use aurora::planner::PlannerDecision;

#[test]
fn parses_chat_decision_from_model_json() {
    let decision = PlannerDecision::parse(
        r#"{
            "mode": "chat",
            "reply": "我是 AuroraPulse。"
        }"#,
    )
    .expect("planner decision should parse");

    assert_eq!(
        decision,
        PlannerDecision::Chat {
            reply: "我是 AuroraPulse。".to_string()
        }
    );
}

#[test]
fn parses_tool_decision_with_arguments_from_model_json() {
    let decision = PlannerDecision::parse(
        r#"{
            "mode": "tool",
            "tool_name": "local_launch.open_app",
            "arguments": {
                "app_name": "Spotify"
            }
        }"#,
    )
    .expect("planner decision should parse");

    assert_eq!(
        decision,
        PlannerDecision::Tool {
            tool_name: "local_launch.open_app".to_string(),
            arguments: serde_json::json!({
                "app_name": "Spotify"
            })
        }
    );
}

#[test]
fn parses_clarify_decision_from_model_json() {
    let decision = PlannerDecision::parse(
        r#"{
            "mode": "clarify",
            "clarify_question": "你想打开哪个应用？"
        }"#,
    )
    .expect("planner decision should parse");

    assert_eq!(
        decision,
        PlannerDecision::Clarify {
            clarify_question: "你想打开哪个应用？".to_string()
        }
    );
}

#[test]
fn parses_retrieve_decision_from_model_json() {
    let decision = PlannerDecision::parse(
        r#"{
            "mode": "retrieve",
            "retrieve_query": "昨天项目日志里的 Day 2 计划"
        }"#,
    )
    .expect("planner decision should parse");

    assert_eq!(
        decision,
        PlannerDecision::Retrieve {
            retrieve_query: "昨天项目日志里的 Day 2 计划".to_string()
        }
    );
}

#[test]
fn rejects_blank_required_text_fields() {
    let err = PlannerDecision::parse(
        r#"{
            "mode": "chat",
            "reply": "   "
        }"#,
    )
    .expect_err("blank reply should be rejected");

    assert!(err.contains("reply"), "unexpected error: {err}");
}

#[test]
fn parses_json_embedded_in_model_chatter() {
    let decision = PlannerDecision::parse(
        r#"好的，我会用 JSON：
```json
{"mode":"chat","reply":"你是 AuroraPulse 的构建者。"}
```
"#,
    )
    .expect("embedded planner decision should parse");

    assert_eq!(
        decision,
        PlannerDecision::Chat {
            reply: "你是 AuroraPulse 的构建者。".to_string()
        }
    );
}

#[test]
fn treats_plain_model_text_as_chat_reply() {
    let decision = PlannerDecision::parse("你是 AuroraPulse 的 owner/builder，最近在做 Phase 1。")
        .expect("plain text should become a chat reply");

    assert_eq!(
        decision,
        PlannerDecision::Chat {
            reply: "你是 AuroraPulse 的 owner/builder，最近在做 Phase 1。".to_string()
        }
    );
}
