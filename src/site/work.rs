use serde::{Deserialize, Serialize};
use toml::value::Date;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CoverOrImage {
    Cover(String),
    Link(Url),
    AudioFile(String),
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct WorkMeta {
    pub title: String,
    pub author: String,
    #[serde(default)]
    pub collaborators: Vec<String>,
    pub date: Date,
    #[serde(default)]
    pub short: Option<String>,
    pub display: CoverOrImage,
    #[serde(default)]
    pub cover_image: Option<String>,
    #[serde(default)]
    pub link: Option<Url>,
    #[serde(default)]
    pub file: Option<String>,
    pub remix_original_work: Option<String>, // The link to the original work if it is a remix.
    #[serde(default)]
    pub featured: bool,

    #[serde(default)]
    pub streaming: Vec<String>,

    #[serde(default)]
    pub duration_seconds: Option<i32>,
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct RawWorkMeta {
    pub title: String,
    pub author: String,
    #[serde(default)]
    pub collaborators: Vec<String>,
    pub date: Date,
    #[serde(default)]
    pub short: Option<String>,
    #[serde(default)]
    pub cover_image: Option<String>,
    #[serde(default)]
    pub link: Option<Url>,
    #[serde(default)]
    pub file: Option<String>,
    pub remix_original_work: Option<String>, // The link to the original work if it is a remix.
    #[serde(default)]
    pub featured: bool,

    #[serde(default)]
    pub streaming: Vec<String>,

    #[serde(default)]
    pub duration_seconds: Option<i32>,
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct DisplayWorkMeta {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub on_site_link: String,
    pub author_displayname: String,
    pub author_link: String,
    pub embed_html: String,
}
