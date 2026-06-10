pub mod app;
pub mod cli;
pub mod config;
pub mod ollama;
pub mod session;

pub fn run(workspace_arg: Option<String>) -> Result<(), String> {
    let config = config::load_config(workspace_arg)?;
    cli::run(&config)?;
    Ok(())
}
