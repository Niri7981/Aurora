use rmcp::{
    ErrorData as McpError, Json, ServerHandler, ServiceExt, handler::server::wrapper::Parameters,
    schemars::JsonSchema, tool, tool_handler, tool_router, transport::stdio,
};
use serde::{Deserialize, Serialize};

use crate::application::context_gateway::ContextGateway;
use crate::application::profile_update_proposal_service::ProfileUpdateProposalService;
use crate::domain::context_pack::ContextPack;
use crate::domain::profile_update_proposal::ProfileUpdateTarget;
use sqlx::PgPool;

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

#[derive(Debug, Deserialize, JsonSchema)]
struct ProposeProfileUpdateRequest {
    /// Profile document to update: identity_card, current_focus, or preferences.
    target: String,
    /// Complete replacement content to show the user for approval.
    proposed_content: String,
    /// Why this change would make the user's profile more accurate or useful.
    reason: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct ProfileUpdateProposalReceipt {
    proposal_id: String,
    target: String,
    status: String,
    created_at: String,
    message: String,
}

#[derive(Clone)]
pub struct AuroraMcpServer {
    gateway: ContextGateway,
    proposal_service: ProfileUpdateProposalService,
    pool: PgPool,
}

#[tool_router]
impl AuroraMcpServer {
    pub fn new(
        gateway: ContextGateway,
        proposal_service: ProfileUpdateProposalService,
        pool: PgPool,
    ) -> Self {
        Self {
            gateway,
            proposal_service,
            pool,
        }
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

    #[tool(
        description = "Create a pending proposal to replace one Aurora profile document. This never changes profile files, cannot approve its own proposal, and cannot target privacy rules.",
        annotations(
            title = "Propose an Aurora profile update",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn propose_profile_update(
        &self,
        Parameters(request): Parameters<ProposeProfileUpdateRequest>,
    ) -> Result<Json<ProfileUpdateProposalReceipt>, McpError> {
        validate_mcp_text(&request.proposed_content, "proposed_content")?;
        validate_mcp_text(&request.reason, "reason")?;
        let target = ProfileUpdateTarget::try_from(request.target.as_str())
            .map_err(|error| McpError::invalid_params(error, None))?;
        let mut connection = self.pool.acquire().await.map_err(|error| {
            internal_mcp_error(format!("failed to acquire PostgreSQL connection: {error}"))
        })?;
        let proposal = self
            .proposal_service
            .propose(
                &mut connection,
                target,
                &request.proposed_content,
                &request.reason,
            )
            .await
            .map_err(internal_mcp_error)?;

        Ok(Json(ProfileUpdateProposalReceipt {
            proposal_id: proposal.id.to_string(),
            target: proposal.target.as_str().to_string(),
            status: proposal.status.as_str().to_string(),
            created_at: proposal.created_at.to_rfc3339(),
            message: "Pending user approval; no profile file was changed.".to_string(),
        }))
    }
}

#[tool_handler(
    name = "aurora",
    version = "0.1.0",
    instructions = "Aurora is the user's local identity and context authority. Call only when personal context materially helps the current request. State a narrow purpose, use the minimum returned context, respect omissions, and never claim access beyond the returned Context Pack."
)]
impl ServerHandler for AuroraMcpServer {}

pub async fn serve(
    gateway: ContextGateway,
    proposal_service: ProfileUpdateProposalService,
    pool: PgPool,
) -> Result<(), String> {
    let service = AuroraMcpServer::new(gateway, proposal_service, pool)
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

#[cfg(test)]
mod tests {
    use super::AuroraMcpServer;

    #[test]
    fn proposal_tool_does_not_accept_status_hash_or_agent_identity() {
        let tools = AuroraMcpServer::tool_router().list_all();
        let tool = tools
            .iter()
            .find(|tool| tool.name == "propose_profile_update")
            .expect("proposal tool should be registered");
        let schema = serde_json::to_string(&tool.input_schema)
            .expect("proposal input schema should serialize");

        assert!(schema.contains("target"));
        assert!(schema.contains("proposed_content"));
        assert!(schema.contains("reason"));
        assert!(!schema.contains("status"));
        assert!(!schema.contains("base_sha256"));
        assert!(!schema.contains("proposed_by"));

        let annotations = tool
            .annotations
            .as_ref()
            .expect("proposal tool should declare annotations");
        assert_eq!(annotations.read_only_hint, Some(false));
        assert_eq!(annotations.destructive_hint, Some(false));
        assert_eq!(annotations.idempotent_hint, Some(false));
        assert_eq!(annotations.open_world_hint, Some(false));
    }
}
