use crate::core::{ActivityCenter, Loadable};
#[allow(dead_code)]
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Screen {
    Splash,
    Home,
    Search,
    Article,
    Bookmarks,
    History,
    Watchlist,
    Editor,
    Media,
    Settings,
    ThemePicker,
    Auth,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Tab {
    pub id: crate::core::TabId,
    pub title: Option<String>,
    pub article: Loadable<crate::core::Article>,
    pub scroll: u16,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct AppState {
    pub screen: Screen,
    pub tabs: Vec<Tab>,
    pub active_tab: Option<crate::core::TabId>,
    pub home: Loadable<Feed>,
    pub search: SearchState,
    pub bookmarks: Loadable<BookmarkList>,
    pub history: Loadable<HistoryPage>,
    pub watchlist: Loadable<WatchlistPage>,
    pub activity: ActivityCenter,
    pub should_quit: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Feed {
    pub sections: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SearchState {
    pub query: String,
    pub results: Loadable<Vec<crate::core::SearchHit>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BookmarkList {
    pub items: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HistoryPage {
    pub items: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WatchlistPage {
    pub items: Vec<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Splash,
            tabs: Vec::new(),
            active_tab: None,
            home: Loadable::Idle,
            search: SearchState {
                query: String::new(),
                results: Loadable::Idle,
            },
            bookmarks: Loadable::Idle,
            history: Loadable::Idle,
            watchlist: Loadable::Idle,
            activity: ActivityCenter::new(),
            should_quit: false,
        }
    }
}

impl AppState {
    #[allow(dead_code)]
    pub fn update(&mut self, msg: crate::core::Msg) -> Vec<crate::core::Cmd> {
        let mut cmds = Vec::new();
        match msg {
            crate::core::Msg::Quit => {
                self.should_quit = true;
            }
            crate::core::Msg::ThemeNext => {}
            crate::core::Msg::ThemePrev => {}
            crate::core::Msg::ShowThemePicker => {
                self.screen = Screen::ThemePicker;
            }
            crate::core::Msg::SwitchTab(id) => {
                self.active_tab = Some(id);
                self.screen = Screen::Article;
            }
            crate::core::Msg::OpenArticle { title, tab } => {
                use crate::core::{RequestId, TabId};
                let tid = tab.unwrap_or_else(|| {
                    let id = TabId::new();
                    self.tabs.push(Tab {
                        id,
                        title: Some(title.clone()),
                        article: crate::core::Loadable::Loading {
                            stage: crate::core::LoadStage::Connecting,
                            progress: None,
                            started_at: std::time::Instant::now(),
                            previous: None,
                            cancel: tokio_util::sync::CancellationToken::new(),
                        },
                        scroll: 0,
                    });
                    self.active_tab = Some(id);
                    id
                });
                self.screen = Screen::Article;
                let req = RequestId::new();
                cmds.push(crate::core::Cmd::FetchArticle {
                    req,
                    tab: tid,
                    title,
                });
            }
            crate::core::Msg::SearchResults { req: _, results } => match results {
                Ok(hits) => {
                    let v = hits
                        .into_iter()
                        .map(|h| crate::core::SearchHit {
                            title: h.title,
                            snippet: h.snippet,
                        })
                        .collect();
                    self.search.results = crate::core::Loadable::Ready {
                        value: std::sync::Arc::new(v),
                        updated_at: std::time::SystemTime::now(),
                        source: crate::core::Source::Network,
                        stale: false,
                    };
                }
                Err(e) => {
                    self.search.results = crate::core::Loadable::Failed {
                        error: crate::core::LoadError {
                            message: e.message,
                            retryable: e.retryable,
                            kind: e.kind,
                        },
                        previous: None,
                        retryable: true,
                        failed_at: std::time::SystemTime::now(),
                    };
                }
            },
            _ => {}
        }
        cmds
    }
}
