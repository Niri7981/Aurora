use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

use crossterm::cursor::MoveUp;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode};

use crate::app::{App, TurnOutcome, should_show_thinking_indicator};
use crate::config::AppConfig;
use crate::harness::ConfirmationDecision;
use crate::model::ConfiguredChatClient;
use crate::startup_animation;

const APP_NAME: &str = "A U R O R A";
const APP_TAGLINE: &str = "local-first assistant shell";
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const ACCENT: &str = "\x1b[38;5;215m";
const MUTED: &str = "\x1b[38;5;246m";

pub fn run(config: &AppConfig) -> Result<(), String> {
    render_banner(
        Path::new(&config.workspace),
        &config.provider,
        config.active_model(),
    )
    .map_err(|err| err.to_string())?;
    repl_loop(config)
}

fn render_banner(workspace: &Path, provider: &str, model: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    let width = env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(80)
        .min(88);
    let rule = "─".repeat(width);
    let is_terminal = stdout.is_terminal();
    let should_animate = is_terminal
        && env::var_os("AURORA_NO_ANIMATION").is_none()
        && env::var("TERM").map(|term| term != "dumb").unwrap_or(true);

    if should_animate {
        startup_animation::play().map_err(io::Error::other)?;
    }

    if !is_terminal || !should_animate {
        write!(
            stdout,
            "{ACCENT}{BOLD}  {APP_NAME}{RESET}\n{DIM}  {APP_TAGLINE}{RESET}\n\n"
        )?;
    }

    write!(
        stdout,
        "{muted}  Provider  {reset}{provider}\n{muted}  Model     {reset}{model}\n{muted}  Mode      {reset}CLI\n{muted}  Workspace {reset}{workspace}\n\n{muted}{rule}{reset}\n{dim}  Type a request, or 'quit' to exit.{reset}\n\n",
        reset = RESET,
        dim = DIM,
        muted = MUTED,
        provider = provider,
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
    let mut app = App::new(config.clone(), ConfiguredChatClient);

    loop {
        write!(stdout, "{ACCENT}> {RESET}").map_err(|err| err.to_string())?;
        stdout.flush().map_err(|err| err.to_string())?;

        let mut input = String::new();
        stdin
            .read_line(&mut input)
            .map_err(|err| format!("failed to read input from terminal: {err}"))?;
        let trimmed = input.trim();

        if should_show_thinking_indicator(trimmed) {
            println!("助手> 正在思考...");
        }

        match app.handle_text(trimmed) {
            Ok(TurnOutcome::Ignored) => continue,
            Ok(TurnOutcome::Exit(message)) => {
                println!("{message}");
                break;
            }
            Ok(TurnOutcome::Cleared(message)) | Ok(TurnOutcome::Reply(message)) => {
                println!("{message}");
            }
            Ok(TurnOutcome::Confirmation { tool_name, prompt }) => {
                let decision = select_confirmation(&prompt, &tool_name)?;
                match app.resolve_confirmation(decision) {
                    Ok(TurnOutcome::Reply(message)) => println!("{message}"),
                    Ok(_) => return Err("确认动作返回了无效状态".to_string()),
                    Err(err) => println!("助手> 执行失败：{err}"),
                }
            }
            Err(err) => {
                println!("助手> 执行失败：{err}");
            }
        }
    }

    Ok(())
}

fn select_confirmation(prompt: &str, tool_name: &str) -> Result<ConfirmationDecision, String> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        println!("助手> {prompt}\n  No (non-interactive terminal)");
        return Ok(ConfirmationDecision::Deny);
    }

    let options = [
        "Yes".to_string(),
        format!("Yes, and always allow {tool_name} for this session"),
        "No".to_string(),
    ];
    let mut selected = 0;
    let mut stdout = io::stdout();
    let _raw_mode = RawModeGuard::enter()?;

    write!(stdout, "助手> {prompt}\r\n").map_err(|err| err.to_string())?;
    render_confirmation_options(&mut stdout, &options, selected, false)?;

    loop {
        let Event::Key(key) = event::read().map_err(|err| err.to_string())? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Up => selected = selected.saturating_sub(1),
            KeyCode::Down => selected = (selected + 1).min(options.len() - 1),
            KeyCode::Char('1') | KeyCode::Char('y') => selected = 0,
            KeyCode::Char('2') | KeyCode::Char('a') => selected = 1,
            KeyCode::Char('3') | KeyCode::Char('n') | KeyCode::Esc => selected = 2,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => selected = 2,
            KeyCode::Enter => {
                render_confirmation_options(&mut stdout, &options, selected, true)?;
                return Ok(match selected {
                    0 => ConfirmationDecision::AllowOnce,
                    1 => ConfirmationDecision::AlwaysAllow,
                    _ => ConfirmationDecision::Deny,
                });
            }
            _ => continue,
        }

        render_confirmation_options(&mut stdout, &options, selected, true)?;
    }
}

fn render_confirmation_options(
    stdout: &mut impl Write,
    options: &[String; 3],
    selected: usize,
    redraw: bool,
) -> Result<(), String> {
    if redraw {
        execute!(stdout, MoveUp(options.len() as u16)).map_err(|err| err.to_string())?;
    }

    for (index, option) in options.iter().enumerate() {
        execute!(stdout, Clear(ClearType::CurrentLine)).map_err(|err| err.to_string())?;
        if index == selected {
            write!(stdout, "{ACCENT}❯ {}. {option}{RESET}\r\n", index + 1)
                .map_err(|err| err.to_string())?;
        } else {
            write!(stdout, "  {}. {option}\r\n", index + 1).map_err(|err| err.to_string())?;
        }
    }
    stdout.flush().map_err(|err| err.to_string())
}

struct RawModeGuard;

impl RawModeGuard {
    fn enter() -> Result<Self, String> {
        enable_raw_mode().map_err(|err| format!("无法进入终端选择模式：{err}"))?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}
