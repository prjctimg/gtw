use crate::{ActivityCenter, Loadable};

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: crate::TabId,
    pub title: Option<String>,
    pub article: Loadable<crate::Article>,
    pub scroll: u16,
}

#[derive(Debug)]
pub struct AppState {
    pub screen: Screen,
    pub tabs: Vec<Tab>,
    pub active_tab: Option<crate::TabId>,
    pub home: Loadable<Feed>,
    pub search: SearchState,
    pub bookmarks: Loadable<BookmarkList>,
    pub history: Loadable<HistoryPage>,
    pub watchlist: Loadable<WatchlistPage>,
    pub activity: ActivityCenter,
    pub should_quit: bool,
}

#[derive(Debug, Clone)]
pub struct Feed {
    pub sections: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub results: Loadable<Vec<crate::SearchHit>>,
}

#[derive(Debug, Clone)]
pub struct BookmarkList {
    pub items: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HistoryPage {
    pub items: Vec<String>,
}

#[derive(Debug, Clone)]
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
    pub fn update(&mut self, msg: crate::Msg) -> Vec<crate::Cmd> {
        let cmds = Vec::new();
        match msg {
            crate::Msg::Quit => {
                self.should_quit = true;
            }
            crate::Msg::ThemeNext => {}
            crate::Msg::ThemePrev => {}
            crate::Msg::ShowThemePicker => {
                self.screen = Screen::ThemePicker;
            }
            crate::Msg::SwitchTab(id) => {
                self.active_tab = Some(id);
                self.screen = Screen::Article;
            }
            _ => {}
        }
        cmds
    }
}
