use aurora::app::TurnOutcome;
use aurora::harness::Harness;
use aurora::planner::PlannerDecision;

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
            "播放音乐",
            PlannerDecision::Tool {
                tool_name: "spotify.play_track".to_string(),
                arguments: serde_json::json!({ "query": "晴天" }),
            },
        )
        .expect("decision should be handled");

    assert_eq!(
        outcome,
        TurnOutcome::Reply(
            "助手> 我没法执行这个工具请求：未知工具：spotify.play_track".to_string()
        )
    );
}
