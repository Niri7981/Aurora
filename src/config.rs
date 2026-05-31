use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_MODEL: &str = "gemma4:e4b";
pub const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";

pub struct AppConfig {
    pub workspace: PathBuf,
    pub model: String,
    pub ollama_url: String,
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

    let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    let ollama_url = env::var("OLLAMA_URL").unwrap_or_else(|_| DEFAULT_OLLAMA_URL.to_string());

    Ok(AppConfig {
        workspace,
        model,
        ollama_url,
    })
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
