use crate::{Session, media::Id, request::ApiVersion, session::SessionError};
use serde::Deserialize;

pub trait IntoArtistId {
    fn into_artist_id(&self) -> Id;
}
impl IntoArtistId for Id {
    fn into_artist_id(&self) -> Id {
        *self
    }
}

/// Artist extension
impl Session {
    pub async fn get_artist<I: IntoArtistId>(&mut self, id: &I) -> Result<Artist, SessionError> {
        let path = format!("artists/{}", id.into_artist_id());
        let resp = self.request(&path, &[], ApiVersion::V1).await?;
        let artist: Artist = resp.json().await?;
        Ok(artist)
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ArtistType {
    Main,
    Featured,
    Contributor,
    Artist,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistRole {
    pub category_id: i32,
    pub category: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShallowArtist {
    pub id: Id,
    pub name: String,
    pub handle: Option<String>,
    #[serde(rename = "type")]
    pub role: ArtistType,
    pub picture: Option<String>,
}
impl IntoArtistId for ShallowArtist {
    fn into_artist_id(&self) -> Id {
        self.id
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: Id,
    pub name: String,
    pub artist_types: Vec<ArtistType>,
    pub url: String,
    pub picture: Option<String>,
    pub selected_album_cover_fallback: Option<String>,
    pub popularity: u32,
    pub artist_roles: Vec<ArtistRole>,
    pub spotlighted: bool,
}
impl IntoArtistId for Artist {
    fn into_artist_id(&self) -> Id {
        self.id
    }
}
