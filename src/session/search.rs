use core::fmt;

use bitflags::bitflags;
use serde::Deserialize;

use crate::{
    Session,
    media::{Album, Artist, ItemList, Track},
    request::ApiVersion,
    session::SessionError,
};

bitflags! {
    // #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SearchType: u8 {
        const Artist = 1 << 0;
        const Album = 1 << 1;
        const Track = 1 << 2;
        const Video = 1 << 3;
        const Playlist = 1 << 4;
        const Mix = 1 << 5;
    }
}
impl fmt::Display for SearchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        // Check in declared order
        if self.contains(SearchType::Artist) {
            parts.push("artists");
        }
        if self.contains(SearchType::Album) {
            parts.push("albums");
        }
        if self.contains(SearchType::Track) {
            parts.push("tracks");
        }
        if self.contains(SearchType::Video) {
            parts.push("videos");
        }
        if self.contains(SearchType::Playlist) {
            parts.push("playlists");
        }
        if self.contains(SearchType::Mix) {
            parts.push("mixs");
        }

        write!(f, "{}", parts.join(", "))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub artists: Option<ItemList<Artist>>,
    pub albums: Option<ItemList<Album>>,
    pub tracks: Option<ItemList<Track>>,
}

impl Session {
    pub async fn search(
        &mut self,
        query: &str,
        types: SearchType,
        limit: u32,
        offset: u32,
    ) -> Result<SearchResult, SessionError> {
        let params = &[
            ("query", query),
            ("limit", &format!("{limit}")),
            ("offset", &format!("{offset}")),
            ("types", &format!("{types}")),
        ];

        let resp = self.request("search", params, ApiVersion::V1).await?;

        let result: SearchResult = resp.json().await?;
        Ok(result)
    }
}
