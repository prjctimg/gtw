use std::time::Duration;

use ::image::AnimationDecoder;
use ::image::codecs::gif::GifDecoder;
use ::image::imageops::FilterType;

use crate::core::{LoadError, MediaKind};
use crate::media::DecodedMedia;

const MAX_DIMENSION: u32 = 4096;
const MAX_FRAMES: usize = 512;

pub fn decode(data: &[u8], kind: MediaKind) -> Result<DecodedMedia, LoadError> {
    match kind {
        MediaKind::Svg => decode_svg(data),
        MediaKind::Gif => decode_gif(data),
        MediaKind::Image => decode_static(data),
        MediaKind::Math => Err(LoadError::parse("math must use the TeX renderer")),
        other => Err(LoadError::parse(format!("unsupported image kind: {other}"))),
    }
}

pub fn decode_static(data: &[u8]) -> Result<DecodedMedia, LoadError> {
    let img = ::image::load_from_memory(data)
        .map_err(|e| LoadError::parse(format!("image decode failed: {e}")))?;
    let rgba = limit_image(img.to_rgba8());
    Ok(DecodedMedia::single(rgba, MediaKind::Image))
}

pub fn decode_gif(data: &[u8]) -> Result<DecodedMedia, LoadError> {
    let decoder = GifDecoder::new(std::io::Cursor::new(data))
        .map_err(|e| LoadError::parse(format!("gif decode failed: {e}")))?;
    let frames = decoder
        .into_frames()
        .collect_frames()
        .map_err(|e| LoadError::parse(format!("gif frame decode failed: {e}")))?;
    if frames.is_empty() {
        return Err(LoadError::parse("gif contains no frames"));
    }
    let mut images = Vec::new();
    let mut delays = Vec::new();
    for frame in frames.into_iter().take(MAX_FRAMES) {
        let delay = frame.delay();
        let delay = Duration::from_millis(
            ((delay.numer_denom_ms().0 as u64) / (delay.numer_denom_ms().1.max(1) as u64)).max(20),
        );
        images.push(limit_image(frame.into_buffer()));
        delays.push(delay);
    }
    Ok(DecodedMedia::animated(images, delays, MediaKind::Gif))
}

pub fn decode_svg(data: &[u8]) -> Result<DecodedMedia, LoadError> {
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(data, &opt)
        .map_err(|e| LoadError::parse(format!("svg parse failed: {e}")))?;
    let size = tree.size();
    let mut width = size.width().max(1.0);
    let mut height = size.height().max(1.0);
    if width > height {
        let scale = 1200.0 / width;
        width *= scale;
        height *= scale;
    } else {
        let scale = 900.0 / height;
        width *= scale;
        height *= scale;
    }
    let width = (width.round() as u32).clamp(1, MAX_DIMENSION);
    let height = (height.round() as u32).clamp(1, MAX_DIMENSION);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| LoadError::parse("svg bitmap allocation failed"))?;
    pixmap.fill(resvg::tiny_skia::Color::TRANSPARENT);
    let scale_x = width as f32 / size.width().max(1.0);
    let scale_y = height as f32 / size.height().max(1.0);
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale_x, scale_y),
        &mut pixmap.as_mut(),
    );
    let image = ::image::RgbaImage::from_raw(width, height, pixmap.data().to_vec())
        .ok_or_else(|| LoadError::parse("svg raster buffer is invalid"))?;
    Ok(DecodedMedia::single(image, MediaKind::Svg))
}

fn limit_image(mut img: ::image::RgbaImage) -> ::image::RgbaImage {
    if img.width() > MAX_DIMENSION || img.height() > MAX_DIMENSION {
        let scale = (MAX_DIMENSION as f32 / img.width() as f32)
            .min(MAX_DIMENSION as f32 / img.height() as f32);
        let w = (img.width() as f32 * scale).round().max(1.0) as u32;
        let h = (img.height() as f32 * scale).round().max(1.0) as u32;
        img = image::imageops::resize(&img, w, h, FilterType::Lanczos3);
    }
    img
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn decodes_png_roundtrip() {
        let mut img = RgbaImage::new(4, 3);
        for (x, y, p) in img.enumerate_pixels_mut() {
            *p = Rgba([(x * 10) as u8, (y * 20) as u8, 80, 255]);
        }
        let mut cursor = std::io::Cursor::new(Vec::new());
        img.write_to(&mut cursor, ::image::ImageFormat::Png)
            .expect("encode");
        let media = decode(cursor.get_ref(), MediaKind::Image).expect("decode");
        assert_eq!(media.width, 4);
        assert_eq!(media.height, 3);
    }

    #[test]
    fn decodes_gif_animation() {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode("R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==")
            .expect("base64 gif");
        let media = decode(&bytes, MediaKind::Gif).expect("gif");
        assert_eq!(media.frames.len(), 1);
        assert_eq!((media.width, media.height), (1, 1));
        assert_eq!(media.frames[0].as_raw().len(), 4);
    }

    #[test]
    fn decodes_svg() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10"><rect width="20" height="10" fill="red"/></svg>"#;
        let media = decode(svg, MediaKind::Svg).expect("svg");
        assert!(media.width >= 1);
        assert_eq!(media.frames.len(), 1);
    }
}
