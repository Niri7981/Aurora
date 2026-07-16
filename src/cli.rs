use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

use crossterm::cursor::{MoveTo, MoveToColumn, MoveUp, RestorePosition, SavePosition};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{self, Clear, ClearType, disable_raw_mode, enable_raw_mode};
use unicode_width::UnicodeWidthChar;

use crate::app::{App, TurnOutcome, should_show_thinking_indicator};
use crate::command_palette::matching_commands;
use crate::config::AppConfig;
use crate::harness::ConfirmationDecision;
use crate::model::ConfiguredChatClient;
use crate::startup_animation;
use crate::theme::{ACCENT, BOLD, DIM, LOGO_HIGHLIGHT, MUTED, PANEL_BACKGROUND, RESET};

const APP_NAME: &str = "A U R O R A";
const APP_TAGLINE: &str = "local-first assistant shell";
pub fn run(config: &AppConfig) -> Result<(), String> {
    let banner = render_banner(
        Path::new(&config.workspace),
        &config.provider,
        config.active_model(),
    )
    .map_err(|err| err.to_string())?;
    repl_loop(config, &banner)
}

struct BannerLayout {
    model_location: Option<ModelLocation>,
}

enum ModelLocation {
    Line(u16),
    Dashboard { row: u16, col: u16, width: usize },
}

impl BannerLayout {
    fn update_model(&self, stdout: &mut impl Write, model: &str) -> io::Result<()> {
        let Some(location) = &self.model_location else {
            return Ok(());
        };

        execute!(stdout, SavePosition)?;
        match location {
            ModelLocation::Line(row) => {
                execute!(stdout, MoveTo(0, *row), Clear(ClearType::CurrentLine))?;
                write!(stdout, "{MUTED}  Model     {RESET}{model}")?;
            }
            ModelLocation::Dashboard { row, col, width } => {
                execute!(
                    stdout,
                    MoveTo(*col, *row),
                    SetBackgroundColor(PANEL_BACKGROUND),
                    SetForegroundColor(LOGO_HIGHLIGHT),
                    Print(startup_animation::fit_text(model, *width)),
                    ResetColor
                )?;
            }
        }
        execute!(stdout, RestorePosition)?;
        stdout.flush()
    }
}

fn render_banner(workspace: &Path, provider: &str, model: &str) -> io::Result<BannerLayout> {
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
        && env::var("TERM").map(|term| term != "dumb").unwrap_or(true)
        && terminal::size()
            .map(|(terminal_width, terminal_height)| terminal_width >= 80 && terminal_height >= 23)
            .unwrap_or(false);

    if should_animate {
        let layout = startup_animation::play(provider, model, &workspace.display().to_string())
            .map_err(io::Error::other)?;
        return Ok(BannerLayout {
            model_location: Some(ModelLocation::Dashboard {
                row: layout.model_row,
                col: layout.model_col,
                width: layout.model_width,
            }),
        });
    }

    let status_start_row: Option<u16> = if is_terminal {
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        write!(
            stdout,
            "{ACCENT}{BOLD}  {APP_NAME}{RESET}\n{DIM}  {APP_TAGLINE}{RESET}\n\n"
        )?;
        Some(3)
    } else {
        write!(
            stdout,
            "{ACCENT}{BOLD}  {APP_NAME}{RESET}\n{DIM}  {APP_TAGLINE}{RESET}\n\n"
        )?;
        None
    };
    let model_location = status_start_row.map(|row| ModelLocation::Line(row.saturating_add(1)));

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
    Ok(BannerLayout { model_location })
}

