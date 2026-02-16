use rspotify::model::{
    CurrentPlaybackContext, FullTrack, SavedTrack, SimplifiedPlaylist,
};

/// Events sent from the event handler to the main loop.
#[derive(Debug)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Tick,
    Resize(u16, u16),
}

/// IO requests sent from the app to the network handler.
#[derive(Debug)]
pub enum IoEvent {
    FetchNowPlaying,
    PlayPause,
    NextTrack,
    PreviousTrack,
    VolumeUp,
    VolumeDown,
    FetchPlaylists,
    FetchPlaylistTracks(String), // playlist ID
    PlayTrackInContext {
        context_uri: String,
        offset: usize,
    },
    PlayTrack(String), // track URI
    Search(String),
    FetchLikedSongs,
    ToggleLike {
        track_id: String,
        currently_liked: bool,
    },
    FetchDevices,
}

/// Actions dispatched to update App state.
#[derive(Debug)]
pub enum Action {
    NowPlayingUpdated(Option<CurrentPlaybackContext>),
    PlaylistsLoaded(Vec<SimplifiedPlaylist>),
    PlaylistTracksLoaded(Vec<FullTrack>),
    SearchResultsLoaded {
        tracks: Vec<FullTrack>,
    },
    LikedSongsLoaded(Vec<SavedTrack>),
    LikeToggled {
        track_id: String,
        is_liked: bool,
    },
    Error(String),
    DevicesLoaded(Vec<rspotify::model::Device>),
}
