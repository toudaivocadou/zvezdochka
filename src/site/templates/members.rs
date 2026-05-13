use eyre::Report;
use maud::{Markup, PreEscaped, html};
use std::collections::HashMap;

use crate::site::{
    album::AlbumMeta,
    member::MemberMeta,
    news::NewsMeta,
    sitemap::SiteMap,
    templates::{
        functions::sns::sns_icon,
        news::news_reference,
        works::{album_reference, work_reference},
    },
    util::{hash, image_or_gray, reference},
    work::WorkMeta,
};

pub fn members(site_map: &SiteMap) -> Result<Markup, Report> {
    Ok(html! {
        section #members-hero {
            .container {
                h2 { "メンバー紹介" }
                p { "東京大学ボカロP同好会で活動する個性豊かなメンバーたちをご紹介します。" }
            }
        }

        section #staff-members {
            .zcontainer {
                .member-grid {
                    @for member in site_map.members.values() {
                        (member_card(member)?)
                    }
                }
            }
        }
    })
}

pub fn member_card(member: &MemberMeta) -> Result<Markup, Report> {
    let member_links_len = member.links.len();
    Ok(html! {
        .member-item {
            a .member-link href=(format!("/members/{}.html", member.ascii_name)) {
                .member-card {
                    .member-image .img-placeholder {
                        img .member-image .img-placeholder src=(format!("icon/{}.jpg", member.ascii_name)) alt=(member.name);
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
                                (sns_icon(link)?)
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
    member: &MemberMeta,
    site_map: &SiteMap,
    content: String,
) -> Result<Markup, Report> {
    let recent_works = site_map
        .works
        .iter()
        .filter(|featured| featured.authors.contains(&member.ascii_name))
        .take(5)
        .collect::<Vec<&WorkMeta>>();

    let recent_posts = site_map
        .news
        .iter()
        .filter(|post| post.author.as_ref() == Some(&member.ascii_name))
        .take(5)
        .collect::<Vec<&NewsMeta>>();

    let recent_albums = site_map
        .albums
        .iter()
        .filter(|album| album.authors.contains(&member.ascii_name))
        .take(5)
        .collect::<Vec<&AlbumMeta>>();

    let inner = html! {
        section #member-detail {
            .member-detail-container {
                .member-profile {
                    .member-profile-image {
                        img .img-placeholder src=(format!("icon/{}.jpg", member.name)) alt=(member.name);
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
                                (sns_icon(link)?)
                            }
                        }
                    }
                }
            }

            .member-works-container {
                .member-featured-works {
                    h3 { "代表作品" }
                    .container {
                        @for featured in &recent_works {
                            (featured_work_item_detail(featured))
                        }
                        @if recent_works.is_empty() {
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
}

pub fn featured_work_item_detail(item: &WorkMeta) -> Markup {
    let work_ref = reference(item);

    html! {
        .work-item-detail id=(work_ref) {
            h4 { (item.title) }
            .work-youtube-container {
                img .work-item-thumb src=(image_or_gray(item.thumbnail.as_ref())) alt=(item.title) {}
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
    }
}

pub fn featured_post_detail(item: &NewsMeta) -> Result<Markup, Report> {
    Ok(html! {
        .post-card style="width: 100%;" {
            .member-profile-image .post-card-image {
                img .post-img src=(image_or_gray(item.thumbnail.as_ref())) {}
            }
            .post-info {
                h3 .post-card-title style="text-align: start; margin-bottom: 0px;" {
                    a href=(format!("/news/{}.html", news_reference(&item.title, hash(&item)))) {
                        (item.title)
                    }
                }
                p .member-role {
                    (item.date)
                }
                @if let Some(short) = item.short {
                    (short)
                }
                .member-links {
                    @for link in &item.sns_links {
                        (sns_icon(link)?)
                    }
                }
            }
        }
    })
}

pub fn featured_album_detail(album_meta: &AlbumMeta, sitemap: &SiteMap) -> Result<Markup, Report> {
    Ok(html! {
        .post-card style="width: 100%;" {
            .member-profile-image .post-card-image {
                img .work-item-thumb src=(image_or_gray(album_meta.thumbnail.as_ref())) alt=(&album_meta.title) {}
            }
            .post-info {
                h3 .post-card-title style="text-align: start; margin-bottom: 0px;" {
                    a href=(format!("/works/albums/{}.html", album_reference(&album_meta.title, &album_meta.front_cover))) {
                        (album_meta.title)
                    }
                }
                p .work-role {
                    (album_meta.date)
                }
                p .member-role {
                    (album_meta.contributors_str(namemap))
                }
            }
        }
    })
}
