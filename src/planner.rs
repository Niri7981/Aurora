use serde_json::Value;

#[derive(Debug, PartialEq, Eq)]
pub enum PlannerDecision {
    Chat { reply: String },
    Tool { tool_name: String, arguments: Value },
    Retrieve { retrieve_query: String },
    Clarify { clarify_question: String },
}

impl PlannerDecision {
    pub fn parse(raw_json: &str) -> Result<Self, String> {
        let value: Value = serde_json::from_str(raw_json)
            .map_err(|err| format!("planner JSON 无法解析：{err}"))?;
        let mode = value
            .get("mode")
            .and_then(Value::as_str)
            .ok_or_else(|| "planner decision 缺少 mode".to_string())?;

        match mode {
            "chat" => {
                let reply = required_text(&value, "reply", "chat decision 缺少 reply")?;
                Ok(Self::Chat {
                    reply: reply.to_string(),
                })
            }
            "tool" => {
                let tool_name = required_text(&value, "tool_name", "tool decision 缺少 tool_name")?;
                let arguments = value
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));

                if !arguments.is_object() {
                    return Err("tool decision 的 arguments 必须是 object".to_string());
                }

                Ok(Self::Tool {
                    tool_name: tool_name.to_string(),
                    arguments,
                })
            }
            "retrieve" => {
                let retrieve_query = required_text(
                    &value,
                    "retrieve_query",
                    "retrieve decision 缺少 retrieve_query",
                )?;
                Ok(Self::Retrieve {
                    retrieve_query: retrieve_query.to_string(),
                })
            }
            "clarify" => {
                let clarify_question = required_text(
                    &value,
                    "clarify_question",
                    "clarify decision 缺少 clarify_question",
                )?;
                Ok(Self::Clarify {
                    clarify_question: clarify_question.to_string(),
                })
            }
            other => Err(format!("未知 planner mode：{other}")),
        }
    }
}

fn required_text<'a>(
    value: &'a Value,
    field: &str,
    missing_message: &str,
) -> Result<&'a str, String> {
    let text = value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| missing_message.to_string())?;

    if text.trim().is_empty() {
        return Err(format!("planner decision 的 {field} 不能为空"));
    }

    Ok(text)
}
