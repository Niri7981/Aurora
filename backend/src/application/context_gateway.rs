use std::collections::HashSet;
use std::path::Path;

use crate::config::AppConfig;
use crate::domain::context::{ContextDocument, LocalContext};
use crate::domain::context_pack::{ContextItem, ContextOmission, ContextPack};
use crate::domain::disclosure::DisclosurePolicy;
use crate::infrastructure::audit_log::AuditLog;
use crate::infrastructure::local_context;

const DEFAULT_SEARCH_LIMIT: usize = 4;
const MAX_SEARCH_LIMIT: usize = 6;
const MAX_ITEM_CHARS: usize = 4_000;

#[derive(Clone)]
pub struct ContextGateway {
    config: AppConfig,
    client: String,
    audit_log: AuditLog,
}

impl ContextGateway {
    pub fn new(config: AppConfig, client: impl Into<String>) -> Self {
        let audit_log = AuditLog::new(config.aurora_home.join("audit/mcp.jsonl"));
        Self {
            config,
            client: client.into(),
            audit_log,
        }
    }

    pub fn audit_log_path(&self) -> &Path {
        self.audit_log.path()
    }

    pub fn get_identity(&self, purpose: &str) -> Result<ContextPack, String> {
        let purpose = required_text(purpose, "purpose")?;
        let context = self.load_context("get_identity", purpose, None)?;
        let documents = context
            .identity_card
            .as_ref()
            .map(|document| vec![("identity", document)])
            .unwrap_or_default();
        self.build_pack(Some("get_identity"), purpose, None, &context, documents)
    }

    pub fn get_current_focus(&self, purpose: &str) -> Result<ContextPack, String> {
        let purpose = required_text(purpose, "purpose")?;
        let context = self.load_context("get_current_focus", purpose, None)?;
        let documents = context
            .current_focus
            .as_ref()
            .map(|document| vec![("current_focus", document)])
            .unwrap_or_default();
        self.build_pack(
            Some("get_current_focus"),
            purpose,
            None,
            &context,
            documents,
        )
    }

    pub fn search_personal_context(
        &self,
        query: &str,
        purpose: &str,
        max_items: Option<usize>,
    ) -> Result<ContextPack, String> {
        self.search_personal_context_with_audit(query, purpose, max_items, true)
    }

    pub(crate) fn search_personal_context_exact_unlogged(
        &self,
        query: &str,
        purpose: &str,
        max_items: Option<usize>,
    ) -> Result<ContextPack, String> {
        let query = required_text(query, "query")?;
        let purpose = required_text(purpose, "purpose")?;
        let context = local_context::load(&self.config)?;
        let mut documents = matching_documents(&context, query);
        let limit = max_items
            .unwrap_or(DEFAULT_SEARCH_LIMIT)
            .clamp(1, MAX_SEARCH_LIMIT);
        documents.truncate(limit);
        let selected = documents
            .into_iter()
            .map(|candidate| (candidate.category, candidate.document))
            .collect();
        self.build_pack(None, purpose, Some(query), &context, selected)
    }

    pub(crate) fn all_personal_context_unlogged(
        &self,
        purpose: &str,
        max_items: Option<usize>,
    ) -> Result<ContextPack, String> {
        let purpose = required_text(purpose, "purpose")?;
        let context = local_context::load(&self.config)?;
        let mut selected = Vec::new();
        if let Some(document) = &context.identity_card {
            selected.push(("identity", document));
        }
        if let Some(document) = &context.current_focus {
            selected.push(("current_focus", document));
        }
        if let Some(document) = &context.preferences {
            selected.push(("preferences", document));
        }
        selected.extend(
            context
                .project_contexts
                .iter()
                .map(|document| ("project_context", document)),
        );
        let limit = max_items
            .unwrap_or(DEFAULT_SEARCH_LIMIT)
            .clamp(1, MAX_SEARCH_LIMIT);
        selected.truncate(limit);
        self.build_pack(None, purpose, None, &context, selected)
    }

    fn search_personal_context_with_audit(
        &self,
        query: &str,
        purpose: &str,
        max_items: Option<usize>,
        audit: bool,
    ) -> Result<ContextPack, String> {
        let query = required_text(query, "query")?;
        let purpose = required_text(purpose, "purpose")?;
        let context = if audit {
            self.load_context("search_personal_context", purpose, Some(query))?
        } else {
            local_context::load(&self.config)?
        };
        let mut documents = scored_documents(&context, query, purpose);
        let limit = max_items
            .unwrap_or(DEFAULT_SEARCH_LIMIT)
            .clamp(1, MAX_SEARCH_LIMIT);
        documents.truncate(limit);
        let selected = documents
            .into_iter()
            .map(|candidate| (candidate.category, candidate.document))
            .collect();

        self.build_pack(
            audit.then_some("search_personal_context"),
            purpose,
            Some(query),
            &context,
            selected,
        )
    }

    fn load_context(
        &self,
        tool: &str,
        purpose: &str,
        query: Option<&str>,
    ) -> Result<LocalContext, String> {
        match local_context::load(&self.config) {
            Ok(context) => Ok(context),
            Err(error) => {
                self.audit_log
                    .append_failure(&self.client, tool, purpose, query, &error)?;
                Err(error)
            }
        }
    }

