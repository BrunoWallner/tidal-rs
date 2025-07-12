use serde::Deserialize;

use crate::{
    Session,
    media::{AudioMode, AudioQuality, Color, Id, ShallowArtist, Track},
    session::SessionError,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShallowAlbum {
    pub id: Id,
    pub title: String,
    pub cover: String,
    pub vibrant_color: Color,
    pub video_cover: Option<String>,
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
    pub copyright: String,
    pub version: Option<String>,
    pub url: String,
    pub cover: String,
    pub vibrant_color: Color,
    pub video_cover: Option<String>,
    pub explicit: bool,
    pub upc: String,
    pub popularity: u32,
    pub audio_quality: AudioQuality,
    pub audio_modes: Vec<AudioMode>,
    pub artist: Option<ShallowArtist>,
    pub artists: Vec<ShallowArtist>,
}
impl Album {
    pub async fn get(id: Id, session: &mut Session) -> Result<Self, SessionError> {
        let Some(oauth) = &mut session.oauth else {
            Err(SessionError::NotLoggedInOauth)?
        };
        let path = format!("albums/{id}");
        let resp = session
            .client
            .tidal_request(&path, &[], oauth, session.info.as_ref())
            .await?;

        let album: Album = resp.json().await?;
        Ok(album)
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
impl AlbumTracks {
    pub async fn get(id: Id, session: &mut Session) -> Result<Self, SessionError> {
        let Some(oauth) = &mut session.oauth else {
            Err(SessionError::NotLoggedInOauth)?
        };
        let path = format!("albums/{id}/tracks");
        let resp = session
            .client
            .tidal_request(&path, &[], oauth, session.info.as_ref())
            .await?;

        // println!("{}", resp.text().await?);
        // todo!()
        let tracks: AlbumTracks = resp.json().await?;
        Ok(tracks)
    }
}
