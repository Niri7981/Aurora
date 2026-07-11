use std::process::Command;

use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolRisk {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgumentKind {
    NonEmptyString,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequiredArgument {
    pub name: &'static str,
    pub kind: ArgumentKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub risk: ToolRisk,
    pub required_arguments: &'static [RequiredArgument],
}

#[derive(Debug, PartialEq, Eq)]
pub enum ToolOutcome {
    Completed(ToolResult),
    NeedsConfirmation(ToolConfirmation),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ToolResult {
    pub tool_name: String,
    pub summary: String,
    pub data: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolConfirmation {
    pub tool_name: String,
    pub risk: ToolRisk,
    pub prompt: String,
    pub data: Value,
}

pub type ToolHandler = fn(&Value) -> Result<ToolOutcome, String>;

pub struct ToolDefinition {
    spec: ToolSpec,
    handler: ToolHandler,
    confirmation_handler: Option<ToolHandler>,
}

impl ToolDefinition {
    pub fn new(spec: ToolSpec, handler: ToolHandler) -> Self {
        Self {
            spec,
            handler,
            confirmation_handler: None,
        }
    }

    pub fn with_confirmation(
        spec: ToolSpec,
        handler: ToolHandler,
        confirmation_handler: ToolHandler,
    ) -> Self {
        Self {
            spec,
            handler,
            confirmation_handler: Some(confirmation_handler),
        }
    }
}

pub struct ToolRegistry {
    tools: Vec<ToolDefinition>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn with_builtin_tools() -> Self {
        let mut registry = Self::new();
        registry.register(ToolDefinition::with_confirmation(
            ToolSpec {
                name: "local_launch.open_app",
                description: "Open a local desktop application after user confirmation.",
                risk: ToolRisk::Medium,
                required_arguments: &[RequiredArgument {
                    name: "app_name",
                    kind: ArgumentKind::NonEmptyString,
                }],
            },
            request_open_app_confirmation,
            execute_open_app,
        ));
        registry.register(ToolDefinition::new(
            ToolSpec {
                name: "spotify.play_artist",
                description: "Search Spotify for an artist and start playing one of their top tracks.",
                risk: ToolRisk::Low,
                required_arguments: &[RequiredArgument {
                    name: "query",
                    kind: ArgumentKind::NonEmptyString,
                }],
            },
            play_spotify_artist,
        ));
        registry.register(ToolDefinition::new(
            ToolSpec {
                name: "spotify.play_track",
                description: "Search Spotify for a track and start playing it.",
                risk: ToolRisk::Low,
                required_arguments: &[RequiredArgument {
                    name: "query",
                    kind: ArgumentKind::NonEmptyString,
                }],
            },
            play_spotify_track,
        ));
        registry
    }

    pub fn register(&mut self, definition: ToolDefinition) {
        self.tools.push(definition);
    }

    pub fn specs(&self) -> impl Iterator<Item = &ToolSpec> {
        self.tools.iter().map(|definition| &definition.spec)
    }

    pub fn invoke(&self, tool_name: &str, arguments: Value) -> Result<ToolOutcome, String> {
        let definition = self
            .definition(tool_name)
            .ok_or_else(|| format!("未知工具：{tool_name}"))?;

        validate_arguments(&definition.spec, &arguments)?;
        (definition.handler)(&arguments)
    }

    pub fn confirm(&self, tool_name: &str, arguments: Value) -> Result<ToolResult, String> {
        let definition = self
            .definition(tool_name)
            .ok_or_else(|| format!("未知工具：{tool_name}"))?;
        let confirmation_handler = definition
            .confirmation_handler
            .ok_or_else(|| format!("{tool_name} 没有可确认执行的动作"))?;

        validate_arguments(&definition.spec, &arguments)?;
        match confirmation_handler(&arguments)? {
            ToolOutcome::Completed(result) => Ok(result),
            ToolOutcome::NeedsConfirmation(_) => Err(format!("{tool_name} 确认后仍然要求再次确认")),
        }
    }

    fn definition(&self, tool_name: &str) -> Option<&ToolDefinition> {
        self.tools
            .iter()
            .find(|definition| definition.spec.name == tool_name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::with_builtin_tools()
    }
}

fn validate_arguments(spec: &ToolSpec, arguments: &Value) -> Result<(), String> {
    if !arguments.is_object() {
        return Err(format!("{} 的 arguments 必须是 object", spec.name));
    }

    for required in spec.required_arguments {
        match required.kind {
            ArgumentKind::NonEmptyString => {
                let value = arguments
                    .get(required.name)
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("{} 缺少参数：{}", spec.name, required.name))?;

                if value.trim().is_empty() {
                    return Err(format!("{} 的 {} 不能为空", spec.name, required.name));
                }
            }
        }
    }

    Ok(())
}

fn request_open_app_confirmation(arguments: &Value) -> Result<ToolOutcome, String> {
    let app_name = arguments
        .get("app_name")
        .and_then(Value::as_str)
        .expect("app_name should be validated before handler runs")
        .trim();

    Ok(ToolOutcome::NeedsConfirmation(ToolConfirmation {
        tool_name: "local_launch.open_app".to_string(),
        risk: ToolRisk::Medium,
        prompt: format!("打开 {app_name} 是一个本地启动动作，需要你确认。"),
        data: json!({
            "app_name": app_name,
            "action": "open_app"
        }),
    }))
}

fn execute_open_app(arguments: &Value) -> Result<ToolOutcome, String> {
    let app_name = arguments
        .get("app_name")
        .and_then(Value::as_str)
        .expect("app_name should be validated before handler runs")
        .trim();

    open_application(app_name)?;

    Ok(ToolOutcome::Completed(ToolResult {
        tool_name: "local_launch.open_app".to_string(),
        summary: format!("已打开 {app_name}。"),
        data: json!({
            "app_name": app_name,
            "action": "open_app"
        }),
    }))
}

#[cfg(target_os = "macos")]
fn open_application(app_name: &str) -> Result<(), String> {
    let output = Command::new("open")
        .args(["-a", app_name])
        .output()
        .map_err(|err| format!("无法启动 macOS open 命令：{err}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        Err(format!("open -a {app_name} 失败：{}", output.status))
    } else {
        Err(stderr)
    }
}

#[cfg(not(target_os = "macos"))]
fn open_application(_app_name: &str) -> Result<(), String> {
    Err("local_launch.open_app 目前只支持 macOS 的 open -a".to_string())
}

fn play_spotify_artist(arguments: &Value) -> Result<ToolOutcome, String> {
    run_spotify_helper("play_artist", required_query(arguments)?)
}

fn play_spotify_track(arguments: &Value) -> Result<ToolOutcome, String> {
    run_spotify_helper("play_track", required_query(arguments)?)
}

fn required_query(arguments: &Value) -> Result<&str, String> {
    arguments
        .get("query")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|query| !query.is_empty())
        .ok_or_else(|| "spotify tool 缺少参数：query".to_string())
}

fn run_spotify_helper(action: &str, query: &str) -> Result<ToolOutcome, String> {
    let output = Command::new("python3")
        .args([
            "-m",
            "aurorapulse.integrations.music.spotify_tool",
            action,
            query,
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .map_err(|err| format!("无法启动 Spotify 工具 helper：{err}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() {
        let detail = if stderr.is_empty() { stdout } else { stderr };
        return Err(if detail.is_empty() {
            format!("Spotify 工具执行失败，退出状态：{}", output.status)
        } else {
            detail
        });
    }

    let summary = if stdout.is_empty() {
        format!("已在 Spotify 开始播放：{query}")
    } else {
        stdout
    };

    Ok(ToolOutcome::Completed(ToolResult {
        tool_name: format!("spotify.{action}"),
        summary,
        data: json!({
            "action": action,
            "query": query
        }),
    }))
}

#[cfg(test)]
mod tests {
    use super::{ToolOutcome, ToolRegistry, ToolRisk};

    #[test]
    fn built_in_registry_exposes_local_launch_tool() {
        let registry = ToolRegistry::with_builtin_tools();

        let names = registry.specs().map(|spec| spec.name).collect::<Vec<_>>();

        assert_eq!(
            names,
            vec![
                "local_launch.open_app",
                "spotify.play_artist",
                "spotify.play_track"
            ]
        );
    }

    #[test]
    fn local_launch_open_app_requires_app_name() {
        let registry = ToolRegistry::with_builtin_tools();

        let err = registry
            .invoke("local_launch.open_app", serde_json::json!({}))
            .expect_err("missing required argument should fail validation");

        assert!(err.contains("app_name"), "unexpected error: {err}");
    }

    #[test]
    fn local_launch_open_app_normalizes_confirmation_request() {
        let registry = ToolRegistry::with_builtin_tools();

        let outcome = registry
            .invoke(
                "local_launch.open_app",
                serde_json::json!({ "app_name": "Spotify" }),
            )
            .expect("tool should validate and produce an outcome");

        match outcome {
            ToolOutcome::NeedsConfirmation(confirmation) => {
                assert_eq!(confirmation.tool_name, "local_launch.open_app");
                assert_eq!(confirmation.risk, ToolRisk::Medium);
                assert!(confirmation.prompt.contains("Spotify"));
                assert_eq!(confirmation.data["app_name"], "Spotify");
            }
            ToolOutcome::Completed(_) => panic!("open_app should require confirmation first"),
        }
    }

    #[test]
    fn spotify_play_artist_requires_query() {
        let registry = ToolRegistry::with_builtin_tools();

        let err = registry
            .invoke("spotify.play_artist", serde_json::json!({}))
            .expect_err("missing required argument should fail validation");

        assert!(err.contains("query"), "unexpected error: {err}");
    }

    #[test]
    fn unknown_tool_is_rejected() {
        let registry = ToolRegistry::with_builtin_tools();

        let err = registry
            .invoke(
                "calendar.create_event",
                serde_json::json!({ "title": "Demo" }),
            )
            .expect_err("unknown tool should fail validation");

        assert!(err.contains("未知工具"), "unexpected error: {err}");
    }
}
