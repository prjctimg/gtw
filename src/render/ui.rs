use crate::core::Loadable;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Paragraph, Widget};

#[allow(dead_code)]
pub fn render_skeleton(area: Rect, buf: &mut Buffer) {
    for y in area.y..area.y + area.height.min(6) {
        for x in area.x..area.x + area.width.saturating_sub(4) {
            if ((x + y) as usize).is_multiple_of(3) {
                buf[(x, y)]
                    .set_char('░')
                    .set_style(Style::default().fg(Color::DarkGray));
            }
        }
    }
}

#[allow(dead_code)]
pub fn render_loadable<T>(
    area: Rect,
    buf: &mut Buffer,
    loadable: &Loadable<T>,
    render_ready: impl FnOnce(Rect, &mut Buffer, &T),
) {
    match loadable {
        Loadable::Idle | Loadable::Queued { .. } => {
            render_skeleton(area, buf);
        }
        Loadable::Loading {
            previous: Some(p), ..
        } => {
            render_ready(area, buf, p.as_ref());
        }
        Loadable::Loading { previous: None, .. } => {
            render_skeleton(area, buf);
        }
        Loadable::Refreshing { previous, .. } => {
            render_ready(area, buf, previous.as_ref());
        }
        Loadable::Ready { value, .. } => {
            render_ready(area, buf, value.as_ref());
        }
        Loadable::Failed {
            previous: Some(p),
            error,
            ..
        } => {
            render_ready(area, buf, p.as_ref());
            let msg = Paragraph::new(format!("Error: {} (r to retry)", error.message))
                .style(Style::default().fg(Color::Red));
            let r = Rect {
                y: area.y + area.height.saturating_sub(1),
                height: 1,
                ..area
            };
            Widget::render(msg, r, buf);
        }
        Loadable::Failed {
            previous: None,
            error,
            ..
        } => {
            let msg = Paragraph::new(format!("Error: {} (r to retry)", error.message))
                .style(Style::default().fg(Color::Red));
            Widget::render(msg, area, buf);
        }
        Loadable::Unsupported { reason, .. } => {
            let msg = Paragraph::new(format!("Unsupported: {}", reason));
            Widget::render(msg, area, buf);
        }
    }
}
