use std::io;
use std::time::Duration;

use crossterm::event::{self, Event};
use gtw_core::{AppState, Msg, Screen};
use gtw_render::{EffectManager, merged_themes};

use crate::{Action, draw, init, map_key, restore};

pub async fn run() -> io::Result<()> {
    let mut terminal = init()?;
    let mut state = AppState::default();
    let themes = merged_themes();
    let mut theme_idx = 0;
    let mut effects = EffectManager::new();
    effects.add_startup();

    let mut tick_count = 0;

    loop {
        terminal.draw(|f| draw(f, &state, &themes, theme_idx))?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => match map_key(key) {
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
                        if matches!(state.screen, Screen::ThemePicker) {
                            state.screen = Screen::Splash;
                        }
                    }
                    _ => {}
                },
                Event::Resize(_, _) => {}
                _ => {}
            }
        }

        tick_count += 1;
        if tick_count > 10 && matches!(state.screen, Screen::Splash) {
            state.screen = Screen::Home;
        }
    }

    restore()?;
    Ok(())
}
