use serde::{Deserialize, Serialize};
use toml::value::Date;
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewsMeta {
    pub title: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub header_image: Option<String>,
    pub date: Date,

    #[serde(default)]
    pub short: String,

    #[serde(default)]
    pub sns_links: Vec<Url>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawNewsMeta {
    pub title: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub header_image: Option<String>,
    pub date: Date,

    #[serde(default)]
    pub short: Option<String>,

    #[serde(default)]
    pub sns_links: Vec<Url>,
}
