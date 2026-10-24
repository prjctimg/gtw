use std::io;
use std::time::Duration;

use crossterm::event::{self, Event};
use gtw_core::{AppState, Msg};
use gtw_render::{merged_themes, EffectManager};
use tokio::sync::mpsc;

use crate::{init, map_key, restore, Action, draw};

pub async fn run() -> io::Result<()> {
    let mut terminal = init()?;
    let mut state = AppState::default();
    let themes = merged_themes();
    let mut theme_idx = 0;
    let mut effects = EffectManager::new();
    effects.add_startup();
    
    // Simple demo: cycle through splash briefly
    let mut tick_count = 0;
    
    loop {
        terminal.draw(|f| draw(f, &state, &themes, theme_idx))?;
        
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    match map_key(key) {
                        Action::Quit => break,
                        Action::ThemeNext => {
                            theme_idx = (theme_idx + 1) % themes.len().max(1);
                        }
                        Action::ThemePrev => {
                            if theme_idx == 0 {
                                theme_idx = themes.len().saturating_sub(1);
                            } else {
                                theme_idx -= 1;
                            }
                        }
                        Action::ShowThemePicker => {
                            let _cmds = state.update(Msg::ShowThemePicker);
                        }
                        Action::Back => {
                            if matches!(state.screen, gtw_core::Screen::ThemePicker) {
                                state.screen = gtw_core::Screen::Splash;
                            }
                        }
                        _ => {}
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        
        tick_count += 1;
        if tick_count > 10 && matches!(state.screen, gtw_core::Screen::Splash) {
            state.screen = gtw_core::Screen::Home;
        }
    }
    
    restore()?;
    Ok(())
}
