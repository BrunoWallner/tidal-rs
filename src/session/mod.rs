// use std::error::Error;
use thiserror::Error;

use crate::{
    request::{ApiVersion, Request, RequestError, Response},
    session::auth::{AuthError, OAuth, OAuthUrl, TokenType},
};

pub mod auth;
mod config;
mod search;

pub use config::*;
pub use search::*;
use serde::Deserialize;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("HTTP request error: {0}")]
    RequestError(#[from] RequestError),
    #[error("login error: {0}")]
    AuthError(#[from] AuthError),
    #[error("not logged in via OAuth")]
    NotLoggedInOauth,
    #[error("no session fetched")]
    NoSession,
    #[error("wrong token type")]
    WrongTokenType,
    #[error("failed to decode manifest")]
    ManifestDecode,
}

#[allow(dead_code)]
pub struct Session {
    pub config: config::Config,
    pub(crate) client: Request,

    pub info: Option<Info>,
    pub oauth: Option<OAuth>,
}

impl Session {
    pub fn new(config: config::Config) -> Self {
        let client = reqwest::Client::new();
        let client = Request::new(client);

        Session {
            config,
            client,
            info: None,
            oauth: None,
        }
    }

    /// Handle all OAuth Login, just pass in a Function that accepts and deals with
    /// with the login url
    /// Alternatively you have to deal with 'auth::request_oauth_login()' and
    /// 'auth::process_oauth_login()' manually
    pub async fn oauth_login_simple<F: FnOnce(&OAuthUrl)>(
        &mut self,
        callback: F,
    ) -> Result<(), SessionError> {
        let oauth_url = auth::request_oauth_url(&self.client).await?;
        callback(&oauth_url);
        let oauth = auth::process_oauth_url(&self.client, oauth_url).await?;
        self.oauth = Some(oauth);
        Ok(())
    }

    pub fn set_oauth(&mut self, oauth: OAuth) {
        self.oauth = Some(oauth);
    }

    pub async fn load_session_info(&mut self) -> Result<(), SessionError> {
        let Some(oauth) = &mut self.oauth else {
            Err(SessionError::NotLoggedInOauth)?
        };
        if oauth.token_type != TokenType::Bearer {
            Err(SessionError::WrongTokenType)?
        }
        let response = self
            .client
            .tidal_request("sessions", &[], oauth, None, ApiVersion::V1)
            .await?;
        let info: Info = response.json().await?;
        self.info = Some(info);
        Ok(())
    }

    pub async fn refresh_oauth_token(&mut self) -> Result<(), SessionError> {
        let Some(oauth) = &mut self.oauth else {
            Err(SessionError::NotLoggedInOauth)?
        };
        auth::refresh_oauth_token(&self.client, oauth).await?;
        Ok(())
    }

    pub async fn request(
        &mut self,
        path: &str,
        query: &[(&str, &str)],
        api_version: ApiVersion,
    ) -> Result<Response, SessionError> {
        let Some(oauth) = &mut self.oauth else {
            Err(SessionError::NotLoggedInOauth)?
        };
        let resp = self
            .client
            .tidal_request(&path, query, oauth, self.info.as_ref(), api_version)
            .await?;

        Ok(resp)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub session_id: String,
    pub user_id: u64,
    pub country_code: String,
    pub channel_id: u64,
    pub partner_id: u64,
    pub client: Client,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub id: u64,
    pub name: String,
    pub authorized_for_offline: bool,
    pub authorized_for_offline_data: Option<u64>,
}
