use std::process::Command;

use serde_json::{Value, json};

use crate::config::AppConfig;
use crate::session::ChatMessage;

use super::ChatClient;
use super::ollama::SYSTEM_PROMPT;

pub(super) struct OpenAiProvider;

impl ChatClient for OpenAiProvider {
    fn provider_name<'a>(&self, _config: &'a AppConfig) -> &'a str {
        "openai"
    }

    fn chat(
        &mut self,
        config: &AppConfig,
        history: &[ChatMessage],
        user_text: &str,
    ) -> Result<String, String> {
        let api_key = config.openai_api_key.as_deref().ok_or_else(|| {
            "OPENAI_API_KEY is not set; add it to .env before using AURORA_PROVIDER=openai"
                .to_string()
        })?;

        let endpoint = build_chat_completions_endpoint(&config.openai_base_url);
        let mut messages = vec![json!({
            "role": "system",
            "content": SYSTEM_PROMPT
        })];

        for message in history {
            messages.push(json!({
                "role": message.role,
                "content": message.content
            }));
        }

        messages.push(json!({
            "role": "user",
            "content": user_text
        }));

        let payload = json!({
            "model": config.openai_model,
            "stream": false,
            "messages": messages,
            "temperature": 0.2
        })
        .to_string();

        let auth_header = format!("Authorization: Bearer {api_key}");
        let output = Command::new("curl")
            .args([
                "-sS",
                "-X",
                "POST",
                endpoint.as_str(),
                "-H",
                "Content-Type: application/json",
                "-H",
                auth_header.as_str(),
                "-d",
                payload.as_str(),
            ])
            .output()
            .map_err(|err| format!("failed to run curl for OpenAI-compatible request: {err}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "OpenAI-compatible request failed with status {}: {}",
                output.status,
                stderr.trim()
            ));
        }

        let body = String::from_utf8(output.stdout)
            .map_err(|err| format!("OpenAI-compatible response was not UTF-8: {err}"))?;
        parse_chat_completions_response(&body)
    }
}

fn build_chat_completions_endpoint(base_url: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        format!("{trimmed}/chat/completions")
    } else {
        format!("{trimmed}/v1/chat/completions")
    }
}

fn parse_chat_completions_response(body: &str) -> Result<String, String> {
    let parsed: Value = serde_json::from_str(body).map_err(|err| {
        format!(
            "failed to parse OpenAI-compatible JSON response: {err}; body: {}",
            body.trim()
        )
    })?;

    if let Some(error_message) = parsed
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
    {
        return Err(format!(
            "OpenAI-compatible API returned an error: {error_message}"
        ));
    }

    let content = parsed
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            "OpenAI-compatible response missing choices[0].message.content".to_string()
        })?;

    Ok(content.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::{build_chat_completions_endpoint, parse_chat_completions_response};

    #[test]
    fn builds_chat_completions_endpoint_from_base_url() {
        assert_eq!(
            build_chat_completions_endpoint("https://example.com"),
            "https://example.com/v1/chat/completions"
        );
        assert_eq!(
            build_chat_completions_endpoint("https://example.com/v1"),
            "https://example.com/v1/chat/completions"
        );
    }

    #[test]
    fn parses_chat_completions_content() {
        let content = parse_chat_completions_response(
            r#"{
                "choices": [
                    {
                        "message": {
                            "content": "{\"mode\":\"chat\",\"reply\":\"ok\"}"
                        }
                    }
                ]
            }"#,
        )
        .expect("response should parse");

        assert_eq!(content, r#"{"mode":"chat","reply":"ok"}"#);
    }

    #[test]
    fn surfaces_api_error_message() {
        let err = parse_chat_completions_response(
            r#"{
                "error": {
                    "message": "bad api key"
                }
            }"#,
        )
        .expect_err("error response should fail");

        assert!(err.contains("bad api key"));
    }
}
