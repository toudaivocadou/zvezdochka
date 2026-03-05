use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use toml::value::Date;

use crate::{metadata::Metadata, templates::partials::navbar::Sections};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlbumMeta {
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    pub release_date: Date,
    pub short: String,
    pub album_type: AlbumType,
    #[serde(default)]
    pub contributors: Vec<String>,
    #[serde(default)]
    pub extra_contributors: Vec<String>,

    #[serde(default)]
    pub crossfade_demonstration: Option<String>,

    pub front_cover: String,
    pub front_cover_illustrator: String,
    #[serde(default)]
    pub front_cover_illustrator_not_on_site: bool,
    #[serde(default)]
    pub other_covers: HashMap<String, Illustration>,

    #[serde(default)]
    pub playlist_link: Option<String>,

    #[serde(default)]
    pub tracklist: Vec<TracklistTrack>,

    #[serde(default)]
    pub sns_links: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Illustration {
    pub link: String,
    pub illustrator: String,
    #[serde(default)]
    pub illustrator_is_not_on_site: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TracklistTrack {
    pub author: String,
    pub title: String,
    #[serde(default)]
    pub duration_seconds: Option<i32>,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub on_site: bool,
    #[serde(default)]
    pub external_author: bool,
}

impl AlbumMeta {
    pub fn contributors_str(&self, name_map: &HashMap<String, String>) -> String {
        let mut all_contributors = HashSet::new();
        all_contributors.extend(
            self.contributors
                .iter()
                .map(|name| match name_map.get(name) {
                    Some(n) => n,
                    None => panic!("{name}: not found"),
                }),
        );
        all_contributors.extend(&self.extra_contributors);

        all_contributors
            .into_iter()
            .map(String::as_str)
            .collect::<Vec<&str>>()
            .join(", ")
    }

    pub fn contributors_str_naive(&self) -> String {
        let mut all_contributors = HashSet::new();
        all_contributors.extend(&self.contributors);
        all_contributors.extend(&self.extra_contributors);

        all_contributors
            .into_iter()
            .map(String::as_str)
            .collect::<Vec<&str>>()
            .join(", ")
    }
}

impl From<AlbumMeta> for Metadata {
    fn from(value: AlbumMeta) -> Self {
        let authors = value.contributors_str_naive();
        Metadata {
            canonical_link: format!("/works/albums/{}", &value.title),
            page_title: value.title,
            page_image: Some(value.front_cover),
            section: Sections::Works,
            description: Some(value.short),
            author: Some(authors),
            date: Some(value.release_date.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum AlbumType {
    Solo,
    GroupExternal,
    ToudaiVocadou,
}
