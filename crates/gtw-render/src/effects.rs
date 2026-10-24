use tachyonfx::{Effect, EffectRenderer, fx};

#[derive(Default)]
pub struct EffectManager {
    effects: Vec<Effect>,
}

impl EffectManager {
    pub fn new() -> Self {
        Self { effects: Vec::new() }
    }

    pub fn add_startup(&mut self) {
        // Basic fade/slide style effect placeholder
        let _ = fx::fade_from_fg(std::time::Duration::from_millis(400), 0);
    }

    pub fn tick(&mut self, _dt: std::time::Duration) {
        // Placeholder
    }
}
