use aurora::app::TurnOutcome;
use aurora::harness::{ConfirmationDecision, Harness};
use aurora::planner::PlannerDecision;
use aurora::tools::{
    ArgumentKind, RequiredArgument, ToolDefinition, ToolRegistry, ToolResult, ToolRisk, ToolSpec,
    ToolStatus,
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
        execute_fake_confirmation,
        fake_confirmation_prompt,
    ));
    registry
}

fn fake_high_risk_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(ToolDefinition::with_confirmation(
        ToolSpec {
            name: "test.high_risk_action",
            description: "Test high-risk action.",
            risk: ToolRisk::High,
            required_arguments: &FAKE_REQUIRED_ARGUMENTS,
        },
        execute_fake_confirmation,
        fake_confirmation_prompt,
    ));
    registry
}

fn fake_confirmation_prompt(arguments: &Value) -> String {
    let target = arguments
        .get("target")
        .and_then(Value::as_str)
        .expect("target should be validated before handler runs");

    format!("执行 {target} 需要确认。")
}

fn execute_fake_confirmation(arguments: &Value) -> Result<ToolResult, String> {
    let target = arguments
        .get("target")
        .and_then(Value::as_str)
        .expect("target should be validated before handler runs");

    Ok(ToolResult::succeeded(
        "test.confirmed_action",
        format!("已执行 {target}。"),
        json!({ "target": target }),
    ))
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
        TurnOutcome::Confirmation {
            tool_name: "local_launch.open_app".to_string(),
            prompt: "打开 Spotify 是一个本地启动动作，需要你确认。".to_string(),
            allow_always: true,
        }
    );
}

#[test]
fn allow_once_executes_pending_tool() {
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
        TurnOutcome::Confirmation {
            tool_name: "test.confirmed_action".to_string(),
            prompt: "执行 测试动作 需要确认。".to_string(),
            allow_always: true,
        }
    );

    let confirmed = harness
        .resolve_confirmation(ConfirmationDecision::AllowOnce)
        .expect("confirmation should succeed");

    assert_eq!(
        confirmed,
        TurnOutcome::Reply("助手> 已执行 测试动作。".to_string())
    );
    assert_eq!(harness.tool_logs().len(), 1);
    assert_eq!(harness.tool_logs()[0].result.status, ToolStatus::Succeeded);
}

#[test]
fn deny_cancels_pending_tool() {
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
        .resolve_confirmation(ConfirmationDecision::Deny)
        .expect("cancellation should succeed");

    assert_eq!(cancelled, TurnOutcome::Reply("助手> 已取消。".to_string()));
    assert_eq!(harness.tool_logs()[0].result.status, ToolStatus::Denied);
    assert!(
        harness
            .resolve_confirmation(ConfirmationDecision::AllowOnce)
            .is_err()
    );
}

#[test]
fn always_allow_skips_future_confirmation_for_the_same_tool() {
    let mut harness = Harness::with_tool_registry(fake_confirmation_registry());

    let first = harness
        .handle_decision(
            "执行第一次动作",
            PlannerDecision::Tool {
                tool_name: "test.confirmed_action".to_string(),
                arguments: json!({ "target": "第一次动作" }),
            },
        )
        .expect("decision should be handled");
    assert!(matches!(first, TurnOutcome::Confirmation { .. }));

    let allowed = harness
        .resolve_confirmation(ConfirmationDecision::AlwaysAllow)
        .expect("always allow should execute the pending action");
    assert_eq!(
        allowed,
        TurnOutcome::Reply("助手> 已执行 第一次动作。".to_string())
    );

    let second = harness
        .handle_decision(
            "执行第二次动作",
            PlannerDecision::Tool {
                tool_name: "test.confirmed_action".to_string(),
                arguments: json!({ "target": "第二次动作" }),
            },
        )
        .expect("always-allowed tool should execute immediately");
    assert_eq!(
        second,
        TurnOutcome::Reply("助手> 已执行 第二次动作。".to_string())
    );
}

#[test]
fn high_risk_tool_never_receives_a_session_bypass() {
    let mut harness = Harness::with_tool_registry(fake_high_risk_registry());

    let first = harness
        .handle_decision(
            "执行高风险动作",
            PlannerDecision::Tool {
                tool_name: "test.high_risk_action".to_string(),
                arguments: json!({ "target": "第一次" }),
            },
        )
        .expect("high-risk decision should be handled");
    assert_eq!(
        first,
        TurnOutcome::Confirmation {
            tool_name: "test.high_risk_action".to_string(),
            prompt: "执行 第一次 需要确认。".to_string(),
            allow_always: false,
        }
    );

    harness
        .resolve_confirmation(ConfirmationDecision::AlwaysAllow)
        .expect("the current high-risk action may still be approved once");

    let second = harness
        .handle_decision(
            "再次执行高风险动作",
            PlannerDecision::Tool {
                tool_name: "test.high_risk_action".to_string(),
                arguments: json!({ "target": "第二次" }),
            },
        )
        .expect("second high-risk decision should be handled");
    assert!(matches!(
        second,
        TurnOutcome::Confirmation {
            allow_always: false,
            ..
        }
    ));
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
    assert_eq!(harness.tool_logs()[0].result.status, ToolStatus::Failed);
    assert!(harness.tool_logs()[0].result.error.is_some());
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
