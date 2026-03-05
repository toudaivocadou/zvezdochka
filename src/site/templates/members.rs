use std::collections::HashMap;

use crate::album::AlbumMeta;
use crate::member::MemberMeta;
use crate::metadata::Metadata;
use crate::news::NewsMeta;
use crate::site::news::SiteData;
use crate::sitemap::SiteMap;
use crate::templates::base::base;
use crate::templates::functions::sns::sns_icon;
use crate::templates::news::{post_reference, post_thumbnail};
use crate::templates::partials::navbar::Sections;
use crate::templates::works::{album_reference, thumbnail_link, work_reference};
use crate::util::image;
use crate::work::WorkMeta;
use hauchiwa::Context;
use hauchiwa::RuntimeError;
use maud::{Markup, PreEscaped, html};

pub fn members(sack: &Context<SiteData>, site_map: &SiteMap) -> Result<Markup, RuntimeError> {
    let inner = html! {
        section #members-hero {
            .container {
                h2 { "メンバー紹介" }
                p { "東京大学ボカロP同好会で活動する個性豊かなメンバーたちをご紹介します。" }
            }
        }

        section #staff-members {
            .zcontainer {
                .member-grid {
                    @for member in &site_map.members {
                        (member_card(sack, member)?)
                    }
                }
            }
        }
    };

    let metadata = Metadata {
        page_title: "メンバー紹介 - 東京大学ボカロP同好会".to_string(),
        page_image: None,
        canonical_link: "/members.html".to_string(),
        section: Sections::Members,
        description: Some("東京大学ボカロP同好会のメンバー紹介".to_string()),
        author: None,
        date: None,
    };

    base(sack, &metadata, None, inner)
}

pub fn member_card(sack: &Context<SiteData>, member: &MemberMeta) -> Result<Markup, RuntimeError> {
    let member_links_len = member.links.len();
    Ok(html! {
        .member-item {
            a .member-link href=(format!("/members/{}.html", member.ascii_name)) {
                .member-card {
                    .member-image .img-placeholder {
                        img .member-image .img-placeholder src=(image(sack, format!("images/icon/{}.jpg", member.ascii_name))?) alt=(member.name);
                    }
                    .member-info #(member.ascii_name) {
                        h3 { (member.name) }
                        @if let Some(role) = &member.position {
                            p .member-role { (role) }
                        }
                        @if let Some(department) = &member.department {
                            p .member-department { (department) }
                        }
                        p .member-description { (&member.short) }
                        .member-links {
                            // dummy div to fill out the size in case the user has no icons
                            @if member_links_len == 0 {
                                .social-icon-size style="visibility: hidden" {}
                            }
                            @for link in &member.links {
                                (sns_icon(sack, link)?)
                            }
                        }
                    }
                }
            }
        }
    })
}

