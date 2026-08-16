use rmcp::{
    ErrorData as McpError, Json, ServerHandler, ServiceExt, handler::server::wrapper::Parameters,
    schemars::JsonSchema, tool, tool_handler, tool_router, transport::stdio,
};
use serde::{Deserialize, Serialize};

use crate::application::aurora_search_service::{AuroraSearchService, SearchAurora};
use crate::application::context_gateway::ContextGateway;
use crate::application::profile_update_proposal_service::{
    ApplyProfileUpdateOutcome, DeleteProfileUpdateOutcome, ProfileUpdateProposalService,
};
use crate::application::telegram_message_service::{SaveTelegramMessage, TelegramMessageService};
use crate::domain::context_pack::ContextPack;
use crate::domain::profile_update_proposal::{ProfileUpdateProposal, ProfileUpdateTarget};
use crate::infrastructure::database::telegram_message_repository::SaveTelegramMessageOutcome;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

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
struct GlobalSearchRequest {
    /// What to find across Aurora's authorized information.
    query: String,
    /// Why the Agent needs these results for the current user request.
    purpose: String,
    /// Optional source filters: personal_context and/or telegram. Omit to search both.
    source_types: Option<Vec<String>>,
    /// Optional exact Telegram channel name filter.
    channel_name: Option<String>,
    /// Optional inclusive lower publication-time bound as RFC 3339.
    from: Option<String>,
    /// Optional exclusive upper publication-time bound as RFC 3339.
    to: Option<String>,
    /// Maximum combined results. Defaults to 10 and is capped at 25.
    max_results: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ProposeProfileUpdateRequest {
    /// Profile document to update: identity_card, current_focus, or preferences.
    target: String,
    /// Complete replacement content to store before a separate apply call.
    proposed_content: String,
    /// Why this change would make the user's profile more accurate or useful.
    reason: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetProfileUpdateProposalRequest {
    /// UUID returned when the proposal was created or listed.
    proposal_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ApplyProfileUpdateRequest {
    /// UUID of the pending proposal to apply exactly as stored.
    proposal_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct DeleteProfileUpdateProposalRequest {
    /// UUID of the pending proposal to permanently delete.
    proposal_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SaveTelegramMessageRequest {
    /// Name of the Telegram channel shown on the forwarded message.
    channel_name: String,
    /// Complete text extracted from the single forwarded message, including relevant image text.
    content_text: String,
    /// Original author shown in the forwarded content, when available.
    author_name: Option<String>,
    /// Original publication time as an RFC 3339 timestamp, when available.
    published_at: Option<String>,
    /// Telegram's original message identifier, when available.
    external_message_id: Option<String>,
    /// Original source URL shown in the forwarded message, when available.
    external_url: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
struct ProfileUpdateProposalReceipt {
    proposal_id: String,
    target: String,
    status: String,
    created_at: String,
    message: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct ProfileUpdateProposalSummary {
    proposal_id: String,
    target: String,
    reason: String,
    proposed_by: String,
    status: String,
    created_at: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct PendingProfileUpdateProposalList {
    proposals: Vec<ProfileUpdateProposalSummary>,
    count: usize,
}

#[derive(Debug, Serialize, JsonSchema)]
struct ProfileUpdateProposalDetails {
    proposal_id: String,
    target: String,
    proposed_content: String,
    reason: String,
    proposed_by: String,
    status: String,
    created_at: String,
    decided_at: Option<String>,
    message: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct ApplyProfileUpdateResponse {
    proposal_id: String,
    target: String,
    status: String,
    applied: bool,
    message: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct DeleteProfileUpdateProposalResponse {
    proposal_id: String,
    target: String,
    deleted: bool,
    message: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct SaveTelegramMessageResponse {
    message_id: String,
    status: String,
    channel_name: String,
    saved_at: String,
    message: String,
}

#[derive(Clone)]
pub struct AuroraMcpServer {
    gateway: ContextGateway,
    search_service: AuroraSearchService,
    proposal_service: ProfileUpdateProposalService,
    pool: PgPool,
}

#[tool_router]
impl AuroraMcpServer {
    pub fn new(
        gateway: ContextGateway,
        search_service: AuroraSearchService,
        proposal_service: ProfileUpdateProposalService,
        pool: PgPool,
    ) -> Self {
        Self {
            gateway,
            search_service,
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
        description = "Search all authorized Aurora information through one bounded, read-only interface. Results always include a server-generated source URI. Searches personal context files and stored Telegram messages by default; never searches privacy rules, profile proposals, hashes, or internal database fields.",
        annotations(
            title = "Search Aurora",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn search_aurora(
        &self,
        Parameters(request): Parameters<GlobalSearchRequest>,
    ) -> Result<Json<ContextPack>, McpError> {
        validate_mcp_text(&request.query, "query")?;
        validate_mcp_text(&request.purpose, "purpose")?;
        validate_optional_mcp_bounded_text(request.channel_name.as_deref(), "channel_name", 500)?;
        let (include_personal_context, include_telegram) =
            parse_search_source_types(request.source_types.as_deref())?;
        let starts_at = parse_optional_rfc3339(request.from.as_deref(), "from")?;
        let ends_at = parse_optional_rfc3339(request.to.as_deref(), "to")?;
        if let (Some(starts_at), Some(ends_at)) = (starts_at, ends_at)
            && starts_at >= ends_at
        {
            return Err(McpError::invalid_params(
                "from must be earlier than to",
                None,
            ));
        }
        if !include_telegram
            && (request.channel_name.is_some() || starts_at.is_some() || ends_at.is_some())
        {
            return Err(McpError::invalid_params(
                "channel_name, from, and to require telegram in source_types",
                None,
            ));
        }
        let max_results = request.max_results.unwrap_or(10);
        if max_results == 0 {
            return Err(McpError::invalid_params(
                "max_results must be at least 1",
                None,
            ));
        }
        let mut connection = self.pool.acquire().await.map_err(|error| {
            internal_mcp_error(format!("failed to acquire PostgreSQL connection: {error}"))
        })?;

        self.search_service
            .search(
                &mut connection,
                SearchAurora {
                    query: &request.query,
                    purpose: &request.purpose,
                    include_personal_context,
                    include_telegram,
                    channel_name: request.channel_name.as_deref(),
                    starts_at,
                    ends_at,
                    max_results: max_results.min(25),
                },
            )
            .await
            .map(Json)
            .map_err(internal_mcp_error)
    }

    #[tool(
        description = "Save exactly one Telegram message that the user explicitly asked to store in Aurora. Extract the complete text from the current forwarded message before calling. This stores source material separately from the user's identity, focus, preferences, and privacy rules.",
        annotations(
            title = "Save one Telegram message",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn save_telegram_message(
        &self,
        Parameters(request): Parameters<SaveTelegramMessageRequest>,
    ) -> Result<Json<SaveTelegramMessageResponse>, McpError> {
        validate_mcp_bounded_text(&request.channel_name, "channel_name", 500)?;
        validate_mcp_bounded_text(&request.content_text, "content_text", 50_000)?;
        validate_optional_mcp_bounded_text(request.author_name.as_deref(), "author_name", 500)?;
        validate_optional_mcp_bounded_text(
            request.external_message_id.as_deref(),
            "external_message_id",
            500,
        )?;
        validate_optional_mcp_bounded_text(request.external_url.as_deref(), "external_url", 2_048)?;
        let published_at = request
            .published_at
            .as_deref()
            .map(|value| {
                DateTime::parse_from_rfc3339(value)
                    .map(|value| value.with_timezone(&Utc))
                    .map_err(|error| {
                        McpError::invalid_params(
                            format!("published_at must be an RFC 3339 timestamp: {error}"),
                            None,
                        )
                    })
            })
            .transpose()?;
        let mut connection = self.pool.acquire().await.map_err(|error| {
            internal_mcp_error(format!("failed to acquire PostgreSQL connection: {error}"))
        })?;
        let outcome = TelegramMessageService::save(
            &mut connection,
            SaveTelegramMessage {
                channel_name: &request.channel_name,
                content_text: &request.content_text,
                author_name: request.author_name.as_deref(),
                published_at,
                external_message_id: request.external_message_id.as_deref(),
                external_url: request.external_url.as_deref(),
            },
        )
        .await
        .map_err(internal_mcp_error)?;

        let (message, status, receipt) = match outcome {
            SaveTelegramMessageOutcome::Created(message) => (
                message,
                "created",
                "The Telegram message was saved separately from the user's profile.",
            ),
            SaveTelegramMessageOutcome::AlreadyExists(message) => (
                message,
                "already_exists",
                "The Telegram message was already stored; no duplicate was created.",
            ),
        };

        Ok(Json(SaveTelegramMessageResponse {
            message_id: message.id.to_string(),
            status: status.to_string(),
            channel_name: message.channel_name,
            saved_at: message.saved_at.to_rfc3339(),
            message: receipt.to_string(),
        }))
    }

    #[tool(
        description = "Create a pending proposal containing the exact replacement for one Aurora profile document. This call never changes profile files and cannot target privacy rules; use apply_profile_update separately to execute it.",
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
            message: "Pending application; no profile file was changed.".to_string(),
        }))
    }

    #[tool(
        description = "List pending Aurora profile update proposals for the user to review. Returns summaries only and never changes proposal state or profile files.",
        annotations(
            title = "List pending Aurora profile updates",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn list_profile_update_proposals(
        &self,
    ) -> Result<Json<PendingProfileUpdateProposalList>, McpError> {
        let mut connection = self.pool.acquire().await.map_err(|error| {
            internal_mcp_error(format!("failed to acquire PostgreSQL connection: {error}"))
        })?;
        let proposals = self
            .proposal_service
            .list_pending(&mut connection)
            .await
            .map_err(internal_mcp_error)?;
        let proposals = proposals
            .iter()
            .map(ProfileUpdateProposalSummary::from)
            .collect::<Vec<_>>();
        let count = proposals.len();

        Ok(Json(PendingProfileUpdateProposalList { proposals, count }))
    }

    #[tool(
        description = "Return one Aurora profile update proposal for user review, including its proposed replacement content. Read-only; this cannot approve, reject, or apply the proposal.",
        annotations(
            title = "Get an Aurora profile update proposal",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn get_profile_update_proposal(
        &self,
        Parameters(request): Parameters<GetProfileUpdateProposalRequest>,
    ) -> Result<Json<ProfileUpdateProposalDetails>, McpError> {
        let proposal_id = Uuid::parse_str(&request.proposal_id).map_err(|error| {
            McpError::invalid_params(format!("proposal_id must be a UUID: {error}"), None)
        })?;
        let mut connection = self.pool.acquire().await.map_err(|error| {
            internal_mcp_error(format!("failed to acquire PostgreSQL connection: {error}"))
        })?;
        let proposal = self
            .proposal_service
            .find_by_id(&mut connection, proposal_id)
            .await
            .map_err(internal_mcp_error)?
            .ok_or_else(|| McpError::invalid_params("profile update proposal not found", None))?;

        Ok(Json(ProfileUpdateProposalDetails::from(proposal)))
    }

    #[tool(
        description = "Apply one stored pending Aurora profile update proposal. This writes the proposal's exact content without an additional user confirmation prompt, while still rejecting privacy-rule targets and stale file versions.",
        annotations(
            title = "Apply an Aurora profile update",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn apply_profile_update(
        &self,
        Parameters(request): Parameters<ApplyProfileUpdateRequest>,
    ) -> Result<Json<ApplyProfileUpdateResponse>, McpError> {
        let proposal_id = Uuid::parse_str(&request.proposal_id).map_err(|error| {
            McpError::invalid_params(format!("proposal_id must be a UUID: {error}"), None)
        })?;

        match self
            .proposal_service
            .apply(&self.pool, proposal_id)
            .await
            .map_err(internal_mcp_error)?
        {
            ApplyProfileUpdateOutcome::Applied(proposal) => {
                Ok(Json(ApplyProfileUpdateResponse::from_proposal(
                    proposal,
                    true,
                    "The stored proposal was applied to the local profile.",
                )))
            }
            ApplyProfileUpdateOutcome::Stale(proposal) => {
                Ok(Json(ApplyProfileUpdateResponse::from_proposal(
                    proposal,
                    false,
                    "The profile changed after this proposal was created. Nothing was overwritten; the proposal is stale.",
                )))
            }
            ApplyProfileUpdateOutcome::NotPending(proposal) => {
                Ok(Json(ApplyProfileUpdateResponse::from_proposal(
                    proposal,
                    false,
                    "The proposal is no longer pending; no additional profile change was made.",
                )))
            }
            ApplyProfileUpdateOutcome::NotFound => Err(McpError::invalid_params(
                "profile update proposal not found",
                None,
            )),
        }
    }

    #[tool(
        description = "Permanently delete one pending Aurora profile update proposal. Applied and stale proposals are retained and cannot be deleted with this tool.",
        annotations(
            title = "Delete a pending Aurora profile update",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn delete_profile_update_proposal(
        &self,
        Parameters(request): Parameters<DeleteProfileUpdateProposalRequest>,
    ) -> Result<Json<DeleteProfileUpdateProposalResponse>, McpError> {
        let proposal_id = Uuid::parse_str(&request.proposal_id).map_err(|error| {
            McpError::invalid_params(format!("proposal_id must be a UUID: {error}"), None)
        })?;

        match self
            .proposal_service
            .delete_pending(&self.pool, proposal_id)
            .await
            .map_err(internal_mcp_error)?
        {
            DeleteProfileUpdateOutcome::Deleted(proposal) => {
                Ok(Json(DeleteProfileUpdateProposalResponse {
                    proposal_id: proposal.id.to_string(),
                    target: proposal.target.as_str().to_string(),
                    deleted: true,
                    message: "The pending proposal was permanently deleted.".to_string(),
                }))
            }
            DeleteProfileUpdateOutcome::NotPending(proposal) => Err(McpError::invalid_params(
                format!(
                    "only pending proposals can be deleted; proposal is {}",
                    proposal.status.as_str()
                ),
                None,
            )),
            DeleteProfileUpdateOutcome::NotFound => Err(McpError::invalid_params(
                "profile update proposal not found",
                None,
            )),
        }
    }
}

#[tool_handler(
    name = "aurora",
    version = "0.1.0",
    instructions = "Aurora is the user's local identity and context authority. Read only the minimum context needed. Save a Telegram message only after the user explicitly asks to store that single forwarded message; Telegram source material is kept separate from profile documents. Agents may create, review, apply, and delete pending profile update proposals. Applying writes the proposal's exact content, checks the original file version, and cannot modify privacy rules. Applied and stale records cannot be deleted."
)]
impl ServerHandler for AuroraMcpServer {}

pub async fn serve(
    gateway: ContextGateway,
    search_service: AuroraSearchService,
    proposal_service: ProfileUpdateProposalService,
    pool: PgPool,
) -> Result<(), String> {
    let service = AuroraMcpServer::new(gateway, search_service, proposal_service, pool)
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

fn validate_mcp_bounded_text(value: &str, field: &str, max_chars: usize) -> Result<(), McpError> {
    validate_mcp_text(value, field)?;
    if value.trim() != value {
        return Err(McpError::invalid_params(
            format!("{field} must not have leading or trailing whitespace"),
            None,
        ));
    }
    if value.chars().count() > max_chars {
        return Err(McpError::invalid_params(
            format!("{field} must be at most {max_chars} characters"),
            None,
        ));
    }
    Ok(())
}

fn validate_optional_mcp_bounded_text(
    value: Option<&str>,
    field: &str,
    max_chars: usize,
) -> Result<(), McpError> {
    match value {
        Some(value) => validate_mcp_bounded_text(value, field, max_chars),
        None => Ok(()),
    }
}

fn parse_optional_rfc3339(
    value: Option<&str>,
    field: &str,
) -> Result<Option<DateTime<Utc>>, McpError> {
    value
        .map(|value| {
            DateTime::parse_from_rfc3339(value)
                .map(|value| value.with_timezone(&Utc))
                .map_err(|error| {
                    McpError::invalid_params(
                        format!("{field} must be an RFC 3339 timestamp: {error}"),
                        None,
                    )
                })
        })
        .transpose()
}

fn parse_search_source_types(source_types: Option<&[String]>) -> Result<(bool, bool), McpError> {
    let Some(source_types) = source_types else {
        return Ok((true, true));
    };
    if source_types.is_empty() {
        return Err(McpError::invalid_params(
            "source_types must contain personal_context and/or telegram",
            None,
        ));
    }

    let mut include_personal_context = false;
    let mut include_telegram = false;
    for source_type in source_types {
        match source_type.as_str() {
            "personal_context" => include_personal_context = true,
            "telegram" => include_telegram = true,
            _ => {
                return Err(McpError::invalid_params(
                    format!("unsupported source_type: {source_type}"),
                    None,
                ));
            }
        }
    }
    Ok((include_personal_context, include_telegram))
}

fn internal_mcp_error(error: String) -> McpError {
    McpError::internal_error(error, None)
}

impl From<&ProfileUpdateProposal> for ProfileUpdateProposalSummary {
    fn from(proposal: &ProfileUpdateProposal) -> Self {
        Self {
            proposal_id: proposal.id.to_string(),
            target: proposal.target.as_str().to_string(),
            reason: proposal.reason.clone(),
            proposed_by: proposal.proposed_by.clone(),
            status: proposal.status.as_str().to_string(),
            created_at: proposal.created_at.to_rfc3339(),
        }
    }
}

impl From<ProfileUpdateProposal> for ProfileUpdateProposalDetails {
    fn from(proposal: ProfileUpdateProposal) -> Self {
        Self {
            proposal_id: proposal.id.to_string(),
            target: proposal.target.as_str().to_string(),
            proposed_content: proposal.proposed_content,
            reason: proposal.reason,
            proposed_by: proposal.proposed_by,
            status: proposal.status.as_str().to_string(),
            created_at: proposal.created_at.to_rfc3339(),
            decided_at: proposal.decided_at.map(|value| value.to_rfc3339()),
            message: "Review only; no proposal state or profile file was changed.".to_string(),
        }
    }
}

impl ApplyProfileUpdateResponse {
    fn from_proposal(proposal: ProfileUpdateProposal, applied: bool, message: &str) -> Self {
        Self {
            proposal_id: proposal.id.to_string(),
            target: proposal.target.as_str().to_string(),
            status: proposal.status.as_str().to_string(),
            applied,
            message: message.to_string(),
        }
    }
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

    #[test]
    fn telegram_save_tool_is_narrow_idempotent_and_non_destructive() {
        let tools = AuroraMcpServer::tool_router().list_all();
        let tool = tools
            .iter()
            .find(|tool| tool.name == "save_telegram_message")
            .expect("Telegram save tool should be registered");
        let schema = serde_json::to_string(&tool.input_schema)
            .expect("Telegram save input schema should serialize");

        for field in [
            "channel_name",
            "content_text",
            "author_name",
            "published_at",
            "external_message_id",
            "external_url",
        ] {
            assert!(schema.contains(field));
        }
        assert!(!schema.contains("identity_card"));
        assert!(!schema.contains("preferences"));
        assert!(!schema.contains("privacy_rules"));

        let annotations = tool
            .annotations
            .as_ref()
            .expect("Telegram save tool should declare annotations");
        assert_eq!(annotations.read_only_hint, Some(false));
        assert_eq!(annotations.destructive_hint, Some(false));
        assert_eq!(annotations.idempotent_hint, Some(true));
        assert_eq!(annotations.open_world_hint, Some(false));
    }

    #[test]
    fn global_search_tool_is_read_only_and_source_filterable() {
        let tools = AuroraMcpServer::tool_router().list_all();
        let tool = tools
            .iter()
            .find(|tool| tool.name == "search_aurora")
            .expect("global search tool should be registered");
        let schema = serde_json::to_string(&tool.input_schema)
            .expect("global search input schema should serialize");

        for field in [
            "query",
            "purpose",
            "source_types",
            "channel_name",
            "from",
            "to",
            "max_results",
        ] {
            assert!(schema.contains(field));
        }
        assert!(!schema.contains("sql"));
        assert!(!schema.contains("privacy_rules"));

        let annotations = tool
            .annotations
            .as_ref()
            .expect("global search tool should declare annotations");
        assert_eq!(annotations.read_only_hint, Some(true));
        assert_eq!(annotations.destructive_hint, Some(false));
        assert_eq!(annotations.idempotent_hint, Some(true));
        assert_eq!(annotations.open_world_hint, Some(false));
    }

    #[test]
    fn proposal_review_tools_are_read_only_and_cannot_accept_a_decision() {
        let tools = AuroraMcpServer::tool_router().list_all();

        for name in [
            "list_profile_update_proposals",
            "get_profile_update_proposal",
        ] {
            let tool = tools
                .iter()
                .find(|tool| tool.name == name)
                .unwrap_or_else(|| panic!("{name} should be registered"));
            let schema = serde_json::to_string(&tool.input_schema)
                .expect("review tool input schema should serialize");
            assert!(!schema.contains("approve"));
            assert!(!schema.contains("reject"));
            assert!(!schema.contains("status"));

            let annotations = tool
                .annotations
                .as_ref()
                .expect("review tool should declare annotations");
            assert_eq!(annotations.read_only_hint, Some(true));
            assert_eq!(annotations.destructive_hint, Some(false));
            assert_eq!(annotations.idempotent_hint, Some(true));
            assert_eq!(annotations.open_world_hint, Some(false));
        }
    }

    #[test]
    fn apply_tool_only_accepts_a_proposal_id_and_is_destructive() {
        let tools = AuroraMcpServer::tool_router().list_all();
        let tool = tools
            .iter()
            .find(|tool| tool.name == "apply_profile_update")
            .expect("apply tool should be registered");
        let schema = serde_json::to_string(&tool.input_schema)
            .expect("apply tool input schema should serialize");

        assert!(schema.contains("proposal_id"));
        assert!(!schema.contains("approved"));
        assert!(!schema.contains("proposed_content"));
        assert!(!schema.contains("target"));

        let annotations = tool
            .annotations
            .as_ref()
            .expect("apply tool should declare annotations");
        assert_eq!(annotations.read_only_hint, Some(false));
        assert_eq!(annotations.destructive_hint, Some(true));
        assert_eq!(annotations.idempotent_hint, Some(false));
        assert_eq!(annotations.open_world_hint, Some(false));
    }

    #[test]
    fn delete_tool_only_accepts_a_proposal_id_and_is_destructive() {
        let tools = AuroraMcpServer::tool_router().list_all();
        let tool = tools
            .iter()
            .find(|tool| tool.name == "delete_profile_update_proposal")
            .expect("delete tool should be registered");
        let schema = serde_json::to_string(&tool.input_schema)
            .expect("delete tool input schema should serialize");

        assert!(schema.contains("proposal_id"));
        assert!(!schema.contains("status"));
        assert!(!schema.contains("target"));

        let annotations = tool
            .annotations
            .as_ref()
            .expect("delete tool should declare annotations");
        assert_eq!(annotations.read_only_hint, Some(false));
        assert_eq!(annotations.destructive_hint, Some(true));
        assert_eq!(annotations.idempotent_hint, Some(false));
        assert_eq!(annotations.open_world_hint, Some(false));
    }
}
