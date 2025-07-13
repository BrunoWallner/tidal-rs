use std::fmt::Display;

use reqwest::{self, header};
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::session::{
    Info,
    auth::{self, OAuth},
};

const USER_AGENT: &'static str = "Mozilla/5.0 (Linux; Android 12; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/91.0.4472.114 Safari/537.36";

pub(super) const API_OAUTH2_DEV_AUTH_URL: &str =
    "https://auth.tidal.com/v1/oauth2/device_authorization";
pub const API_OAUTH2_URL: &str = "https://auth.tidal.com/v1/oauth2/token";
pub const CLIENT_ID: &str = "zU4XHVVkc2tDPo4t";
pub const CLIENT_SECRET: &str = "VJKhDFqJPqvsPVNBV6ukXTJmwlvbttP7wlMlrc72se4=";
pub const API_URL: &str = "https://api.tidal.com";

#[derive(Copy, Clone, Debug)]
pub enum ApiVersion {
    V1,
    V2,
}
impl Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiVersion::V1 => write!(f, "v1"),
            ApiVersion::V2 => write!(f, "v2"),
        }
    }
}

#[derive(Error)]
pub enum RequestError {
    Backend(reqwest::Error),
    InvalidTokenType,
    JsonParse(Vec<u8>),
    Authentification,
}
impl From<reqwest::Error> for RequestError {
    fn from(err: reqwest::Error) -> Self {
        RequestError::Backend(err.without_url())
    }
}
impl Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestError::Backend(error) => write!(f, "backend error: {error}"),
            RequestError::InvalidTokenType => write!(f, "invalid token type"),
            RequestError::JsonParse(bytes) => write!(
                f,
                "failed to parse Json: {}",
                String::from_utf8_lossy(bytes)
            ),
            RequestError::Authentification => write!(f, "failed at authentification"),
        }
    }
}
impl std::fmt::Debug for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self) // reuse Display logic
    }
}

pub struct Response(reqwest::Response);
impl Response {
    pub async fn json<T: DeserializeOwned>(self) -> Result<T, RequestError> {
        let bytes = self.0.bytes().await?;
        let deserialized =
            serde_json::from_slice(&bytes).map_err(|_| RequestError::JsonParse(bytes.into()))?;
        Ok(deserialized)
    }
    pub fn is_ok(&self) -> bool {
        self.0.error_for_status_ref().is_ok()
    }
    pub fn status(&self) -> u16 {
        self.0.status().as_u16()
    }
    pub async fn text(self) -> Result<String, RequestError> {
        Ok(self.0.text().await?)
    }
}

pub struct Request {
    client: reqwest::Client,
}
impl Request {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    // internal use only, can be simple -> no generics :)
    pub async fn post(&self, url: &str, form: &[(&str, &str)]) -> Result<Response, RequestError> {
        let resp = self.client.post(url).form(form).send().await?;
        Ok(Response(resp))
    }

    pub async fn tidal_request(
        &self,
        path: &str,
        query: &[(&str, &str)],
        oauth: &mut OAuth,
        info: Option<&Info>,
        api_version: ApiVersion,
    ) -> Result<Response, RequestError> {
        // helper function
        async fn send(
            client: &Request,
            path: &str,
            query: &[(&str, &str)],
            oauth: &mut OAuth,
            info_query: &[(&str, &str)],
            api_version: ApiVersion,
        ) -> Result<reqwest::Response, RequestError> {
            let resp = client
                .client
                .get(&format!("{API_URL}/{api_version}/{path}"))
                .query(query)
                .query(info_query)
                .header(header::USER_AGENT, USER_AGENT)
                .header(
                    header::AUTHORIZATION,
                    format!("{} {}", oauth.token_type, oauth.access_token),
                )
                .send()
                .await?;
            Ok(resp)
        }

        let info_query: &[(&str, &str)] = if let Some(info) = info {
            &[
                ("sessionId", info.session_id.as_str()),
                ("countryCode", info.country_code.as_str()),
                ("limit", "1000"),
            ]
        } else {
            &[]
        };
        let mut response = send(self, path, query, oauth, info_query, api_version).await?;

        // refresh token and retry if token is expired
        // @todo: check if the expired token really is the cause and not something else
        if response.status().is_client_error() {
            auth::refresh_oauth_token(self, oauth)
                .await
                .map_err(|_| RequestError::Authentification)?;

            response = send(self, path, query, oauth, info_query, api_version).await?;
        }

        Ok(Response(response))
    }
}
