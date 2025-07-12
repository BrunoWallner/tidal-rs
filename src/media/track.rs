use serde::Deserialize;

use super::AudioQuality;
use crate::media::Id;
use crate::media::ShallowAlbum;
use crate::media::ShallowArtist;
use crate::session::{Session, SessionError};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: Id,
    pub title: String,
    pub duration: u64,
    pub replay_gain: f32,
    pub peak: f32,
    pub allow_streaming: bool,
    pub stream_ready: bool,
    pub dj_ready: bool,
    pub stem_ready: bool,
    pub stream_start_date: chrono::DateTime<chrono::Utc>,
    pub track_number: u32,
    pub volume_number: u32,
    pub popularity: u32,
    pub copyright: String,
    pub bpm: u32,
    pub url: String,
    pub isrc: String,
    pub explicit: bool,
    pub audio_quality: AudioQuality,
    pub audio_modes: Vec<AudioMode>,
    pub artist: Option<ShallowArtist>,
    pub artists: Vec<ShallowArtist>,
    pub album: ShallowAlbum,
}
impl Track {
    pub async fn get(id: Id, session: &mut Session) -> Result<Self, SessionError> {
        let Some(oauth) = &mut session.oauth else {
            Err(SessionError::NotLoggedInOauth)?
        };
        let path = format!("tracks/{id}");
        let resp = session
            .client
            .tidal_request(&path, &[], oauth, session.info.as_ref())
            .await?;

        let track: Track = resp.json().await?;
        Ok(track)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackStream {
    pub track_id: Id,
    pub asset_presentation: String,
    pub audio_mode: AudioMode,
    pub audio_quality: AudioQuality,
    pub manifest_mime_type: String,
    pub manifest_hash: String,
    pub manifest: String,
    pub album_replay_gain: f32,
    pub album_peak_amplitude: f32,
    pub track_replay_gain: f32,
    pub track_peak_amplitude: f32,
    pub bit_depth: u8,
    pub sample_rate: u32,
}
impl TrackStream {
    pub async fn get(id: Id, session: &mut Session) -> Result<Self, SessionError> {
        let Some(oauth) = &mut session.oauth else {
            Err(SessionError::NotLoggedInOauth)?
        };
        let path = format!("tracks/{id}/playbackinfopostpaywall");
        let query = &[
            ("playbackmode", "STREAM"),
            ("audioquality", session.config.audio_quality.into()),
            ("assetpresentation", "FULL"),
        ];
        let resp = session
            .client
            .tidal_request(&path, query, oauth, session.info.as_ref())
            .await?;

        let track_stream: TrackStream = resp.json().await?;
        Ok(track_stream)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackStreamUrl {
    pub urls: Vec<String>,
    pub track_id: Id,
    pub asset_presentation: String,
    pub audio_quality: AudioQuality,
    pub audio_mode: AudioMode,
    pub streaming_session_id: Option<String>,
    pub codec: Codec,
    pub security_type: Option<String>,
    pub security_token: Option<String>,
}
impl TrackStreamUrl {
    pub async fn get(id: Id, session: &mut Session) -> Result<Self, SessionError> {
        let Some(oauth) = &mut session.oauth else {
            Err(SessionError::NotLoggedInOauth)?
        };
        let path = format!("tracks/{id}/urlpostpaywall");
        let query = &[
            ("urlusagemode", "STREAM"),
            ("audioquality", session.config.audio_quality.into()),
            ("assetpresentation", "FULL"),
        ];
        let resp = session
            .client
            .tidal_request(&path, query, oauth, session.info.as_ref())
            .await?;

        let track_stream_url: TrackStreamUrl = resp.json().await?;
        Ok(track_stream_url)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AudioMode {
    // STEREO
    Stereo,
    // DOLBY_ATMOS
    DolbyAtmos,
}

#[derive(Debug, Deserialize)]
pub enum Codec {
    MP3,
    AAC,
    MP4A,
    FLAC,
    /// Atmos
    EAC3,
    AC4,
}
