pub mod app;
pub mod cli;
pub mod config;
pub mod context;
pub mod harness;
pub mod model_provider;
pub mod ollama;
pub mod planner;
pub mod session;

pub fn run(workspace_arg: Option<String>) -> Result<(), String> {
    let config = config::load_config(workspace_arg)?;
    cli::run(&config)?;
    Ok(())
}
