use std::sync::Arc;
use std::time::SystemTime;

use crate::{Loadable, LoadError, Progress, RequestId, Source, TabId};

#[derive(Debug, Clone)]
pub enum Msg {
    // Input
    Quit,
    Resize(u16, u16),
    Tick,

    // Navigation
    OpenArticle { title: String, tab: Option<TabId> },
    GoBack,
    GoForward,
    SwitchTab(TabId),
    CloseTab(TabId),

    // Commands results
    ArticleLoaded {
        req: RequestId,
        tab: TabId,
        result: Result<Article, LoadError>,
        source: Source,
    },
    SearchResults {
        req: RequestId,
        results: Result<Vec<SearchHit>, LoadError>,
    },
    ImageReady {
        req: RequestId,
        data: Vec<u8>,
    },
    ImageFailed {
        req: RequestId,
        err: LoadError,
    },

    // UI
    ThemeNext,
    ThemePrev,
    ShowThemePicker,
    ShowActivityCenter,

    // Effects
    EffectDone,
}

#[derive(Debug, Clone)]
pub struct Article {
    pub title: String,
    pub body_html: String,
    pub revision: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub title: String,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub enum Cmd {
    FetchArticle { req: RequestId, tab: TabId, title: String },
    Search { req: RequestId, query: String },
    FetchImage { req: RequestId, url: String },
    Cancel(RequestId),
}
