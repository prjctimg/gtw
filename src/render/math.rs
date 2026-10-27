use image::RgbaImage;
use ratex_layout::{LayoutOptions, layout, to_display_list};
use ratex_parser::parser::parse;
use ratex_svg::{SvgOptions, render_to_svg};
use ratex_types::color::Color;
use ratex_types::math_style::MathStyle;
use resvg::{tiny_skia, usvg};

/// Render a TeX expression to an RGBA bitmap using RaTeX + resvg.
///
/// The returned image is sized for a terminal cell grid; callers may scale it
/// further. `light` selects a light background with dark glyphs.
pub fn render_math(tex: &str, light: bool, target_height: u32) -> Result<RgbaImage, String> {
    let tex = tex.trim();
    if tex.is_empty() {
        return Err("empty math expression".to_string());
    }
    validate(tex)?;
    let tex = normalize_tex(tex);
    let ast = parse(&tex).map_err(|err| format!("TeX parse error: {err}"))?;
    let fg = if light { Color::BLACK } else { Color::WHITE };
    let layout_opts = LayoutOptions::default()
        .with_style(MathStyle::Display)
        .with_color(fg);
    let layout_box = layout(&ast, &layout_opts);
    let display_list = to_display_list(&layout_box);
    let svg_opts = SvgOptions {
        font_size: 40.0,
        padding: 10.0,
        stroke_width: 1.5,
        embed_glyphs: true,
        font_dir: String::new(),
    };
    let svg = render_to_svg(&display_list, &svg_opts);
    let opt = usvg::Options {
        font_family: "KaTeX_Main".to_string(),
        ..Default::default()
    };
    let tree =
        usvg::Tree::from_str(&svg, &opt).map_err(|err| format!("math SVG parse error: {err}"))?;
    let size = tree.size();
    let source_width = size.width().max(1.0);
    let source_height = size.height().max(1.0);
    let scale = (target_height.max(32) as f32 / source_height).max(0.05);
    let width = (source_width * scale).ceil().max(1.0) as u32;
    let height = (source_height * scale).ceil().max(1.0) as u32;
    if width > 8192 || height > 4096 {
        return Err("math expression is too wide to render".to_string());
    }
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "math bitmap allocation failed".to_string())?;
    let bg = if light {
        tiny_skia::Color::from_rgba8(250, 250, 250, 255)
    } else {
        tiny_skia::Color::from_rgba8(24, 24, 24, 255)
    };
    pixmap.fill(bg);
    let mut pixmap_mut = pixmap.as_mut();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap_mut,
    );
    let image = RgbaImage::from_raw(width, height, pixmap.data().to_vec())
        .ok_or_else(|| "invalid math raster buffer".to_string())?;
    if !has_ink(&image) {
        return Err("math expression rendered no visible glyphs".to_string());
    }
    Ok(image)
}

pub fn validate(tex: &str) -> Result<(), String> {
    let tex = tex.trim();
    if tex.is_empty() {
        return Err("empty math expression".to_string());
    }
    parse(&normalize_tex(tex))
        .map(|_| ())
        .map_err(|err| format!("TeX parse error: {err}"))
}

fn normalize_tex(tex: &str) -> String {
    let mut out = tex.to_string();
    // Wikipedia wraps display math in {\displaystyle ...}; RaTeX accepts the
    // content directly and it keeps formulas much more compact.
    for prefix in [
        "{\\displaystyle ",
        "{\\textstyle ",
        "\\displaystyle ",
        "\\textstyle ",
    ] {
        if out.starts_with(prefix) {
            out = out[prefix.len()..].to_string();
            if out.ends_with('}') {
                out.pop();
            }
            break;
        }
    }
    out.trim().to_string()
}

/// A tiny helper for tests: the resulting buffer must contain ink.
pub fn has_ink(image: &RgbaImage) -> bool {
    let bg = image.get_pixel(0, 0).0;
    image.pixels().any(|p| {
        let [r, g, b, a] = p.0;
        a > 0 && (r.abs_diff(bg[0]) > 8 || g.abs_diff(bg[1]) > 8 || b.abs_diff(bg[2]) > 8)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_basic_formula() {
        let img = render_math("x^2 + y^2 = z^2", true, 96).expect("render");
        assert!(img.width() > 8);
        assert!(img.height() >= 32);
        assert!(has_ink(&img));
    }

    #[test]
    fn normalizes_display_style() {
        assert_eq!(normalize_tex("{\\displaystyle x^2}"), "x^2");
    }
}
