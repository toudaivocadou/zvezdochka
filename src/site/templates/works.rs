use crate::site::album::AlbumMeta;
use crate::site::sitemap::SiteMap;
use crate::site::templates::functions::embed::embed;
use crate::site::templates::functions::sns::sns_icon;
use crate::site::util::image_or_gray;
use crate::site::util::{author_list, reference};
use crate::site::work::WorkMeta;
use eyre::Report;
use maud::{Markup, PreEscaped, html};

pub fn works(site_map: &SiteMap) -> Result<Markup, Report> {
    Ok(html! {
        section #hero {
            .container {
                h2 { "リリース" }
                p { "東京大学ボカロP同好会のメンバーの作品目録です。" }
            }
        }

        section #filters {
            .container .filters {
                .click-button {
                    a .filter-link href="#songs" {
                        p { "リリース" }
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
                        @for work in &site_map.works {
                            (work_card(site_map, work)?)
                        }
                        @if site_map.works.is_empty() {
                            p .work-description style="text-align: center;" {
                                em {
                                    "リリースがありません。"
                                }
                            }
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
                        @for album in &site_map.albums {
                            (album_card(site_map, album)?)
                        }
                        @if site_map.albums.is_empty() {
                            p .work-description style="text-align: center;" {
                                em {
                                    "アルバムがありません。"
                                }
                            }
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

pub fn work_card(sitemap: &SiteMap, work_meta: &WorkMeta) -> Result<Markup, Report> {
    Ok(html! {
        .work-item {
            a .member-link href=(format!("/works/releases/{}.html", reference(&work_meta.title, &work_meta.authors, &work_meta.additional_authors))) {
                .work-card {
                    h4 .member-info {
                        a .member-link href=(format!("/works/releases/{}.html", reference(&work_meta.title, &work_meta.authors, &work_meta.additional_authors))){
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

pub fn album_card(sitemap: &SiteMap, album_meta: &AlbumMeta) -> Result<Markup, Report> {
    Ok(html! {
        .work-item {
            a .member-link href=(
                format!("/works/albums/{}.html", reference(&album_meta.title, &album_meta.authors, &album_meta.additional_authors))
            ) {
                .work-card {
                    h4 .member-info {
                        a href=(
                            format!("/works/albums/{}.html", reference(&album_meta.title, &album_meta.authors, &album_meta.additional_authors))
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
    sitemap: &SiteMap,
    album_meta: &AlbumMeta,
    content: &str,
) -> Result<Markup, Report> {
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
                        @for (number, track) in album_meta.tracks.iter().enumerate() {
                            .tracklist-track {
                                dt .track-title {
                                    h2 {
                                        (number + 1) ". "
                                        @if !track.external {
                                            a href=(reference(&track.title, &track.authors, &track.additional_authors)) {
                                                (track.title)
                                            }
                                        } @else {
                                            (track.title)
                                        }
                                    }
                                }
                                dd .track-author {
                                    (author_list(sitemap, &track.authors, &track.additional_authors))
                                }
                                @if let Some(duration) = track.duration {
                                    dd .track-length {
                                        (duration)
                                    }
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
                    h2 { "説明" }
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
                        @for illustration in &album_meta.illustrations {
                            .work-item-detail #(format!("illustration-{}", illustration.title)) {
                                h4 { (illustration.title) }
                                .work-illustration-container {
                                    img .img-placeholder src=(illustration.image) alt=(illustration.title);
                                }
                                p {"イラスト: " (author_list(sitemap, &album_meta.thumbnail.illustrators, &album_meta.thumbnail.additional_illustrators)) }
                            }
                        }
                    }
                }


                .back-button{
                    a href="../../works.html" {
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
    sitemap: &SiteMap,
    work_meta: &WorkMeta,
    content: &str,
) -> Result<Markup, Report> {
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
                    .description {
                        (PreEscaped(content))
                    }
                    @if content.is_empty() {
                        p .work-no-description {
                            em { "説明がありません。" }
                        }
                    }
                }

                .back-button{
                    a href="../../works.html" {
                        "リリース集合一覧に戻る"
                    }
                }
            }
        }
    })
}
