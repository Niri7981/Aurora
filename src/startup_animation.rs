use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{self, Clear, ClearType};

use crate::theme::{
    LOGO_BLOCK_SHADOW, LOGO_HIGHLIGHT, LOGO_MAIN, LOGO_SHADOW, LOGO_SIDE, LOGO_TRACE_BACK,
    LOGO_TRACE_MID, PANEL_BACKGROUND, PANEL_BORDER, PANEL_GRID, PANEL_TEXT, SCREEN_BACKGROUND,
    STATUS_READY,
};

const LOGO_LINE_DELAY: Duration = Duration::from_millis(42);
const TYPEWRITER_DELAY: Duration = Duration::from_millis(26);
const TAGLINE: &str = "BRAILLE ANIMATIONS";

const PANEL_WIDTH: u16 = 78;
const PANEL_TOP: u16 = 1;
const PANEL_BOTTOM: u16 = 20;
const LOGO_TOP: u16 = 4;
const LOGO_LAYER_OFFSET: u16 = 2;
const TAGLINE_ROW: u16 = 15;
const STATUS_ROW: u16 = 17;
const WORKSPACE_ROW: u16 = 18;
const TELEMETRY_ROW: u16 = 19;
pub const END_ROW: u16 = 22;

pub const LOGO_ASCII: [&str; 9] = [
    "  ▄████▄     ██▌  ▐██    ██████▄      ▄████▄     ██████▄      ▄████▄  ",
    " ▟██▀▀██▙    ██▌  ▐██    ██▌  ▐██    ▟██▀▀██▙    ██▌  ▐██    ▟██▀▀██▙ ",
    " ██▌  ▐██    ██▌  ▐██    ██▌  ▐██    ██▌  ▐██    ██▌  ▐██    ██▌  ▐██ ",
    " ██▌  ▐██    ██▌  ▐██    ██████▀     ██▌  ▐██    ██████▀     ██▌  ▐██ ",
    " ████████    ██▌  ▐██    ██▌▜█▄      ██▌  ▐██    ██▌▜█▄      ████████ ",
    " ██▌  ▐██    ██▌  ▐██    ██▌ ▜█▄     ██▌  ▐██    ██▌ ▜█▄     ██▌  ▐██ ",
    " ██▌  ▐██    ██▌  ▐██    ██▌  ▜██    ██▌  ▐██    ██▌  ▜██    ██▌  ▐██ ",
    " ██▌  ▐██    ▜██▄▄██▛    ██▌  ▐██    ▜██▄▄██▛    ██▌  ▐██    ██▌  ▐██ ",
    " ▀▀    ▀▀     ▀████▀     ▀▀    ▀▀     ▀████▀     ▀▀    ▀▀    ▀▀    ▀▀ ",
];

const CIRCUIT_ASCII: [&str; 10] = [
    "        ┌┐          ┌┐          ┌┐          ┌┐          ┌┐          ┌┐",
    "        ││           │           │           │           │          ││",
    "         │           │      ┌────┤           │      ┌────┤           │",
    "    ┌────┤           │      └──┐ │           │      └──┐ │      ┌────┤",
    "    └─┐  │           │         └─┤           │         └─┤      └─┐  │",
    "      └──┘           │          ┌┘           │          ┌┘        └──┘",
    "                     │          └┐           │          └┐            ",
    "         ╵          ┌┘           │          ┌┘           │           ╵",
    "              ┌─────┘            ╵    ┌─────┘            ╵            ",
    "  └──────┘    └──────┘    └──────┘    └──────┘    └──────┘    └──────┘",
];

#[derive(Debug, Clone, Copy)]
pub struct StartupLayout {
    pub model_row: u16,
    pub model_col: u16,
    pub model_width: usize,
}

