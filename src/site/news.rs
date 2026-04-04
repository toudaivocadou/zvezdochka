use hauchiwa::{
    Tracker,
    loader::{Image, image::ImageFormat},
};
use maud::{Render, html};
use serde::{Deserialize, Serialize};
use time::Date;
use url::Url;

use crate::site::{
    metadata::RenderImageMetadata, templates::partials::navbar::Sections, util::make_path_relative,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewsMeta {
    pub title: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub thumbnail: Option<String>,
    pub date: Date,

    #[serde(default)]
    pub short: Option<String>,

    #[serde(default)]
    pub sns_links: Vec<Url>,
}

impl RenderImageMetadata for &NewsMeta {
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

impl Render for &NewsMeta {
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
            @if let Some(author) = &self.author {
                meta property="og:article:author" content=(author);
            }
        }
    }
}
