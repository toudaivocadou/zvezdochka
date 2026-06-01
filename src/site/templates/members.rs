use anyhow::Error;
use hauchiwa::{Tracker, loader::Document};
use maud::{Markup, PreEscaped, html};

use crate::site::{
    album::AlbumMeta,
    member::MemberMeta,
    namemap::NameMap,
    news::NewsMeta,
    templates::{functions::sns::sns_icon, news::NEWS_MISSING_AUTHOR},
    util::{author_list, image_or_gray, reference},
    work::WorkMeta,
};

pub fn member_index(members: Tracker<'_, Document<MemberMeta>>) -> Result<Markup, Error> {
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
                    @for (_, member) in members {
                        (member_card(&member.matter)?)
                    }
                }
            }
        }
    })
}

fn member_card(member: &MemberMeta) -> Result<Markup, Error> {
    let member_links_len = member.links.len();
    Ok(html! {
        .member-item {
            a .member-link href=(format!("/members/{}/index.html", member.ascii_name)) {
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
    names: &NameMap,
    all_works: Tracker<'_, Document<WorkMeta>>,
    all_albums: Tracker<'_, Document<AlbumMeta>>,
    all_news: Tracker<'_, Document<NewsMeta>>,
    content: String,
) -> Result<Markup, Error> {
    let recent_works = all_works
        .iter()
        .map(|(_, meta)| &meta.matter)
        .filter(|featured| featured.authors.contains(&member.ascii_name))
        .take(5)
        .collect::<Vec<_>>();

    let recent_news = all_news
        .iter()
        .map(|(_, meta)| &meta.matter)
        .filter(|post| post.author.as_ref() == Some(&member.ascii_name))
        .take(5)
        .collect::<Vec<_>>();

    let recent_albums = all_albums
        .iter()
        .map(|(_, meta)| &meta.matter)
        .filter(|album| album.authors.contains(&member.ascii_name))
        .take(5)
        .collect::<Vec<_>>();

    Ok(html! {
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
                    h3 { "最近投稿作品" }
                    .container {
                        @for featured in recent_works {
                            (featured_work_detail(featured))
                        }
                    }
                }

                .member-featured-works {
                    h3 { "最近投稿ニュース" }
                    .container {
                        @for news in recent_news {
                            (featured_post_detail(news)?)
                        }
                    }
                }

                .member-featured-works {
                    h3 { "最近投稿アルバム" }
                    .container {
                        @for featured in recent_albums {
                            (featured_album_detail(names, featured)?)
                        }
                    }
                }

                .back-button  {
                    a href="../members/index.html" class="margin-top: 50px;" {
                        "メンバー一覧に戻る"
                    }
                }
            }
        }
    })
}

fn featured_work_detail(work: &WorkMeta) -> Markup {
    html! {
        .work-item-detail id=(urlencoding::encode(&work.title)) {
            h4 { (work.title) }
            .work-youtube-container {
                img .work-item-thumb src=(image_or_gray(work.thumbnail.as_ref().map(|x| &x.image))) alt=(work.title) {}
            }

            .work-description {
                p {
                    @if let Some(short) = &work.short {
                        (short)
                    } @else {
                        ("説明がありません。")
                    }
                }
            }

            .click-button{
                a href=(format!("/works/releases/{}/index.html", reference(&work.title, &work.authors, &work.additional_authors))) {
                    p { "詳しく見る" }
                }
            }
        }
    }
}

fn featured_post_detail(news: &NewsMeta) -> Result<Markup, Error> {
    Ok(html! {
        .post-card style="width: 100%;" {
            .member-profile-image .post-card-image {
                img .post-img src=(image_or_gray(news.thumbnail.as_ref())) {}
            }
            .post-info {
                h3 .post-card-title style="text-align: start; margin-bottom: 0px;" {
                    a href=(format!("/news/{}/index.html", reference(&news.title, &[news.author.as_ref().map(|x| x.as_str()).unwrap_or(NEWS_MISSING_AUTHOR)], &[]))) {
                        (news.title)
                    }
                }
                p .member-role {
                    (news.date)
                }
                p {
                    @if let Some(short) = &news.short {
                        (short)
                    } @else {
                        ("説明がありません。")
                    }
                }

                .member-links {
                    @for link in &news.sns_links {
                        (sns_icon(link)?)
                    }
                }
            }
        }
    })
}

fn featured_album_detail(name_map: &NameMap, album_meta: &AlbumMeta) -> Result<Markup, Error> {
    Ok(html! {
        .post-card style="width: 100%;" {
            .member-profile-image .post-card-image {
                img .work-item-thumb src=(&album_meta.thumbnail.image) alt=(&album_meta.thumbnail.title) {}
            }
            .post-info {
                h3 .post-card-title style="text-align: start; margin-bottom: 0px;" {
                    a href=(format!("/works/albums/{}/index.html", reference(&album_meta.title, &album_meta.authors, &album_meta.additional_authors))) {
                        (album_meta.title)
                    }
                }
                p .work-role {
                    (album_meta.date)
                }
                p .member-role {
                    (author_list(name_map, &album_meta.authors, &album_meta.additional_authors))
                }
            }
        }
    })
}
