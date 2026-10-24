use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;
use tui_big_text::{BigText, PixelSize};

pub fn render_wordmark(area: Rect, buf: &mut Buffer, color: Color) {
    if area.width < 8 || area.height < 3 {
        return;
    }
    let big = BigText::builder()
        .pixel_size(PixelSize::Sextant)
        .style(Style::default().fg(color))
        .lines(vec!["gtw".into()])
        .build();
    Widget::render(big, area, buf);
}
