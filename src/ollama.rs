use std::io::{self, Write};
use std::net::TcpStream;

use serde_json::{Value, json};

use crate::session::ChatMessage;

const SYSTEM_PROMPT: &str = "你是 AuroraPulse，本地终端助手。直接回答用户当前问题，简洁、自然、少套话。你必须结合当前会话上下文理解代词、省略和追问，不要把每条输入都当成全新会话。不要每次都重复自我介绍，除非用户明确要求你介绍自己。";

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
