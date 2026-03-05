use crate::SiteData;
use crate::templates::partials::navbar::Sections;
use crate::util::image;
use hauchiwa::{Context, RuntimeError};
use maud::{Markup, html};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub page_title: String,
    pub page_image: Option<String>,
    pub canonical_link: String,
    pub section: Sections,
    pub description: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
}

pub fn render_metadata(
    sack: &Context<SiteData>,
    metadata: &Metadata,
) -> Result<Markup, RuntimeError> {
    let page_type = match metadata.section {
        Sections::Home => "website",
        Sections::Members => "website",
        Sections::MemberProfile => "profile",
        Sections::Activities => "website",
        Sections::Join => "website",
        Sections::News => "website",
        Sections::NewsPost => "article",
        Sections::Works => "website",
        Sections::WorksPost => "article",
        Sections::AlbumPost => "albums",
    };

    let others = match page_type {
        "article" => {
            html! {
                meta property="og:article:author" content=[&metadata.author];
                meta property="og:article:published_time" content=[&metadata.date];
            }
        }
        "profile" => {
            html! { meta property="og:profile:username" content=[&metadata.author]; }
        }
        _ => html! {},
    };

    let canonical_link = &metadata.canonical_link;

    let image_lnk = metadata
        .page_image
        .as_ref()
        .map(|img| {
            if img.starts_with("https://") || img.ends_with(".webp") {
                Ok(img.clone())
            } else {
                image(sack, img)
            }
        })
        .map_or(Ok(None), |v| v.map(Some))?;

    Ok(html! {
        title { (&metadata.page_title) }
        meta property="og:title" content=(&metadata.page_title);
        meta property="og:url" content=(canonical_link);
        meta property="og:type" content=(page_type);
        meta property="og:site_name" content="東京大学ボカロP同好会 - University of Tokyo Vocaloid Producer Club"; // production -> producer - ありがとーnekojitalter
        meta property="og:locale" content="ja_JP";
        @if let Some(img) = &image_lnk {
            meta property="og:image" content=(img);
        }
        @if let Some(desc) = &metadata.description {
            meta property="og:description" content=(desc);
        }
        (others)
    })
}
