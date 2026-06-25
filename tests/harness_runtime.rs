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
fn tool_decision_stops_at_unimplemented_execution_layer() {
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
        TurnOutcome::Reply("助手> 工具 local_launch.open_app 还没有接入执行层。".to_string())
    );
}
