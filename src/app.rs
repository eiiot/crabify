use rspotify::model::{
    CurrentPlaybackContext, Device, FullTrack, PlayableItem, SavedTrack, SimplifiedPlaylist,
};
use tokio::sync::mpsc;

use crate::action::{Action, IoEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Library,
    Search,
    LikedSongs,
}

impl Screen {
    pub fn all() -> &'static [Screen] {
        &[Screen::Library, Screen::Search, Screen::LikedSongs]
    }

    pub fn label(&self) -> &str {
        match self {
            Screen::Library => "Library",
            Screen::Search => "Search",
            Screen::LikedSongs => "Liked Songs",
        }
    }

    pub fn next(&self) -> Screen {
        match self {
            Screen::Library => Screen::Search,
            Screen::Search => Screen::LikedSongs,
            Screen::LikedSongs => Screen::Library,
        }
    }

    pub fn prev(&self) -> Screen {
        match self {
            Screen::Library => Screen::LikedSongs,
            Screen::Search => Screen::Library,
            Screen::LikedSongs => Screen::Search,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Left,
    Right,
}

pub struct App {
    pub running: bool,
    pub screen: Screen,
    pub input_mode: InputMode,
    pub active_panel: Panel,
    pub show_help: bool,

    // Now playing
    pub now_playing: Option<CurrentPlaybackContext>,
    pub is_playing: bool,
    pub volume: u8,

    // Library
    pub playlists: Vec<SimplifiedPlaylist>,
    pub playlist_index: usize,
    pub playlist_tracks: Vec<FullTrack>,
    pub track_index: usize,
    pub selected_playlist_id: Option<String>,

    // Search
    pub search_input: String,
    pub search_results: Vec<FullTrack>,
    pub search_index: usize,

    // Liked songs
    pub liked_songs: Vec<SavedTrack>,
    pub liked_index: usize,
    pub liked_track_ids: std::collections::HashSet<String>,

    // Devices
    pub devices: Vec<Device>,

    // Status/error messages
    pub flash_message: Option<(String, std::time::Instant)>,
    pub loading: bool,

    // IO channel
    pub io_tx: mpsc::UnboundedSender<IoEvent>,

    // Tick counter for polling
    tick_count: u32,

    // Local progress interpolation
    last_playback_update: Option<std::time::Instant>,
}

impl App {
    pub fn new(io_tx: mpsc::UnboundedSender<IoEvent>) -> Self {
        Self {
            running: true,
            screen: Screen::Library,
            input_mode: InputMode::Normal,
            active_panel: Panel::Left,
            show_help: false,
            now_playing: None,
            is_playing: false,
            volume: 50,
            playlists: Vec::new(),
            playlist_index: 0,
            playlist_tracks: Vec::new(),
            track_index: 0,
            selected_playlist_id: None,
            search_input: String::new(),
            search_results: Vec::new(),
            search_index: 0,
            liked_songs: Vec::new(),
            liked_index: 0,
            liked_track_ids: std::collections::HashSet::new(),
            devices: Vec::new(),
            flash_message: None,
            loading: false,
            io_tx,
            tick_count: 0,
            last_playback_update: None,
        }
    }

    pub fn dispatch_io(&self, event: IoEvent) {
        let _ = self.io_tx.send(event);
    }

    pub fn on_tick(&mut self) {
        self.tick_count += 1;

        // Poll now playing every ~5 seconds (20 ticks at 250ms)
        if self.tick_count % 20 == 0 {
            self.dispatch_io(IoEvent::FetchNowPlaying);
        }

        // Clear flash messages after 5 seconds
        if let Some((_, instant)) = &self.flash_message {
            if instant.elapsed() > std::time::Duration::from_secs(5) {
                self.flash_message = None;
            }
        }
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::NowPlayingUpdated(ctx) => {
                if let Some(ref ctx) = ctx {
                    self.is_playing = ctx.is_playing;
                    if let Some(ref device) = ctx.device.volume_percent {
                        self.volume = *device as u8;
                    }
                }
                self.now_playing = ctx;
                self.last_playback_update = Some(std::time::Instant::now());
            }
            Action::PlaylistsLoaded(playlists) => {
                self.playlists = playlists;
                self.playlist_index = 0;
                self.loading = false;
            }
            Action::PlaylistTracksLoaded(tracks) => {
                self.playlist_tracks = tracks;
                self.track_index = 0;
                self.loading = false;
            }
            Action::SearchResultsLoaded { tracks } => {
                self.search_results = tracks;
                self.search_index = 0;
                self.loading = false;
            }
            Action::LikedSongsLoaded(songs) => {
                self.liked_track_ids.clear();
                for song in &songs {
                    if let Some(ref id) = song.track.id {
                        self.liked_track_ids.insert(id.to_string());
                    }
                }
                self.liked_songs = songs;
                self.liked_index = 0;
                self.loading = false;
            }
            Action::LikeToggled { track_id, is_liked } => {
                if is_liked {
                    self.liked_track_ids.insert(track_id);
                } else {
                    self.liked_track_ids.remove(&track_id);
                }
            }
            Action::Error(msg) => {
                self.flash_message = Some((msg, std::time::Instant::now()));
                self.loading = false;
            }
            Action::DevicesLoaded(devices) => {
                self.devices = devices;
            }
        }
    }

    pub fn set_flash(&mut self, msg: String) {
        self.flash_message = Some((msg, std::time::Instant::now()));
    }

    // Navigation helpers

    pub fn next_screen(&mut self) {
        self.screen = self.screen.next();
        self.on_screen_change();
    }

    pub fn prev_screen(&mut self) {
        self.screen = self.screen.prev();
        self.on_screen_change();
    }

    fn on_screen_change(&mut self) {
        self.active_panel = Panel::Left;
        match self.screen {
            Screen::Library => {
                if self.playlists.is_empty() {
                    self.loading = true;
                    self.dispatch_io(IoEvent::FetchPlaylists);
                }
            }
            Screen::LikedSongs => {
                if self.liked_songs.is_empty() {
                    self.loading = true;
                    self.dispatch_io(IoEvent::FetchLikedSongs);
                }
            }
            Screen::Search => {}
        }
    }

    pub fn move_up(&mut self) {
        match self.screen {
            Screen::Library => {
                if self.active_panel == Panel::Left {
                    if self.playlist_index > 0 {
                        self.playlist_index -= 1;
                    }
                } else if self.track_index > 0 {
                    self.track_index -= 1;
                }
            }
            Screen::Search => {
                if self.search_index > 0 {
                    self.search_index -= 1;
                }
            }
            Screen::LikedSongs => {
                if self.liked_index > 0 {
                    self.liked_index -= 1;
                }
            }
        }
    }

    pub fn move_down(&mut self) {
        match self.screen {
            Screen::Library => {
                if self.active_panel == Panel::Left {
                    if !self.playlists.is_empty()
                        && self.playlist_index < self.playlists.len() - 1
                    {
                        self.playlist_index += 1;
                    }
                } else if !self.playlist_tracks.is_empty()
                    && self.track_index < self.playlist_tracks.len() - 1
                {
                    self.track_index += 1;
                }
            }
            Screen::Search => {
                if !self.search_results.is_empty()
                    && self.search_index < self.search_results.len() - 1
                {
                    self.search_index += 1;
                }
            }
            Screen::LikedSongs => {
                if !self.liked_songs.is_empty()
                    && self.liked_index < self.liked_songs.len() - 1
                {
                    self.liked_index += 1;
                }
            }
        }
    }

    pub fn toggle_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Left => Panel::Right,
            Panel::Right => Panel::Left,
        };
    }

    pub fn on_enter(&mut self) {
        match self.screen {
            Screen::Library => {
                if self.active_panel == Panel::Left {
                    // Select playlist, fetch tracks
                    if let Some(playlist) = self.playlists.get(self.playlist_index) {
                        let id = playlist.id.to_string();
                        self.selected_playlist_id = Some(id.clone());
                        self.loading = true;
                        self.dispatch_io(IoEvent::FetchPlaylistTracks(id));
                        self.active_panel = Panel::Right;
                    }
                } else {
                    // Play selected track in playlist context
                    if let Some(ref playlist_id) = self.selected_playlist_id {
                        self.dispatch_io(IoEvent::PlayTrackInContext {
                            context_uri: playlist_id.clone(),
                            offset: self.track_index,
                        });
                    }
                }
            }
            Screen::Search => {
                if self.input_mode == InputMode::Editing {
                    // Submit search
                    let query = self.search_input.clone();
                    if !query.is_empty() {
                        self.loading = true;
                        self.dispatch_io(IoEvent::Search(query));
                    }
                    self.input_mode = InputMode::Normal;
                } else {
                    // Play selected search result
                    if let Some(track) = self.search_results.get(self.search_index) {
                        if let Some(ref id) = track.id {
                            self.dispatch_io(IoEvent::PlayTrack(id.to_string()));
                        }
                    }
                }
            }
            Screen::LikedSongs => {
                if let Some(saved_track) = self.liked_songs.get(self.liked_index) {
                    if let Some(ref id) = saved_track.track.id {
                        self.dispatch_io(IoEvent::PlayTrack(id.to_string()));
                    }
                }
            }
        }
    }

    pub fn now_playing_track_id(&self) -> Option<String> {
        self.now_playing.as_ref().and_then(|ctx| {
            ctx.item.as_ref().and_then(|item| match item {
                PlayableItem::Track(t) => t.id.as_ref().map(|id| id.to_string()),
                _ => None,
            })
        })
    }

    pub fn toggle_like(&mut self) {
        let track_id = match self.screen {
            Screen::Library => {
                self.playlist_tracks
                    .get(self.track_index)
                    .and_then(|t| t.id.as_ref())
                    .map(|id| id.to_string())
            }
            Screen::Search => {
                self.search_results
                    .get(self.search_index)
                    .and_then(|t| t.id.as_ref())
                    .map(|id| id.to_string())
            }
            Screen::LikedSongs => {
                self.liked_songs
                    .get(self.liked_index)
                    .and_then(|t| t.track.id.as_ref())
                    .map(|id| id.to_string())
            }
        }
        .or_else(|| self.now_playing_track_id());

        if let Some(id) = track_id {
            let currently_liked = self.liked_track_ids.contains(&id);
            self.dispatch_io(IoEvent::ToggleLike {
                track_id: id,
                currently_liked,
            });
        }
    }

    pub fn toggle_like_now_playing(&mut self) {
        if let Some(id) = self.now_playing_track_id() {
            let currently_liked = self.liked_track_ids.contains(&id);
            self.dispatch_io(IoEvent::ToggleLike {
                track_id: id,
                currently_liked,
            });
        }
    }

    pub fn current_track_name(&self) -> Option<String> {
        self.now_playing.as_ref().and_then(|ctx| {
            ctx.item.as_ref().map(|item| match item {
                PlayableItem::Track(track) => {
                    let artists: Vec<&str> =
                        track.artists.iter().map(|a| a.name.as_str()).collect();
                    format!("{} - {}", track.name, artists.join(", "))
                }
                PlayableItem::Episode(ep) => ep.name.clone(),
            })
        })
    }

    fn interpolated_progress_ms(&self) -> Option<(i64, i64)> {
        let ctx = self.now_playing.as_ref()?;
        let base_ms = ctx.progress.map(|d| d.num_milliseconds()).unwrap_or(0);
        let duration_ms = ctx.item.as_ref().map(|item| match item {
            PlayableItem::Track(t) => t.duration.num_milliseconds(),
            PlayableItem::Episode(e) => e.duration.num_milliseconds(),
        }).unwrap_or(0);

        let elapsed = self.last_playback_update
            .map(|t| t.elapsed().as_millis() as i64)
            .unwrap_or(0);

        let progress = if self.is_playing {
            (base_ms + elapsed).min(duration_ms)
        } else {
            base_ms
        };

        Some((progress, duration_ms))
    }

    pub fn progress_fraction(&self) -> f64 {
        self.now_playing
            .as_ref()
            .map(|ctx| {
                let progress = ctx
                    .progress
                    .map(|d| d.num_milliseconds() as f64)
                    .unwrap_or(0.0);
                let duration = ctx
                    .item
                    .as_ref()
                    .map(|item| match item {
                        PlayableItem::Track(t) => t.duration.num_milliseconds() as f64,
                        PlayableItem::Episode(e) => e.duration.num_milliseconds() as f64,
                    })
                    .unwrap_or(1.0);
                if duration > 0.0 {
                    progress / duration
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0)
    }

    pub fn progress_text(&self) -> String {
        self.interpolated_progress_ms()
            .map(|(progress_ms, duration_ms)| {
                format!(
                    "{} / {}",
                    format_duration(progress_ms),
                    format_duration(duration_ms)
                )
            })
            .unwrap_or_default()
    }

    pub fn init(&mut self) {
        self.dispatch_io(IoEvent::FetchNowPlaying);
        self.dispatch_io(IoEvent::FetchPlaylists);
        self.dispatch_io(IoEvent::FetchDevices);
    }
}

fn format_duration(ms: i64) -> String {
    let total_secs = ms / 1000;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{}:{:02}", mins, secs)
}
