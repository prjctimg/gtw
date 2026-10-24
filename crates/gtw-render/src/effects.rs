#[derive(Default)]
pub struct EffectManager {
    effects: Vec<()>,
}

impl EffectManager {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    pub fn add_startup(&mut self) {}

    pub fn tick(&mut self, _dt: std::time::Duration) {}
}
