#[derive(Debug, Clone)]
pub struct FeedSection {
    pub title: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FeaturedFeed {
    pub sections: Vec<FeedSection>,
    pub date: String,
}

pub struct FeedManager;

impl FeedManager {
    pub fn new() -> Self {
        Self
    }

    pub fn get_featured(&self, date: &str) -> FeaturedFeed {
        FeaturedFeed {
            sections: vec![
                FeedSection {
                    title: "Featured".to_string(),
                    items: vec![],
                },
                FeedSection {
                    title: "On this day".to_string(),
                    items: vec![],
                },
            ],
            date: date.to_string(),
        }
    }
}

impl Default for FeedManager {
    fn default() -> Self {
        Self::new()
    }
}