pub fn play(provider: &str, model: &str, workspace: &str) -> Result<StartupLayout, String> {
    install_interrupt_restore_handler()?;
    let _terminal = TerminalSession::enter().map_err(|err| err.to_string())?;
    let (terminal_width, _) = terminal::size().unwrap_or((80, 24));
    let panel_col = center_col(terminal_width, PANEL_WIDTH as usize);
    let mut stdout = io::stdout();

    draw_panel_shell(&mut stdout, panel_col).map_err(|err| err.to_string())?;
    play_logo_scan(&mut stdout, panel_col).map_err(|err| err.to_string())?;
    play_tagline_typewriter(&mut stdout, panel_col).map_err(|err| err.to_string())?;
    let layout = draw_runtime_status(&mut stdout, panel_col, provider, model, workspace)
        .map_err(|err| err.to_string())?;

    execute!(stdout, ResetColor, MoveTo(0, END_ROW)).map_err(|err| err.to_string())?;
    stdout.flush().map_err(|err| err.to_string())?;
    Ok(layout)
}

struct TerminalSession;

impl TerminalSession {
    fn enter() -> io::Result<Self> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            Hide,
            SetBackgroundColor(SCREEN_BACKGROUND),
            Clear(ClearType::All),
            MoveTo(0, 0)
        )?;
        stdout.flush()?;
        Ok(Self)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, ResetColor, Show);
        let _ = stdout.flush();
    }
}

fn install_interrupt_restore_handler() -> Result<(), String> {
    ctrlc::set_handler(|| {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, ResetColor, Show);
        let _ = stdout.flush();
        std::process::exit(130);
    })
    .map_err(|err| format!("failed to install terminal cleanup handler: {err}"))
}

fn draw_panel_shell(stdout: &mut impl Write, panel_col: u16) -> io::Result<()> {
    let panel_fill = " ".repeat(PANEL_WIDTH as usize);
    for row in PANEL_TOP..=PANEL_BOTTOM {
        execute!(
            stdout,
            SetBackgroundColor(PANEL_BACKGROUND),
            MoveTo(panel_col, row),
            Print(&panel_fill)
        )?;
    }

    draw_frame_row(stdout, panel_col, PANEL_TOP, '╭', '─', '╮')?;
    draw_frame_row(stdout, panel_col, 3, '├', '─', '┤')?;
    draw_frame_row(stdout, panel_col, 16, '├', '─', '┤')?;
    draw_frame_row(stdout, panel_col, PANEL_BOTTOM, '╰', '─', '╯')?;

    for row in (LOGO_TOP..TAGLINE_ROW).step_by(2) {
        let grid = "·   ".repeat(((PANEL_WIDTH - 2) / 4) as usize + 1);
        draw_layer(
            stdout,
            PANEL_GRID,
            panel_col + 1,
            row,
            &fit_text(&grid, (PANEL_WIDTH - 2) as usize),
        )?;
    }

    draw_panel_text(
        stdout,
        panel_col,
        2,
        "AURORA // BOOT.CONSOLE",
        "● SYSTEM READY",
    )?;
    stdout.flush()
}

fn play_logo_scan(stdout: &mut impl Write, panel_col: u16) -> io::Result<()> {
    let logo_width = visible_width(&LOGO_ASCII) + LOGO_LAYER_OFFSET as usize;
    let start_col = panel_col + center_col(PANEL_WIDTH, logo_width);

    for (row, circuit) in CIRCUIT_ASCII.iter().enumerate() {
        let y = LOGO_TOP + row as u16;

        if let Some(logo) = LOGO_ASCII.get(row) {
            draw_layer(
                stdout,
                LOGO_SHADOW,
                start_col + 2,
                y + 2,
                &shade_line(logo, '░'),
            )?;
            draw_layer(
                stdout,
                LOGO_BLOCK_SHADOW,
                start_col + 1,
                y + 1,
                &shade_line(logo, '▒'),
            )?;
        }

        draw_layer(stdout, LOGO_TRACE_BACK, start_col + 1, y + 1, circuit)?;
        draw_layer(stdout, LOGO_HIGHLIGHT, start_col, y, circuit)?;

        if let Some(logo) = LOGO_ASCII.get(row) {
            draw_layer(stdout, logo_color(row), start_col, y, &segmented_line(logo))?;
        }

        io::stdout().flush().unwrap();
        thread::sleep(LOGO_LINE_DELAY);
    }

    Ok(())
}

