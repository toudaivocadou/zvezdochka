use crate::site::album::AlbumMeta;
use crate::site::namemap::NameMap;
use crate::site::templates::functions::embed::embed;
use crate::site::templates::functions::sns::sns_icon;
use crate::site::util::image_or_gray;
use crate::site::util::{author_list, reference};
use crate::site::work::WorkMeta;
use anyhow::Error;
use hauchiwa::Tracker;
use hauchiwa::loader::Document;
use maud::{Markup, PreEscaped, html};

pub fn work_album_index(
    names: &NameMap,
    works: &Tracker<'_, Document<WorkMeta>>,
    albums: &Tracker<'_, Document<AlbumMeta>>,
) -> Result<Markup, Error> {
    Ok(html! {
        section #hero {
            .container {
                h2 { "作品" }
                p { "東京大学ボカロP同好会のメンバーの作品目録です。" }
            }
        }

        section #filters {
            .container .filters {
                .click-button {
                    a .filter-link href="#songs" {
                        p { "音楽のリリース" }
                    }
                }
                .click-button {
                    a .filter-link href="#albums" {
                        p { "アルバム" }
                    }
                }
            }
        }

        section #songs .list {
            .container {
                h2 {
                    "リリース"
                }
                .zcontainer {
                    .member-grid {
                        @for (_, work) in works.iter() {
                            (work_card(names, &work.matter)?)
                        }
                    }
                }
            }
        }

        section #albums .list {
            .container {
                h2 {
                "アルバム"
                }
                .zcontainer {
                    .member-grid {
                        @for (_, album) in albums.iter() {
                            (album_card(names, &album.matter)?)
                        }
                    }
                }
            }
        }
    })

    // let metadata = Metadata {
    //     page_title: "リリース".to_string(),
    //     page_image: None,
    //     canonical_link: "/works.html".to_string(),
    //     section: Sections::Works,
    //     description: Some("東京大学ボカロP同好会のメンバーの作品展示館".to_string()),
    //     author: None,
    //     date: None,
    // };
}

fn work_card(sitemap: &NameMap, work_meta: &WorkMeta) -> Result<Markup, Error> {
    Ok(html! {
        .work-item {
            a .member-link href=(format!("/works/releases/{}/index.html", reference(&work_meta.title, &work_meta.authors, &work_meta.additional_authors))) {
                .work-card {
                    h4 .member-info {
                        a .member-link href=(format!("/works/releases/{}/index.html", reference(&work_meta.title, &work_meta.authors, &work_meta.additional_authors))){
                            (work_meta.title)
                        }
                    }
                    .work-thumbnail {
                        img .work-item-thumb src=(image_or_gray(work_meta.thumbnail.as_ref().map(|x| &x.image))) alt=(work_meta.title) {}
                    }
                    .work-description {
                        (author_list(sitemap, &work_meta.authors, &work_meta.additional_authors))
                        p .work-date {
                            (work_meta.date)
                        }
                        p {
                            (work_meta.short.clone().unwrap_or_default())
                        }
                    }
                }
            }
        }
    })
}

fn album_card(sitemap: &NameMap, album_meta: &AlbumMeta) -> Result<Markup, Error> {
    Ok(html! {
        .work-item {
            a .member-link href=(
                format!("/works/albums/{}/index.html", reference(&album_meta.title, &album_meta.authors, &album_meta.additional_authors))
            ) {
                .work-card {
                    h4 .member-info {
                        a href=(
                            format!("/works/albums/{}/index.html", reference(&album_meta.title, &album_meta.authors, &album_meta.additional_authors))
                        ) {
                            (album_meta.title)
                        }
                    }
                    .work-thumbnail {
                        img .work-item-thumb src=(&album_meta.thumbnail.image) alt=(&album_meta.title) {}
                    }
                    .work-description {
                        p .member-role {
                            (author_list(sitemap, &album_meta.authors, &album_meta.additional_authors))
                        }
                        p .work-date {
                            (album_meta.date)
                        }
                        p {
                            @if let Some(short) = &album_meta.short {
                                (short)
                            }
                            @else {
                                i { "説明がありません" }
                            }
                        }
                    }
                }
            }
        }
    })
}

