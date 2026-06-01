use crate::site::namemap::NameMap;
use crate::site::templates::functions::sns::sns_icon;
use crate::site::util::image_or_gray;
use crate::site::{news::NewsMeta, util::reference};
use anyhow::Error;
use hauchiwa::Tracker;
use hauchiwa::loader::Document;
use maud::{Markup, PreEscaped, html};

pub const NEWS_MISSING_AUTHOR: &'static str = "東大ボカロP同好会";

pub fn news_index(
    names: &NameMap,
    news_posts: &Tracker<'_, Document<NewsMeta>>,
) -> Result<Markup, Error> {
    Ok(html! {
        section #hero {
            .container {
                h2 { "ニュース" }
                p { "東京大学ボカロP同好会のニュース目録" }
            }
        }

        section #list {
            .listcontainer .flex-container style="align-items: center;"{
                @for (_, news_posts) in news_posts {
                    (news_card(names, &news_posts.matter)?)
                }
            }
        }
    })

    // let metadata = Metadata {
    //     page_title: "ニュース".to_string(),
    //     page_image: None,
    //     canonical_link: "/news.html".to_string(),
    //     section: Sections::News,
    //     description: Some("東京大学ボカロP同好会のニュース".to_string()),
    //     author: None,
    //     date: None,
    // };

    // base(sack, &metadata, Some(&[]), inner)
}

fn news_card(names: &NameMap, news_meta: &NewsMeta) -> Result<Markup, Error> {
    Ok(html! {
        .post-card {
            .member-profile-image .post-card-image {
                img .post-img src=(image_or_gray(news_meta.thumbnail.as_ref())) {}
            }
            .post-info {
                h3 .post-card-title {
                    a href=(format!("/news/{}/index.html", reference(&news_meta.title, &[news_meta.author.as_ref().map(|x| x.as_str()).unwrap_or(NEWS_MISSING_AUTHOR)], &[]))) {
                        (news_meta.title)
                    }
                }
                p .member-role {
                    (news_meta.date)
                }
                @if let Some(ascii_author) = &news_meta.author {
                    a href=(format!("/members/{}/index.html", ascii_author)) { p { (names.members.get(ascii_author).unwrap()) } }
                } @else {
                    p { "東大ボカロP同好会" }
                }
                p {
                    @if let Some(short) = news_meta.short.as_ref() {
                        (short)
                    }
                }

                .member-links {
                    @for link in &news_meta.sns_links {
                        (sns_icon(link)?)
                    }
                }
            }
        }
    })
}

pub fn news_detail(
    site_map: &NameMap,
    news_meta: &NewsMeta,
    content: String,
) -> Result<Markup, Error> {
    Ok(html! {
        section #post-detail {
            .member-detail-container {
                .member-profile {
                    .work-image {
                        img src=(image_or_gray(news_meta.thumbnail.as_ref())) alt="header image" { }
                    }
                    .member-profile-info {
                        h2 { (news_meta.title) }
                        p { (news_meta.date) }
                        @if let Some(ascii_author) = &news_meta.author {
                            a href=(format!("/members/{}/index.html", ascii_author)) { p { (site_map.members.get(ascii_author).unwrap()) } }
                        } @else {
                            p { "東大ボカロP同好会" }
                        }
                        .member-links {
                            @for link in &news_meta.sns_links {
                                (sns_icon(link)?)
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
                a href="../news/index.html" {
                    "ニュース目録一覧に戻る"
                }
            }
        }
    })
}
