use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{self, Clear, ClearType};

const LOGO_LINE_DELAY: Duration = Duration::from_millis(50);
const TYPEWRITER_DELAY: Duration = Duration::from_millis(30);
const TAGLINE: &str = "BRAILLE ANIMATIONS";

pub const LOGO_ASCII: [&str; 7] = [
    " ██████╔╗   ██    ██╗   ███████╗   ██████╗    ███████╗   ██████╔╗ ",
    "██    ██║   ██    ██║   ██    ██╗ ██    ██╗   ██    ██╗ ██    ██║ ",
    "██    ██║   ██    ██║   ██    ██║ ██    ██║   ██    ██║ ██    ██║ ",
    "████████╣   ██    ██║   ███████╔╝ ██    ██║   ███████╔╝ ████████╣ ",
    "██    ██║   ██    ██║   ██  ██╔╝  ██    ██║   ██  ██╔╝  ██    ██║ ",
    "██    ██║   ██    ██║   ██   ██╗  ██    ██║   ██   ██╗  ██    ██║ ",
    "██    ██╝    ██████╝    ██    ██╗  ██████╝    ██    ██╗ ██    ██╝ ",
];

pub const END_ROW: u16 = 5 + LOGO_ASCII.len() as u16 + 5;

pub fn play() -> Result<(), String> {
    install_interrupt_restore_handler()?;
    let _terminal = TerminalSession::enter().map_err(|err| err.to_string())?;
    play_logo_scan().map_err(|err| err.to_string())?;
    play_tagline_typewriter().map_err(|err| err.to_string())?;
    Ok(())
}

struct TerminalSession;

impl TerminalSession {
    fn enter() -> io::Result<Self> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            Hide,
            ResetColor,
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

fn play_logo_scan() -> io::Result<()> {
    let mut stdout = io::stdout();
    let (terminal_width, _) = terminal::size().unwrap_or((120, 40));
    let logo_width = visible_width(&LOGO_ASCII);
    let start_col = center_col(terminal_width, logo_width);
    let start_row = 5;

    draw_braille_corners(&mut stdout, terminal_width)?;

    for (row, line) in LOGO_ASCII.iter().enumerate() {
        let y = start_row + row as u16;
        execute!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            MoveTo(start_col + 2, y + 2),
            Print(shadow_line(line))
        )?;
        execute!(stdout, ResetColor, MoveTo(start_col, y), Print(*line))?;
        io::stdout().flush().unwrap();
        thread::sleep(LOGO_LINE_DELAY);
    }

    Ok(())
}

fn play_tagline_typewriter() -> io::Result<()> {
    let mut stdout = io::stdout();
    let (terminal_width, _) = terminal::size().unwrap_or((120, 40));
    let logo_width = visible_width(&LOGO_ASCII);
    let start_col = center_col(terminal_width, logo_width);
    let row = 5 + LOGO_ASCII.len() as u16 + 3;
    execute!(
        stdout,
        SetForegroundColor(Color::Grey),
        MoveTo(start_col, row)
    )?;

    for ch in TAGLINE.chars() {
        execute!(stdout, Print(ch))?;
        io::stdout().flush().unwrap();
        thread::sleep(TYPEWRITER_DELAY);
    }

    execute!(stdout, MoveTo(0, END_ROW))?;
    stdout.flush()?;
    Ok(())
}

fn draw_braille_corners(stdout: &mut impl Write, terminal_width: u16) -> io::Result<()> {
    let right_col = terminal_width.saturating_sub(13);
    let left_corner = ["⠉⠉⠉⠁", "⠇", "⠇", "⠁"];
    let right_corner = ["⠈⠉⠉⠉", "   ⠸", "   ⠸", "   ⠈"];

    for (index, line) in left_corner.iter().enumerate() {
        execute!(
            stdout,
            ResetColor,
            MoveTo(5, 2 + index as u16),
            Print(*line)
        )?;
    }

    for (index, line) in right_corner.iter().enumerate() {
        execute!(
            stdout,
            ResetColor,
            MoveTo(right_col, 2 + index as u16),
            Print(*line)
        )?;
    }

    stdout.flush()?;
    Ok(())
}

fn center_col(terminal_width: u16, logo_width: usize) -> u16 {
    terminal_width
        .saturating_sub(logo_width as u16)
        .saturating_div(2)
        .max(2)
}

fn visible_width(lines: &[&str]) -> usize {
    lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
}

fn shadow_line(line: &str) -> String {
    line.chars()
        .map(|ch| match ch {
            '█' => '░',
            '╔' | '╗' | '╚' | '╝' | '═' | '║' | '╠' | '╣' | '╦' | '╩' => ' ',
            other => other,
        })
        .collect()
}
