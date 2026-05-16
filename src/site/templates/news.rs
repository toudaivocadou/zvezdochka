use crate::site::sitemap::SiteMap;
use crate::site::templates::functions::sns::sns_icon;
use crate::site::util::image_or_gray;
use crate::site::{news::NewsMeta, util::reference};
use eyre::Report;
use maud::{Markup, PreEscaped, html};

pub const NEWS_MISSING_AUTHOR: &'static str = "東大ボカロP同好会";

pub fn news_posts(site_map: &SiteMap) -> Result<Markup, Report> {
    // TODO: pagination. this will get long! yell at peng if we get >100!

    Ok(html! {
        section #hero {
            .container {
                h2 { "ニュース" }
                p { "東京大学ボカロP同好会のニュース目録" }
            }
        }

        section #list {
            .listcontainer .flex-container style="align-items: center;"{
                @for news_posts in &site_map.news {
                    (news_card(site_map, news_posts)?)
                }
                @if site_map.news.is_empty() {
                    p .work-description style="text-align: center;" {
                        i {
                            "ニュースがありません。"
                        }
                    }
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

pub fn news_card(site_map: &SiteMap, news_meta: &NewsMeta) -> Result<Markup, Report> {
    Ok(html! {
        .post-card {
            .member-profile-image .post-card-image {
                img .post-img src=(image_or_gray(news_meta.thumbnail.as_ref())) {}
            }
            .post-info {
                h3 .post-card-title {
                    a href=(format!("/news/{}.html", reference(&news_meta.title, &[news_meta.author.as_ref().map(|x| x.as_str()).unwrap_or(NEWS_MISSING_AUTHOR)], &[]))) {
                        (news_meta.title)
                    }
                }
                p .member-role {
                    (news_meta.date)
                }
                @if let Some(ascii_author) = &news_meta.author {
                    a href=(format!("/members/{}.html", ascii_author)) { p { (site_map.members.get(ascii_author).unwrap()) } }
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
    site_map: &SiteMap,
    news_meta: &NewsMeta,
    content: &str,
) -> Result<Markup, Report> {
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
                            a href=(format!("/members/{}.html", ascii_author)) { p { (site_map.members.get(ascii_author).unwrap()) } }
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
                a href="../news.html" {
                    "ニュース目録一覧に戻る"
                }
            }
        }
    })

    // let metadata = Metadata {
    //     page_title: post_meta.title.clone(),
    //     page_image: Some(news_thumbnail(sack, post_meta)?),
    //     canonical_link: format!("/news/{}.html", news_reference(post_meta)),
    //     section: Sections::NewsPost,
    //     description: Some(shorten(content)),
    //     author: post_meta.author.clone(),
    //     date: Some(post_meta.date.to_string()),
    // };

    // base(sack, &metadata, Some(&[]), inner)
}
