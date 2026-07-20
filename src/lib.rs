pub mod cli;
pub mod config;
pub mod context;
pub mod mcp;

pub fn run_args(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    cli::run_args(args)
}
