use crate::site::{
    metadata::RenderableMetadata, namemap::MemberRef, templates::partials::navbar::Sections,
    util::format_date,
};
use fancy_duration::FancyDuration;
use indexmap::IndexMap;
use maud::{Markup, Render, html};
use serde::{Deserialize, Serialize};
use std::{hash::Hash, time::Duration};
use time::Date;
use url::Url;

// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
// pub enum AlbumType {
//     ToudaiVocadou,
//     Solo,
//     Collaboration,
// }

pub type Title = String;

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Illustration {
    pub image: String,
    #[serde(default)]
    pub illustrators: Vec<MemberRef>,
    #[serde(default)]
    pub additional_illustrators: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    pub illustrations: IndexMap<Title, Illustration>,

    #[serde(default)]
    pub tracks: IndexMap<Title, Track>,
}

impl Hash for AlbumMeta {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.authors.hash(state);
        self.additional_authors.hash(state);
        self.date.hash(state);
        self.link.hash(state);
        self.demonstration.hash(state);
        self.sns_links.hash(state);
        self.short.hash(state);
        self.thumbnail.hash(state);
        for (title, illust) in &self.illustrations {
            title.hash(state);
            illust.hash(state);
        }
        for (title, track) in &self.tracks {
            title.hash(state);
            track.hash(state);
        }
    }
}

impl RenderableMetadata for AlbumMeta {
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

impl Render for AlbumMeta {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Track {
    #[serde(default)]
    pub authors: Vec<MemberRef>,
    #[serde(default)]
    pub additional_authors: Vec<String>,
    pub duration: FancyDuration<Duration>,
    #[serde(default)]
    pub external: bool,
}

impl Hash for Track {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.authors.hash(state);
        self.additional_authors.hash(state);
        self.duration.format().hash(state);
        self.external.hash(state);
    }
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.authors == other.authors
            && self.additional_authors == other.additional_authors
            && self.duration == other.duration
            && self.external == other.external
    }
}

impl PartialOrd for Track {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.authors.partial_cmp(&other.authors) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self
            .additional_authors
            .partial_cmp(&other.additional_authors)
        {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.external.partial_cmp(&other.external)
    }
}
