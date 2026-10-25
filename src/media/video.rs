#[derive(Debug, Clone)]
#[allow(dead_code)]
#[allow(dead_code)]
pub struct VideoTrack {
    pub src: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VideoState {
    pub playing: bool,
    pub position: f64,
    pub duration: f64,
    pub buffered: f64,
}

#[allow(dead_code)]
pub struct VideoPlayer {
    state: VideoState,
}

impl VideoPlayer {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            state: VideoState {
                playing: false,
                position: 0.0,
                duration: 0.0,
                buffered: 0.0,
            },
        }
    }

    #[allow(dead_code)]
    pub fn play(&mut self) {
        self.state.playing = true;
    }

    #[allow(dead_code)]
    pub fn pause(&mut self) {
        self.state.playing = false;
    }

    #[allow(dead_code)]
    pub fn seek(&mut self, pct: f64) {
        self.state.position = self.state.duration * pct.clamp(0.0, 1.0);
    }

    #[allow(dead_code)]
    pub fn state(&self) -> &VideoState {
        &self.state
    }
}

impl Default for VideoPlayer {
    fn default() -> Self {
        Self::new()
    }
}
