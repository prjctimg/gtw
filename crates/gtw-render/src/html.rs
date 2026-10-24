pub struct Extracted {
    pub title: String,
    pub body_html: String,
}

pub fn extract_article(_html: &str) -> Extracted {
    Extracted {
        title: String::new(),
        body_html: String::new(),
    }
}
