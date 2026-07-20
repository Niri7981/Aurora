use crate::{config, context, mcp};

const USAGE: &str = "Aurora is a local personal memory layer for authorized MCP clients.\n\nUsage:\n  aurora serve [workspace]   Start the stdio MCP server\n  aurora init [workspace]    Create missing local identity files\n  aurora preview [workspace] Preview externally disclosable context\n  aurora audit [workspace]   Show recent MCP context access\n  aurora help                Show this help";

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Serve(Option<String>),
    Init(Option<String>),
    Preview(Option<String>),
    Audit(Option<String>),
    Help,
}

pub fn run_args(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    match parse_args(args)? {
        Command::Serve(workspace) => mcp::run(config::load_config(workspace)?),
        Command::Init(workspace) => {
            let config = config::load_config(workspace)?;
            println!("{}", context::init_files(&config)?.render());
            Ok(())
        }
        Command::Preview(workspace) => {
            let config = config::load_config(workspace)?;
            println!("{}", context::load(&config)?.render_preview());
            Ok(())
        }
        Command::Audit(workspace) => {
            let config = config::load_config(workspace)?;
            println!("{}", mcp::render_audit_log(&config, 20)?);
            Ok(())
        }
        Command::Help => {
            println!("{USAGE}");
            Ok(())
        }
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        return Ok(Command::Help);
    };

    if matches!(command.as_str(), "help" | "-h" | "--help") {
        reject_extra(&mut args, "help")?;
        return Ok(Command::Help);
    }

    let workspace = args.next();
    reject_extra(&mut args, &command)?;
    match command.as_str() {
        "serve" => Ok(Command::Serve(workspace)),
        "init" => Ok(Command::Init(workspace)),
        "preview" => Ok(Command::Preview(workspace)),
        "audit" => Ok(Command::Audit(workspace)),
        _ => Err(format!("unknown command `{command}`\n\n{USAGE}")),
    }
}

fn reject_extra(args: &mut impl Iterator<Item = String>, command: &str) -> Result<(), String> {
    if let Some(extra) = args.next() {
        return Err(format!("{command} received an extra argument: {extra}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Command, parse_args};

    fn parse(args: &[&str]) -> Result<Command, String> {
        parse_args(args.iter().map(|value| value.to_string()))
    }

    #[test]
    fn no_arguments_show_help() {
        assert_eq!(parse(&[]), Ok(Command::Help));
    }

    #[test]
    fn parses_serve_with_workspace() {
        assert_eq!(
            parse(&["serve", "/tmp/workspace"]),
            Ok(Command::Serve(Some("/tmp/workspace".to_string())))
        );
    }

    #[test]
    fn rejects_unknown_commands_and_extra_arguments() {
        assert!(parse(&["chat"]).unwrap_err().contains("unknown command"));
        assert!(parse(&["audit", ".", "extra"]).is_err());
    }
}
