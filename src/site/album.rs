use std::fmt::Display;

use crate::site::{
    metadata::RenderableMetadata, sitemap::MemberRef, templates::partials::navbar::Sections,
    util::format_date,
};
use maud::{Markup, Render, html};
use serde::{Deserialize, Serialize};
use time::Date;
use url::Url;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlbumType {
    ToudaiVocadou,
    Solo,
    Collaboration,
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Illustration {
    pub title: String,
    pub image: String,
    #[serde(default)]
    pub illustrators: Vec<MemberRef>,
    #[serde(default)]
    pub additional_illustrators: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AlbumMeta {
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub additional_authors: Vec<String>,
    pub date: Date,

    pub link: Option<Url>,
    #[serde(default)]
    pub demonstration: Option<Url>,
    #[serde(default)]
    pub sns_links: Vec<Url>,

    #[serde(default)]
    pub short: Option<String>,

    pub thumbnail: Illustration,

    #[serde(default)]
    pub illustrations: Vec<Illustration>,

    #[serde(default)]
    pub tracks: Vec<Track>,
}

impl RenderableMetadata for &AlbumMeta {
    fn render_image_meta(&self) -> Option<Markup> {
        Some(html! {
            meta property="og:image" content=(self.thumbnail.image);
        })
    }

    fn section(&self) -> Sections {
        Sections::AlbumPost
    }

    fn title(&self) -> &str {
        &self.title
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

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SongLength {
    pub minutes: u8,
    pub seconds: u8,
}

impl Display for SongLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.minutes, self.seconds)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Track {
    pub title: String,
    #[serde(default)]
    pub authors: Vec<MemberRef>,
    #[serde(default)]
    pub additional_authors: Vec<String>,
    #[serde(default)]
    pub duration: Option<SongLength>,
    #[serde(default)]
    pub external: bool,
}
