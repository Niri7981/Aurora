use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_MODEL: &str = "gemma4:e4b";
pub const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";
pub const DEFAULT_PROVIDER: &str = "ollama";
pub const DEFAULT_OPENAI_MODEL: &str = "gpt-4o-mini";
pub const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com";

#[derive(Clone)]
pub struct AppConfig {
    pub workspace: PathBuf,
    pub provider: String,
    pub model: String,
    pub ollama_url: String,
    pub openai_api_key: Option<String>,
    pub openai_base_url: String,
    pub openai_model: String,
    pub aurora_home: PathBuf,
    pub identity_card_path: PathBuf,
    pub current_focus_path: PathBuf,
    pub preferences_path: PathBuf,
    pub privacy_rules_path: PathBuf,
}

pub fn load_config(workspace_arg: Option<String>) -> Result<AppConfig, String> {
    let workspace_input = workspace_arg.unwrap_or_else(|| ".".to_string());
    let workspace = resolve_workspace(Path::new(&workspace_input))?;
    env::set_current_dir(&workspace).map_err(|err| {
        format!(
            "failed to switch into workspace {}: {err}",
            workspace.display()
        )
    })?;

    load_dotenv(&workspace)?;

    let provider = env::var("AURORA_PROVIDER").unwrap_or_else(|_| DEFAULT_PROVIDER.to_string());
    let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    let ollama_url = env::var("OLLAMA_URL").unwrap_or_else(|_| DEFAULT_OLLAMA_URL.to_string());
    let openai_api_key = env::var("OPENAI_API_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let openai_base_url =
        env::var("OPENAI_BASE_URL").unwrap_or_else(|_| DEFAULT_OPENAI_BASE_URL.to_string());
    let openai_model =
        env::var("OPENAI_MODEL").unwrap_or_else(|_| DEFAULT_OPENAI_MODEL.to_string());
    let aurora_home = resolve_aurora_home()?;
    let identity_card_path =
        env_path_or_default("AURORA_IDENTITY_CARD", "identity-card.md", &aurora_home);
    let current_focus_path =
        env_path_or_default("AURORA_CURRENT_FOCUS", "current-focus.md", &aurora_home);
    let preferences_path =
        env_path_or_default("AURORA_PREFERENCES", "preferences.json", &aurora_home);
    let privacy_rules_path =
        env_path_or_default("AURORA_PRIVACY_RULES", "privacy-rules.json", &aurora_home);

    Ok(AppConfig {
        workspace,
        provider,
        model,
        ollama_url,
        openai_api_key,
        openai_base_url,
        openai_model,
        aurora_home,
        identity_card_path,
        current_focus_path,
        preferences_path,
        privacy_rules_path,
    })
}

fn resolve_aurora_home() -> Result<PathBuf, String> {
    if let Ok(raw) = env::var("AURORA_HOME") {
        return Ok(PathBuf::from(raw));
    }

    let home = env::var("HOME").map_err(|_| {
        "HOME is not set; set AURORA_HOME to choose where AuroraPulse stores local identity context"
            .to_string()
    })?;
    Ok(PathBuf::from(home).join(".aurorapulse"))
}

fn env_path_or_default(env_key: &str, filename: &str, aurora_home: &Path) -> PathBuf {
    env::var(env_key)
        .map(PathBuf::from)
        .unwrap_or_else(|_| aurora_home.join(filename))
}

fn resolve_workspace(input: &Path) -> Result<PathBuf, String> {
    if !input.exists() {
        return Err(format!("workspace not found: {}", input.display()));
    }
    if !input.is_dir() {
        return Err(format!("workspace is not a directory: {}", input.display()));
    }
    input
        .canonicalize()
        .map_err(|err| format!("failed to resolve {}: {err}", input.display()))
}

fn load_dotenv(workspace: &Path) -> Result<(), String> {
    let env_path = workspace.join(".env");
    if !env_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&env_path)
        .map_err(|err| format!("failed to read {}: {err}", env_path.display()))?;

    for line in content.lines() {
        let raw = line.trim();
        if raw.is_empty() || raw.starts_with('#') || raw.starts_with('[') {
            continue;
        }
        if let Some((key, value)) = raw.split_once('=') {
            if env::var_os(key.trim()).is_none() {
                // Safety: we set process env during single-threaded startup before any
                // worker threads exist, which avoids concurrent environment access.
                unsafe {
                    env::set_var(key.trim(), value.trim());
                }
            }
        }
    }

    Ok(())
}
