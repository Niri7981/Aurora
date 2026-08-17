use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;

use crate::domain::context_pack::ContextPack;

#[derive(Clone)]
pub struct AuditLog {
    path: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl AuditLog {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append_success(
        &self,
        client: &str,
        tool: &str,
        pack: &ContextPack,
    ) -> Result<(), String> {
        self.append(AuditEvent::success(client, tool, pack))
    }

    pub fn append_failure(
        &self,
        client: &str,
        tool: &str,
        purpose: &str,
        query: Option<&str>,
        error: &str,
    ) -> Result<(), String> {
        self.append(AuditEvent::failure(client, tool, purpose, query, error))
    }

    pub fn append_search_success<'a>(
        &self,
        client: &'a str,
        tool: &'a str,
        purpose: &'a str,
        query: &'a str,
        returned_sources: &[&'a str],
        omitted_lines: usize,
    ) -> Result<(), String> {
        self.append(AuditEvent::search_success(
            client,
            tool,
            purpose,
            query,
            returned_sources,
            omitted_lines,
        ))
    }

    pub fn render_recent(&self, limit: usize) -> Result<String, String> {
        if !self.path.exists() {
            return Ok("暂无 MCP 上下文调用记录。".to_string());
        }
        let content = fs::read_to_string(&self.path)
            .map_err(|error| format!("failed to read MCP audit log: {error}"))?;
        let mut rendered = content
            .lines()
            .rev()
            .take(limit)
            .map(render_audit_line)
            .collect::<Result<Vec<_>, _>>()?;
        if rendered.is_empty() {
            return Ok("暂无 MCP 上下文调用记录。".to_string());
        }
        rendered.reverse();
        Ok(rendered.join("\n"))
    }

    fn append(&self, event: AuditEvent<'_>) -> Result<(), String> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| "MCP audit lock is poisoned".to_string())?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create MCP audit directory: {error}"))?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| format!("failed to open MCP audit log: {error}"))?;
        serde_json::to_writer(&mut file, &event)
            .map_err(|error| format!("failed to serialize MCP audit event: {error}"))?;
        file.write_all(b"\n")
            .map_err(|error| format!("failed to write MCP audit log: {error}"))
    }
}

#[derive(Serialize)]
struct AuditEvent<'a> {
    timestamp_unix_ms: u64,
    client: &'a str,
    tool: &'a str,
    purpose: &'a str,
    query: Option<&'a str>,
    status: &'a str,
    returned_sources: Vec<&'a str>,
    omitted_lines: usize,
    error: Option<&'a str>,
}

impl<'a> AuditEvent<'a> {
    fn success(client: &'a str, tool: &'a str, pack: &'a ContextPack) -> Self {
        Self {
            timestamp_unix_ms: timestamp_unix_ms(),
            client,
            tool,
            purpose: &pack.purpose,
            query: pack.query.as_deref(),
            status: "succeeded",
            returned_sources: pack.items.iter().map(|item| item.source.as_str()).collect(),
            omitted_lines: pack
                .omissions
                .iter()
                .map(|omission| omission.line_count)
                .sum(),
            error: None,
        }
    }

    fn search_success(
        client: &'a str,
        tool: &'a str,
        purpose: &'a str,
        query: &'a str,
        returned_sources: &[&'a str],
        omitted_lines: usize,
    ) -> Self {
        Self {
            timestamp_unix_ms: timestamp_unix_ms(),
            client,
            tool,
            purpose,
            query: Some(query),
            status: "succeeded",
            returned_sources: returned_sources.to_vec(),
            omitted_lines,
            error: None,
        }
    }

    fn failure(
        client: &'a str,
        tool: &'a str,
        purpose: &'a str,
        query: Option<&'a str>,
        error: &'a str,
    ) -> Self {
        Self {
            timestamp_unix_ms: timestamp_unix_ms(),
            client,
            tool,
            purpose,
            query,
            status: "failed",
            returned_sources: Vec::new(),
            omitted_lines: 0,
            error: Some(error),
        }
    }
}

fn render_audit_line(line: &str) -> Result<String, String> {
    let event: Value = serde_json::from_str(line)
        .map_err(|error| format!("failed to parse MCP audit event: {error}"))?;
    let timestamp = event
        .get("timestamp_unix_ms")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let client = event
        .get("client")
        .and_then(Value::as_str)
        .unwrap_or("unknown-agent");
    let tool = event
        .get("tool")
        .and_then(Value::as_str)
        .unwrap_or("unknown-tool");
    let status = event
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let purpose = event
        .get("purpose")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let (purpose, _) = truncate_chars(purpose, 80);
    let sources = event
        .get("returned_sources")
        .and_then(Value::as_array)
        .map(|sources| {
            sources
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|sources| !sources.is_empty())
        .unwrap_or_else(|| "none".to_string());
    let omitted = event
        .get("omitted_lines")
        .and_then(Value::as_u64)
        .unwrap_or_default();

    Ok(format!(
        "{timestamp}  {client:<12}  {tool:<25}  {status:<9}  sources={sources}  omitted={omitted}  purpose={purpose}"
    ))
}

fn truncate_chars(content: &str, max_chars: usize) -> (String, bool) {
    let mut characters = content.chars();
    let truncated = characters.by_ref().take(max_chars).collect::<String>();
    let was_truncated = characters.next().is_some();
    (truncated, was_truncated)
}

fn timestamp_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
