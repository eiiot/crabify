mod action;
mod app;
mod auth;
mod config;
mod error;
mod event;
mod spotify;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::KeyCode;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;

use action::{Action, Event, IoEvent};
use app::{App, InputMode, Screen};
use event::EventHandler;
use spotify::SpotifyClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Authenticate with Spotify
    eprintln!("Authenticating with Spotify...");
    let spotify_auth = auth::authenticate().await?;
    let spotify_client = SpotifyClient::new(spotify_auth);

    // Verify connection
    eprintln!("Connected! Starting TUI...");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create channels
    let (io_tx, mut io_rx) = mpsc::unbounded_channel::<IoEvent>();
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();

    // Create app
    let mut app = App::new(io_tx);
    app.init();

    // Create event handler
    let mut events = EventHandler::new(Duration::from_millis(250));

    // Spawn network handler task
    let net_action_tx = action_tx.clone();
    tokio::spawn(async move {
        while let Some(io_event) = io_rx.recv().await {
            let result = handle_io_event(&spotify_client, io_event).await;
            let _ = net_action_tx.send(result);
        }
    });

    // Main loop
    loop {
        // Draw
        terminal.draw(|f| ui::render(f, &app))?;

        // Handle events
        tokio::select! {
            event = events.next() => {
                match event? {
                    Event::Key(key) => {
                        handle_key_event(&mut app, key);
                    }
                    Event::Tick => {
                        app.on_tick();
                    }
                    Event::Resize(_, _) => {
                        // Terminal will re-draw automatically
                    }
                }
            }
            Some(action) = action_rx.recv() => {
                app.update(action);
            }
        }

        if !app.running {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_key_event(app: &mut App, key: crossterm::event::KeyEvent) {
    // In editing mode, handle text input
    if app.input_mode == InputMode::Editing {
        match key.code {
            KeyCode::Enter => {
                app.on_enter();
            }
            KeyCode::Char(c) => {
                app.search_input.push(c);
            }
            KeyCode::Backspace => {
                app.search_input.pop();
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        return;
    }

    // Help overlay
    if app.show_help {
        match key.code {
            KeyCode::Char('?') | KeyCode::Esc => {
                app.show_help = false;
            }
            _ => {}
        }
        return;
    }

    // Normal mode
    match key.code {
        KeyCode::Char('q') => {
            app.running = false;
        }
        KeyCode::Char('?') => {
            app.show_help = true;
        }

        // Screen navigation
        KeyCode::BackTab => {
            app.prev_screen();
        }
        KeyCode::Tab => {
            if app.screen == Screen::Library {
                app.toggle_panel();
            } else {
                app.next_screen();
            }
        }

        // List navigation
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_down();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_up();
        }
        KeyCode::Enter => {
            app.on_enter();
        }

        // Search
        KeyCode::Char('/') => {
            app.screen = Screen::Search;
            app.input_mode = InputMode::Editing;
            app.search_input.clear();
        }

        // Playback controls
        KeyCode::Char(' ') => {
            app.dispatch_io(IoEvent::PlayPause);
        }
        KeyCode::Char('n') => {
            app.dispatch_io(IoEvent::NextTrack);
        }
        KeyCode::Char('p') => {
            app.dispatch_io(IoEvent::PreviousTrack);
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            app.dispatch_io(IoEvent::VolumeUp);
        }
        KeyCode::Char('-') => {
            app.dispatch_io(IoEvent::VolumeDown);
        }

        // Like toggle
        KeyCode::Char('s') => {
            app.toggle_like();
        }

        _ => {}
    }
}

async fn handle_io_event(client: &SpotifyClient, event: IoEvent) -> Action {
    match event {
        IoEvent::FetchNowPlaying => match client.fetch_now_playing().await {
            Ok(ctx) => Action::NowPlayingUpdated(ctx),
            Err(e) => Action::Error(format!("Failed to fetch playback: {}", e)),
        },
        IoEvent::PlayPause => {
            // First fetch current state to know if playing
            match client.fetch_now_playing().await {
                Ok(Some(ctx)) => {
                    let is_playing = ctx.is_playing;
                    match client.play_pause(is_playing).await {
                        Ok(()) => {
                            tokio::time::sleep(Duration::from_millis(200)).await;
                            match client.fetch_now_playing().await {
                                Ok(ctx) => Action::NowPlayingUpdated(ctx),
                                Err(e) => Action::Error(format!("Failed to fetch playback: {}", e)),
                            }
                        }
                        Err(e) => Action::Error(format!("Playback control failed: {}", e)),
                    }
                }
                Ok(None) => Action::Error("No active device found".to_string()),
                Err(e) => Action::Error(format!("Failed to fetch playback: {}", e)),
            }
        }
        IoEvent::NextTrack => match client.next_track().await {
            Ok(()) => {
                tokio::time::sleep(Duration::from_millis(300)).await;
                match client.fetch_now_playing().await {
                    Ok(ctx) => Action::NowPlayingUpdated(ctx),
                    Err(e) => Action::Error(format!("Failed to fetch playback: {}", e)),
                }
            }
            Err(e) => Action::Error(format!("Next track failed: {}", e)),
        },
        IoEvent::PreviousTrack => match client.previous_track().await {
            Ok(()) => {
                tokio::time::sleep(Duration::from_millis(300)).await;
                match client.fetch_now_playing().await {
                    Ok(ctx) => Action::NowPlayingUpdated(ctx),
                    Err(e) => Action::Error(format!("Failed to fetch playback: {}", e)),
                }
            }
            Err(e) => Action::Error(format!("Previous track failed: {}", e)),
        },
        IoEvent::VolumeUp => {
            match client.fetch_now_playing().await {
                Ok(Some(ctx)) => {
                    let current = ctx.device.volume_percent.unwrap_or(50) as u8;
                    let new_vol = (current + 5).min(100);
                    match client.set_volume(new_vol).await {
                        Ok(()) => {
                            tokio::time::sleep(Duration::from_millis(200)).await;
                            match client.fetch_now_playing().await {
                                Ok(ctx) => Action::NowPlayingUpdated(ctx),
                                Err(e) => Action::Error(format!("{}", e)),
                            }
                        }
                        Err(e) => Action::Error(format!("Volume change failed: {}", e)),
                    }
                }
                Ok(None) => Action::Error("No active device".to_string()),
                Err(e) => Action::Error(format!("{}", e)),
            }
        }
        IoEvent::VolumeDown => {
            match client.fetch_now_playing().await {
                Ok(Some(ctx)) => {
                    let current = ctx.device.volume_percent.unwrap_or(50) as u8;
                    let new_vol = current.saturating_sub(5);
                    match client.set_volume(new_vol).await {
                        Ok(()) => {
                            tokio::time::sleep(Duration::from_millis(200)).await;
                            match client.fetch_now_playing().await {
                                Ok(ctx) => Action::NowPlayingUpdated(ctx),
                                Err(e) => Action::Error(format!("{}", e)),
                            }
                        }
                        Err(e) => Action::Error(format!("Volume change failed: {}", e)),
                    }
                }
                Ok(None) => Action::Error("No active device".to_string()),
                Err(e) => Action::Error(format!("{}", e)),
            }
        }
        IoEvent::FetchPlaylists => match client.fetch_playlists().await {
            Ok(playlists) => Action::PlaylistsLoaded(playlists),
            Err(e) => Action::Error(format!("Failed to fetch playlists: {}", e)),
        },
        IoEvent::FetchPlaylistTracks(id) => match client.fetch_playlist_tracks(&id).await {
            Ok(tracks) => Action::PlaylistTracksLoaded(tracks),
            Err(e) => Action::Error(format!("Failed to fetch tracks: {}", e)),
        },
        IoEvent::PlayTrackInContext { context_uri, offset } => {
            match client.play_track_in_context(&context_uri, offset).await {
                Ok(()) => {
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    match client.fetch_now_playing().await {
                        Ok(ctx) => Action::NowPlayingUpdated(ctx),
                        Err(e) => Action::Error(format!("{}", e)),
                    }
                }
                Err(e) => Action::Error(format!("Failed to play track: {}", e)),
            }
        }
        IoEvent::PlayTrack(uri) => match client.play_track(&uri).await {
            Ok(()) => {
                tokio::time::sleep(Duration::from_millis(300)).await;
                match client.fetch_now_playing().await {
                    Ok(ctx) => Action::NowPlayingUpdated(ctx),
                    Err(e) => Action::Error(format!("{}", e)),
                }
            }
            Err(e) => Action::Error(format!("Failed to play track: {}", e)),
        },
        IoEvent::Search(query) => match client.search_tracks(&query).await {
            Ok(tracks) => Action::SearchResultsLoaded { tracks },
            Err(e) => Action::Error(format!("Search failed: {}", e)),
        },
        IoEvent::FetchLikedSongs => match client.fetch_liked_songs().await {
            Ok(songs) => Action::LikedSongsLoaded(songs),
            Err(e) => Action::Error(format!("Failed to fetch liked songs: {}", e)),
        },
        IoEvent::ToggleLike {
            track_id,
            currently_liked,
        } => {
            if currently_liked {
                match client.remove_track(&track_id).await {
                    Ok(()) => Action::LikeToggled {
                        track_id,
                        is_liked: false,
                    },
                    Err(e) => Action::Error(format!("Failed to unlike: {}", e)),
                }
            } else {
                match client.save_track(&track_id).await {
                    Ok(()) => Action::LikeToggled {
                        track_id,
                        is_liked: true,
                    },
                    Err(e) => Action::Error(format!("Failed to like: {}", e)),
                }
            }
        }
        IoEvent::FetchDevices => match client.fetch_devices().await {
            Ok(devices) => Action::DevicesLoaded(devices),
            Err(e) => Action::Error(format!("Failed to fetch devices: {}", e)),
        },
    }
}
