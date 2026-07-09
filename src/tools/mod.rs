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

#[derive(Debug, PartialEq, Eq)]
pub struct ToolConfirmation {
    pub tool_name: String,
    pub risk: ToolRisk,
    pub prompt: String,
    pub data: Value,
}

type ToolHandler = fn(&Value) -> ToolOutcome;

pub struct ToolDefinition {
    spec: ToolSpec,
    handler: ToolHandler,
}

impl ToolDefinition {
    pub fn new(spec: ToolSpec, handler: ToolHandler) -> Self {
        Self { spec, handler }
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
        registry.register(ToolDefinition::new(
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
            .tools
            .iter()
            .find(|definition| definition.spec.name == tool_name)
            .ok_or_else(|| format!("未知工具：{tool_name}"))?;

        validate_arguments(&definition.spec, &arguments)?;
        Ok((definition.handler)(&arguments))
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

fn request_open_app_confirmation(arguments: &Value) -> ToolOutcome {
    let app_name = arguments
        .get("app_name")
        .and_then(Value::as_str)
        .expect("app_name should be validated before handler runs")
        .trim();

    ToolOutcome::NeedsConfirmation(ToolConfirmation {
        tool_name: "local_launch.open_app".to_string(),
        risk: ToolRisk::Medium,
        prompt: format!("打开 {app_name} 是一个本地启动动作，需要你确认。"),
        data: json!({
            "app_name": app_name,
            "action": "open_app"
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::{ToolOutcome, ToolRegistry, ToolRisk};

    #[test]
    fn built_in_registry_exposes_local_launch_tool() {
        let registry = ToolRegistry::with_builtin_tools();

        let names = registry.specs().map(|spec| spec.name).collect::<Vec<_>>();

        assert_eq!(names, vec!["local_launch.open_app"]);
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
    fn unknown_tool_is_rejected() {
        let registry = ToolRegistry::with_builtin_tools();

        let err = registry
            .invoke("spotify.play_track", serde_json::json!({ "query": "晴天" }))
            .expect_err("unknown tool should fail validation");

        assert!(err.contains("未知工具"), "unexpected error: {err}");
    }
}
