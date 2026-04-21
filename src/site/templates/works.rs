use crate::SiteData;
use crate::album::AlbumMeta;
use crate::die_linky::SocialLinkType;
use crate::metadata::Metadata;
use crate::site::sitemap::{AlbumRef, WorkRef};
use crate::sitemap::SiteMap;
use crate::templates::base::base;
use crate::templates::functions::embed::embed;
use crate::templates::functions::sns::sns_icon;
use crate::templates::partials::navbar::Sections;
use crate::util::{image, shorten};
use crate::work::WorkMeta;
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use hauchiwa::{Context, RuntimeError};
use maud::{Markup, PreEscaped, html};
use std::collections::HashMap;
use std::str::FromStr;
use url::Url;

pub fn works(
    sack: &Context<SiteData>,
    site_map: &SiteMap,
    name_map: &HashMap<String, String>,
) -> Result<Markup, RuntimeError> {
    // TODO: pagination. this will get ungodly long. yell at peng if we get >100!

    let inner = html! {
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
                            (work_card(sack, work, name_map)?)
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
                            (album_card(sack, album, name_map)?)
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
    };

    let metadata = Metadata {
        page_title: "リリース".to_string(),
        page_image: None,
        canonical_link: "/works.html".to_string(),
        section: Sections::Works,
        description: Some("東京大学ボカロP同好会のメンバーの作品展示館".to_string()),
        author: None,
        date: None,
    };

    base(sack, &metadata, Some(&[]), inner)
}

