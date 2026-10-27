use std::sync::OnceLock;

use ratatui::style::{Color, Modifier, Style};
use syntect::easy::HighlightLines;
use syntect::highlighting::{
    FontStyle, ScopeSelectors, StyleModifier, Theme, ThemeItem, ThemeSettings,
};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::core::AppTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaletteKind {
    Dark,
    Light,
}

fn syntaxes() -> &'static SyntaxSet {
    static SET: OnceLock<SyntaxSet> = OnceLock::new();
    SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

pub fn highlight(
    code: &str,
    language: Option<&str>,
    theme: AppTheme,
    enabled: bool,
    width: usize,
) -> Vec<ratatui::text::Line<'static>> {
    let max_lines = 800;
    if !enabled {
        return code
            .lines()
            .take(max_lines)
            .map(|line| {
                ratatui::text::Line::from(ratatui::text::Span::styled(
                    truncate(line, width),
                    Style::default().fg(theme.fg_bright),
                ))
            })
            .collect();
    }

    let set = syntaxes();
    let syntax = language
        .and_then(|lang| set.find_syntax_by_token(lang))
        .unwrap_or_else(|| set.find_syntax_plain_text());
    let syntect_theme = build_theme(theme);
    let mut highlighter = HighlightLines::new(syntax, &syntect_theme);
    let mut out = Vec::new();
    for line in LinesWithEndings::from(code).take(max_lines) {
        let ranges = highlighter.highlight_line(line, set).unwrap_or_default();
        let mut spans = Vec::new();
        for (style, text) in ranges {
            let text = text.trim_end_matches(['\r', '\n']);
            if text.is_empty() {
                continue;
            }
            spans.push(ratatui::text::Span::styled(
                text.to_string(),
                convert_style(style),
            ));
        }
        if spans.is_empty() {
            spans.push(ratatui::text::Span::raw(" "));
        }
        let mut line = ratatui::text::Line::from(spans);
        // Keep the code inside the terminal and preserve a visible right edge.
        let mut used = 0usize;
        line.spans.retain(|span| {
            let w = unicode_width::UnicodeWidthStr::width(span.content.as_ref());
            if used >= width {
                false
            } else {
                used += w;
                true
            }
        });
        out.push(line);
    }
    if out.is_empty() {
        out.push(ratatui::text::Line::from(" "));
    }
    out
}

fn convert_style(style: syntect::highlighting::Style) -> Style {
    let mut out = Style::default().fg(Color::Rgb(
        style.foreground.r,
        style.foreground.g,
        style.foreground.b,
    ));
    if style.font_style.contains(FontStyle::BOLD) {
        out = out.add_modifier(Modifier::BOLD);
    }
    if style.font_style.contains(FontStyle::ITALIC) {
        out = out.add_modifier(Modifier::ITALIC);
    }
    if style.font_style.contains(FontStyle::UNDERLINE) {
        out = out.add_modifier(Modifier::UNDERLINED);
    }
    out
}

fn build_theme(theme: AppTheme) -> Theme {
    let kind = if luminance(theme.bg) > 0.5 {
        PaletteKind::Light
    } else {
        PaletteKind::Dark
    };
    let mut scopes = Vec::new();
    let mut item = |scope: &str, fg: Color, font: Option<FontStyle>| {
        if let Ok(selectors) = scope.parse::<ScopeSelectors>() {
            scopes.push(ThemeItem {
                scope: selectors,
                style: StyleModifier {
                    foreground: Some(syntect_color(fg)),
                    background: None,
                    font_style: font,
                },
            });
        }
    };

    item("comment", theme.fg_dim, Some(FontStyle::ITALIC));
    item("string", theme.success, None);
    item("constant.numeric", theme.tertiary_accent, None);
    item("constant", theme.warning, None);
    item("keyword", theme.accent, Some(FontStyle::BOLD));
    item("storage", theme.accent, Some(FontStyle::BOLD));
    item("entity.name.function", theme.secondary_accent, None);
    item(
        "entity.name.type",
        theme.secondary_accent,
        Some(FontStyle::ITALIC),
    );
    item("variable", theme.fg, None);
    item("support", theme.tertiary_accent, None);
    item("markup.heading", theme.accent, Some(FontStyle::BOLD));
    item("markup.italic", theme.fg, Some(FontStyle::ITALIC));
    item("markup.bold", theme.fg, Some(FontStyle::BOLD));
    item("invalid", theme.error, Some(FontStyle::UNDERLINE));

    let _ = kind;
    Theme {
        name: Some(format!("gtw-{}", theme.accent)),
        author: Some("gtw".to_string()),
        settings: ThemeSettings {
            foreground: Some(syntect_color(theme.fg)),
            background: Some(syntect_color(theme.bg)),
            ..Default::default()
        },
        scopes,
    }
}

fn syntect_color(color: Color) -> syntect::highlighting::Color {
    let (r, g, b) = match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::White => (255, 255, 255),
        Color::Red => (220, 80, 80),
        Color::Green => (80, 180, 90),
        Color::Yellow => (220, 200, 80),
        Color::Blue => (80, 120, 220),
        Color::Magenta => (200, 90, 200),
        Color::Cyan => (70, 190, 200),
        Color::Gray => (160, 160, 160),
        Color::DarkGray => (80, 80, 80),
        Color::LightRed => (255, 120, 120),
        Color::LightGreen => (140, 230, 140),
        Color::LightYellow => (250, 240, 140),
        Color::LightBlue => (140, 180, 255),
        Color::LightMagenta => (240, 150, 240),
        Color::LightCyan => (150, 240, 250),
        _ => (200, 200, 200),
    };
    syntect::highlighting::Color { r, g, b, a: 255 }
}

fn luminance(color: Color) -> f32 {
    let (r, g, b) = match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::White => (255, 255, 255),
        _ => (0, 0, 0),
    };
    (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) / 255.0
}

fn truncate(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut used = 0;
    for c in text.chars() {
        let w = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if used + w > width {
            break;
        }
        out.push(c);
        used += w;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_rust() {
        let theme = AppTheme {
            bg: Color::Rgb(10, 10, 10),
            pane_bg: Color::Rgb(10, 10, 10),
            picker_bg: Color::Rgb(20, 20, 20),
            elevated_bg: Color::Rgb(30, 30, 30),
            muted_border: Color::Rgb(60, 60, 60),
            fg: Color::Rgb(220, 220, 220),
            fg_dim: Color::Rgb(120, 120, 120),
            fg_bright: Color::Rgb(255, 255, 255),
            accent: Color::Rgb(80, 180, 240),
            secondary_accent: Color::Rgb(180, 120, 240),
            tertiary_accent: Color::Rgb(240, 180, 80),
            error: Color::Rgb(240, 80, 80),
            warning: Color::Rgb(240, 200, 80),
            success: Color::Rgb(80, 220, 120),
            selection_fg: Color::Rgb(10, 10, 10),
            selection_bg: Color::Rgb(80, 180, 240),
            border: Color::Rgb(60, 60, 60),
            border_active: Color::Rgb(80, 180, 240),
            volume_low: Color::Rgb(80, 220, 120),
            volume_medium: Color::Rgb(240, 200, 80),
            volume_high: Color::Rgb(240, 80, 80),
            sidebar_active_border: Color::Rgb(80, 180, 240),
            notification_border: Color::Rgb(80, 180, 240),
            monochromatic: false,
        };
        let lines = highlight("fn main() {}\n", Some("rs"), theme, true, 80);
        assert!(!lines.is_empty());
    }
}
