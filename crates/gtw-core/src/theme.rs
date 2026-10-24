use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct AppTheme {
    pub bg: Color,
    pub pane_bg: Color,
    pub picker_bg: Color,
    pub elevated_bg: Color,
    pub muted_border: Color,
    pub fg: Color,
    pub fg_dim: Color,
    pub fg_bright: Color,
    pub accent: Color,
    pub secondary_accent: Color,
    pub tertiary_accent: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub selection_fg: Color,
    pub selection_bg: Color,
    pub border: Color,
    pub border_active: Color,
    pub volume_low: Color,
    pub volume_medium: Color,
    pub volume_high: Color,
    pub sidebar_active_border: Color,
    pub notification_border: Color,
    pub monochromatic: bool,
}

#[derive(Debug, Clone)]
pub struct ThemeEntry {
    pub name: String,
    pub light: bool,
    pub theme: AppTheme,
}
