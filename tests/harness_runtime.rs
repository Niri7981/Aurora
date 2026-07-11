use aurora::app::TurnOutcome;
use aurora::harness::Harness;
use aurora::planner::PlannerDecision;
use aurora::tools::{
    ArgumentKind, RequiredArgument, ToolConfirmation, ToolDefinition, ToolOutcome, ToolRegistry,
    ToolResult, ToolRisk, ToolSpec,
};
use serde_json::{Value, json};

const FAKE_REQUIRED_ARGUMENTS: [RequiredArgument; 1] = [RequiredArgument {
    name: "target",
    kind: ArgumentKind::NonEmptyString,
}];

fn fake_confirmation_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(ToolDefinition::with_confirmation(
        ToolSpec {
            name: "test.confirmed_action",
            description: "Test confirmation action.",
            risk: ToolRisk::Medium,
            required_arguments: &FAKE_REQUIRED_ARGUMENTS,
        },
        request_fake_confirmation,
        execute_fake_confirmation,
    ));
    registry
}

fn request_fake_confirmation(arguments: &Value) -> Result<ToolOutcome, String> {
    let target = arguments
        .get("target")
        .and_then(Value::as_str)
        .expect("target should be validated before handler runs");

    Ok(ToolOutcome::NeedsConfirmation(ToolConfirmation {
        tool_name: "test.confirmed_action".to_string(),
        risk: ToolRisk::Medium,
        prompt: format!("执行 {target} 需要确认。"),
        data: json!({ "target": target }),
    }))
}

fn execute_fake_confirmation(arguments: &Value) -> Result<ToolOutcome, String> {
    let target = arguments
        .get("target")
        .and_then(Value::as_str)
        .expect("target should be validated before handler runs");

    Ok(ToolOutcome::Completed(ToolResult {
        tool_name: "test.confirmed_action".to_string(),
        summary: format!("已执行 {target}。"),
        data: json!({ "target": target }),
    }))
}

#[test]
fn chat_decision_becomes_user_facing_reply() {
    let mut harness = Harness::new();

    let outcome = harness
        .handle_decision(
            "你是谁？",
            PlannerDecision::Chat {
                reply: "我是 AuroraPulse。".to_string(),
            },
        )
        .expect("decision should be handled");

    assert_eq!(
        outcome,
        TurnOutcome::Reply("助手> 我是 AuroraPulse。".to_string())
    );
}

#[test]
fn clarify_decision_becomes_user_facing_question() {
    let mut harness = Harness::new();

    let outcome = harness
        .handle_decision(
            "打开那个应用",
            PlannerDecision::Clarify {
                clarify_question: "你想打开哪个应用？".to_string(),
            },
        )
        .expect("decision should be handled");

    assert_eq!(
        outcome,
        TurnOutcome::Reply("助手> 你想打开哪个应用？".to_string())
    );
}

#[test]
fn tool_decision_is_routed_through_tool_registry() {
    let mut harness = Harness::new();

    let outcome = harness
        .handle_decision(
            "打开 Spotify",
            PlannerDecision::Tool {
                tool_name: "local_launch.open_app".to_string(),
                arguments: serde_json::json!({
                    "app_name": "Spotify"
                }),
            },
        )
        .expect("decision should be handled");

    assert_eq!(
        outcome,
        TurnOutcome::Reply(
            "助手> 打开 Spotify 是一个本地启动动作，需要你确认。 回复“确认”后我再执行。"
                .to_string()
        )
    );
}

#[test]
fn confirmation_reply_executes_pending_tool_without_model() {
    let mut harness = Harness::with_tool_registry(fake_confirmation_registry());

    let first = harness
        .handle_decision(
            "执行测试动作",
            PlannerDecision::Tool {
                tool_name: "test.confirmed_action".to_string(),
                arguments: json!({ "target": "测试动作" }),
            },
        )
        .expect("decision should be handled");

    assert_eq!(
        first,
        TurnOutcome::Reply("助手> 执行 测试动作 需要确认。 回复“确认”后我再执行。".to_string())
    );

    let confirmed = harness
        .handle_pending_input("确认")
        .expect("pending action should be handled")
        .expect("confirmation should succeed");

    assert_eq!(
        confirmed,
        TurnOutcome::Reply("助手> 已执行 测试动作。".to_string())
    );
}

#[test]
fn cancellation_reply_clears_pending_tool() {
    let mut harness = Harness::with_tool_registry(fake_confirmation_registry());

    harness
        .handle_decision(
            "执行测试动作",
            PlannerDecision::Tool {
                tool_name: "test.confirmed_action".to_string(),
                arguments: json!({ "target": "测试动作" }),
            },
        )
        .expect("decision should be handled");

    let cancelled = harness
        .handle_pending_input("取消")
        .expect("pending action should be handled")
        .expect("cancellation should succeed");

    assert_eq!(cancelled, TurnOutcome::Reply("助手> 已取消。".to_string()));
    assert!(harness.handle_pending_input("确认").is_none());
}

#[test]
fn invalid_tool_arguments_become_bounded_failure_reply() {
    let mut harness = Harness::new();

    let outcome = harness
        .handle_decision(
            "打开那个应用",
            PlannerDecision::Tool {
                tool_name: "local_launch.open_app".to_string(),
                arguments: serde_json::json!({}),
            },
        )
        .expect("decision should be handled");

    assert_eq!(
        outcome,
        TurnOutcome::Reply(
            "助手> 我没法执行这个工具请求：local_launch.open_app 缺少参数：app_name".to_string()
        )
    );
}

#[test]
fn unknown_tool_becomes_bounded_failure_reply() {
    let mut harness = Harness::new();

    let outcome = harness
        .handle_decision(
            "帮我安排会议",
            PlannerDecision::Tool {
                tool_name: "calendar.create_event".to_string(),
                arguments: serde_json::json!({ "title": "Demo" }),
            },
        )
        .expect("decision should be handled");

    assert_eq!(
        outcome,
        TurnOutcome::Reply(
            "助手> 我没法执行这个工具请求：未知工具：calendar.create_event".to_string()
        )
    );
}

#[test]
fn spotify_tool_requires_query_before_execution() {
    let mut harness = Harness::new();

    let outcome = harness
        .handle_decision(
            "我想在 Spotify 听歌",
            PlannerDecision::Tool {
                tool_name: "spotify.play_artist".to_string(),
                arguments: serde_json::json!({}),
            },
        )
        .expect("decision should be handled");

    assert_eq!(
        outcome,
        TurnOutcome::Reply(
            "助手> 我没法执行这个工具请求：spotify.play_artist 缺少参数：query".to_string()
        )
    );
}
