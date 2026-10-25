#[derive(Debug, Clone)]
#[allow(dead_code)]
#[allow(dead_code)]
pub struct ImageSource {
    pub url: String,
}

#[allow(dead_code)]
pub struct ImageDecoder;

impl ImageDecoder {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
}

impl Default for ImageDecoder {
    fn default() -> Self {
        Self::new()
    }
}