pub fn album_detail(
    sitemap: &NameMap,
    album_meta: &AlbumMeta,
    content: String,
) -> Result<Markup, Error> {
    Ok(html! {
        section #work-section {
            .work-detail-container {
                .work-detail {
                    .work-image {
                        img .img-placeholder src=(album_meta.thumbnail.image) alt=(album_meta.title);
                    }
                    .work-info {
                        h2 { (album_meta.title) }
                        @if let Some(short) = &album_meta.short {
                            p { (short) }
                        }
                        .work-contributors {
                            "投稿者: " (author_list(sitemap, &album_meta.authors, &album_meta.additional_authors))
                        }
                        p {
                            @if let Some(short) = &album_meta.short {
                                (short)
                            }
                            @else {
                                i { "説明がありません" }
                            }
                        }
                        .member-links {
                            @for link in &album_meta.sns_links {
                                (sns_icon(link)?)
                            }
                        }
                    }
                }
            }

            .member-works-container {
                section #tracklist {
                    h2 { "トラックリスト" }
                    dl .tracklist-list  {
                        @for (number, (title, track)) in album_meta.tracks.iter().enumerate() {
                            .tracklist-track {
                                dt .track-title {
                                    h2 {
                                        (number + 1) ". "
                                        @if !track.external {
                                            a href=(reference(title, &track.authors, &track.additional_authors)) {
                                                (title)
                                            }
                                        } @else {
                                            (title)
                                        }
                                    }
                                }
                                dd .track-author {
                                    (author_list(sitemap, &track.authors, &track.additional_authors))
                                }
                                dd .track-length {
                                    (track.duration.format())
                                }
                            }
                        }
                    }
                    @if let Some(link) = &album_meta.link {
                        .click-button {
                            a href=(link) alt=(&album_meta.title) {
                                p { "プレイリストに行く" }
                            }
                        }
                    }
                }

                section #description .work-description {
                    h2 { "歌詞" }
                    .description {
                        (PreEscaped(content))
                    }
                }

                section #crossfade {
                    h2 { "試聴動画" }
                    @if let Some(crossfade_demonstration) = &album_meta.demonstration {
                        .work-youtube-container {
                            .youtube-embed-container {
                                (embed(crossfade_demonstration.as_str())?)
                            }
                        }
                    } @else {
                        p .work-no-description {
                            em { "試聴動画がありません。" }
                        }
                    }
                }


                section #additional-album-images {
                    h2 { "イラスト" }
                    .container {
                        .work-item-detail #frontcover {
                            h4 { "フロントカーバー" }
                            .work-illustration-container {
                                img .work-item-thumb src=(album_meta.thumbnail.image) alt=(album_meta.title);
                            }
                            p {"イラスト: " (author_list(sitemap, &album_meta.thumbnail.illustrators, &album_meta.thumbnail.additional_illustrators)) }
                        }
                        @for (title, illustration) in &album_meta.illustrations {
                            .work-item-detail #(format!("illustration-{}", title)) {
                                h4 { (title) }
                                .work-illustration-container {
                                    img .img-placeholder src=(illustration.image) alt=(title);
                                }
                                p {"イラスト: " (author_list(sitemap, &album_meta.thumbnail.illustrators, &album_meta.thumbnail.additional_illustrators)) }
                            }
                        }
                    }
                }


                .back-button{
                    a href="../../works/index.html" {
                        "リリース集合一覧に戻る"
                    }
                }
            }
        }
    })

    // let metadata = Metadata {
    //     page_title: album_meta.title.clone(),
    //     page_image: None,
    //     canonical_link: format!(
    //         "/works/albums/{}.html",
    //         album_reference(&album_meta.title, &album_meta.front_cover)
    //     ),
    //     section: Sections::AlbumPost,
    //     description: Some(album_meta.short.clone()),
    //     author: Some(album_meta.contributors_str(name_map)),
    //     date: Some(album_meta.release_date.to_string()),
    // };
}

pub fn work_detail(
    sitemap: &NameMap,
    work_meta: &WorkMeta,
    content: String,
) -> Result<Markup, Error> {
    Ok(html! {
        section #work-section {
            .work-detail-container {
                .work-detail {
                    .work-thumbnail {
                        img .img-placeholder src=(image_or_gray(work_meta.thumbnail.as_ref().map(|x| &x.image))) alt=(work_meta.title);
                    }
                    .work-info {
                        h2 { (work_meta.title) }
                        // .work-featured-work {
                        //     @if work_meta.featured {
                        //         h5 { "⭐: このリリースはメンバーページでフィーチャーされています。" }
                        //     }
                        // }
                        .work-date {
                            p { (work_meta.date) }
                        }
                        p { "投稿者: " (author_list(sitemap, &work_meta.authors, &work_meta.additional_authors)) }
                        @if let Some(short) = &work_meta.short {
                            p .work-bio { (short) }
                        }
                        .member-links {
                            @for link in &work_meta.sns_links {
                                (sns_icon(link)?)
                            }
                        }
                    }
                }
            }

            .member-works-container {
                section .work-featured-work-container {
                    h2 { "作品リンク" }
                    @if let Some(link) = &work_meta.source {
                        .youtube-embed-container {
                            (embed(link.as_str())?)
                        }
                        .click-button {
                            a href=(link) alt=(&work_meta.title) {
                                p { "現本に行く" }
                            }
                        }
                    } @else {
                        p .work-no-description {
                            em { "リンクがありません。" }
                        }
                    }

                }

                section #description .work-description {
                    h2 { "作品説明" }

                    @if content.is_empty() {
                        p .work-no-description {
                            em { "説明がありません。" }
                        }
                    } @else {
                        .description {
                            (PreEscaped(content))
                        }
                    }
                }

                .back-button{
                    a href="../../works/index.html" {
                        "リリース集合一覧に戻る"
                    }
                }
            }
        }
    })
}