fn play_tagline_typewriter(stdout: &mut impl Write, panel_col: u16) -> io::Result<()> {
    execute!(
        stdout,
        SetBackgroundColor(PANEL_BACKGROUND),
        SetForegroundColor(LOGO_TRACE_MID),
        MoveTo(panel_col + 4, TAGLINE_ROW),
        Print("⠿  ")
    )?;

    for ch in TAGLINE.chars() {
        execute!(stdout, Print(ch))?;
        io::stdout().flush().unwrap();
        thread::sleep(TYPEWRITER_DELAY);
    }

    execute!(
        stdout,
        SetForegroundColor(PANEL_TEXT),
        MoveTo(panel_col + PANEL_WIDTH - 20, TAGLINE_ROW),
        Print("CRT/TEAL  SEQ.03")
    )?;
    stdout.flush()
}

fn draw_runtime_status(
    stdout: &mut impl Write,
    panel_col: u16,
    provider: &str,
    model: &str,
    workspace: &str,
) -> io::Result<StartupLayout> {
    for row in STATUS_ROW..PANEL_BOTTOM {
        draw_panel_borders(stdout, panel_col, row)?;
    }

    draw_label(stdout, panel_col + 3, STATUS_ROW, "PROVIDER")?;
    draw_value(stdout, panel_col + 12, STATUS_ROW, provider, 10)?;
    draw_label(stdout, panel_col + 25, STATUS_ROW, "MODEL")?;
    draw_value(stdout, panel_col + 32, STATUS_ROW, model, 18)?;
    draw_label(stdout, panel_col + 53, STATUS_ROW, "MODE")?;
    draw_value(stdout, panel_col + 59, STATUS_ROW, "CLI", 4)?;
    execute!(
        stdout,
        SetBackgroundColor(PANEL_BACKGROUND),
        SetForegroundColor(STATUS_READY),
        MoveTo(panel_col + 67, STATUS_ROW),
        Print("● READY")
    )?;

    draw_label(stdout, panel_col + 3, WORKSPACE_ROW, "WORKSPACE")?;
    draw_value(stdout, panel_col + 13, WORKSPACE_ROW, workspace, 60)?;

    draw_label(stdout, panel_col + 3, TELEMETRY_ROW, "BUS")?;
    execute!(
        stdout,
        SetBackgroundColor(PANEL_BACKGROUND),
        SetForegroundColor(LOGO_MAIN),
        MoveTo(panel_col + 8, TELEMETRY_ROW),
        Print("▁▂▃▅▇▆▄▅▇")
    )?;
    draw_label(stdout, panel_col + 20, TELEMETRY_ROW, "CONTEXT")?;
    execute!(
        stdout,
        SetBackgroundColor(PANEL_BACKGROUND),
        SetForegroundColor(LOGO_SIDE),
        MoveTo(panel_col + 29, TELEMETRY_ROW),
        Print("▰▰▰▰▰▰▱▱")
    )?;
    draw_label(stdout, panel_col + 40, TELEMETRY_ROW, "HARNESS")?;
    draw_value(stdout, panel_col + 49, TELEMETRY_ROW, "ARMED", 7)?;
    draw_label(stdout, panel_col + 59, TELEMETRY_ROW, "LINK")?;
    draw_value(stdout, panel_col + 65, TELEMETRY_ROW, "LOCAL", 8)?;
    stdout.flush()?;

    Ok(StartupLayout {
        model_row: STATUS_ROW,
        model_col: panel_col + 32,
        model_width: 18,
    })
}

fn draw_frame_row(
    stdout: &mut impl Write,
    panel_col: u16,
    row: u16,
    left: char,
    fill: char,
    right: char,
) -> io::Result<()> {
    let line = format!(
        "{left}{}{right}",
        fill.to_string().repeat((PANEL_WIDTH - 2) as usize)
    );
    execute!(
        stdout,
        SetBackgroundColor(PANEL_BACKGROUND),
        SetForegroundColor(PANEL_BORDER),
        MoveTo(panel_col, row),
        Print(line)
    )
}

