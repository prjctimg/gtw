#[derive(Default)]
pub struct EffectManager {}

impl EffectManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn add_startup(&mut self) {}

    pub fn tick(&mut self, _dt: std::time::Duration) {}
}
