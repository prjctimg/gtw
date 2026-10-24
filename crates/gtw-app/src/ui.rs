use gtw_core::{AppState, Screen};
use gtw_render::{merged_themes, render_loadable, render_skeleton, render_wordmark};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs};

pub fn draw(f: &mut Frame, state: &AppState, themes: &[gtw_core::ThemeEntry], theme_idx: usize) {
    let size = f.area();
    let theme = themes.get(theme_idx).map(|t| t.theme).unwrap_or_else(|| {
        gtw_core::ThemeEntry {
            name: "default".to_string(),
            light: false,
            theme: gtw_core::AppTheme {
                bg: Color::Black,
                pane_bg: Color::Black,
                picker_bg: Color::DarkGray,
                elevated_bg: Color::Gray,
                muted_border: Color::DarkGray,
                fg: Color::White,
                fg_dim: Color::Gray,
                fg_bright: Color::White,
                accent: Color::Cyan,
                secondary_accent: Color::Green,
                tertiary_accent: Color::Yellow,
                error: Color::Red,
                warning: Color::Yellow,
                success: Color::Green,
                selection_fg: Color::Black,
                selection_bg: Color::Cyan,
                border: Color::DarkGray,
                border_active: Color::Cyan,
                volume_low: Color::Green,
                volume_medium: Color::Yellow,
                volume_high: Color::Red,
                sidebar_active_border: Color::Cyan,
                notification_border: Color::Cyan,
                monochromatic: false,
            },
        }
        .theme
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    // Header with wordmark
    let header = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(theme.bg).fg(theme.fg));
    f.render_widget(header, chunks[0]);
    render_wordmark(
        Rect {
            x: chunks[0].x + 1,
            y: chunks[0].y,
            width: 10,
            height: 3,
        },
        f.buffer_mut(),
        theme.accent,
    );

    // Main area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(0)])
        .split(chunks[1]);

    // Sidebar
    let items = vec![
        ListItem::new("Home"),
        ListItem::new("Search"),
        ListItem::new("Bookmarks"),
        ListItem::new("History"),
        ListItem::new("Watchlist"),
    ];
    let sidebar = List::new(items).block(
        Block::default()
            .title("Menu")
            .borders(Borders::ALL)
            .style(Style::default().bg(theme.pane_bg).fg(theme.fg)),
    );
    f.render_widget(sidebar, main_chunks[0]);

    // Content
    match state.screen {
        Screen::Splash => {
            render_skeleton(main_chunks[1], f.buffer_mut());
            let msg = Paragraph::new("Loading gtw...").style(Style::default().fg(theme.fg));
            let r = Rect {
                x: main_chunks[1].x + 2,
                y: main_chunks[1].y + 8,
                width: main_chunks[1].width - 4,
                height: 1,
            };
            f.render_widget(msg, r);
        }
        Screen::ThemePicker => {
            let theme_names: Vec<String> = themes.iter().map(|t| t.name.clone()).collect();
            let items: Vec<ListItem> = theme_names
                .iter()
                .map(|n| ListItem::new(n.as_str()))
                .collect();
            let list = List::new(items)
                .block(
                    Block::default()
                        .title("Theme Picker (t/T to cycle, p to close)")
                        .borders(Borders::ALL)
                        .style(Style::default().bg(theme.pane_bg).fg(theme.fg)),
                )
                .highlight_style(
                    Style::default()
                        .bg(theme.selection_bg)
                        .fg(theme.selection_fg)
                        .add_modifier(Modifier::BOLD),
                );
            f.render_widget(list, main_chunks[1]);
        }
        _ => {
            let msg = Paragraph::new(format!("Screen: {:?}", state.screen))
                .style(Style::default().fg(theme.fg));
            f.render_widget(msg, main_chunks[1]);
        }
    }

    // Footer
    let footer = Paragraph::new(format!(
        "gtw | theme: {} | q:quit t:next p:picker",
        themes
            .get(theme_idx)
            .map(|t| t.name.as_str())
            .unwrap_or("default")
    ))
    .style(Style::default().bg(theme.bg).fg(theme.fg_dim));
    f.render_widget(footer, chunks[2]);
}
