use chrono::NaiveDate;
use fancy_duration::FancyDuration;
use maud::{Render, html};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash, time::Duration};
use url::Url;

use crate::site::{
    album::Illustration, metadata::RenderableMetadata, namemap::MemberRef,
    templates::partials::navbar::Sections, util::get_link_image_thumb,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkMeta {
    pub title: String,
    #[serde(default)]
    pub authors: Vec<MemberRef>,
    #[serde(default)]
    pub additional_authors: Vec<String>,
    pub date: NaiveDate,
    pub duration: FancyDuration<Duration>,

    #[serde(default)]
    pub short: Option<String>,

    #[serde(default)]
    pub thumbnail: Option<Illustration>,

    #[serde(default)]
    pub link: Option<Url>,
    #[serde(default)]
    pub sns_links: Vec<Url>,
}

impl WorkMeta {
    pub fn thumbnail_or_none(&self) -> String {
        if let Some(thumb) = &self.thumbnail {
            return thumb.image.clone();
        }
        if let Some(source) = &self.link {
            if let Ok(link) = get_link_image_thumb(source) {
                return link;
            }
        }

        return "images/gray.jpg".to_string();
    }
}

impl Hash for WorkMeta {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.authors.hash(state);
        self.additional_authors.hash(state);
        self.date.hash(state);
        self.duration.format().hash(state);
        self.short.hash(state);
        self.thumbnail.hash(state);
        self.link.hash(state);
        self.sns_links.hash(state);
    }
}

impl RenderableMetadata for WorkMeta {
    fn render_image_meta(&self) -> Option<maud::Markup> {
        Some(html! {
            meta property="og:image" content="thumbnail.jpg";
        })
    }

    fn section(&self) -> Sections {
        Sections::WorksPost
    }

    fn title(&self) -> &str {
        &self.title
    }
}

impl Render for WorkMeta {
    fn render(&self) -> maud::Markup {
        let og_type = Sections::WorksPost.opengraph_type();

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

            @for a_author in &self.additional_authors {
                meta property="og:music:musician" content=(a_author);
            }
            meta property="og:music:release_date" content=(self.date.to_string());
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkListWork {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub on_site_link: String,
    pub authors: HashMap<String, String>,
    pub embed_html: String,
}
