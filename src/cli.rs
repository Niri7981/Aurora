use std::env;
use std::io::{self, Write};
use std::path::Path;

use crate::config::AppConfig;
use crate::ollama;
use crate::session::Session;

const APP_NAME: &str = "A U R O R A";
const APP_TAGLINE: &str = "local-first assistant shell";
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const ACCENT: &str = "\x1b[38;5;215m";
const MUTED: &str = "\x1b[38;5;246m";

pub fn run(config: &AppConfig) -> Result<(), String> {
    render_banner(Path::new(&config.workspace), &config.model).map_err(|err| err.to_string())?;
    repl_loop(config)
}

fn render_banner(workspace: &Path, model: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    let width = env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(80)
        .min(88);
    let rule = "─".repeat(width);

    write!(
        stdout,
        "\x1b[2J\x1b[H{accent}{bold}  {name}{reset}\n{dim}  {tagline}{reset}\n\n{muted}  Model     {reset}{model}\n{muted}  Mode      {reset}CLI\n{muted}  Workspace {reset}{workspace}\n\n{muted}{rule}{reset}\n{dim}  Type a request, or 'quit' to exit.{reset}\n\n",
        accent = ACCENT,
        bold = BOLD,
        name = APP_NAME,
        reset = RESET,
        dim = DIM,
        tagline = APP_TAGLINE,
        muted = MUTED,
        model = model,
        workspace = workspace.display(),
        rule = rule
    )?;
    stdout.flush()?;
    Ok(())
}

fn repl_loop(config: &AppConfig) -> Result<(), String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut session = Session::new();

    loop {
        write!(stdout, "{ACCENT}> {RESET}").map_err(|err| err.to_string())?;
        stdout.flush().map_err(|err| err.to_string())?;

        let mut input = String::new();
        stdin
            .read_line(&mut input)
            .map_err(|err| format!("failed to read input from terminal: {err}"))?;
        let trimmed = input.trim();

        if trimmed.is_empty() {
            continue;
        }

        if matches!(trimmed, "quit" | "exit") {
            println!("助手> 下次见。");
            break;
        }

        if trimmed == "/clear" {
            session.clear();
            println!("助手> 已清空当前会话。");
            continue;
        }

        println!("助手> 正在思考...");
        match ollama::chat(&config.ollama_url, &config.model, session.history(), trimmed) {
            Ok(reply) => {
                session.push_turn(trimmed, &reply);
                println!("助手> {reply}");
            }
            Err(err) => println!("助手> 执行失败：{err}"),
        }
    }

    Ok(())
}
