use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::config::AppConfig;

const PROJECT_CONTEXT_FILES: [&str; 3] = ["CONTEXT.md", "AGENTS.md", "CLAUDE.md"];
const IDENTITY_CARD_TEMPLATE: &str = include_str!("../../examples/identity-card.md");
const CURRENT_FOCUS_TEMPLATE: &str = include_str!("../../examples/current-focus.md");
const PREFERENCES_TEMPLATE: &str = include_str!("../../examples/preferences.json");
const PRIVACY_RULES_TEMPLATE: &str = include_str!("../../examples/privacy-rules.json");
const DEFAULT_REDACTION_MARKERS: [&str; 2] = ["private:", "local-only:"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextDocument {
    pub label: String,
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalContext {
    pub identity_card: Option<ContextDocument>,
    pub current_focus: Option<ContextDocument>,
    pub preferences: Option<ContextDocument>,
    pub privacy_rules: Option<ContextDocument>,
    pub project_contexts: Vec<ContextDocument>,
    pub missing: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisclosurePolicy {
    redaction_markers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilteredContent {
    pub content: String,
    pub omitted_line_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitReport {
    pub created: Vec<PathBuf>,
    pub existing: Vec<PathBuf>,
}

impl InitReport {
    pub fn render(&self) -> String {
        let mut output = String::from("Context files ready.\n");

        if !self.created.is_empty() {
            output.push_str("\nCreated:\n");
            for path in &self.created {
                output.push_str(&format!("- {}\n", path.display()));
            }
        }

        if !self.existing.is_empty() {
            output.push_str("\nAlready existed:\n");
            for path in &self.existing {
                output.push_str(&format!("- {}\n", path.display()));
            }
        }

        output.push_str("\nEdit identity-card.md and current-focus.md with your real information.");
        output
    }
}

impl DisclosurePolicy {
    pub fn from_context(context: &LocalContext) -> Self {
        let configured = context
            .privacy_rules
            .as_ref()
            .and_then(|document| serde_json::from_str::<Value>(&document.content).ok())
            .and_then(|value| value.get("redaction_markers").cloned())
            .and_then(|markers| markers.as_array().cloned())
            .map(|markers| {
                markers
                    .into_iter()
                    .filter_map(|marker| {
                        marker.as_str().map(str::trim).map(str::to_ascii_lowercase)
                    })
                    .filter(|marker| !marker.is_empty())
                    .collect::<Vec<_>>()
            })
            .filter(|markers| !markers.is_empty());

        Self {
            redaction_markers: configured.unwrap_or_else(|| {
                DEFAULT_REDACTION_MARKERS
                    .iter()
                    .map(|marker| marker.to_string())
                    .collect()
            }),
        }
    }

    pub fn filter_external(&self, content: &str) -> FilteredContent {
        let mut omitted_line_count = 0;
        let lines = content
            .lines()
            .filter(|line| {
                let lower = line.to_ascii_lowercase();
                let blocked = self
                    .redaction_markers
                    .iter()
                    .any(|marker| lower.contains(marker));
                omitted_line_count += usize::from(blocked);
                !blocked
            })
            .collect::<Vec<_>>();

        FilteredContent {
            content: lines.join("\n"),
            omitted_line_count,
        }
    }
}

impl LocalContext {
    pub fn load(config: &AppConfig) -> Result<Self, String> {
        let mut missing = Vec::new();

        let identity_card =
            read_optional("Identity Card", &config.identity_card_path, &mut missing)?;
        let current_focus =
            read_optional("Current Focus", &config.current_focus_path, &mut missing)?;
        let preferences = read_optional("Preferences", &config.preferences_path, &mut missing)?;
        if let Some(document) = &preferences {
            validate_json("preferences", document)?;
        }
        let privacy_rules =
            read_optional("Privacy Rules", &config.privacy_rules_path, &mut missing)?;
        if let Some(document) = &privacy_rules {
            validate_json("privacy rules", document)?;
        }

        let mut project_contexts = Vec::new();
        for filename in PROJECT_CONTEXT_FILES {
            let path = config.workspace.join(filename);
            if let Some(document) = read_if_present(filename, &path)? {
                project_contexts.push(document);
            }
        }

        Ok(Self {
            identity_card,
            current_focus,
            preferences,
            privacy_rules,
            project_contexts,
            missing,
        })
    }

    pub fn render_preview(&self) -> String {
        let disclosure_policy = DisclosurePolicy::from_context(self);
        let mut output = String::from(
            "External context preview\nPolicy: read-only, minimum-necessary disclosure\n\n",
        );

        render_external_document(&mut output, &self.identity_card, &disclosure_policy);
        render_external_document(&mut output, &self.current_focus, &disclosure_policy);
        render_external_document(&mut output, &self.preferences, &disclosure_policy);

        if self.project_contexts.is_empty() {
            output.push_str("Project Context: not found\n\n");
        } else {
            for document in &self.project_contexts {
                render_external_document(&mut output, &Some(document.clone()), &disclosure_policy);
            }
        }

        if !self.missing.is_empty() {
            output.push_str("Missing optional files:\n");
            for path in &self.missing {
                output.push_str(&format!("- {}\n", path.display()));
            }
        }

        output.trim_end().to_string()
    }
}

pub fn load(config: &AppConfig) -> Result<LocalContext, String> {
    LocalContext::load(config)
}

pub fn init_files(config: &AppConfig) -> Result<InitReport, String> {
    fs::create_dir_all(&config.aurora_home).map_err(|err| {
        format!(
            "failed to create AuroraPulse home {}: {err}",
            config.aurora_home.display()
        )
    })?;

    let mut report = InitReport {
        created: Vec::new(),
        existing: Vec::new(),
    };

    write_template_if_missing(
        &config.identity_card_path,
        IDENTITY_CARD_TEMPLATE,
        &mut report,
    )?;
    write_template_if_missing(
        &config.current_focus_path,
        CURRENT_FOCUS_TEMPLATE,
        &mut report,
    )?;
    write_template_if_missing(&config.preferences_path, PREFERENCES_TEMPLATE, &mut report)?;
    write_template_if_missing(
        &config.privacy_rules_path,
        PRIVACY_RULES_TEMPLATE,
        &mut report,
    )?;

    Ok(report)
}

fn write_template_if_missing(
    path: &Path,
    template: &str,
    report: &mut InitReport,
) -> Result<(), String> {
    if path.exists() {
        report.existing.push(path.to_path_buf());
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }

    fs::write(path, template)
        .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    report.created.push(path.to_path_buf());
    Ok(())
}

fn read_optional(
    label: &str,
    path: &Path,
    missing: &mut Vec<PathBuf>,
) -> Result<Option<ContextDocument>, String> {
    match read_if_present(label, path)? {
        Some(document) => Ok(Some(document)),
        None => {
            missing.push(path.to_path_buf());
            Ok(None)
        }
    }
}

fn read_if_present(label: &str, path: &Path) -> Result<Option<ContextDocument>, String> {
    if !path.exists() {
        return Ok(None);
    }
    if !path.is_file() {
        return Err(format!("context source is not a file: {}", path.display()));
    }

    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    Ok(Some(ContextDocument {
        label: label.to_string(),
        path: path.to_path_buf(),
        content: trimmed.to_string(),
    }))
}

fn validate_json(label: &str, document: &ContextDocument) -> Result<(), String> {
    serde_json::from_str::<Value>(&document.content).map_err(|err| {
        format!(
            "failed to parse {label} JSON at {}: {err}",
            document.path.display()
        )
    })?;
    Ok(())
}

fn render_external_document(
    output: &mut String,
    document: &Option<ContextDocument>,
    disclosure_policy: &DisclosurePolicy,
) {
    if let Some(document) = document {
        output.push_str(&format!(
            "{}: {}\n",
            document.label,
            document.path.display()
        ));
        let filtered = disclosure_policy.filter_external(&document.content);
        output.push_str(&filtered.content);
        if filtered.omitted_line_count > 0 {
            output.push_str(&format!(
                "\n[{} line(s) omitted by privacy policy]",
                filtered.omitted_line_count
            ));
        }
        output.push_str("\n\n");
    }
}