// TODO: add "worked on albums" and "posts".
pub fn member_detail(
    sack: &Context<SiteData>,
    member: &MemberMeta,
    site_map: &SiteMap,
    namemap: &HashMap<String, String>,
    content: &str,
) -> Result<Markup, RuntimeError> {
    let this_featured_work = site_map
        .works
        .iter()
        .filter(|featured| featured.author == member.ascii_name && featured.featured)
        .take(5)
        .collect::<Vec<&WorkMeta>>();

    let featured_posts = site_map
        .news
        .iter()
        .filter(|post| post.author.as_ref() == Some(&member.ascii_name))
        .take(5)
        .collect::<Vec<&NewsMeta>>();

    let featured_albums = site_map
        .albums
        .iter()
        .filter(|album| album.contributors.contains(&member.ascii_name))
        .take(5)
        .collect::<Vec<&AlbumMeta>>();

    let inner = html! {
        section #member-detail {
            .member-detail-container {
                .member-profile {
                    .member-profile-image {
                        img .img-placeholder src=(image(sack, format!("images/icon/{}.jpg", member.ascii_name))?) alt=(member.name);
                    }
                    .member-profile-info {
                        h2 { (member.name) }
                        @if let Some(role) = &member.position {
                            p .member-role { (role) }
                        }
                        .member-bio {
                            (PreEscaped(content))
                        }
                        .member-links {
                            @for link in &member.links {
                                (sns_icon(sack, link)?)
                            }
                        }
                    }
                }
            }

            .member-works-container {
                .member-featured-works {
                    h3 { "代表作品" }
                    .container {
                        @for featured in &this_featured_work {
                            (featured_work_item_detail(sack, featured)?)
                        }
                        @if this_featured_work.is_empty() {
                            p .work-description style="text-align: center;" {
                                em {
                                    "代表作品がありません。"
                                }
                            }
                        }
                    }
                }

                .member-featured-works {
                    h3 { "最近のポスト" }
                    .container {
                        @for featured in featured_posts.iter() {
                            (featured_post_detail(sack, featured)?)
                        }
                        @if featured_posts.is_empty() {
                            p .work-description style="text-align: center;" {
                                em {
                                    "ポストがありません。"
                                }
                            }
                        }
                    }
                }

                .member-featured-works {
                    h3 { "最近のアルバム" }
                    .container {
                        @for featured in featured_albums.iter() {
                            (featured_album_detail(sack, featured, namemap)?)
                        }
                        @if featured_albums.is_empty() {
                            p .work-description style="text-align: center;" {
                                em {
                                    "アルバムがありません。"
                                }
                            }
                        }
                    }
                }

                .back-button  {
                    a href="../members.html" class="margin-top: 50px;" {
                        "メンバー一覧に戻る"
                    }
                }
            }
        }
    };

    let metadata = MemberMeta::to_metadata(member.clone());

    base(sack, &metadata, None, inner)
}

pub fn featured_work_item_detail(
    sack: &Context<SiteData>,
    item: &WorkMeta,
) -> Result<Markup, RuntimeError> {
    let work_ref = work_reference(&item.title, &item.author);

    Ok(html! {
        .work-item-detail id=(work_ref) {
            h4 { (item.title) }
            .work-youtube-container {
                img .work-item-thumb src=(thumbnail_link(sack, item)?) alt=(item.title) {}
            }

            .work-description {
                p { (item.short.as_deref().unwrap_or_default()) }
            }

            .click-button{
                a href=(format!("/works/releases/{}.html", work_ref)) {
                    p { "詳しく見る" }
                }
            }
        }
    })
}

pub fn featured_post_detail(
    sack: &Context<SiteData>,
    item: &NewsMeta,
) -> Result<Markup, RuntimeError> {
    Ok(html! {
        .post-card style="width: 100%;" {
            .member-profile-image .post-card-image {
                img .post-img src=(post_thumbnail(sack, item)?) {}
            }
            .post-info {
                h3 .post-card-title style="text-align: start; margin-bottom: 0px;" {
                    a href=(format!("/news/{}.html", post_reference(item))) {
                        (item.title)
                    }
                }
                p .member-role {
                    (item.date)
                }
                p {
                    (item.short)
                }
                .member-links {
                    @for link in &item.sns_links {
                        (sns_icon(sack, link.as_str())?)
                    }
                }
            }
        }
    })
}

pub fn featured_album_detail(
    sack: &Context<SiteData>,
    album_meta: &AlbumMeta,
    namemap: &HashMap<String, String>,
) -> Result<Markup, RuntimeError> {
    Ok(html! {
        .post-card style="width: 100%;" {
            .member-profile-image .post-card-image {
                img .work-item-thumb src=(image(sack, format!("images/{}", &album_meta.front_cover))?) alt=(&album_meta.title) {}
            }
            .post-info {
                h3 .post-card-title style="text-align: start; margin-bottom: 0px;" {
                    a href=(format!("/works/albums/{}.html", album_reference(&album_meta.title, &album_meta.front_cover))) {
                        (album_meta.title)
                    }
                }
                p .work-role {
                    (album_meta.release_date)
                }
                p .member-role {
                    (album_meta.contributors_str(namemap))
                }
            }
        }
    })
}
