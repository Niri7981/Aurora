use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

use crossterm::cursor::{MoveToColumn, MoveUp};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode};
use unicode_width::UnicodeWidthChar;

use crate::app::{App, TurnOutcome, should_show_thinking_indicator};
use crate::command_palette::matching_commands;
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
        let Some(input) = read_request(&stdin, &mut stdout)? else {
            break;
        };
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

fn read_request(stdin: &io::Stdin, stdout: &mut io::Stdout) -> Result<Option<String>, String> {
    if !stdin.is_terminal() || !stdout.is_terminal() {
        write!(stdout, "{ACCENT}> {RESET}").map_err(|err| err.to_string())?;
        stdout.flush().map_err(|err| err.to_string())?;

        let mut input = String::new();
        let bytes_read = stdin
            .read_line(&mut input)
            .map_err(|err| format!("failed to read input from terminal: {err}"))?;
        return Ok((bytes_read > 0).then_some(input));
    }

    let _raw_mode = RawModeGuard::enter()?;
    let mut input = Vec::<char>::new();
    let mut cursor = 0;
    let mut selected = 0;
    let mut previous_suggestions = 0;
    render_request_editor(stdout, &input, cursor, selected, &mut previous_suggestions)?;

    loop {
        let Event::Key(key) = event::read().map_err(|err| err.to_string())? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        let current = input.iter().collect::<String>();
        let suggestions = matching_commands(&current);

        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                clear_request_suggestions(stdout, previous_suggestions)?;
                execute!(stdout, MoveToColumn(0), Clear(ClearType::CurrentLine))
                    .map_err(|err| err.to_string())?;
                write!(stdout, "^C\r\n").map_err(|err| err.to_string())?;
                stdout.flush().map_err(|err| err.to_string())?;
                return Ok(Some("quit".to_string()));
            }
            KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                input.insert(cursor, ch);
                cursor += 1;
                selected = 0;
            }
            KeyCode::Backspace if cursor > 0 => {
                cursor -= 1;
                input.remove(cursor);
                selected = 0;
            }
            KeyCode::Delete if cursor < input.len() => {
                input.remove(cursor);
                selected = 0;
            }
            KeyCode::Left => cursor = cursor.saturating_sub(1),
            KeyCode::Right => cursor = (cursor + 1).min(input.len()),
            KeyCode::Up if !suggestions.is_empty() => {
                selected = selected.saturating_sub(1);
            }
            KeyCode::Down if !suggestions.is_empty() => {
                selected = (selected + 1).min(suggestions.len() - 1);
            }
            KeyCode::Tab if !suggestions.is_empty() => {
                input = suggestions[selected].name.chars().collect();
                cursor = input.len();
                selected = 0;
            }
            KeyCode::Esc if !suggestions.is_empty() => {
                input.clear();
                cursor = 0;
                selected = 0;
            }
            KeyCode::Enter => {
                let result = if !suggestions.is_empty() {
                    suggestions[selected].name.to_string()
                } else {
                    current
                };
                clear_request_suggestions(stdout, previous_suggestions)?;
                execute!(stdout, MoveToColumn(0), Clear(ClearType::CurrentLine))
                    .map_err(|err| err.to_string())?;
                write!(stdout, "{ACCENT}> {RESET}{result}\r\n").map_err(|err| err.to_string())?;
                stdout.flush().map_err(|err| err.to_string())?;
                return Ok(Some(result));
            }
            _ => continue,
        }

        render_request_editor(stdout, &input, cursor, selected, &mut previous_suggestions)?;
    }
}

fn render_request_editor(
    stdout: &mut impl Write,
    input: &[char],
    cursor: usize,
    selected: usize,
    previous_suggestions: &mut usize,
) -> Result<(), String> {
    let input_text = input.iter().collect::<String>();
    let suggestions = matching_commands(&input_text);
    let rows = (*previous_suggestions).max(suggestions.len());

    execute!(stdout, MoveToColumn(0), Clear(ClearType::CurrentLine))
        .map_err(|err| err.to_string())?;
    write!(stdout, "{ACCENT}> {RESET}{input_text}").map_err(|err| err.to_string())?;

    for index in 0..rows {
        write!(stdout, "\r\n").map_err(|err| err.to_string())?;
        execute!(stdout, Clear(ClearType::CurrentLine)).map_err(|err| err.to_string())?;
        if let Some(command) = suggestions.get(index) {
            if index == selected {
                write!(
                    stdout,
                    "{ACCENT}❯ {:<18}{RESET}{DIM}{}{RESET}",
                    command.name, command.description
                )
                .map_err(|err| err.to_string())?;
            } else {
                write!(
                    stdout,
                    "  {:<18}{DIM}{}{RESET}",
                    command.name, command.description
                )
                .map_err(|err| err.to_string())?;
            }
        }
    }

    if rows > 0 {
        execute!(stdout, MoveUp(rows as u16)).map_err(|err| err.to_string())?;
    }
    execute!(stdout, MoveToColumn(input_cursor_column(input, cursor)))
        .map_err(|err| err.to_string())?;
    stdout.flush().map_err(|err| err.to_string())?;
    *previous_suggestions = suggestions.len();
    Ok(())
}

fn clear_request_suggestions(
    stdout: &mut impl Write,
    suggestion_count: usize,
) -> Result<(), String> {
    for _ in 0..suggestion_count {
        write!(stdout, "\r\n").map_err(|err| err.to_string())?;
        execute!(stdout, Clear(ClearType::CurrentLine)).map_err(|err| err.to_string())?;
    }
    if suggestion_count > 0 {
        execute!(stdout, MoveUp(suggestion_count as u16)).map_err(|err| err.to_string())?;
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

fn input_cursor_column(input: &[char], cursor: usize) -> u16 {
    let input_width = input[..cursor]
        .iter()
        .map(|ch| ch.width().unwrap_or(0))
        .sum::<usize>();
    (2 + input_width).min(u16::MAX as usize) as u16
}

#[cfg(test)]
mod tests {
    use super::input_cursor_column;

    #[test]
    fn cursor_column_uses_terminal_display_width_for_chinese_input() {
        let input = "你好".chars().collect::<Vec<_>>();

        assert_eq!(input_cursor_column(&input, input.len()), 6);
    }
}
