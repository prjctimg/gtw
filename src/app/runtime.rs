use std::io;
use std::time::Duration;

use crate::core::{AppState, Msg, Screen};
use crate::render::{EffectManager, merged_themes};
use crossterm::event::{self, Event};

use crate::app::{Action, draw, init, map_key, restore};

pub async fn run() -> io::Result<()> {
    let mut terminal = init()?;
    let mut state = AppState::default();
    let themes = merged_themes();
    let mut theme_idx = 0;
    let mut effects = EffectManager::new();
    effects.add_startup();

    let mut tick_count = 0;

    loop {
        if state.should_quit {
            break;
        }
        terminal.draw(|f| draw(f, &state, &themes, theme_idx))?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => match map_key(key) {
                    Action::Quit => {
                        let _ = state.update(Msg::Quit);
                        break;
                    }
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
                        if matches!(state.screen, Screen::ThemePicker | Screen::Search) {
                            state.screen = Screen::Home;
                        }
                    }
                    _ => {}
                },
                Event::Resize(_, _) => {}
                _ => {}
            }
        }

        tick_count += 1;
        if tick_count > 15 && matches!(state.screen, Screen::Splash) {
            state.screen = Screen::Home;
        }
        effects.tick(Duration::from_millis(50));
    }

    restore()?;
    Ok(())
}
