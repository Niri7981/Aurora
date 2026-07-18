use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use rmcp::{
    ErrorData as McpError, Json, ServerHandler, ServiceExt, handler::server::wrapper::Parameters,
    schemars::JsonSchema, tool, tool_handler, tool_router, transport::stdio,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::AppConfig;
use crate::context::{self, ContextDocument, DisclosurePolicy, LocalContext};

const DEFAULT_SEARCH_LIMIT: usize = 4;
const MAX_SEARCH_LIMIT: usize = 6;
const MAX_ITEM_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
pub struct ContextPack {
    pub purpose: String,
    pub query: Option<String>,
    pub client: String,
    pub access: String,
    pub items: Vec<ContextItem>,
    pub omissions: Vec<ContextOmission>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
pub struct ContextItem {
    pub category: String,
    pub label: String,
    pub source: String,
    pub content: String,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
pub struct ContextOmission {
    pub source: String,
    pub reason: String,
    pub line_count: usize,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct PurposeRequest {
    /// Why the Agent needs this personal context for the current user request.
    purpose: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchRequest {
    /// The personal context to find. Keep this focused on the current task.
    query: String,
    /// Why the Agent needs this personal context for the current user request.
    purpose: String,
    /// Maximum number of context items to return. Values above 6 are clamped.
    max_items: Option<usize>,
}

#[derive(Clone)]
pub struct AuroraContextService {
    config: AppConfig,
    client: String,
    audit_log_path: PathBuf,
    audit_lock: Arc<Mutex<()>>,
}

impl AuroraContextService {
    pub fn new(config: AppConfig, client: impl Into<String>) -> Self {
        let audit_log_path = config.aurora_home.join("audit/mcp.jsonl");
        Self {
            config,
            client: client.into(),
            audit_log_path,
            audit_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn audit_log_path(&self) -> &Path {
        &self.audit_log_path
    }

    pub fn get_identity(&self, purpose: &str) -> Result<ContextPack, String> {
        let purpose = required_text(purpose, "purpose")?;
        let context = self.load_context("get_identity", purpose, None)?;
        let documents = context
            .identity_card
            .as_ref()
            .map(|document| vec![("identity", document)])
            .unwrap_or_default();
        self.build_and_audit_pack("get_identity", purpose, None, &context, documents)
    }

    pub fn get_current_focus(&self, purpose: &str) -> Result<ContextPack, String> {
        let purpose = required_text(purpose, "purpose")?;
        let context = self.load_context("get_current_focus", purpose, None)?;
        let documents = context
            .current_focus
            .as_ref()
            .map(|document| vec![("current_focus", document)])
            .unwrap_or_default();
        self.build_and_audit_pack("get_current_focus", purpose, None, &context, documents)
    }

    pub fn search_personal_context(
        &self,
        query: &str,
        purpose: &str,
        max_items: Option<usize>,
    ) -> Result<ContextPack, String> {
        let query = required_text(query, "query")?;
        let purpose = required_text(purpose, "purpose")?;
        let context = self.load_context("search_personal_context", purpose, Some(query))?;
        let mut documents = scored_documents(&context, query, purpose);
        let limit = max_items
            .unwrap_or(DEFAULT_SEARCH_LIMIT)
            .clamp(1, MAX_SEARCH_LIMIT);
        documents.truncate(limit);
        let selected = documents
            .into_iter()
            .map(|candidate| (candidate.category, candidate.document))
            .collect();

        self.build_and_audit_pack(
            "search_personal_context",
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
        match context::load(&self.config) {
            Ok(context) => Ok(context),
            Err(error) => {
                self.append_audit(AuditEvent::failure(
                    &self.client,
                    tool,
                    purpose,
                    query,
                    &error,
                ))?;
                Err(error)
            }
        }
    }

    fn build_and_audit_pack(
        &self,
        tool: &str,
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

        self.append_audit(AuditEvent::success(&self.client, tool, &pack))?;
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

    fn append_audit(&self, event: AuditEvent<'_>) -> Result<(), String> {
        let _guard = self
            .audit_lock
            .lock()
            .map_err(|_| "MCP audit lock is poisoned".to_string())?;
        if let Some(parent) = self.audit_log_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create MCP audit directory: {error}"))?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.audit_log_path)
            .map_err(|error| format!("failed to open MCP audit log: {error}"))?;
        serde_json::to_writer(&mut file, &event)
            .map_err(|error| format!("failed to serialize MCP audit event: {error}"))?;
        file.write_all(b"\n")
            .map_err(|error| format!("failed to write MCP audit log: {error}"))
    }
}

#[derive(Serialize)]
struct AuditEvent<'a> {
    timestamp_unix_ms: u64,
    client: &'a str,
    tool: &'a str,
    purpose: &'a str,
    query: Option<&'a str>,
    status: &'a str,
    returned_sources: Vec<&'a str>,
    omitted_lines: usize,
    error: Option<&'a str>,
}

impl<'a> AuditEvent<'a> {
    fn success(client: &'a str, tool: &'a str, pack: &'a ContextPack) -> Self {
        Self {
            timestamp_unix_ms: timestamp_unix_ms(),
            client,
            tool,
            purpose: &pack.purpose,
            query: pack.query.as_deref(),
            status: "succeeded",
            returned_sources: pack.items.iter().map(|item| item.source.as_str()).collect(),
            omitted_lines: pack
                .omissions
                .iter()
                .map(|omission| omission.line_count)
                .sum(),
            error: None,
        }
    }

    fn failure(
        client: &'a str,
        tool: &'a str,
        purpose: &'a str,
        query: Option<&'a str>,
        error: &'a str,
    ) -> Self {
        Self {
            timestamp_unix_ms: timestamp_unix_ms(),
            client,
            tool,
            purpose,
            query,
            status: "failed",
            returned_sources: Vec::new(),
            omitted_lines: 0,
            error: Some(error),
        }
    }
}

#[derive(Clone)]
pub struct AuroraMcpServer {
    service: AuroraContextService,
}

#[tool_router]
impl AuroraMcpServer {
    pub fn new(config: AppConfig, client: impl Into<String>) -> Self {
        Self {
            service: AuroraContextService::new(config, client),
        }
    }

    #[tool(
        description = "Return the user's filtered identity card for a stated task purpose. Read-only; never returns privacy rules or unmarked files.",
        annotations(
            title = "Get Aurora identity",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn get_identity(
        &self,
        Parameters(request): Parameters<PurposeRequest>,
    ) -> Result<Json<ContextPack>, McpError> {
        validate_mcp_text(&request.purpose, "purpose")?;
        self.service
            .get_identity(&request.purpose)
            .map(Json)
            .map_err(internal_mcp_error)
    }

    #[tool(
        description = "Return the user's filtered current focus for a stated task purpose. Use when the request depends on what the user is doing now.",
        annotations(
            title = "Get Aurora current focus",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn get_current_focus(
        &self,
        Parameters(request): Parameters<PurposeRequest>,
    ) -> Result<Json<ContextPack>, McpError> {
        validate_mcp_text(&request.purpose, "purpose")?;
        self.service
            .get_current_focus(&request.purpose)
            .map(Json)
            .map_err(internal_mcp_error)
    }

    #[tool(
        description = "Search the user's authorized local personal context for the current task. Returns a small, filtered Context Pack with source URIs and omission metadata.",
        annotations(
            title = "Search Aurora personal context",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn search_personal_context(
        &self,
        Parameters(request): Parameters<SearchRequest>,
    ) -> Result<Json<ContextPack>, McpError> {
        validate_mcp_text(&request.query, "query")?;
        validate_mcp_text(&request.purpose, "purpose")?;
        self.service
            .search_personal_context(&request.query, &request.purpose, request.max_items)
            .map(Json)
            .map_err(internal_mcp_error)
    }
}

#[tool_handler(
    name = "aurora",
    version = "0.1.0",
    instructions = "Aurora is the user's local identity and context authority. Call only when personal context materially helps the current request. State a narrow purpose, use the minimum returned context, respect omissions, and never claim access beyond the returned Context Pack."
)]
impl ServerHandler for AuroraMcpServer {}

pub fn run(config: AppConfig) -> Result<(), String> {
    let client = std::env::var("AURORA_MCP_CLIENT").unwrap_or_else(|_| "unknown-agent".to_string());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("failed to start MCP runtime: {error}"))?;
    runtime.block_on(async move {
        let server = AuroraMcpServer::new(config, client);
        let service = server
            .serve(stdio())
            .await
            .map_err(|error| format!("failed to serve Aurora MCP: {error}"))?;
        service
            .waiting()
            .await
            .map_err(|error| format!("Aurora MCP stopped with an error: {error}"))?;
        Ok(())
    })
}

pub fn render_audit_log(config: &AppConfig, limit: usize) -> Result<String, String> {
    let path = config.aurora_home.join("audit/mcp.jsonl");
    if !path.exists() {
        return Ok("暂无 MCP 上下文调用记录。".to_string());
    }
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read MCP audit log: {error}"))?;
    let mut rendered = content
        .lines()
        .rev()
        .take(limit)
        .map(render_audit_line)
        .collect::<Result<Vec<_>, _>>()?;
    if rendered.is_empty() {
        return Ok("暂无 MCP 上下文调用记录。".to_string());
    }
    rendered.reverse();
    Ok(rendered.join("\n"))
}

fn render_audit_line(line: &str) -> Result<String, String> {
    let event: Value = serde_json::from_str(line)
        .map_err(|error| format!("failed to parse MCP audit event: {error}"))?;
    let timestamp = event
        .get("timestamp_unix_ms")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let client = event
        .get("client")
        .and_then(Value::as_str)
        .unwrap_or("unknown-agent");
    let tool = event
        .get("tool")
        .and_then(Value::as_str)
        .unwrap_or("unknown-tool");
    let status = event
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let purpose = event
        .get("purpose")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let (purpose, _) = truncate_chars(purpose, 80);
    let sources = event
        .get("returned_sources")
        .and_then(Value::as_array)
        .map(|sources| {
            sources
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|sources| !sources.is_empty())
        .unwrap_or_else(|| "none".to_string());
    let omitted = event
        .get("omitted_lines")
        .and_then(Value::as_u64)
        .unwrap_or_default();

    Ok(format!(
        "{timestamp}  {client:<12}  {tool:<25}  {status:<9}  sources={sources}  omitted={omitted}  purpose={purpose}"
    ))
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

fn validate_mcp_text(value: &str, field: &str) -> Result<(), McpError> {
    required_text(value, field)
        .map(|_| ())
        .map_err(|error| McpError::invalid_params(error, None))
}

fn internal_mcp_error(error: String) -> McpError {
    McpError::internal_error(error, None)
}

fn timestamp_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
