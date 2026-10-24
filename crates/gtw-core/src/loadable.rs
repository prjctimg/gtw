use std::sync::Arc;
use std::time::{Instant, SystemTime};

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Interactive,
    Background,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Source {
    Network,
    Cache,
    Prefetch,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressUnit {
    Bytes,
    Items,
    Frames,
    Percent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub current: u64,
    pub total: Option<u64>,
    pub unit: ProgressUnit,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadStage {
    Connecting,
    Downloading,
    Parsing,
    Extracting,
    LayingOut,
    Decoding,
    Buffering,
    Indexing,
    Saving,
    Committing,
}

#[derive(Debug, Clone)]
pub struct CancelHandle(pub CancellationToken);

impl Default for CancelHandle {
    fn default() -> Self {
        Self(CancellationToken::new())
    }
}

#[derive(Debug, Clone)]
pub enum Fallback {
    None,
    Placeholder,
    External,
    Cached,
}

#[derive(Debug, Clone)]
pub struct LoadError {
    pub message: String,
    pub retryable: bool,
    pub kind: LoadErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadErrorKind {
    Network,
    NotFound,
    Protected,
    Parse,
    Offline,
    Auth,
    RateLimited,
    Cancelled,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum Loadable<T> {
    Idle,
    Queued {
        queued_at: Instant,
        priority: Priority,
    },
    Loading {
        stage: LoadStage,
        progress: Option<Progress>,
        started_at: Instant,
        previous: Option<Arc<T>>,
        cancel: CancellationToken,
    },
    Refreshing {
        stage: LoadStage,
        progress: Option<Progress>,
        previous: Arc<T>,
        started_at: Instant,
        cancel: CancellationToken,
    },
    Ready {
        value: Arc<T>,
        updated_at: SystemTime,
        source: Source,
        stale: bool,
    },
    Failed {
        error: LoadError,
        previous: Option<Arc<T>>,
        retryable: bool,
        failed_at: SystemTime,
    },
    Unsupported {
        reason: String,
        fallback: Fallback,
    },
}

impl<T> Default for Loadable<T> {
    fn default() -> Self {
        Loadable::Idle
    }
}

impl<T> Loadable<T> {
    pub fn value(&self) -> Option<&T> {
        match self {
            Loadable::Ready { value, .. } => Some(value.as_ref()),
            Loadable::Loading {
                previous: Some(p), ..
            } => Some(p.as_ref()),
            Loadable::Refreshing { previous, .. } => Some(previous.as_ref()),
            Loadable::Failed {
                previous: Some(p), ..
            } => Some(p.as_ref()),
            _ => None,
        }
    }

    pub fn is_loading(&self) -> bool {
        matches!(
            self,
            Loadable::Loading { .. } | Loadable::Refreshing { .. } | Loadable::Queued { .. }
        )
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Loadable::Ready { .. })
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Loadable::Failed { .. })
    }

    pub fn begin_refresh(&mut self, previous: Arc<T>) {
        *self = Loadable::Refreshing {
            stage: LoadStage::Connecting,
            progress: None,
            previous,
            started_at: Instant::now(),
            cancel: CancellationToken::new(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_idle() {
        let v: Loadable<String> = Loadable::default();
        assert!(matches!(v, Loadable::Idle));
    }

    #[test]
    fn value_from_ready() {
        let v = Loadable::Ready {
            value: Arc::new("hi".to_string()),
            updated_at: SystemTime::now(),
            source: Source::Network,
            stale: false,
        };
        assert_eq!(v.value().unwrap(), "hi");
    }
}
