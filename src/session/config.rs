use std::ops::Deref;

use crate::media::{AudioQuality, VideoQuality};

#[derive(Clone, Debug)]
pub struct ItemLimit(u32);
impl TryFrom<u32> for ItemLimit {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            x if x < 10_000 => Ok(ItemLimit(value)),
            _ => Err(()),
        }
    }
}
impl Deref for ItemLimit {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub audio_quality: AudioQuality,
    pub video_quality: VideoQuality,
    pub item_limit: ItemLimit,
    pub alac: bool,
}

impl Config {
    pub fn new(
        audio_quality: AudioQuality,
        video_quality: VideoQuality,
        item_limit: ItemLimit,
        alac: bool,
    ) -> Self {
        // let client_unique_key = format!("{:016x}", rand::rng().random::<u64>());

        // let code_verifier = {
        //     let random_bytes: [u8; 32] = rand::rng().random();
        //     BASE64_URL_SAFE_NO_PAD.encode(random_bytes)
        // };

        // let code_challenge = {
        //     let hash = Sha256::digest(code_verifier.as_bytes());
        //     BASE64_URL_SAFE_NO_PAD.encode(hash)
        // };

        // // PKCE client_id and secret (decoded twice from base64)
        // let client_id_pkce = "6BDSRdpK9hqEBTgU".to_string();
        // let client_secret_pkce = "xeuPmY7nbpZ9IIbLAcQ93shka1VNheUAqN6IcszjTG8=".to_string();

        // let api_token = "zU4XHVVkc2tDPo4t".to_string();
        // let client_id = "zU4XHVVkc2tDPo4t".to_string();
        // let client_secret = "VJKhDFqJPqvsPVNBV6ukXTJmwlvbttP7wlMlrc72se4=".to_string();

        // let quality: &str = quality.into();
        // let video_quality: &str = video_quality.into();

        Config {
            // // api_oauth2_token: "https://auth.tidal.com/v1/oauth2/token".to_string(),
            // api_pkce_auth: "https://login.tidal.com/authorize".to_string(),
            // api_v1_location: "https://api.tidal.com/v1/".to_string(),
            // api_v2_location: "https://api.tidal.com/v2/".to_string(),
            // openapi_v2_location: "https://openapi.tidal.com/v2/".to_string(),
            // image_url: "https://resources.tidal.com/images/%s/%ix%i.jpg".to_string(),
            // video_url: "https://resources.tidal.com/videos/%s/%ix%i.mp4".to_string(),
            // listen_base_url: "https://listen.tidal.com".to_string(),
            // share_base_url: "https://tidal.com/browse".to_string(),
            // pkce_uri_redirect: "https://tidal.com/android/login/auth".to_string(),

            // api_token,
            // client_id,
            // client_secret,
            audio_quality,
            video_quality: video_quality,
            item_limit,
            alac,
            // client_unique_key,
            // code_verifier,
            // code_challenge,
            // client_id_pkce,
            // client_secret_pkce,
        }
    }
}
