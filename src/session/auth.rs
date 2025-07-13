use std::{collections::HashMap, fmt::Display, mem, str::FromStr, time};
use thiserror::Error;

use chrono;
use serde::{Deserialize, Serialize};
// use tokio::time;
// use async_std::time;
// use async_std::
use smol::{Timer, stream::StreamExt};

use crate::request::{self, Request, RequestError};

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("token has expired")]
    RefreshTokenExpired,
    #[error("unable to authenticate")]
    Authentication,
    #[error("OAuth request token has expired")]
    TokenExpired,
    #[error("failed to parse token type")]
    TokenTypeParseError,
    #[error("request error: {0}")]
    RequestError(#[from] RequestError),
}

#[derive(Deserialize)]
pub struct OAuthResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    token_type: String,
}

#[derive(Deserialize)]
pub struct RefreshResponse {
    access_token: String,
    expires_in: i64,
    token_type: String,
}
impl RefreshResponse {
    pub fn into_oauth_response(self, refresh_token: String) -> OAuthResponse {
        OAuthResponse {
            access_token: self.access_token,
            refresh_token,
            expires_in: self.expires_in,
            token_type: self.token_type,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuth {
    pub access_token: String,
    pub refresh_token: String,
    // #[serde(with = "ts_seconds")]
    pub expiry: chrono::DateTime<chrono::Utc>,
    pub token_type: TokenType,
}
impl TryFrom<OAuthResponse> for OAuth {
    type Error = AuthError;

    fn try_from(value: OAuthResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            access_token: value.access_token,
            refresh_token: value.refresh_token,
            expiry: chrono::Utc::now() + chrono::Duration::seconds(value.expires_in),
            token_type: value.token_type.parse()?,
        })
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum TokenType {
    Bearer,
    // idk if there is any other :(
}
impl FromStr for TokenType {
    type Err = AuthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "bearer" => Ok(Self::Bearer),
            _ => Err(AuthError::TokenTypeParseError),
        }
    }
}
impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Bearer => write!(f, "Bearer"),
        }
    }
}

#[derive(Debug)]
pub struct OAuthUrl {
    pub user_code: String,
    pub device_code: String,
    pub verification_uri: String,
    pub interval: Timer,
    pub expiry: chrono::DateTime<chrono::Utc>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OAuthUrlResponse {
    expires_in: i64,
    user_code: String,
    verification_uri_complete: String,
    interval: u64,
    device_code: String,
}

pub async fn refresh_oauth_token(request: &Request, oauth: &mut OAuth) -> Result<(), AuthError> {
    let params = &[
        ("grant_type", "refresh_token"),
        ("refresh_token", oauth.refresh_token.as_str()),
        ("client_id", request::CLIENT_ID),
        ("client_secret", request::CLIENT_SECRET),
    ];
    let response = request
        .post(request::API_OAUTH2_URL, params)
        .await
        .map_err(|_| AuthError::RefreshTokenExpired)?;

    let refresh_resp: RefreshResponse = match response.status() {
        x if x >= 300 || x < 200 => Err(AuthError::Authentication)?,
        _ => response.json().await?,
    };
    let refresh_token = mem::take(&mut oauth.refresh_token);
    let new_oauth = OAuth::try_from(refresh_resp.into_oauth_response(refresh_token))?;
    *oauth = new_oauth;

    Ok(())
}

// impl super::Session {
pub async fn request_oauth_url(request: &Request) -> Result<OAuthUrl, AuthError> {
    let form = &[
        ("client_id", request::CLIENT_ID),
        ("scope", "r_usr w_usr w_sub"),
    ];

    let resp = request.post(request::API_OAUTH2_DEV_AUTH_URL, form).await?;

    let response: OAuthUrlResponse = resp.json().await?;
    let interval = Timer::interval(time::Duration::from_secs(response.interval));
    let expiry = chrono::Utc::now() + chrono::Duration::seconds(response.expires_in);

    Ok(OAuthUrl {
        user_code: response.user_code,
        device_code: response.device_code,
        verification_uri: response.verification_uri_complete,
        interval,
        expiry,
    })
}

pub async fn process_oauth_url(
    request: &Request,
    mut oauth_url: OAuthUrl,
) -> Result<OAuth, AuthError> {
    let url = request::API_OAUTH2_URL;
    let params = &[
        ("client_id", request::CLIENT_ID),
        ("client_secret", request::CLIENT_SECRET),
        ("device_code", oauth_url.device_code.as_str()),
        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ("scope", "r_usr w_usr w_sub"),
    ];

    while chrono::Utc::now() < oauth_url.expiry {
        oauth_url.interval.next().await;
        let response = request.post(url, params).await?;
        match response.is_ok() {
            true => {
                let oauth_resp: OAuthResponse = response.json().await?;
                let oauth =
                    OAuth::try_from(oauth_resp).map_err(|_| AuthError::TokenTypeParseError)?;
                return Ok(oauth);
            }
            false => {
                let res: HashMap<String, serde_json::Value> = response.json().await?;
                if let Some(err) = res.get("error")
                    && err == "expired_token"
                {
                    Err(AuthError::TokenExpired)?
                }
            }
        }
    }

    Err(AuthError::TokenExpired)?
}
// }
