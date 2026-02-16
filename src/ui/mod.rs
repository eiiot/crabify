pub mod layout;
pub mod library;
pub mod search;
pub mod now_playing;
pub mod liked_songs;
pub mod help;

use ratatui::Frame;

use crate::app::{App, Screen};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = layout::main_layout(f.area());

    // Header (tabs)
    layout::render_tabs(f, app, chunks[0]);

    // Body (screen-specific content)
    match app.screen {
        Screen::Library => library::render(f, app, chunks[1]),
        Screen::Search => search::render(f, app, chunks[1]),
        Screen::LikedSongs => liked_songs::render(f, app, chunks[1]),
    }

    // Footer (now playing)
    now_playing::render(f, app, chunks[2]);

    // Flash message overlay
    if let Some((ref msg, _)) = app.flash_message {
        layout::render_flash(f, msg);
    }

    // Help overlay
    if app.show_help {
        help::render(f);
    }
}