    fn build_pack(
        &self,
        tool: Option<&str>,
        purpose: &str,
        query: Option<&str>,
        context: &LocalContext,
        documents: Vec<(&str, &ContextDocument)>,
    ) -> Result<ContextPack, String> {
        let policy = DisclosurePolicy::from_context(context);
        let mut items = Vec::new();
        let mut omissions = Vec::new();

        for (category, document) in documents {
            let source = self.source_uri(&document.path);
            let filtered = policy.filter_external(&document.content);
            if filtered.omitted_line_count > 0 {
                omissions.push(ContextOmission {
                    source: source.clone(),
                    reason: "redaction_marker".to_string(),
                    line_count: filtered.omitted_line_count,
                });
            }

            let (content, truncated) = truncate_chars(filtered.content.trim(), MAX_ITEM_CHARS);
            if !content.is_empty() {
                items.push(ContextItem {
                    category: category.to_string(),
                    label: document.label.clone(),
                    source,
                    content,
                    truncated,
                });
            }
        }

        let pack = ContextPack {
            purpose: purpose.to_string(),
            query: query.map(str::to_string),
            client: self.client.clone(),
            access: "read-only, minimum-necessary disclosure".to_string(),
            items,
            omissions,
        };

        if let Some(tool) = tool {
            self.audit_log.append_success(&self.client, tool, &pack)?;
        }
        Ok(pack)
    }

    fn source_uri(&self, path: &Path) -> String {
        if let Ok(relative) = path.strip_prefix(&self.config.aurora_home) {
            return format!("aurora://{}", relative.display());
        }
        if let Ok(relative) = path.strip_prefix(&self.config.workspace) {
            return format!("workspace://{}", relative.display());
        }
        format!(
            "local://{}",
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("context")
        )
    }
}

struct ScoredDocument<'a> {
    category: &'static str,
    document: &'a ContextDocument,
    score: usize,
}

fn scored_documents<'a>(
    context: &'a LocalContext,
    query: &str,
    purpose: &str,
) -> Vec<ScoredDocument<'a>> {
    let terms = search_terms(&format!("{query} {purpose}"));
    let mut candidates = Vec::new();
    if let Some(document) = &context.identity_card {
        candidates.push(scored("identity", document, 30, &terms));
    }
    if let Some(document) = &context.current_focus {
        candidates.push(scored("current_focus", document, 25, &terms));
    }
    if let Some(document) = &context.preferences {
        candidates.push(scored("preferences", document, 10, &terms));
    }
    for document in &context.project_contexts {
        let candidate = scored("project_context", document, 0, &terms);
        if candidate.score > 0 {
            candidates.push(candidate);
        }
    }
    candidates.sort_by(|left, right| right.score.cmp(&left.score));
    candidates
}

fn matching_documents<'a>(context: &'a LocalContext, query: &str) -> Vec<ScoredDocument<'a>> {
    let terms = search_terms(query);
    let mut candidates = Vec::new();
    if let Some(document) = &context.identity_card {
        candidates.push(scored("identity", document, 0, &terms));
    }
    if let Some(document) = &context.current_focus {
        candidates.push(scored("current_focus", document, 0, &terms));
    }
    if let Some(document) = &context.preferences {
        candidates.push(scored("preferences", document, 0, &terms));
    }
    for document in &context.project_contexts {
        candidates.push(scored("project_context", document, 0, &terms));
    }
    candidates.retain(|candidate| candidate.score > 0);
    candidates.sort_by(|left, right| right.score.cmp(&left.score));
    candidates
}

fn scored<'a>(
    category: &'static str,
    document: &'a ContextDocument,
    base_score: usize,
    terms: &HashSet<String>,
) -> ScoredDocument<'a> {
    let document_terms = search_terms(&document.content);
    let overlap = terms.intersection(&document_terms).count();
    ScoredDocument {
        category,
        document,
        score: base_score + overlap * 5,
    }
}

fn search_terms(text: &str) -> HashSet<String> {
    let mut terms = HashSet::new();
    let mut ascii = String::new();
    let mut cjk = Vec::new();

    let flush_ascii = |value: &mut String, terms: &mut HashSet<String>| {
        if value.len() >= 2 {
            terms.insert(value.clone());
        }
        value.clear();
    };
    let flush_cjk = |value: &mut Vec<char>, terms: &mut HashSet<String>| {
        if value.len() >= 2 {
            terms.insert(value.iter().collect());
            for pair in value.windows(2) {
                terms.insert(pair.iter().collect());
            }
        }
        value.clear();
    };

    for character in text.chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            flush_cjk(&mut cjk, &mut terms);
            ascii.push(character.to_ascii_lowercase());
        } else if is_cjk(character) {
            flush_ascii(&mut ascii, &mut terms);
            cjk.push(character);
        } else {
            flush_ascii(&mut ascii, &mut terms);
            flush_cjk(&mut cjk, &mut terms);
        }
    }
    flush_ascii(&mut ascii, &mut terms);
    flush_cjk(&mut cjk, &mut terms);
    terms
}

fn is_cjk(character: char) -> bool {
    matches!(character, '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}')
}

fn truncate_chars(content: &str, max_chars: usize) -> (String, bool) {
    let mut characters = content.chars();
    let truncated = characters.by_ref().take(max_chars).collect::<String>();
    let was_truncated = characters.next().is_some();
    (truncated, was_truncated)
}

fn required_text<'a>(value: &'a str, field: &str) -> Result<&'a str, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(value)
}
