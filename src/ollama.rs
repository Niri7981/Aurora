use std::io::{self, Write};
use std::net::TcpStream;

use serde_json::{Value, json};

use crate::session::ChatMessage;

pub const SYSTEM_PROMPT: &str = r#"你是 AuroraPulse 的 planner。你必须只输出 JSON，不要输出 Markdown、解释或额外文本。

根据用户当前请求和最近会话，选择一个 mode：
- chat：可以直接短回复用户
- clarify：信息不足，需要先问一个短澄清问题
- tool：需要调用工具；目前只做决策，不执行
- retrieve：需要检索本地知识；目前只做决策，不执行

当前用户请求里可能已经包含 AuroraPulse 注入的本地上下文，例如 Identity Card、Current Focus、Preferences、Project Context。
如果用户问“我是谁”“我最近在做什么”“这个项目是什么”，并且这些信息已经出现在当前请求的上下文里，必须选择 chat 并基于上下文简短回答。
只有当前请求提供的上下文不足以回答时，才选择 retrieve 或 clarify。

输出格式必须是以下之一：
{"mode":"chat","reply":"..."}
{"mode":"clarify","clarify_question":"..."}
{"mode":"tool","tool_name":"...","arguments":{}}
{"mode":"retrieve","retrieve_query":"..."}

要求：
- 字段值必须非空
- arguments 必须是 object
- 回复和问题都要简短自然
- 不要声称已经执行工具或检索
"#;

pub fn chat(
    ollama_url: &str,
    model: &str,
    history: &[ChatMessage],
    user_text: &str,
) -> Result<String, String> {
    let endpoint = build_chat_endpoint(ollama_url);
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
        "model": model,
        "stream": false,
        "messages": messages,
        "options": {
            "temperature": 0.2
        }
    })
    .to_string();

    let (host, port, path) = parse_http_endpoint(&endpoint)?;
    let mut stream = TcpStream::connect((host.as_str(), port))
        .map_err(|err| format!("无法连接到 Ollama 服务 {host}:{port}：{err}"))?;
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    );

    stream
        .write_all(request.as_bytes())
        .map_err(|err| format!("发送请求到 Ollama 失败：{err}"))?;

    let mut response = String::new();
    io::Read::read_to_string(&mut stream, &mut response)
        .map_err(|err| format!("读取 Ollama 响应失败：{err}"))?;

    let (head, raw_body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| "Ollama 响应格式不正确".to_string())?;
    let status_line = head
        .lines()
        .next()
        .ok_or_else(|| "Ollama 响应缺少状态行".to_string())?;
    if !status_line.contains(" 200 ") {
        return Err(format!("Ollama 返回错误：{status_line}"));
    }
    let body = if header_has_chunked_encoding(head) {
        decode_chunked_body(raw_body)?
    } else {
        raw_body.to_string()
    };

    let parsed: Value =
        serde_json::from_str(&body).map_err(|err| format!("解析 Ollama JSON 失败：{err}"))?;
    let content = parsed
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .ok_or_else(|| "Ollama 响应里没有 message.content".to_string())?;

    Ok(content.trim().to_string())
}

fn build_chat_endpoint(base_url: &str) -> String {
    format!("{}/api/chat", base_url.trim_end_matches('/'))
}

fn parse_http_endpoint(endpoint: &str) -> Result<(String, u16, String), String> {
    let without_scheme = endpoint
        .strip_prefix("http://")
        .ok_or_else(|| "目前只支持 http:// 开头的 OLLAMA_URL".to_string())?;
    let (host_port, path) = without_scheme
        .split_once('/')
        .ok_or_else(|| "OLLAMA_URL 缺少路径".to_string())?;
    let path = format!("/{}", path);

    let (host, port) = match host_port.split_once(':') {
        Some((host, port)) => {
            let port = port
                .parse::<u16>()
                .map_err(|err| format!("OLLAMA_URL 端口无效：{err}"))?;
            (host.to_string(), port)
        }
        None => (host_port.to_string(), 80),
    };

    Ok((host, port, path))
}

fn header_has_chunked_encoding(headers: &str) -> bool {
    headers.lines().any(|line| {
        let lower = line.to_ascii_lowercase();
        lower.starts_with("transfer-encoding:") && lower.contains("chunked")
    })
}

fn decode_chunked_body(body: &str) -> Result<String, String> {
    let mut rest = body;
    let mut decoded = String::new();

    loop {
        let (size_line, next) = rest
            .split_once("\r\n")
            .ok_or_else(|| "chunked 响应缺少 size 行".to_string())?;
        let size_hex = size_line.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_hex, 16)
            .map_err(|err| format!("chunk size 无法解析：{err}"))?;

        if size == 0 {
            break;
        }

        if next.len() < size + 2 {
            return Err("chunked 响应长度不完整".to_string());
        }

        decoded.push_str(&next[..size]);
        rest = &next[size + 2..];
    }

    Ok(decoded)
}
