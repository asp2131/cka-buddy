use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
};

pub const CANONICAL_VIEWPORT_WIDTH: u16 = 160;
pub const CANONICAL_VIEWPORT_HEIGHT: u16 = 48;

pub const BG_DARK: Color = Color::Rgb(14, 18, 28);
pub const BG_PANEL: Color = Color::Rgb(20, 25, 38);
pub const BG_ACCENT: Color = Color::Rgb(28, 34, 52);

pub fn centered_clamped_viewport(area: Rect) -> Rect {
    let width = area.width.min(CANONICAL_VIEWPORT_WIDTH);
    let height = area.height.min(CANONICAL_VIEWPORT_HEIGHT);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;

    Rect::new(x, y, width, height)
}

pub struct UiStyle;

impl UiStyle {
    pub const DEFAULT: Style = Style {
        fg: None,
        bg: None,
        underline_color: None,
        add_modifier: Modifier::empty(),
        sub_modifier: Modifier::empty(),
    };

    pub const SELECTED: Style = Self::DEFAULT.bg(Color::Rgb(40, 44, 62));
    pub const HEADER: Style = Self::DEFAULT.fg(Color::Rgb(50, 130, 240));
    pub const HIGHLIGHT: Style = Self::DEFAULT.fg(Color::Rgb(0, 200, 210));
    pub const OK: Style = Self::DEFAULT.fg(Color::Rgb(80, 220, 120));
    pub const WARNING: Style = Self::DEFAULT.fg(Color::Rgb(255, 185, 50));
    pub const ERROR: Style = Self::DEFAULT.fg(Color::Rgb(240, 70, 70));
    pub const MUTED: Style = Self::DEFAULT.fg(Color::Rgb(80, 88, 105));
    pub const TEXT_PRIMARY: Style = Self::DEFAULT.fg(Color::Rgb(230, 235, 245));
    pub const TEXT_SECONDARY: Style = Self::DEFAULT.fg(Color::Rgb(140, 150, 170));
    pub const COMMAND: Style = Self::DEFAULT.fg(Color::Rgb(255, 220, 100));
    pub const PROMPT: Style = Self::DEFAULT.fg(Color::Rgb(0, 220, 230));
    pub const BORDER: Style = Self::DEFAULT.fg(Color::Rgb(55, 62, 80));
    pub const FOOTER_KEY: Style = Self::DEFAULT
        .fg(Color::Rgb(20, 24, 34))
        .bg(Color::Rgb(150, 160, 180));
    pub const FOOTER_DESC: Style = Self::DEFAULT
        .fg(Color::Rgb(205, 210, 225))
        .bg(Color::Rgb(36, 41, 56));
    pub const HOVER_HELP: Style = Self::DEFAULT.fg(Color::Rgb(170, 180, 200));

    pub const BORDER_ACTIVE: Style = Self::DEFAULT.fg(Color::Rgb(80, 100, 140));
    pub const PANEL_BG: Style = Self::DEFAULT.bg(Color::Rgb(20, 25, 38));
    pub const ACCENT_BG: Style = Self::DEFAULT.bg(Color::Rgb(28, 34, 52));

    pub const GLOW_CYAN: Style = Self::DEFAULT.fg(Color::Rgb(0, 220, 230));
    pub const GLOW_GREEN: Style = Self::DEFAULT.fg(Color::Rgb(80, 220, 120));
    pub const GLOW_AMBER: Style = Self::DEFAULT.fg(Color::Rgb(255, 185, 50));

    pub const DOMAIN_STORAGE: Style = Self::DEFAULT.fg(Color::Rgb(180, 120, 255));
    pub const DOMAIN_NETWORKING: Style = Self::DEFAULT.fg(Color::Rgb(100, 200, 255));
    pub const DOMAIN_WORKLOADS: Style = Self::DEFAULT.fg(Color::Rgb(255, 150, 100));
    pub const DOMAIN_CLUSTER: Style = Self::DEFAULT.fg(Color::Rgb(120, 230, 180));
    pub const DOMAIN_SECURITY: Style = Self::DEFAULT.fg(Color::Rgb(255, 100, 130));
    pub const DOMAIN_TROUBLESHOOTING: Style = Self::DEFAULT.fg(Color::Rgb(255, 210, 80));

    pub const DIFF_EASY: Style = Self::DEFAULT.fg(Color::Rgb(80, 220, 120));
    pub const DIFF_MEDIUM: Style = Self::DEFAULT.fg(Color::Rgb(255, 185, 50));
    pub const DIFF_HARD: Style = Self::DEFAULT.fg(Color::Rgb(240, 70, 70));

    pub const STAR_DIM: Style = Self::DEFAULT.fg(Color::Rgb(50, 55, 70));
    pub const STAR_BRIGHT: Style = Self::DEFAULT.fg(Color::Rgb(200, 210, 230));
}
