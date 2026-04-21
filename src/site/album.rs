use hauchiwa::{
    Tracker,
    loader::{Image, image::ImageFormat},
};
use maud::{Render, html};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use time::{Date, Duration};
use url::Url;

use crate::site::{
    metadata::RenderImageMetadata,
    templates::partials::navbar::Sections,
    util::{format_date, make_path_relative},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlbumType {
    ToudaiVocadou,
    Solo,
    Collaboration,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Illustration {
    image: String,
    illustrator: Option<String>,
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AlbumMeta {
    pub title: String,
    pub authors: Vec<String>,
    #[serde(default)]
    pub extra_authors: Vec<String>,
    pub date: Date,

    pub link: Url,
    #[serde(default)]
    pub demonstration: Option<Url>,
    #[serde(default)]
    pub sns_links: Vec<Url>,

    #[serde(default)]
    pub short: Option<String>,

    pub thumbnail: Option<String>,
    #[serde(default)]
    pub thumbnail_illustrator: Option<String>,

    #[serde(default)]
    pub illustrations: Vec<Illustration>,

    pub tracks: Vec<Track>,
}

impl RenderImageMetadata for &AlbumMeta {
    fn render_image_meta(&self, image: Tracker<'_, Image>) -> maud::Markup {
        if let Some(thumbnail) = &self.thumbnail {
            let path = make_path_relative("images", thumbnail);
            match image
                .get(path)
                .ok()
                .map(|img| img.get(ImageFormat::WebP))
                .flatten()
                .map(|path| path.as_str())
            {
                Some(img) => html! {
                    meta property="og:image" content=(img);
                },
                None => html! {},
            }
        } else {
            html! {}
        }
    }
}

impl Render for &AlbumMeta {
    fn render(&self) -> maud::Markup {
        let og_type = Sections::AlbumPost.opengraph_type();

        html! {
            meta property="og:title" content=(&self.title);
            meta property="og:site_name" content="東京大学ボカロP同好会 - University of Tokyo Vocaloid Producer Club";
            meta property="og:locale" content="ja_JP";
            meta property="og:type" content=(og_type);
            @if let Some(shrt) = &self.short {
                meta property="og:description" content=(&shrt);
            }
            @for author in &self.authors {
                meta property="og:music:musician" content=(author);
            }
            meta property="og:music:release_date" content=(format_date(self.date));
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Track {
    pub title: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub duration: Option<Duration>,
    #[serde(default)]
    pub external: bool,
}

// #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
// pub struct AlbumMeta {
//     pub title: String,
//     #[serde(default)]
//     pub subtitle: Option<String>,
//     pub release_date: Date,
//     pub short: String,
//     pub album_type: AlbumType,
//     #[serde(default)]
//     pub contributors: Vec<String>,
//     #[serde(default)]
//     pub extra_contributors: Vec<String>,

//     #[serde(default)]
//     pub crossfade_demonstration: Option<String>,

//     pub front_cover: String,
//     pub front_cover_illustrator: String,
//     #[serde(default)]
//     pub front_cover_illustrator_not_on_site: bool,
//     #[serde(default)]
//     pub other_covers: HashMap<String, Illustration>,

//     #[serde(default)]
//     pub playlist_link: Option<String>,

//     #[serde(default)]
//     pub tracklist: Vec<TracklistTrack>,

//     #[serde(default)]
//     pub sns_links: Vec<String>,
// }

// #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
// pub struct Illustration {
//     pub link: String,
//     pub illustrator: String,
//     #[serde(default)]
//     pub illustrator_is_not_on_site: bool,
// }

// #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
// pub struct TracklistTrack {
//     pub author: String,
//     pub title: String,
//     #[serde(default)]
//     pub duration_seconds: Option<i32>,
//     #[serde(default)]
//     pub link: Option<String>,
//     #[serde(default)]
//     pub on_site: bool,
//     #[serde(default)]
//     pub external_author: bool,
// }

// impl AlbumMeta {
//     pub fn contributors_str(&self, name_map: &HashMap<String, String>) -> String {
//         let mut all_contributors = HashSet::new();
//         all_contributors.extend(
//             self.contributors
//                 .iter()
//                 .map(|name| match name_map.get(name) {
//                     Some(n) => n,
//                     None => panic!("{name}: not found"),
//                 }),
//         );
//         all_contributors.extend(&self.extra_contributors);

//         all_contributors
//             .into_iter()
//             .map(String::as_str)
//             .collect::<Vec<&str>>()
//             .join(", ")
//     }

//     pub fn contributors_str_naive(&self) -> String {
//         let mut all_contributors = HashSet::new();
//         all_contributors.extend(&self.contributors);
//         all_contributors.extend(&self.extra_contributors);

//         all_contributors
//             .into_iter()
//             .map(String::as_str)
//             .collect::<Vec<&str>>()
//             .join(", ")
//     }
// }

// impl From<AlbumMeta> for Metadata {
//     fn from(value: AlbumMeta) -> Self {
//         let authors = value.contributors_str_naive();
//         Metadata {
//             canonical_link: format!("/works/albums/{}", &value.title),
//             page_title: value.title,
//             page_image: Some(value.front_cover),
//             section: Sections::Works,
//             description: Some(value.short),
//             author: Some(authors),
//             date: Some(value.release_date.to_string()),
//         }
//     }
// }

// #[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
// pub enum AlbumType {
//     Solo,
//     GroupExternal,
//     ToudaiVocadou,
// }