fn draw_panel_text(
    stdout: &mut impl Write,
    panel_col: u16,
    row: u16,
    left: &str,
    right: &str,
) -> io::Result<()> {
    draw_panel_borders(stdout, panel_col, row)?;
    execute!(
        stdout,
        SetBackgroundColor(PANEL_BACKGROUND),
        SetForegroundColor(PANEL_TEXT),
        MoveTo(panel_col + 3, row),
        Print(left),
        SetForegroundColor(STATUS_READY),
        MoveTo(
            panel_col + PANEL_WIDTH - right.chars().count() as u16 - 3,
            row
        ),
        Print(right)
    )
}

fn draw_panel_borders(stdout: &mut impl Write, panel_col: u16, row: u16) -> io::Result<()> {
    execute!(
        stdout,
        SetBackgroundColor(PANEL_BACKGROUND),
        SetForegroundColor(PANEL_BORDER),
        MoveTo(panel_col, row),
        Print('│'),
        MoveTo(panel_col + PANEL_WIDTH - 1, row),
        Print('│')
    )
}

fn draw_label(stdout: &mut impl Write, x: u16, y: u16, label: &str) -> io::Result<()> {
    execute!(
        stdout,
        SetBackgroundColor(PANEL_BACKGROUND),
        SetForegroundColor(PANEL_TEXT),
        MoveTo(x, y),
        Print(label)
    )
}

fn draw_value(
    stdout: &mut impl Write,
    x: u16,
    y: u16,
    value: &str,
    width: usize,
) -> io::Result<()> {
    execute!(
        stdout,
        SetBackgroundColor(PANEL_BACKGROUND),
        SetForegroundColor(LOGO_HIGHLIGHT),
        MoveTo(x, y),
        Print(fit_text(value, width))
    )
}

fn draw_layer(stdout: &mut impl Write, color: Color, x: u16, y: u16, line: &str) -> io::Result<()> {
    let mut run_start = 0;
    let mut run = String::new();

    for (column, ch) in line.chars().chain(std::iter::once(' ')).enumerate() {
        if ch == ' ' {
            if !run.is_empty() {
                execute!(
                    stdout,
                    SetBackgroundColor(PANEL_BACKGROUND),
                    SetForegroundColor(color),
                    MoveTo(x + run_start as u16, y),
                    Print(&run)
                )?;
                run.clear();
            }
        } else {
            if run.is_empty() {
                run_start = column;
            }
            run.push(ch);
        }
    }

    Ok(())
}

fn center_col(container_width: u16, content_width: usize) -> u16 {
    container_width
        .saturating_sub(content_width as u16)
        .saturating_div(2)
}

fn visible_width(lines: &[&str]) -> usize {
    lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
}

fn logo_color(row: usize) -> Color {
    match row {
        0 | 1 | 4 => LOGO_HIGHLIGHT,
        2 | 3 | 5 | 6 => LOGO_MAIN,
        7 => LOGO_SIDE,
        _ => LOGO_TRACE_MID,
    }
}

fn shade_line(line: &str, shade: char) -> String {
    line.chars()
        .map(|ch| if ch == ' ' { ' ' } else { shade })
        .collect()
}

fn segmented_line(line: &str) -> String {
    line.chars()
        .map(|ch| if ch == '█' { '▉' } else { ch })
        .collect()
}

pub fn fit_text(text: &str, width: usize) -> String {
    let mut output = text.chars().take(width).collect::<String>();
    let used = output.chars().count();
    output.extend(std::iter::repeat_n(' ', width.saturating_sub(used)));
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logo_and_circuit_share_the_same_compact_canvas() {
        assert!(LOGO_ASCII.iter().all(|line| line.chars().count() == 70));
        assert!(CIRCUIT_ASCII.iter().all(|line| line.chars().count() == 70));
        assert_eq!(LOGO_ASCII.len(), 9);
    }

    #[test]
    fn title_uses_cut_pixel_shapes_instead_of_plain_blocks_only() {
        let logo = LOGO_ASCII.join("\n");
        assert!(logo.contains(['▄', '▀', '▟', '▙', '▜', '▛']));
        assert!(!logo.contains("AURORA"));
    }

    #[test]
    fn fit_text_truncates_and_pads_dashboard_fields() {
        assert_eq!(fit_text("gpt-5", 8), "gpt-5   ");
        assert_eq!(fit_text("123456789", 5), "12345");
    }
}
