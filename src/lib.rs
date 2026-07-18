pub mod app;
pub mod cli;
pub mod command_palette;
pub mod config;
pub mod context;
pub mod harness;
pub mod mcp;
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

pub fn run_args(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    let mut args = args.into_iter();
    match args.next() {
        Some(command) if command == "serve" => {
            let config = config::load_config(args.next())?;
            if let Some(extra) = args.next() {
                return Err(format!("serve 收到了多余参数：{extra}"));
            }
            mcp::run(config)
        }
        workspace => {
            if let Some(extra) = args.next() {
                return Err(format!("收到了多余参数：{extra}"));
            }
            run(workspace)
        }
    }
}
