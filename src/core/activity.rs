use std::time::Instant;
#[allow(dead_code)]
use tokio_util::sync::CancellationToken;

use crate::core::{Priority, Progress, RequestId};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ActivityId(pub u64);

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ActivityKind {
    FetchArticle,
    Search,
    DownloadImage,
    PlayAudio,
    PlayVideo,
    Prefetch,
    Cache,
    Auth,
    Edit,
    Other,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum SurfaceId {
    Home,
    Search,
    Article,
    Bookmarks,
    History,
    Watchlist,
    Editor,
    Media,
    Settings,
    Other,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Activity {
    pub id: RequestId,
    pub kind: ActivityKind,
    pub label: String,
    pub surface: SurfaceId,
    pub priority: Priority,
    pub progress: Option<Progress>,
    pub cancellable: bool,
    pub started_at: Instant,
    pub cancel: CancellationToken,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct ActivityCenter {
    pub activities: Vec<Activity>,
}

impl ActivityCenter {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            activities: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn push(&mut self, activity: Activity) {
        self.activities.push(activity);
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, id: RequestId) {
        self.activities.retain(|a| a.id.0 != id.0);
    }

    #[allow(dead_code)]
    pub fn highest_priority(&self) -> Option<&Activity> {
        self.activities
            .iter()
            .filter(|a| a.cancellable || a.progress.is_some())
            .min_by_key(|a| match a.priority {
                Priority::Interactive => 0,
                Priority::Background => 1,
            })
            .or_else(|| self.activities.first())
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.activities.is_empty()
    }
}
