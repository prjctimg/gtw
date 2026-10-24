use std::time::Instant;

use tokio_util::sync::CancellationToken;

use crate::{Priority, Progress, RequestId};

#[derive(Debug, Clone)]
pub struct ActivityId(pub u64);

#[derive(Debug, Clone)]
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
pub struct ActivityCenter {
    pub activities: Vec<Activity>,
}

impl ActivityCenter {
    pub fn new() -> Self {
        Self { activities: Vec::new() }
    }

    pub fn push(&mut self, activity: Activity) {
        self.activities.push(activity);
    }

    pub fn remove(&mut self, id: RequestId) {
        self.activities.retain(|a| a.id.0 != id.0);
    }

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
}
