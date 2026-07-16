use std::fmt;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Map, Value, json};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(20);
const COMMAND_POLL_INTERVAL: Duration = Duration::from_millis(20);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolRisk {
    Low,
    Medium,
    High,
}

impl ToolRisk {
    pub fn requires_confirmation(self) -> bool {
        !matches!(self, Self::Low)
    }

    pub fn allows_session_bypass(self) -> bool {
        matches!(self, Self::Medium)
    }
}

impl fmt::Display for ToolRisk {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgumentKind {
    NonEmptyString,
}

impl ArgumentKind {
    fn planner_description(self) -> &'static str {
        match self {
            Self::NonEmptyString => "non-empty string",
        }
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolStatus {
    Succeeded,
    Failed,
    Denied,
}

impl fmt::Display for ToolStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Denied => "denied",
        };
        formatter.write_str(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ToolOutcome {
    Completed(ToolResult),
    NeedsConfirmation(ToolConfirmation),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolResult {
    pub tool_name: String,
    pub status: ToolStatus,
    pub summary: String,
    pub data: Value,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn succeeded(
        tool_name: impl Into<String>,
        summary: impl Into<String>,
        data: Value,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            status: ToolStatus::Succeeded,
            summary: summary.into(),
            data,
            error: None,
        }
    }

    pub fn failed(tool_name: impl Into<String>, error: impl Into<String>, data: Value) -> Self {
        let tool_name = tool_name.into();
        let error = error.into();
        Self {
            tool_name,
            status: ToolStatus::Failed,
            summary: format!("我没法执行这个工具请求：{error}"),
            data,
            error: Some(error),
        }
    }

    pub fn denied(tool_name: impl Into<String>, data: Value) -> Self {
        Self {
            tool_name: tool_name.into(),
            status: ToolStatus::Denied,
            summary: "已取消。".to_string(),
            data,
            error: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolConfirmation {
    pub tool_name: String,
    pub risk: ToolRisk,
    pub prompt: String,
    pub data: Value,
}

pub type ToolHandler = fn(&Value) -> Result<ToolResult, String>;
pub type ConfirmationPrompt = fn(&Value) -> String;

pub struct ToolDefinition {
    spec: ToolSpec,
    handler: ToolHandler,
    confirmation_prompt: Option<ConfirmationPrompt>,
}

impl ToolDefinition {
    pub fn new(spec: ToolSpec, handler: ToolHandler) -> Self {
        Self {
            spec,
            handler,
            confirmation_prompt: None,
        }
    }

    pub fn with_confirmation(
        spec: ToolSpec,
        handler: ToolHandler,
        confirmation_prompt: ConfirmationPrompt,
    ) -> Self {
        Self {
            spec,
            handler,
            confirmation_prompt: Some(confirmation_prompt),
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
                description: "Open a named local desktop application.",
                risk: ToolRisk::Medium,
                required_arguments: &[RequiredArgument {
                    name: "app_name",
                    kind: ArgumentKind::NonEmptyString,
                }],
            },
            execute_open_app,
            open_app_confirmation_prompt,
        ));
        registry.register(ToolDefinition::new(
            ToolSpec {
                name: "spotify.play_artist",
                description: "Play music by an artist in Spotify; use this for artist-level requests.",
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
                description: "Play a specific track in Spotify; use this only when a track is named.",
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

    pub fn planner_catalog(&self) -> String {
        let tools = self
            .specs()
            .map(|spec| {
                let arguments = spec
                    .required_arguments
                    .iter()
                    .map(|argument| {
                        (
                            argument.name.to_string(),
                            Value::String(argument.kind.planner_description().to_string()),
                        )
                    })
                    .collect::<Map<String, Value>>();
                json!({
                    "name": spec.name,
                    "description": spec.description,
                    "risk": spec.risk.to_string(),
                    "arguments": arguments
                })
            })
            .collect::<Vec<_>>();

        serde_json::to_string_pretty(&json!({ "available_tools": tools }))
            .expect("tool catalog contains only serializable values")
    }

    pub fn invoke(&self, tool_name: &str, arguments: Value) -> ToolOutcome {
        let Some(definition) = self.definition(tool_name) else {
            return ToolOutcome::Completed(ToolResult::failed(
                tool_name,
                format!("未知工具：{tool_name}"),
                arguments,
            ));
        };

        if let Err(error) = validate_arguments(&definition.spec, &arguments) {
            return ToolOutcome::Completed(ToolResult::failed(tool_name, error, arguments));
        }

        if definition.spec.risk.requires_confirmation() {
            let prompt = definition
                .confirmation_prompt
                .map(|builder| builder(&arguments))
                .unwrap_or_else(|| format!("执行 {tool_name} 需要你确认。"));
            return ToolOutcome::NeedsConfirmation(ToolConfirmation {
                tool_name: tool_name.to_string(),
                risk: definition.spec.risk,
                prompt,
                data: arguments,
            });
        }

        ToolOutcome::Completed(execute_definition(definition, &arguments))
    }

    pub fn confirm(&self, tool_name: &str, arguments: Value) -> ToolResult {
        let Some(definition) = self.definition(tool_name) else {
            return ToolResult::failed(tool_name, format!("未知工具：{tool_name}"), arguments);
        };

        if let Err(error) = validate_arguments(&definition.spec, &arguments) {
            return ToolResult::failed(tool_name, error, arguments);
        }

        execute_definition(definition, &arguments)
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

fn execute_definition(definition: &ToolDefinition, arguments: &Value) -> ToolResult {
    match (definition.handler)(arguments) {
        Ok(mut result) => {
            result.tool_name = definition.spec.name.to_string();
            result
        }
        Err(error) => ToolResult::failed(definition.spec.name, error, arguments.clone()),
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

fn open_app_confirmation_prompt(arguments: &Value) -> String {
    let app_name = arguments
        .get("app_name")
        .and_then(Value::as_str)
        .expect("app_name should be validated before prompt rendering")
        .trim();
    format!("打开 {app_name} 是一个本地启动动作，需要你确认。")
}

fn execute_open_app(arguments: &Value) -> Result<ToolResult, String> {
    let app_name = arguments
        .get("app_name")
        .and_then(Value::as_str)
        .expect("app_name should be validated before handler runs")
        .trim();

    open_application(app_name)?;

    Ok(ToolResult::succeeded(
        "local_launch.open_app",
        format!("已打开 {app_name}。"),
        json!({
            "app_name": app_name,
            "action": "open_app"
        }),
    ))
}

#[cfg(target_os = "macos")]
fn open_application(app_name: &str) -> Result<(), String> {
    let mut command = Command::new("open");
    command.args(["-a", app_name]);
    let output = run_command_with_timeout(&mut command, COMMAND_TIMEOUT, "macOS open")?;

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

fn play_spotify_artist(arguments: &Value) -> Result<ToolResult, String> {
    run_spotify_helper("play_artist", required_query(arguments)?)
}

fn play_spotify_track(arguments: &Value) -> Result<ToolResult, String> {
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

fn run_spotify_helper(action: &str, query: &str) -> Result<ToolResult, String> {
    let mut command = Command::new("python3");
    command
        .args([
            "-m",
            "aurorapulse.integrations.music.spotify_tool",
            action,
            query,
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"));
    let output = run_command_with_timeout(&mut command, COMMAND_TIMEOUT, "Spotify helper")?;

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

    Ok(ToolResult::succeeded(
        format!("spotify.{action}"),
        summary,
        json!({
            "action": action,
            "query": query
        }),
    ))
}

fn run_command_with_timeout(
    command: &mut Command,
    timeout: Duration,
    label: &str,
) -> Result<Output, String> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("无法启动 {label}：{error}"))?;
    let started = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .map_err(|error| format!("无法读取 {label} 的执行结果：{error}"));
            }
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("{label} 执行超时（{} 秒）", timeout.as_secs()));
            }
            Ok(None) => thread::sleep(COMMAND_POLL_INTERVAL),
            Err(error) => return Err(format!("无法等待 {label}：{error}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{
        ToolDefinition, ToolOutcome, ToolRegistry, ToolResult, ToolRisk, ToolSpec, ToolStatus,
        run_command_with_timeout,
    };

    fn custom_tool(_arguments: &Value) -> Result<ToolResult, String> {
        Ok(ToolResult::succeeded("test.custom", "done", json!({})))
    }

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
    fn planner_catalog_is_generated_from_registered_specs() {
        let mut registry = ToolRegistry::new();
        registry.register(ToolDefinition::new(
            ToolSpec {
                name: "test.custom",
                description: "A dynamically registered test capability.",
                risk: ToolRisk::Low,
                required_arguments: &[],
            },
            custom_tool,
        ));
        let catalog = registry.planner_catalog();

        assert!(catalog.contains("test.custom"));
        assert!(catalog.contains("dynamically registered"));
        assert!(catalog.contains("low"));
    }

    #[test]
    fn local_launch_open_app_requires_app_name() {
        let registry = ToolRegistry::with_builtin_tools();
        let ToolOutcome::Completed(result) =
            registry.invoke("local_launch.open_app", serde_json::json!({}))
        else {
            panic!("invalid arguments should be normalized as a completed failure");
        };

        assert_eq!(result.status, ToolStatus::Failed);
        assert!(
            result
                .error
                .expect("failure should include error")
                .contains("app_name")
        );
    }

    #[test]
    fn medium_risk_tool_is_confirmed_by_registry_policy() {
        let registry = ToolRegistry::with_builtin_tools();
        let outcome = registry.invoke(
            "local_launch.open_app",
            serde_json::json!({ "app_name": "Spotify" }),
        );

        match outcome {
            ToolOutcome::NeedsConfirmation(confirmation) => {
                assert_eq!(confirmation.tool_name, "local_launch.open_app");
                assert_eq!(confirmation.risk, ToolRisk::Medium);
                assert!(confirmation.prompt.contains("Spotify"));
                assert_eq!(confirmation.data["app_name"], "Spotify");
            }
            ToolOutcome::Completed(_) => panic!("medium-risk tools must require confirmation"),
        }
    }

    #[test]
    fn unknown_tool_is_a_structured_failure() {
        let registry = ToolRegistry::with_builtin_tools();
        let ToolOutcome::Completed(result) = registry.invoke(
            "calendar.create_event",
            serde_json::json!({ "title": "Demo" }),
        ) else {
            panic!("unknown tool should fail without confirmation");
        };

        assert_eq!(result.status, ToolStatus::Failed);
        assert!(
            result
                .error
                .expect("failure should include error")
                .contains("未知工具")
        );
    }

    #[test]
    fn external_commands_are_stopped_at_the_execution_boundary() {
        let mut command = std::process::Command::new("sleep");
        command.arg("1");

        let error = run_command_with_timeout(
            &mut command,
            std::time::Duration::from_millis(30),
            "test command",
        )
        .expect_err("long-running command should time out");

        assert!(error.contains("超时"));
    }
}
