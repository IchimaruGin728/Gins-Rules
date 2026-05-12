use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UpstreamSource {
    pub name: String,
    pub url: String,
    pub category: String,
    pub target: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct IconSource {
    pub name: String,
    pub url: String,
    pub theme: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NormalizedIcon {
    pub name: String,
    pub url: String,
    pub source: String,
    pub theme: String,
}

#[derive(Debug, Deserialize)]
pub struct RawIcon {
    pub name: Option<String>,
    pub tag: Option<String>,
    pub url: Option<String>,
}
