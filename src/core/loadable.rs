use std::sync::Arc;
use std::time::{Instant, SystemTime};

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum Priority {
    Interactive,
    Background,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum Source {
    Network,
    Cache,
    Prefetch,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum ProgressUnit {
    Bytes,
    Items,
    Frames,
    Percent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Progress {
    pub current: u64,
    pub total: Option<u64>,
    pub unit: ProgressUnit,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct CancelHandle(pub CancellationToken);

impl Default for CancelHandle {
    fn default() -> Self {
        Self(CancellationToken::new())
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Fallback {
    None,
    Placeholder,
    External,
    Cached,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LoadError {
    pub message: String,
    pub retryable: bool,
    pub kind: LoadErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
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

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub enum Loadable<T> {
    #[default]
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

impl<T> Loadable<T> {
    #[allow(dead_code)]
    #[allow(dead_code)]
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

    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn is_loading(&self) -> bool {
        matches!(
            self,
            Loadable::Loading { .. } | Loadable::Refreshing { .. } | Loadable::Queued { .. }
        )
    }

    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn is_ready(&self) -> bool {
        matches!(self, Loadable::Ready { .. })
    }

    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn is_failed(&self) -> bool {
        matches!(self, Loadable::Failed { .. })
    }

    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn begin_refresh(&mut self, previous: Arc<T>) {
        *self = Loadable::Refreshing {
            stage: LoadStage::Connecting,
            progress: None,
            previous,
            started_at: Instant::now(),
            cancel: CancellationToken::new(),
        };
    }

    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn cancel(&mut self) {
        match self {
            Loadable::Loading { cancel, .. } => cancel.cancel(),
            Loadable::Refreshing { cancel, .. } => cancel.cancel(),
            _ => {}
        }
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

    #[test]
    fn failed_with_previous_shows_previous() {
        let prev = Arc::new("old".to_string());
        let v = Loadable::Failed {
            error: LoadError {
                message: "err".to_string(),
                retryable: true,
                kind: LoadErrorKind::Network,
            },
            previous: Some(prev.clone()),
            retryable: true,
            failed_at: SystemTime::now(),
        };
        assert_eq!(v.value().unwrap(), "old");
        assert!(v.is_failed());
    }
}