fn repl_loop(config: &AppConfig, banner: &BannerLayout) -> Result<(), String> {
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
            Ok(TurnOutcome::Confirmation {
                tool_name,
                prompt,
                allow_always,
            }) => {
                let decision = select_confirmation(&prompt, &tool_name, allow_always)?;
                match app.resolve_confirmation(decision) {
                    Ok(TurnOutcome::Reply(message)) => println!("{message}"),
                    Ok(_) => return Err("确认动作返回了无效状态".to_string()),
                    Err(err) => println!("助手> 执行失败：{err}"),
                }
            }
            Ok(TurnOutcome::ModelSelection {
                current_model,
                models,
            }) => match select_model(&current_model, &models)? {
                Some(model) => match app.select_model(&model) {
                    Ok(TurnOutcome::ModelChanged { model, message }) => {
                        banner
                            .update_model(&mut stdout, &model)
                            .map_err(|err| format!("更新顶部模型状态失败：{err}"))?;
                        println!("{message}");
                    }
                    Ok(_) => return Err("模型切换返回了无效状态".to_string()),
                    Err(err) => println!("助手> 切换模型失败：{err}"),
                },
                None => println!("助手> 已取消模型切换。"),
            },
            Ok(TurnOutcome::ModelChanged { .. }) => {
                return Err("模型切换结果出现在了无效的请求阶段".to_string());
            }
            Err(err) => {
                println!("助手> 执行失败：{err}");
            }
        }
    }

    Ok(())
}

const MODEL_MENU_SIZE: usize = 8;
const MODEL_MENU_LINES: u16 = (MODEL_MENU_SIZE + 2) as u16;

fn select_model(current_model: &str, models: &[String]) -> Result<Option<String>, String> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("/model 需要交互式终端来选择模型".to_string());
    }
    if models.is_empty() {
        return Err("API 没有返回可选择的模型".to_string());
    }

    let mut stdout = io::stdout();
    let _raw_mode = RawModeGuard::enter()?;
    let mut filter = String::new();
    let mut selected = models
        .iter()
        .position(|model| model == current_model)
        .unwrap_or(0);

    write!(stdout, "当前模型：{current_model}\r\n").map_err(|err| err.to_string())?;
    render_model_menu(&mut stdout, models, current_model, &filter, selected, false)?;

    loop {
        let Event::Key(key) = event::read().map_err(|err| err.to_string())? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        let filtered = filtered_models(models, &filter);
        match key.code {
            KeyCode::Up if !filtered.is_empty() => selected = selected.saturating_sub(1),
            KeyCode::Down if !filtered.is_empty() => {
                selected = (selected + 1).min(filtered.len() - 1)
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                clear_model_menu(&mut stdout)?;
                return Ok(None);
            }
            KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                filter.push(ch);
                selected = 0;
            }
            KeyCode::Backspace => {
                filter.pop();
                selected = 0;
            }
            KeyCode::Esc => {
                clear_model_menu(&mut stdout)?;
                return Ok(None);
            }
            KeyCode::Enter if !filtered.is_empty() => {
                let model = filtered[selected].clone();
                clear_model_menu(&mut stdout)?;
                return Ok(Some(model));
            }
            _ => continue,
        }

        render_model_menu(&mut stdout, models, current_model, &filter, selected, true)?;
    }
}

fn filtered_models<'a>(models: &'a [String], filter: &str) -> Vec<&'a String> {
    let normalized_filter = filter.to_ascii_lowercase();
    models
        .iter()
        .filter(|model| model.to_ascii_lowercase().contains(&normalized_filter))
        .collect()
}

fn render_model_menu(
    stdout: &mut impl Write,
    models: &[String],
    current_model: &str,
    filter: &str,
    selected: usize,
    redraw: bool,
) -> Result<(), String> {
    if redraw {
        execute!(stdout, MoveUp(MODEL_MENU_LINES)).map_err(|err| err.to_string())?;
    }

    execute!(stdout, Clear(ClearType::CurrentLine)).map_err(|err| err.to_string())?;
    write!(stdout, "筛选：{filter}\r\n").map_err(|err| err.to_string())?;

    let filtered = filtered_models(models, filter);
    let selected = selected.min(filtered.len().saturating_sub(1));
    let window_start = selected
        .saturating_sub(MODEL_MENU_SIZE / 2)
        .min(filtered.len().saturating_sub(MODEL_MENU_SIZE));

    for row in 0..MODEL_MENU_SIZE {
        execute!(stdout, Clear(ClearType::CurrentLine)).map_err(|err| err.to_string())?;
        if let Some(model) = filtered.get(window_start + row) {
            let current_marker = if model.as_str() == current_model {
                " *"
            } else {
                ""
            };
            if window_start + row == selected {
                write!(stdout, "{ACCENT}❯ {}{current_marker}{RESET}", model)
                    .map_err(|err| err.to_string())?;
            } else {
                write!(stdout, "  {}{current_marker}", model).map_err(|err| err.to_string())?;
            }
        } else if row == 0 && filtered.is_empty() {
            write!(stdout, "{DIM}  没有匹配的模型{RESET}").map_err(|err| err.to_string())?;
        }
        write!(stdout, "\r\n").map_err(|err| err.to_string())?;
    }

    execute!(stdout, Clear(ClearType::CurrentLine)).map_err(|err| err.to_string())?;
    write!(
        stdout,
        "{DIM}输入筛选 · ↑/↓ 选择 · Enter 切换 · Esc 取消{RESET}\r\n"
    )
    .map_err(|err| err.to_string())?;
    stdout.flush().map_err(|err| err.to_string())
}