pub fn work_card(
    sack: &Context<SiteData>,
    work_meta: &WorkMeta,
    name_map: &HashMap<String, String>,
) -> Result<Markup, RuntimeError> {
    let author_name = name_map.get(&work_meta.author).ok_or(RuntimeError::msg("Could not find author. Does the member page exist? Did you remember to type in the ascii name? Did you mistype it?".to_string()))?;

    Ok(html! {
        .work-item {
            a .member-link href=(format!("/works/releases/{}.html", work_reference(&work_meta.title, &work_meta.author))) {
                .work-card {
                    h4 .member-info {
                        a .member-link href=(format!("/works/releases/{}.html", work_reference(&work_meta.title, &work_meta.author))){
                            (work_meta.title)
                        }
                    }
                    .work-thumbnail {
                        img .work-item-thumb src=(thumbnail_link(sack, work_meta)?) alt=(work_meta.title) {}
                    }
                    .work-description {
                        a href=(format!("/members/{}.html", work_meta.author)) {
                            p .member-role {
                                (author_name)
                            }
                        }
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

pub fn work_reference(title: &str, hash: u64) -> WorkRef {
    let cachebust = BASE64_URL_SAFE_NO_PAD.encode(hash.to_le_bytes());

    WorkRef(format!("{}-{}", urlencoding::encode(title), cachebust))
}

pub fn album_reference(title: &str, hash: u64) -> AlbumRef {
    let cachebust = BASE64_URL_SAFE_NO_PAD.encode(hash.to_le_bytes());

    AlbumRef(format!("{}-{}", urlencoding::encode(title), cachebust))
}

pub fn album_card(
    sack: &Context<SiteData>,
    album_meta: &AlbumMeta,
    name_map: &HashMap<String, String>,
) -> Result<Markup, RuntimeError> {
    let contribs = format!(
        "{}...",
        album_meta
            .contributors_str(name_map)
            .chars()
            .take(18)
            .collect::<String>()
    );
    Ok(html! {
        .work-item {
            a .member-link href=(
                format!("/works/albums/{}.html", album_reference(&album_meta.title, &album_meta.front_cover))
            ) {
                .work-card {
                    h4 .member-info {
                        a href=(
                            format!("/works/albums/{}.html", album_reference(&album_meta.title, &album_meta.front_cover))
                        ) {
                            (album_meta.title)
                        }
                    }
                    .work-thumbnail {
                        img .work-item-thumb src=(image(sack, format!("images/{}", &album_meta.front_cover))?) alt=(&album_meta.title) {}
                    }
                    .work-description {
                        p .member-role {
                            (contribs)
                        }
                        p .work-date {
                            (album_meta.release_date)
                        }
                        @if let Some(subtitle) = &album_meta.subtitle {
                            p { (subtitle) }
                        }
                        p {
                            (album_meta.short)
                        }
                    }
                }
            }
        }
    })
}

pub fn album_detail(
    sack: &Context<SiteData>,
    album_meta: &AlbumMeta,
    name_map: &HashMap<String, String>,
    content: &str,
) -> Result<Markup, RuntimeError> {
    let contributors = album_meta.contributors.iter().map(|contributor| {
        let ascii_name = name_map.get(contributor).unwrap();
        html! {
            a href=(format!("/members/{}.html", ascii_name)) {
                (contributor)
            }
        }
    });

    let extra_contributors = album_meta.extra_contributors.iter();

    let inner = html! {
        section #work-section {
            .work-detail-container {
                .work-detail {
                    .work-image {
                        img .img-placeholder src=(image(sack, format!("images/{}", &album_meta.front_cover))?) alt=(album_meta.title);
                    }
                    .work-info {
                        h2 { (album_meta.title) }
                        @if let Some(subtitle) = &album_meta.subtitle {
                            p { (subtitle) }
                        }
                        .work-contributors {
                            p {
                                "投稿者: "
                                @for contrib in contributors {
                                    (contrib) " "
                                }
                                @for extrac in extra_contributors {
                                    (extrac) " "
                                }
                            }
                        }
                        p {
                            (album_meta.short)
                        }
                        .member-links {
                            @for link in &album_meta.sns_links {
                                (sns_icon(sack, link.as_str())?)
                            }
                        }
                    }
                }
            }

            .member-works-container {
                section #tracklist {
                    h2 { "トラックリスト" }
                    dl .tracklist-list  {
                        @for (number, track) in album_meta.tracklist.iter().enumerate() {
                            .tracklist-track {
                                dt .track-title {
                                    h2 {
                                        (number + 1) ". "
                                        @if track.on_site {
                                            a href=(work_reference(&track.title, &track.author)) {
                                                (track.title)
                                            }
                                        } @else if let Some(link) = &track.link {
                                            a href=(link) {
                                                (track.title)
                                            }
                                        } @else {
                                            (track.title)
                                        }
                                    }
                                }
                                dd .track-author {
                                    "投稿者: "
                                    @if track.external_author {
                                        (track.author)
                                    } @else {
                                        a href=(format!("/members/{}.html", &track.author)) {
                                            (name_map.get(&track.author).ok_or(RuntimeError::msg("User does not exist in album"))?)
                                        }
                                    }
                                }
                                @if let Some(duration_seconds) = track.duration_seconds {
                                    dd .track-length {
                                        ({
                                            let minutes = duration_seconds / 60;
                                            let seconds = duration_seconds % 60;
                                            format!("{}:{:02}", minutes, seconds)
                                        })
                                    }
                                }
                            }
                        }
                    }
                    @if let Some(link) = &album_meta.playlist_link {
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
                    @if let Some(crossfade_demonstration) = &album_meta.crossfade_demonstration {
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
                                img .work-item-thumb src=(image(sack, format!("images/{}", &album_meta.front_cover))?) alt=(album_meta.title);
                            }
                            @if album_meta.front_cover_illustrator_not_on_site {
                                p {"イラスト: " (album_meta.front_cover_illustrator) }
                            }
                            @else {
                                a href=(format!("/members/{}.html", album_meta.front_cover_illustrator)) {
                                    p { "イラスト: " (name_map.get(&album_meta.front_cover_illustrator).ok_or(RuntimeError::msg("did not find front cover illustrator on site"))?) }
                                }
                            }
                        }
                        @for (header, imglnk) in &album_meta.other_covers {
                            .work-item-detail #(header) {
                                h4 { (header) }
                                .work-illustration-container {
                                    img .img-placeholder src=(image(sack, format!("images/{}", imglnk.link))?) alt=(header);
                                }
                                @if imglnk.illustrator_is_not_on_site {
                                    p { "イラスト: " (imglnk.illustrator) }
                                } @else {
                                    a href=(format!("/members/{}.html", imglnk.illustrator)) {
                                        p { "イラスト: " (name_map.get(&imglnk.illustrator).ok_or(RuntimeError::msg("did not find illustrator on site"))?) }
                                    }
                                }
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
    };

    let metadata = Metadata {
        page_title: album_meta.title.clone(),
        page_image: None,
        canonical_link: format!(
            "/works/albums/{}.html",
            album_reference(&album_meta.title, &album_meta.front_cover)
        ),
        section: Sections::AlbumPost,
        description: Some(album_meta.short.clone()),
        author: Some(album_meta.contributors_str(name_map)),
        date: Some(album_meta.release_date.to_string()),
    };
    base(sack, &metadata, Some(&[]), inner)
}

pub fn work_detail(
    sack: &Context<SiteData>,
    work_meta: &WorkMeta,
    name_map: &HashMap<String, String>,
    content: &str,
) -> Result<Markup, RuntimeError> {
    let author_name = name_map.get(&work_meta.author).expect("Could not find author. Does the member page exist? Did you remember to type in the ascii name? Did you mistype it? Yell at peg for more info");

    let inner = html! {
        section #work-section {
            .work-detail-container {
                .work-detail {
                    .work-thumbnail {
                        img .img-placeholder src=(thumbnail_link(sack, work_meta)?) alt=(work_meta.title);
                    }
                    .work-info {
                        h2 { (work_meta.title) }
                        .work-featured-work {
                            @if work_meta.featured {
                                h5 { "⭐: このリリースはメンバーページでフィーチャーされています。" }
                            }
                        }
                        .work-date {
                            p { (work_meta.date) }
                        }
                        a .member-role .member-bio href=(format!("/members/{}.html", work_meta.author)) { p { (author_name) } }
                        @if let Some(short) = &work_meta.short {
                            p .work-bio { (short) }
                        }
                        .member-links {
                            @for link in &work_meta.streaming {
                                (sns_icon(sack, link)?)
                            }
                        }
                    }
                }
            }

            .member-works-container {
                section .work-featured-work-container {
                    h2 { "作品リンク" }
                    @if let Some(link) = &work_meta.link {
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
    };

    let page_image = match &work_meta.display {
        crate::work::CoverOrImage::Cover(cover) => Some(cover.to_string()),
        crate::work::CoverOrImage::Link(url) => Some(url.to_string()),
        crate::work::CoverOrImage::AudioFile(_) => None,
    };

    let metadata = Metadata {
        page_title: work_meta.title.clone(),
        page_image,
        canonical_link: format!("/works/releases/{}.html", work_meta.title),
        section: Sections::WorksPost,
        description: Some(work_meta.short.clone().unwrap_or(shorten(content))),
        author: Some(work_meta.author.clone()),
        date: Some(work_meta.date.to_string()),
    };
    base(sack, &metadata, Some(&[]), inner)
}

pub fn thumbnail_link(sack: &Context<SiteData>, meta: &WorkMeta) -> Result<String, RuntimeError> {
    match &meta.display {
        crate::work::CoverOrImage::Cover(cover) => image(sack, cover),
        crate::work::CoverOrImage::Link(url) => get_link_image_thumb(sack, url.as_str()),
        crate::work::CoverOrImage::AudioFile(_audio_file) => Ok("".to_string()),
    }
}

pub fn get_link_image_thumb(sack: &Context<SiteData>, link: &str) -> Result<String, RuntimeError> {
    let url_type =
        SocialLinkType::from_str(link).map_err(|why| RuntimeError::msg(why.to_string()))?;
    let url_parse = Url::parse(link).map_err(|why| RuntimeError::msg(why.to_string()))?;

    match url_type {
        SocialLinkType::Youtube => {
            let youtube_video_id = url_parse
                .query_pairs()
                .find(|(key, _)| key == "v")
                .ok_or(RuntimeError::msg("Invalid youtube id"))?
                .1;
            Ok(format!(
                "https://img.youtube.com/vi/{}/maxresdefault.jpg",
                youtube_video_id
            ))
        }
        SocialLinkType::NicoDouga => {
            // FIXME: NND is fucking cringe and you need some sort of key to download their thumbs.
            // use request and fetch the thumbnails and host them locally.
            // Until then, lol.
            image(sack, "images/gray.jpg")
        }
        _ => image(sack, "images/gray.jpg"),
    }
}
