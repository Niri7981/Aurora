use rmcp::{
    ErrorData as McpError, Json, ServerHandler, ServiceExt, handler::server::wrapper::Parameters,
    schemars::JsonSchema, tool, tool_handler, tool_router, transport::stdio,
};
use serde::Deserialize;

use crate::application::context_gateway::ContextGateway;
use crate::domain::context_pack::ContextPack;

#[derive(Debug, Deserialize, JsonSchema)]
struct PurposeRequest {
    /// Why the Agent needs this personal context for the current user request.
    purpose: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchRequest {
    /// The personal context to find. Keep this focused on the current task.
    query: String,
    /// Why the Agent needs this personal context for the current user request.
    purpose: String,
    /// Maximum number of context items to return. Values above 6 are clamped.
    max_items: Option<usize>,
}

#[derive(Clone)]
pub struct AuroraMcpServer {
    gateway: ContextGateway,
}

#[tool_router]
impl AuroraMcpServer {
    pub fn new(gateway: ContextGateway) -> Self {
        Self { gateway }
    }

    #[tool(
        description = "Return the user's filtered identity card for a stated task purpose. Read-only; never returns privacy rules or unmarked files.",
        annotations(
            title = "Get Aurora identity",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn get_identity(
        &self,
        Parameters(request): Parameters<PurposeRequest>,
    ) -> Result<Json<ContextPack>, McpError> {
        validate_mcp_text(&request.purpose, "purpose")?;
        self.gateway
            .get_identity(&request.purpose)
            .map(Json)
            .map_err(internal_mcp_error)
    }

    #[tool(
        description = "Return the user's filtered current focus for a stated task purpose. Use when the request depends on what the user is doing now.",
        annotations(
            title = "Get Aurora current focus",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn get_current_focus(
        &self,
        Parameters(request): Parameters<PurposeRequest>,
    ) -> Result<Json<ContextPack>, McpError> {
        validate_mcp_text(&request.purpose, "purpose")?;
        self.gateway
            .get_current_focus(&request.purpose)
            .map(Json)
            .map_err(internal_mcp_error)
    }

    #[tool(
        description = "Search the user's authorized local personal context for the current task. Returns a small, filtered Context Pack with source URIs and omission metadata.",
        annotations(
            title = "Search Aurora personal context",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn search_personal_context(
        &self,
        Parameters(request): Parameters<SearchRequest>,
    ) -> Result<Json<ContextPack>, McpError> {
        validate_mcp_text(&request.query, "query")?;
        validate_mcp_text(&request.purpose, "purpose")?;
        self.gateway
            .search_personal_context(&request.query, &request.purpose, request.max_items)
            .map(Json)
            .map_err(internal_mcp_error)
    }
}

#[tool_handler(
    name = "aurora",
    version = "0.1.0",
    instructions = "Aurora is the user's local identity and context authority. Call only when personal context materially helps the current request. State a narrow purpose, use the minimum returned context, respect omissions, and never claim access beyond the returned Context Pack."
)]
impl ServerHandler for AuroraMcpServer {}

pub async fn serve(gateway: ContextGateway) -> Result<(), String> {
    let service = AuroraMcpServer::new(gateway)
        .serve(stdio())
        .await
        .map_err(|error| format!("failed to serve Aurora MCP: {error}"))?;
    service
        .waiting()
        .await
        .map(|_| ())
        .map_err(|error| format!("Aurora MCP stopped with an error: {error}"))
}

fn validate_mcp_text(value: &str, field: &str) -> Result<(), McpError> {
    if value.trim().is_empty() {
        return Err(McpError::invalid_params(
            format!("{field} must not be empty"),
            None,
        ));
    }
    Ok(())
}

fn internal_mcp_error(error: String) -> McpError {
    McpError::internal_error(error, None)
}