fn clear_model_menu(stdout: &mut impl Write) -> Result<(), String> {
    execute!(stdout, MoveUp(MODEL_MENU_LINES)).map_err(|err| err.to_string())?;
    for _ in 0..MODEL_MENU_LINES {
        execute!(stdout, Clear(ClearType::CurrentLine)).map_err(|err| err.to_string())?;
        write!(stdout, "\r\n").map_err(|err| err.to_string())?;
    }
    execute!(stdout, MoveUp(MODEL_MENU_LINES)).map_err(|err| err.to_string())?;
    stdout.flush().map_err(|err| err.to_string())
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

fn select_confirmation(
    prompt: &str,
    tool_name: &str,
    allow_always: bool,
) -> Result<ConfirmationDecision, String> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        println!("助手> {prompt}\n  No (non-interactive terminal)");
        return Ok(ConfirmationDecision::Deny);
    }

    let options = if allow_always {
        vec![
            "Yes".to_string(),
            format!("Yes, and always allow {tool_name} for this session"),
            "No".to_string(),
        ]
    } else {
        vec!["Yes".to_string(), "No".to_string()]
    };
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
            KeyCode::Char('2') if allow_always => selected = 1,
            KeyCode::Char('a') if allow_always => selected = 1,
            KeyCode::Char('3') if allow_always => selected = 2,
            KeyCode::Char('2') | KeyCode::Char('n') | KeyCode::Esc => selected = options.len() - 1,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                selected = options.len() - 1
            }
            KeyCode::Enter => {
                render_confirmation_options(&mut stdout, &options, selected, true)?;
                return Ok(if selected == 0 {
                    ConfirmationDecision::AllowOnce
                } else if allow_always && selected == 1 {
                    ConfirmationDecision::AlwaysAllow
                } else {
                    ConfirmationDecision::Deny
                });
            }
            _ => continue,
        }

        render_confirmation_options(&mut stdout, &options, selected, true)?;
    }
}

fn render_confirmation_options(
    stdout: &mut impl Write,
    options: &[String],
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
    use super::{BannerLayout, ModelLocation, input_cursor_column};

    #[test]
    fn cursor_column_uses_terminal_display_width_for_chinese_input() {
        let input = "你好".chars().collect::<Vec<_>>();

        assert_eq!(input_cursor_column(&input, input.len()), 6);
    }

    #[test]
    fn model_change_rewrites_the_banner_model_row() {
        let banner = BannerLayout {
            model_location: Some(ModelLocation::Line(7)),
        };
        let mut output = Vec::new();

        banner
            .update_model(&mut output, "gpt-5.5")
            .expect("banner update should render");

        let rendered = String::from_utf8(output).expect("terminal output should be UTF-8");
        assert!(rendered.contains("Model"));
        assert!(rendered.contains("gpt-5.5"));
    }

    #[test]
    fn model_change_rewrites_only_the_dashboard_value_field() {
        let banner = BannerLayout {
            model_location: Some(ModelLocation::Dashboard {
                row: 17,
                col: 32,
                width: 10,
            }),
        };
        let mut output = Vec::new();

        banner
            .update_model(&mut output, "gpt-5.5")
            .expect("dashboard model update should render");

        let rendered = String::from_utf8(output).expect("terminal output should be UTF-8");
        assert!(rendered.contains("gpt-5.5  "));
        assert!(!rendered.contains("Model"));
    }
}
