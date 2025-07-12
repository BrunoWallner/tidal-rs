mod album;
pub mod artist;
pub mod track;

pub use album::*;
pub use artist::*;
pub use track::*;

pub type Id = u64;

use serde::{
    Deserialize, Deserializer,
    de::{self, Error, Unexpected, Visitor},
};
use std::fmt;

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl Color {
    fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#')?;
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self { r, g, b })
            }
            _ => None,
        }
    }
}
impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ColorVisitor;

        impl<'de> Visitor<'de> for ColorVisitor {
            type Value = Color;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a hex color string like \"#ffffff\"")
            }

            fn visit_str<E>(self, v: &str) -> Result<Color, E>
            where
                E: de::Error,
            {
                Color::from_hex(v).ok_or_else(|| E::custom(format!("invalid hex color: {v}")))
            }
        }

        deserializer.deserialize_str(ColorVisitor)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AudioQuality {
    Low96K,
    Low320K,
    HighLossless,
    HiResLossless,
}
impl<'a> Into<&'a str> for AudioQuality {
    fn into(self) -> &'a str {
        match self {
            AudioQuality::Low96K => "LOW",
            AudioQuality::Low320K => "HIGH",
            AudioQuality::HighLossless => "LOSSLESS",
            AudioQuality::HiResLossless => "HI_RES_LOSSLESS",
        }
    }
}
// Manual implementation of Deserialize
impl<'de> Deserialize<'de> for AudioQuality {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        match s {
            "LOW" => Ok(AudioQuality::Low96K),
            "HIGH" => Ok(AudioQuality::Low320K),
            "LOSSLESS" => Ok(AudioQuality::HighLossless),
            "HI_RES_LOSSLESS" => Ok(AudioQuality::HiResLossless),
            _ => Err(D::Error::invalid_value(
                Unexpected::Str(s),
                &"a valid audio quality",
            )),
        }
    }
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VideoQuality {
    Low,
    Medium,
    High,
    AudioOnly,
}
impl<'a> Into<&'a str> for VideoQuality {
    fn into(self) -> &'a str {
        match self {
            VideoQuality::Low => "LOW",
            VideoQuality::Medium => "MEDIUM",
            VideoQuality::High => "HIGH",
            VideoQuality::AudioOnly => "AUDIO_ONLY",
        }
    }
}
