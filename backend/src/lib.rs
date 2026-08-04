pub mod api;
pub mod application;
pub mod config;
pub mod domain;
pub mod infrastructure;

use application::context_gateway::ContextGateway;
use application::profile_update_proposal_service::ProfileUpdateProposalService;
use config::AppConfig;
use infrastructure::database;

pub fn run_args(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    api::cli::run_args(args)
}

fn run_mcp(config: AppConfig) -> Result<(), String> {
    let client = std::env::var("AURORA_MCP_CLIENT").unwrap_or_else(|_| "unknown-agent".to_string());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("failed to start MCP runtime: {error}"))?;
    runtime.block_on(async move {
        let pool = database::connect_and_migrate().await?;
        let gateway = ContextGateway::new(config.clone(), client.clone());
        let proposal_service = ProfileUpdateProposalService::new(config, client);
        api::mcp::serve(gateway, proposal_service, pool).await
    })
}
