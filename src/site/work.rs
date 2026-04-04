use hauchiwa::{
    Tracker,
    loader::{Image, image::ImageFormat},
};
use maud::{Render, html};
use serde::{Deserialize, Serialize};
use time::Date;
use url::Url;

use crate::site::{
    metadata::RenderImageMetadata,
    templates::partials::navbar::Sections,
    util::{format_date, make_path_relative},
};

#[derive(Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct WorkMeta {
    pub title: String,
    pub authors: Vec<String>,
    pub date: Date,

    #[serde(default)]
    pub short: Option<String>,

    #[serde(default)]
    pub thumbnail: Option<String>,

    pub source: Url,
    #[serde(default)]
    pub sns_links: Vec<Url>,
}

impl RenderImageMetadata for &WorkMeta {
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
