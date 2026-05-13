use maud::{Render, html};
use serde::{Deserialize, Serialize};
use time::Date;
use url::Url;

use crate::site::{
    album::Illustration, metadata::RenderableMetadata, sitemap::MemberRef,
    templates::partials::navbar::Sections, util::format_date,
};

#[derive(Clone, Debug, Hash, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct WorkMeta {
    pub title: String,
    #[serde(default)]
    pub authors: Vec<MemberRef>,
    #[serde(default)]
    pub additional_authors: Vec<String>,
    pub date: Date,

    #[serde(default)]
    pub short: Option<String>,

    #[serde(default)]
    pub thumbnail: Option<Illustration>,

    #[serde(default)]
    pub source: Option<Url>,
    #[serde(default)]
    pub sns_links: Vec<Url>,
}

impl RenderableMetadata for &WorkMeta {
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

impl Render for &WorkMeta {
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
            meta property="og:music:release_date" content=(format_date(self.date));
        }
    }
}
