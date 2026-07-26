use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::config::AppConfig;
use crate::domain::context::{ContextDocument, LocalContext};

const PROJECT_CONTEXT_FILES: [&str; 3] = ["CONTEXT.md", "AGENTS.md", "CLAUDE.md"];
const IDENTITY_CARD_TEMPLATE: &str = include_str!("../../../examples/identity-card.md");
const CURRENT_FOCUS_TEMPLATE: &str = include_str!("../../../examples/current-focus.md");
const PREFERENCES_TEMPLATE: &str = include_str!("../../../examples/preferences.json");
const PRIVACY_RULES_TEMPLATE: &str = include_str!("../../../examples/privacy-rules.json");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitReport {
    pub created: Vec<PathBuf>,
    pub existing: Vec<PathBuf>,
}

pub fn load(config: &AppConfig) -> Result<LocalContext, String> {
    let mut missing = Vec::new();

    let identity_card = read_optional("Identity Card", &config.identity_card_path, &mut missing)?;
    let current_focus = read_optional("Current Focus", &config.current_focus_path, &mut missing)?;
    let preferences = read_optional("Preferences", &config.preferences_path, &mut missing)?;
    if let Some(document) = &preferences {
        validate_json("preferences", document)?;
    }
    let privacy_rules = read_optional("Privacy Rules", &config.privacy_rules_path, &mut missing)?;
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

    Ok(LocalContext {
        identity_card,
        current_focus,
        preferences,
        privacy_rules,
        project_contexts,
        missing,
    })
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
