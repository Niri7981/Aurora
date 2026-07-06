use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::config::AppConfig;

const PROJECT_CONTEXT_FILES: [&str; 3] = ["CONTEXT.md", "AGENTS.md", "CLAUDE.md"];
const IDENTITY_CARD_TEMPLATE: &str = include_str!("../examples/identity-card.md");
const CURRENT_FOCUS_TEMPLATE: &str = include_str!("../examples/current-focus.md");
const PREFERENCES_TEMPLATE: &str = include_str!("../examples/preferences.json");
const PRIVACY_RULES_TEMPLATE: &str = include_str!("../examples/privacy-rules.json");

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Local,
    Cloud,
}

impl ProviderKind {
    pub fn from_name(provider: &str) -> Self {
        match provider {
            "openai" | "anthropic" | "gemini" => Self::Cloud,
            _ => Self::Local,
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

    pub fn render_preview(&self, provider: &str) -> String {
        let provider_kind = ProviderKind::from_name(provider);
        let mut output = String::from("Context preview\n");
        output.push_str(&format!("Provider: {provider}\n"));
        output.push_str(&format!("Policy: {:?}\n\n", provider_kind));

        render_document(&mut output, &self.identity_card, provider_kind);
        render_document(&mut output, &self.current_focus, provider_kind);
        render_document(&mut output, &self.preferences, provider_kind);
        if provider_kind == ProviderKind::Local {
            render_document(&mut output, &self.privacy_rules, provider_kind);
        }

        if self.project_contexts.is_empty() {
            output.push_str("Project Context: not found\n\n");
        } else {
            for document in &self.project_contexts {
                render_document(&mut output, &Some(document.clone()), provider_kind);
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

    pub fn render_model_context(&self, provider: &str) -> String {
        let provider_kind = ProviderKind::from_name(provider);
        let mut output = String::from(
            "Use this local AuroraPulse context to understand who you are helping. Do not claim these files exist unless the user asks about sources.\n\n",
        );

        append_prompt_document(&mut output, &self.identity_card, provider_kind);
        append_prompt_document(&mut output, &self.current_focus, provider_kind);
        append_prompt_document(&mut output, &self.preferences, provider_kind);

        if provider_kind == ProviderKind::Local {
            append_prompt_document(&mut output, &self.privacy_rules, provider_kind);
        }

        for document in &self.project_contexts {
            append_prompt_document(&mut output, &Some(document.clone()), provider_kind);
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

pub fn compose_user_prompt(context: &LocalContext, provider: &str, user_text: &str) -> String {
    let context_text = context.render_model_context(provider);
    if context_text.trim().is_empty() {
        return user_text.to_string();
    }

    format!(
        "{context_text}\n\nCurrent user request:\n{user_text}",
        context_text = context_text,
        user_text = user_text
    )
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

fn render_document(
    output: &mut String,
    document: &Option<ContextDocument>,
    provider_kind: ProviderKind,
) {
    if let Some(document) = document {
        output.push_str(&format!(
            "{}: {}\n",
            document.label,
            document.path.display()
        ));
        output.push_str(&redact_for_provider(&document.content, provider_kind));
        output.push_str("\n\n");
    }
}

fn append_prompt_document(
    output: &mut String,
    document: &Option<ContextDocument>,
    provider_kind: ProviderKind,
) {
    if let Some(document) = document {
        output.push_str(&format!("## {}\n", document.label));
        output.push_str(&redact_for_provider(&document.content, provider_kind));
        output.push_str("\n\n");
    }
}

fn redact_for_provider(content: &str, provider_kind: ProviderKind) -> String {
    if provider_kind == ProviderKind::Local {
        return content.to_string();
    }

    content
        .lines()
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            !(lower.contains("private:") || lower.contains("local-only:"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}
