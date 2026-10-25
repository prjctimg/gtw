use crate::core::{LoadError, RequestId, Source, TabId};
#[allow(dead_code)]
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Msg {
    // Input
    Quit,
    Resize(u16, u16),
    Tick,

    // Navigation
    OpenArticle {
        title: String,
        tab: Option<TabId>,
    },
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
#[allow(dead_code)]
pub struct Article {
    pub title: String,
    pub body_html: String,
    pub revision: Option<u64>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SearchHit {
    pub title: String,
    pub snippet: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Cmd {
    FetchArticle {
        req: RequestId,
        tab: TabId,
        title: String,
    },
    Search {
        req: RequestId,
        query: String,
    },
    FetchImage {
        req: RequestId,
        url: String,
    },
    Cancel(RequestId),
}
