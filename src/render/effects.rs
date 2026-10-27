use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use tachyonfx::{EffectManager as Tachyon, Interpolation, Motion, fx};

/// Wraps tachyonfx's effect manager so the UI can own one canvas-wide
/// transition pipeline. Effects are purely visual and never block input.
pub struct EffectManager {
    manager: Tachyon<()>,
    enabled: bool,
}

impl Default for EffectManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectManager {
    pub fn new() -> Self {
        Self {
            manager: Tachyon::default(),
            enabled: true,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.manager = Tachyon::default();
        }
    }

    pub fn add_startup(&mut self) {
        if !self.enabled {
            return;
        }
        self.manager.add_effect(fx::fade_from(
            Color::Rgb(0, 0, 0),
            Color::Rgb(0, 0, 0),
            (520, Interpolation::QuadOut),
        ));
        self.manager.add_effect(fx::sweep_in(
            Motion::LeftToRight,
            18,
            8,
            Color::Rgb(0, 0, 0),
            (680, Interpolation::CubicOut),
        ));
    }

    pub fn page_in(&mut self) {
        if !self.enabled {
            return;
        }
        self.manager.add_effect(fx::sweep_in(
            Motion::UpToDown,
            12,
            4,
            Color::Rgb(0, 0, 0),
            (300, Interpolation::QuadOut),
        ));
    }

    pub fn feed_in(&mut self) {
        if !self.enabled {
            return;
        }
        self.manager.add_effect(fx::sweep_in(
            Motion::RightToLeft,
            10,
            6,
            Color::Rgb(0, 0, 0),
            (360, Interpolation::SineOut),
        ));
    }

    pub fn process(&mut self, dt: std::time::Duration, buf: &mut Buffer, area: Rect) {
        if self.enabled {
            self.manager.process_effects(dt.into(), buf, area);
        }
    }
}
