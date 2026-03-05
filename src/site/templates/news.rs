use crate::news::NewsMeta;
use crate::sitemap::SiteMap;
use crate::templates::base::base;
use crate::templates::functions::sns::sns_icon;
use crate::templates::partials::navbar::Sections;
use crate::util::{image, shorten};
use crate::{SiteData, metadata::Metadata};
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use hauchiwa::Context;
use hauchiwa::RuntimeError;
use maud::{Markup, PreEscaped, html};
use std::collections::HashMap;

pub fn news_posts(
    sack: &Context<SiteData>,
    site_map: &SiteMap,
    name_map: &HashMap<String, String>,
) -> Result<Markup, RuntimeError> {
    // TODO: pagination. this will get long! yell at peng if we get >100!

    let inner = html! {
        section #hero {
            .container {
                h2 { "ニュース" }
                p { "東京大学ボカロP同好会のニュース目録です。" }
            }
        }

        section #list {
            .listcontainer .flex-container style="align-items: center;"{
                @for post_meta in &site_map.news {
                    (post_card(sack, post_meta, name_map)?)
                }
                @if site_map.news.is_empty() {
                    p .work-description style="text-align: center;" {
                        em {
                            "ニュースがありません。"
                        }
                    }
                }
            }
        }
    };

    let metadata = Metadata {
        page_title: "ニュース".to_string(),
        page_image: None,
        canonical_link: "/news.html".to_string(),
        section: Sections::News,
        description: Some("東京大学ボカロP同好会のニュース".to_string()),
        author: None,
        date: None,
    };

    base(sack, &metadata, Some(&[]), inner)
}

pub fn post_card(
    context: &Context<SiteData>,
    post_meta: &NewsMeta,
    name_map: &HashMap<String, String>,
) -> Result<Markup, RuntimeError> {
    let author_name = post_meta.author.as_ref().map(|author| name_map.get(author).ok_or(RuntimeError::msg("Could not find author. Does the member page exist? Did you remember to type in the ascii name? Did you mistype it? Yell at peg for more info".to_string())));

    Ok(html! {
        .post-card {
            .member-profile-image .post-card-image {
                img .post-img src=(post_thumbnail(context, post_meta)?) {}
            }
            .post-info {
                h3 .post-card-title {
                    a href=(format!("/news/{}.html", post_reference(post_meta))) {
                        (post_meta.title)
                    }
                }
                p .member-role {
                    (post_meta.date)
                }
                @if let Some(author) = author_name && let Some(ascii_author) = &post_meta.author {
                    a href=(format!("/members/{}.html", ascii_author)) { p { (author?) } }
                } @else {
                    p { "東大ボカロP同好会" }
                }
                p {
                    (post_meta.short)
                }
                .member-links {
                    @for link in &post_meta.sns_links {
                        (sns_icon(context, link.as_str())?)
                    }
                }
            }
        }
    })
}

pub fn post_detail(
    sack: &Context<SiteData>,
    post_meta: &NewsMeta,
    content: &str,
    name_map: &HashMap<String, String>,
) -> Result<Markup, RuntimeError> {
    let author_name = post_meta.author.as_ref().map(|author| name_map.get(author).ok_or(RuntimeError::msg("Could not find author. Does the member page exist? Did you remember to type in the ascii name? Did you mistype it? Yell at peg for more info".to_string())));

    let inner = html! {
        section #post-detail {
            .member-detail-container {
                .member-profile {
                    .work-image {
                        img src=(post_thumbnail(sack, post_meta)?) alt="header image" { }
                    }
                    .member-profile-info {
                        h2 { (post_meta.title) }
                        p { (post_meta.date) }
                        @if let Some(author) = author_name && let Some(ascii_author) = &post_meta.author {
                            a href=(format!("/members/{}.html", ascii_author)) { p { (author?) } }
                        } @else {
                            p { "東大ボカロP同好会" }
                        }
                        .member-links {
                            @for link in &post_meta.sns_links {
                                (sns_icon(sack, link.as_str())?)
                            }
                        }
                    }
                }
            }
        }

        .member-works-container {
            section #description .work-description {
                .description {
                    (PreEscaped(content))
                }
            }

            .back-button{
                a href="../news.html" {
                    "ニュース目録一覧に戻る"
                }
            }
        }
    };

    let metadata = Metadata {
        page_title: post_meta.title.clone(),
        page_image: Some(post_thumbnail(sack, post_meta)?),
        canonical_link: format!("/news/{}.html", post_reference(post_meta)),
        section: Sections::NewsPost,
        description: Some(shorten(content)),
        author: post_meta.author.clone(),
        date: Some(post_meta.date.to_string()),
    };

    base(sack, &metadata, Some(&[]), inner)
}

pub fn post_thumbnail(sack: &Context<SiteData>, item: &NewsMeta) -> Result<String, RuntimeError> {
    match &item.header_image {
        Some(header) => Ok(image(sack, format!("images/{}", header))?),
        None => Ok(image(sack, "images/gray.jpg")?),
    }

    // TODO: Get thumbnail from SNS post.
}

pub fn post_reference(meta: &NewsMeta) -> String {
    let authorhash = seahash::hash(
        meta.author
            .as_deref()
            .unwrap_or("東大ボカロP同好会")
            .as_bytes(),
    ) as u128;
    let titlehash = seahash::hash(meta.title.as_bytes()) as u128;
    let combined = (authorhash << 64) + titlehash;
    BASE64_URL_SAFE_NO_PAD.encode(combined.to_le_bytes())
}
