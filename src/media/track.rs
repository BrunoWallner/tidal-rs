use base64::prelude::*;
use serde::Deserialize;

use super::AudioQuality;
use crate::media::Id;
use crate::media::MimeType;
use crate::media::ShallowAlbum;
use crate::media::ShallowArtist;
use crate::request::ApiVersion;
use crate::session::{Session, SessionError};

pub trait IntoTrackId {
    fn into_track_id(&self) -> Id;
}
impl IntoTrackId for Id {
    fn into_track_id(&self) -> Id {
        *self
    }
}

/// Track extension
impl Session {
    pub async fn get_track<I: IntoTrackId>(&mut self, id: &I) -> Result<Track, SessionError> {
        let path = format!("tracks/{}", id.into_track_id());
        let resp = self.request(&path, &[], ApiVersion::V1).await?;
        let track: Track = resp.json().await?;
        Ok(track)
    }

    pub async fn get_track_stream<I: IntoTrackId>(
        &mut self,
        id: &I,
    ) -> Result<TrackStream, SessionError> {
        let path = format!("tracks/{}/playbackinfo", id.into_track_id());
        let query = &[
            ("playbackmode", "STREAM"),
            ("audioquality", self.config.audio_quality.into()),
            ("assetpresentation", "FULL"),
        ];
        let resp = self.request(&path, query, ApiVersion::V1).await?;
        let track_stream_response: TrackStreamResponse = resp.json().await?;
        let track_stream = TrackStream::try_from(track_stream_response)?;
        Ok(track_stream)
    }
}

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
    pub copyright: Option<String>,
    pub bpm: Option<u32>,
    pub url: String,
    pub isrc: String,
    pub explicit: bool,
    pub audio_quality: AudioQuality,
    pub audio_modes: Vec<AudioMode>,
    pub artist: Option<ShallowArtist>,
    pub artists: Vec<ShallowArtist>,
    pub album: ShallowAlbum,
}
impl IntoTrackId for Track {
    fn into_track_id(&self) -> Id {
        self.id
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrackStreamResponse {
    track_id: Id,
    asset_presentation: String,
    audio_mode: AudioMode,
    audio_quality: AudioQuality,
    manifest_mime_type: String,
    manifest_hash: String,
    manifest: String,
    album_replay_gain: f32,
    album_peak_amplitude: f32,
    track_replay_gain: f32,
    track_peak_amplitude: f32,
    bit_depth: u8,
    sample_rate: u32,
}
impl IntoTrackId for TrackStreamResponse {
    fn into_track_id(&self) -> Id {
        self.track_id
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub mime_type: MimeType,
    pub codecs: String,
    pub encryption_type: String,
    pub urls: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TrackStream {
    pub track_id: Id,
    pub asset_presentation: String,
    pub audio_mode: AudioMode,
    pub audio_quality: AudioQuality,
    pub manifest_mime_type: String,
    pub manifest_hash: String,
    pub manifest: Manifest,
    pub album_replay_gain: f32,
    pub album_peak_amplitude: f32,
    pub track_replay_gain: f32,
    pub track_peak_amplitude: f32,
    pub bit_depth: u8,
    pub sample_rate: u32,
}
impl TryFrom<TrackStreamResponse> for TrackStream {
    type Error = SessionError;

    fn try_from(value: TrackStreamResponse) -> Result<Self, Self::Error> {
        let manifest = BASE64_STANDARD
            .decode(value.manifest)
            .map_err(|_| SessionError::ManifestDecode)?;
        let manifest: Manifest =
            serde_json::from_slice(&manifest).map_err(|_| SessionError::ManifestDecode)?;
        let track_stream = TrackStream {
            track_id: value.track_id,
            asset_presentation: value.asset_presentation,
            audio_mode: value.audio_mode,
            audio_quality: value.audio_quality,
            manifest_mime_type: value.manifest_mime_type,
            manifest_hash: value.manifest_hash,
            manifest: manifest,
            album_replay_gain: value.album_replay_gain,
            album_peak_amplitude: value.album_peak_amplitude,
            track_replay_gain: value.track_replay_gain,
            track_peak_amplitude: value.track_peak_amplitude,
            bit_depth: value.bit_depth,
            sample_rate: value.sample_rate,
        };
        Ok(track_stream)
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
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
