use serde::Deserialize;

use crate::{
    ApiVersion, Session, SessionError,
    media::{ItemList, Track},
};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteItem<T> {
    pub created: chrono::DateTime<chrono::Utc>,
    pub item: T,
}

impl Session {
    pub async fn get_favorite_tracks(
        &mut self,
        limit: u32,
        offset: u32,
    ) -> Result<ItemList<FavoriteItem<Track>>, SessionError> {
        let Some(info) = &self.info else {
            Err(SessionError::NoSession)?
        };
        let path = format!("users/{}/favorites/tracks", info.user_id);
        let resp = self
            .request(
                &path,
                &[
                    ("limit", &limit.to_string()),
                    ("offset", &offset.to_string()),
                ],
                ApiVersion::V1,
            )
            .await?;
        let tracks: ItemList<FavoriteItem<Track>> = resp.json().await?;
        Ok(tracks)
    }
}
