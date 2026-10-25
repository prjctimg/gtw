#[derive(Debug, Clone)]
#[allow(dead_code)]
#[allow(dead_code)]
pub struct AudioTrack {
    pub src: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AudioState {
    pub playing: bool,
    pub position: f64,
    pub duration: f64,
    pub volume: f32,
}

#[allow(dead_code)]
pub struct AudioPlayer {
    state: AudioState,
}

impl AudioPlayer {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            state: AudioState {
                playing: false,
                position: 0.0,
                duration: 0.0,
                volume: 1.0,
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
    pub fn set_volume(&mut self, v: f32) {
        self.state.volume = v.clamp(0.0, 1.0);
    }

    #[allow(dead_code)]
    pub fn state(&self) -> &AudioState {
        &self.state
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}
