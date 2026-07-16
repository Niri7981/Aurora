use serde_json::Value;

pub const SYSTEM_PROMPT_BASE: &str = r#"你是 AuroraPulse 的 planner。你必须只输出 JSON，不要输出 Markdown、解释或额外文本。

根据用户当前请求和最近会话，选择一个 mode：
- chat：可以直接短回复用户
- clarify：信息不足，需要先问一个短澄清问题
- tool：需要调用 AuroraPulse 工具；harness 会校验参数、风险和权限
- retrieve：需要检索本地知识；目前只做决策，不执行

当前用户请求可能包含 AuroraPulse 注入的 Identity Card、Current Focus、Preferences 和 Project Context。已有上下文足够时直接选择 chat，只有信息不足时才选择 retrieve 或 clarify。

输出格式必须是以下之一：
{"mode":"chat","reply":"..."}
{"mode":"clarify","clarify_question":"..."}
{"mode":"tool","tool_name":"...","arguments":{}}
{"mode":"retrieve","retrieve_query":"..."}

要求：
- 只能使用下方工具目录中存在的工具名和参数
- 字段值必须非空，arguments 必须是 object
- 回复和问题要简短自然
- 不要声称工具已经成功；真实结果以 harness 返回的 ToolResult 为准"#;

pub fn build_system_prompt(tool_catalog: &str) -> String {
    format!("{SYSTEM_PROMPT_BASE}\n\n当前工具目录（由 ToolRegistry 生成）：\n{tool_catalog}")
}

#[derive(Debug, PartialEq, Eq)]
pub enum PlannerDecision {
    Chat { reply: String },
    Tool { tool_name: String, arguments: Value },
    Retrieve { retrieve_query: String },
    Clarify { clarify_question: String },
}

impl PlannerDecision {
    pub fn parse(raw_json: &str) -> Result<Self, String> {
        let value = match parse_planner_value(raw_json)? {
            Some(value) => value,
            None => {
                return Ok(Self::Chat {
                    reply: raw_json.trim().to_string(),
                });
            }
        };
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

fn parse_planner_value(raw_json: &str) -> Result<Option<Value>, String> {
    let trimmed = raw_json.trim();
    if trimmed.is_empty() {
        return Err("planner 输出为空".to_string());
    }

    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        return Ok(Some(value));
    }

    if let Some(candidate) = extract_json_object(trimmed) {
        let value = serde_json::from_str::<Value>(&candidate)
            .map_err(|err| format!("planner JSON 无法解析：{err}"))?;
        return Ok(Some(value));
    }

    Ok(None)
}

fn extract_json_object(text: &str) -> Option<String> {
    for (start, _) in text.match_indices('{') {
        let candidate = &text[start..];
        if let Some(end) = find_json_object_end(candidate) {
            return Some(candidate[..=end].to_string());
        }
    }

    None
}

fn find_json_object_end(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in text.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }

    None
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
