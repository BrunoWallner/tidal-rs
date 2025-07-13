use serde::Deserialize;

use crate::{
    Session,
    media::{AudioMode, AudioQuality, Color, Id, ShallowArtist, Track},
    request::ApiVersion,
    session::SessionError,
};

pub trait IntoAlbumId {
    fn into_album_id(&self) -> Id;
}
impl IntoAlbumId for Id {
    fn into_album_id(&self) -> Id {
        *self
    }
}

/// Album extension
impl Session {
    pub async fn get_album<I: IntoAlbumId>(&mut self, id: &I) -> Result<Album, SessionError> {
        let path = format!("albums/{}", id.into_album_id());
        let resp = self.request(&path, &[], ApiVersion::V1).await?;
        let album: Album = resp.json().await?;
        Ok(album)
    }

    pub async fn get_album_tracks<I: IntoAlbumId>(
        &mut self,
        id: &I,
    ) -> Result<AlbumTracks, SessionError> {
        let path = format!("albums/{}/tracks", id.into_album_id());
        let resp = self.request(&path, &[], ApiVersion::V1).await?;
        let album: AlbumTracks = resp.json().await?;
        Ok(album)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShallowAlbum {
    pub id: Id,
    pub title: String,
    pub cover: String,
    pub vibrant_color: Option<Color>,
    pub video_cover: Option<String>,
}
impl IntoAlbumId for ShallowAlbum {
    fn into_album_id(&self) -> Id {
        self.id
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: Id,
    pub title: String,
    pub duration: u32,
    pub dj_ready: bool,
    pub stem_ready: bool,
    pub stream_start_date: chrono::DateTime<chrono::Utc>,
    pub allow_streaming: bool,
    pub number_of_tracks: u32,
    pub number_of_videos: u32,
    pub number_of_volumes: u32,
    // @todo: impl proper parsing: yyyy-mm-dd?
    pub release_date: String,
    pub copyright: Option<String>,
    pub version: Option<String>,
    pub url: String,
    pub cover: String,
    pub vibrant_color: Option<Color>,
    pub video_cover: Option<String>,
    pub explicit: bool,
    pub upc: String,
    pub popularity: u32,
    pub audio_quality: AudioQuality,
    pub audio_modes: Vec<AudioMode>,
    pub artist: Option<ShallowArtist>,
    pub artists: Vec<ShallowArtist>,
}
impl IntoAlbumId for Album {
    fn into_album_id(&self) -> Id {
        self.id
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumTracks {
    pub limit: u32,
    pub offset: u32,
    pub total_number_of_items: u32,
    pub items: Vec<Track>,
}
