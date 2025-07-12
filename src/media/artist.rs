use serde::Deserialize;

use crate::{Session, media::Id, session::SessionError};

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
impl Artist {
    pub async fn get(id: Id, session: &mut Session) -> Result<Self, SessionError> {
        let Some(oauth) = &mut session.oauth else {
            Err(SessionError::NotLoggedInOauth)?
        };
        let path = format!("artists/{id}");
        let resp = session
            .client
            .tidal_request(&path, &[], oauth, session.info.as_ref())
            .await?;

        let artist: Artist = resp.json().await?;
        Ok(artist)
    }
}
