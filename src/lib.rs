pub mod app;
pub mod cli;
pub mod command_palette;
pub mod config;
pub mod context;
pub mod harness;
pub mod model;
pub mod planner;
pub mod session;
pub mod startup_animation;
pub mod theme;
pub mod tools;

pub fn run(workspace_arg: Option<String>) -> Result<(), String> {
    let config = config::load_config(workspace_arg)?;
    cli::run(&config)?;
    Ok(())
}
