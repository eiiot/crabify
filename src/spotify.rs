use anyhow::Result;
use rspotify::model::{
    CurrentPlaybackContext, Device, FullTrack, Market, SavedTrack, SearchType,
    SimplifiedPlaylist, PlayableItem, PlaylistId, TrackId,
};
use rspotify::prelude::*;
use rspotify::AuthCodePkceSpotify;

pub struct SpotifyClient {
    client: AuthCodePkceSpotify,
}

impl SpotifyClient {
    pub fn new(client: AuthCodePkceSpotify) -> Self {
        Self { client }
    }

    pub async fn fetch_now_playing(&self) -> Result<Option<CurrentPlaybackContext>> {
        let market = Some(Market::FromToken);
        let result = self
            .client
            .current_playback(market, None::<Vec<_>>)
            .await?;
        Ok(result)
    }

    pub async fn play_pause(&self, is_playing: bool) -> Result<()> {
        if is_playing {
            self.client.pause_playback(None).await?;
        } else {
            self.client.resume_playback(None, None).await?;
        }
        Ok(())
    }

    pub async fn next_track(&self) -> Result<()> {
        self.client.next_track(None).await?;
        Ok(())
    }

    pub async fn previous_track(&self) -> Result<()> {
        self.client.previous_track(None).await?;
        Ok(())
    }

    pub async fn set_volume(&self, volume_percent: u8) -> Result<()> {
        self.client
            .volume(volume_percent, None)
            .await?;
        Ok(())
    }

    pub async fn fetch_playlists(&self) -> Result<Vec<SimplifiedPlaylist>> {
        let mut playlists = Vec::new();
        let mut offset = 0;
        let limit = 50;
        loop {
            let page = self
                .client
                .current_user_playlists_manual(Some(limit), Some(offset))
                .await?;
            let total = page.total;
            playlists.extend(page.items);
            offset += limit;
            if offset >= total {
                break;
            }
        }
        Ok(playlists)
    }

    pub async fn fetch_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<FullTrack>> {
        let playlist_id = PlaylistId::from_id_or_uri(playlist_id)?;
        let mut tracks = Vec::new();
        let mut offset = 0;
        let limit = 100;
        loop {
            let page = self
                .client
                .playlist_items_manual(playlist_id.as_ref(), None, None, Some(limit), Some(offset))
                .await?;
            let total = page.total;
            for item in page.items {
                if let Some(PlayableItem::Track(track)) = item.track {
                    tracks.push(track);
                }
            }
            offset += limit;
            if offset >= total {
                break;
            }
        }
        Ok(tracks)
    }

    pub async fn play_track_in_context(
        &self,
        context_uri: &str,
        offset: usize,
    ) -> Result<()> {
        use rspotify::model::PlayContextId;

        // Parse the context URI to determine the type
        let context_id: PlayContextId = if context_uri.contains(":playlist:") {
            let id = context_uri.split(':').last().unwrap_or("");
            PlayContextId::Playlist(PlaylistId::from_id(id)?.into_static())
        } else if context_uri.contains(":album:") {
            let id = context_uri.split(':').last().unwrap_or("");
            PlayContextId::Album(rspotify::model::AlbumId::from_id(id)?.into_static())
        } else if context_uri.contains(":artist:") {
            let id = context_uri.split(':').last().unwrap_or("");
            PlayContextId::Artist(rspotify::model::ArtistId::from_id(id)?.into_static())
        } else {
            anyhow::bail!("Unsupported context URI: {}", context_uri);
        };

        let offset = Some(rspotify::model::Offset::Position(
            chrono::Duration::milliseconds(offset as i64),
        ));

        self.client
            .start_context_playback(context_id, None, offset, None)
            .await?;
        Ok(())
    }

    pub async fn play_track(&self, track_uri: &str) -> Result<()> {
        let track_id = track_uri.split(':').last().unwrap_or(track_uri);
        let track_id = TrackId::from_id(track_id)?;
        let uris = [PlayableId::Track(track_id)];
        self.client
            .start_uris_playback(uris, None, None, None)
            .await?;
        Ok(())
    }

    pub async fn search_tracks(&self, query: &str) -> Result<Vec<FullTrack>> {
        let result = self
            .client
            .search(query, SearchType::Track, None, None, Some(20), Some(0))
            .await?;

        let mut tracks = Vec::new();
        if let rspotify::model::SearchResult::Tracks(page) = result {
            tracks = page.items;
        }
        Ok(tracks)
    }

    pub async fn fetch_liked_songs(&self) -> Result<Vec<SavedTrack>> {
        let mut songs = Vec::new();
        let mut offset = 0;
        let limit = 50;
        loop {
            let page = self
                .client
                .current_user_saved_tracks_manual(None, Some(limit), Some(offset))
                .await?;
            let total = page.total;
            songs.extend(page.items);
            offset += limit;
            if offset >= total || songs.len() >= 200 {
                break;
            }
        }
        Ok(songs)
    }

    pub async fn save_track(&self, track_id: &str) -> Result<()> {
        let track_id = TrackId::from_id(track_id)?;
        self.client
            .current_user_saved_tracks_add([track_id])
            .await?;
        Ok(())
    }

    pub async fn remove_track(&self, track_id: &str) -> Result<()> {
        let track_id = TrackId::from_id(track_id)?;
        self.client
            .current_user_saved_tracks_delete([track_id])
            .await?;
        Ok(())
    }

    pub async fn check_saved_tracks(&self, track_ids: &[String]) -> Result<Vec<bool>> {
        let ids: Vec<TrackId> = track_ids
            .iter()
            .filter_map(|id| TrackId::from_id(id).ok())
            .collect();
        let result = self.client.current_user_saved_tracks_contains(ids).await?;
        Ok(result)
    }

    pub async fn fetch_devices(&self) -> Result<Vec<Device>> {
        let devices = self.client.device().await?;
        Ok(devices)
    }
}
