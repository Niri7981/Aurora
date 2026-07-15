use crossterm::style::Color;

pub const RESET: &str = "\x1b[0m";
pub const DIM: &str = "\x1b[2m";
pub const BOLD: &str = "\x1b[1m";
pub const ACCENT: &str = "\x1b[38;2;80;163;177m";
pub const MUTED: &str = "\x1b[38;2;58;123;139m";

pub const SCREEN_BACKGROUND: Color = Color::Rgb {
    r: 15,
    g: 23,
    b: 26,
};
pub const PANEL_BACKGROUND: Color = Color::Rgb {
    r: 17,
    g: 24,
    b: 32,
};
pub const PANEL_BORDER: Color = Color::Rgb {
    r: 36,
    g: 70,
    b: 82,
};
pub const PANEL_GRID: Color = Color::Rgb {
    r: 24,
    g: 47,
    b: 55,
};
pub const PANEL_TEXT: Color = Color::Rgb {
    r: 142,
    g: 166,
    b: 172,
};
pub const STATUS_READY: Color = Color::Rgb {
    r: 113,
    g: 198,
    b: 163,
};
pub const LOGO_HIGHLIGHT: Color = Color::Rgb {
    r: 108,
    g: 203,
    b: 214,
};
pub const LOGO_MAIN: Color = Color::Rgb {
    r: 80,
    g: 163,
    b: 177,
};
pub const LOGO_SIDE: Color = Color::Rgb {
    r: 68,
    g: 142,
    b: 158,
};
pub const LOGO_TRACE_MID: Color = Color::Rgb {
    r: 58,
    g: 123,
    b: 139,
};
pub const LOGO_TRACE_BACK: Color = Color::Rgb {
    r: 38,
    g: 86,
    b: 100,
};
pub const LOGO_BLOCK_SHADOW: Color = Color::Rgb {
    r: 43,
    g: 97,
    b: 112,
};
pub const LOGO_SHADOW: Color = Color::Rgb {
    r: 24,
    g: 44,
    b: 48,
};
