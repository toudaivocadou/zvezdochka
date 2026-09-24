use chrono::NaiveDate;
use maud::{Markup, Render, html};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::site::{
    album::Illustration, metadata::RenderableMetadata, templates::partials::navbar::Sections,
};

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct NewsMeta {
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub additional_authors: Vec<String>,
    #[serde(default)]
    pub thumbnail: Option<Illustration>,
    pub date: NaiveDate,

    #[serde(default)]
    pub short: Option<String>,

    #[serde(default)]
    pub sns_links: Vec<Url>,
}

impl RenderableMetadata for NewsMeta {
    fn render_image_meta(&self) -> Option<Markup> {
        self.thumbnail.as_ref().map(|th| {
            html! {
                meta property="og:image" content=(&th.image);
            }
        })
    }

    fn section(&self) -> Sections {
        Sections::NewsPost
    }

    fn title(&self) -> &str {
        &self.title
    }
}

impl Render for NewsMeta {
    fn render(&self) -> maud::Markup {
        let og_type = Sections::NewsPost.opengraph_type();

        html! {
            meta property="og:title" content=(&self.title);
            meta property="og:site_name" content="東京大学ボカロP同好会 - University of Tokyo Vocaloid Producer Club";
            meta property="og:locale" content="ja_JP";
            meta property="og:type" content=(og_type);
            @if let Some(desc) = &self.short {
                meta property="og:description" content=(desc);
            }
            @for author in &self.authors {
                meta property="og:article:author" content=(author);
            }
            @for a_author in &self.additional_authors {
                meta property="og:article:author" content=(a_author);
            }
        }
    }
}
