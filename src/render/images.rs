use std::collections::HashMap;

use ::image::RgbaImage;
use ratatui::Frame;
use ratatui::layout::{Rect, Size};
use ratatui_image::picker::Picker;
use ratatui_image::protocol::Protocol;
use ratatui_image::{Image, Resize};

use crate::media::DecodedMedia;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    key: String,
    width: u16,
    height: u16,
    frame: usize,
}

/// Terminal image backend. Uses the best protocol the terminal advertises and
/// falls back to Unicode half blocks, which work everywhere.
pub struct ImageRenderer {
    picker: Picker,
    protocols: HashMap<CacheKey, Protocol>,
}

impl ImageRenderer {
    pub fn new() -> Self {
        let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());
        Self {
            picker,
            protocols: HashMap::new(),
        }
    }

    #[cfg(test)]
    pub fn headless() -> Self {
        Self {
            picker: Picker::halfblocks(),
            protocols: HashMap::new(),
        }
    }

    pub fn protocol_name(&self) -> &'static str {
        // Picker does not expose a stable protocol name; halfblocks is the
        // universal fallback and the doctor reports the same distinction.
        if self.picker.font_size().width == 0 || self.picker.font_size().height == 0 {
            "halfblocks"
        } else {
            "terminal graphics"
        }
    }

    pub fn render_media(
        &mut self,
        frame: &mut Frame<'_>,
        key: &str,
        media: &DecodedMedia,
        area: Rect,
        frame_index: usize,
    ) {
        if area.width < 2 || area.height < 2 || media.frames.is_empty() {
            return;
        }
        let index = frame_index.min(media.frames.len() - 1);
        self.render_rgba(frame, &format!("{key}#{index}"), &media.frames[index], area);
    }

    pub fn render_rgba(&mut self, frame: &mut Frame<'_>, key: &str, image: &RgbaImage, area: Rect) {
        if area.width < 2 || area.height < 2 || image.width() == 0 || image.height() == 0 {
            return;
        }
        let cache_key = CacheKey {
            key: key.to_string(),
            width: area.width,
            height: area.height,
            frame: 0,
        };
        if !self.protocols.contains_key(&cache_key) {
            if self.protocols.len() > 48 {
                self.protocols.clear();
            }
            let dynamic = ::image::DynamicImage::ImageRgba8(image.clone());
            match self.picker.new_protocol(
                dynamic,
                Size::new(area.width, area.height),
                Resize::Fit(None),
            ) {
                Ok(protocol) => {
                    self.protocols.insert(cache_key.clone(), protocol);
                }
                Err(error) => {
                    tracing::debug!("image protocol encode failed: {error}");
                    return;
                }
            }
        }
        if let Some(protocol) = self.protocols.get(&cache_key) {
            frame.render_widget(Image::new(protocol).allow_clipping(true), area);
        }
    }

    pub fn clear(&mut self) {
        self.protocols.clear();
    }
}

impl Default for ImageRenderer {
    fn default() -> Self {
        Self::new()
    }
}
