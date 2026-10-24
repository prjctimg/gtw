use gtw_core::Screen;

#[derive(Debug, Clone)]
pub struct ScreenRouter;

impl ScreenRouter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ScreenRouter {
    fn default() -> Self {
        Self::new()
    }
}
