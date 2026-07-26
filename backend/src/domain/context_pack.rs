use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
pub struct ContextPack {
    pub purpose: String,
    pub query: Option<String>,
    pub client: String,
    pub access: String,
    pub items: Vec<ContextItem>,
    pub omissions: Vec<ContextOmission>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
pub struct ContextItem {
    pub category: String,
    pub label: String,
    pub source: String,
    pub content: String,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
pub struct ContextOmission {
    pub source: String,
    pub reason: String,
    pub line_count: usize,
}
