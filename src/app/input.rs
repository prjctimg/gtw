use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
#[allow(dead_code)]
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Action {
    Quit,
    Up,
    Down,
    Left,
    Right,
    Enter,
    Back,
    TabNext,
    TabPrev,
    ThemeNext,
    ThemePrev,
    ShowThemePicker,
    ShowActivityCenter,
    Other,
}

#[allow(dead_code)]
pub fn map_key(key: KeyEvent) -> Action {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::NONE) => Action::Quit,
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Action::Quit,
        (KeyCode::Esc, _) => Action::Back,
        (KeyCode::Up, _) => Action::Up,
        (KeyCode::Down, _) => Action::Down,
        (KeyCode::Left, _) => Action::Left,
        (KeyCode::Right, _) => Action::Right,
        (KeyCode::Enter, _) => Action::Enter,
        (KeyCode::Tab, KeyModifiers::NONE) => Action::TabNext,
        (KeyCode::BackTab, _) => Action::TabPrev,
        (KeyCode::Char('t'), KeyModifiers::NONE) => Action::ThemeNext,
        (KeyCode::Char('T'), KeyModifiers::NONE) => Action::ThemePrev,
        (KeyCode::Char('p'), KeyModifiers::NONE) => Action::ShowThemePicker,
        (KeyCode::Char('j'), KeyModifiers::CONTROL) => Action::ShowActivityCenter,
        _ => Action::Other,
    }
}
