use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};

const LOGO_LINE_DELAY: Duration = Duration::from_millis(50);
const TYPEWRITER_DELAY: Duration = Duration::from_millis(30);
const TAGLINE: &str = "BRAILLE ANIMATIONS";

pub const LOGO_ASCII: [&str; 11] = [
    "╔════════╗  ╔╗    ╔╗  ╔══════╗   ╔══════╗   ╔══════╗   ╔════════╗",
    "║ ██████ ║  ║║    ║║  ║ ████ ╚╗  ║ ████ ║   ║ ████ ╚╗  ║ ██████ ║",
    "║ ██  ██ ║  ║║    ║║  ║ ██ ╔╗ ║  ║ ██ ██║   ║ ██ ╔╗ ║  ║ ██  ██ ║",
    "║ ██████ ╠══╣║    ║╠══╣ ████ ╔╝══╣ ██ ██╠═══╣ ████ ╔╝══╣ ██████ ║",
    "║ ██  ██ ║  ║║    ║║  ║ ██╔═██╗  ║ ██ ██║   ║ ██╔═██╗  ║ ██  ██ ║",
    "║ ██  ██ ║  ║╚════╝║  ║ ██║ ╚██╗ ║ ████ ║   ║ ██║ ╚██╗ ║ ██  ██ ║",
    "╚═╝  ╚═╝  ╚════════╝  ╚═╝   ╚═╝  ╚══════╝   ╚═╝   ╚═╝  ╚═╝  ╚═╝",
    "   ╚═╦═══╦═╝    ╚═════╦════╝       ╚══╦══╝       ╚═════╦════╝",
    "     ║ █ ║            ║   ████════╗   ║   ╔════████   ║",
    "     ╚═══╝            ╚═══════════╝   ╚═══╝   ╚════════╝",
    "        ⠁⠂⠄                         AURORA                         ⠠⠐⠈",
];

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
        execute!(stdout, Hide, Clear(ClearType::All), MoveTo(0, 0))?;
        stdout.flush()?;
        Ok(Self)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, Show);
        let _ = stdout.flush();
    }
}

fn install_interrupt_restore_handler() -> Result<(), String> {
    ctrlc::set_handler(|| {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, Show);
        let _ = stdout.flush();
        std::process::exit(130);
    })
    .map_err(|err| format!("failed to install terminal cleanup handler: {err}"))
}

fn play_logo_scan() -> io::Result<()> {
    let mut stdout = io::stdout();

    for (row, line) in LOGO_ASCII.iter().enumerate() {
        execute!(stdout, MoveTo(0, row as u16), Print(*line))?;
        io::stdout().flush().unwrap();
        thread::sleep(LOGO_LINE_DELAY);
    }

    Ok(())
}

fn play_tagline_typewriter() -> io::Result<()> {
    let row = LOGO_ASCII.len() as u16 + 1;
    let mut stdout = io::stdout();
    execute!(stdout, MoveTo(0, row))?;

    for ch in TAGLINE.chars() {
        execute!(stdout, Print(ch))?;
        io::stdout().flush().unwrap();
        thread::sleep(TYPEWRITER_DELAY);
    }

    execute!(stdout, MoveTo(0, row + 2))?;
    stdout.flush()?;
    Ok(())
}
