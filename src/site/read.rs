use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use crate::FRONT_MATTER_SPLIT;
use crate::news::{NewsMeta, RawNewsMeta};
use crate::work::{CoverOrImage, RawWorkMeta, WorkMeta};
use hauchiwa::Page;
use hauchiwa::RuntimeError;
use serde::de::DeserializeOwned;

pub fn parse_front_matter_and_fetch_contents<Metadata>(
    file: &str,
) -> Result<(Metadata, String), anyhow::Error>
where
    Metadata: DeserializeOwned,
{
    let (front_matter, content) = match file.split_once(FRONT_MATTER_SPLIT) {
        Some(v) => v,
        None => {
            return Err(anyhow::Error::msg(format!(
                "Failed to split front matter! Ensure that the front matter splitter \"{FRONT_MATTER_SPLIT}\" exists! - フロントデータを分離できませんでした。フロントデータ分離マーカー「{FRONT_MATTER_SPLIT}」があるのかを確認してください！"
            )));
        }
    };

    let toml_parsed = toml::from_str::<Metadata>(front_matter)?;

    Ok((toml_parsed, content.to_string()))
}

pub fn parse_post_meta(file: &str) -> Result<(NewsMeta, String), anyhow::Error> {
    let (raw_post, content) = parse_front_matter_and_fetch_contents::<RawNewsMeta>(file)?;

    let new_short = match raw_post.short {
        Some(shrt) => shrt,
        None => format!("{}...", content.chars().take(50).collect::<String>()),
    };

    Ok((
        NewsMeta {
            title: raw_post.title,
            author: raw_post.author,
            header_image: raw_post.header_image,
            date: raw_post.date,
            short: new_short,
            sns_links: raw_post.sns_links,
        },
        content,
    ))
}

pub fn parse_work_meta(file: &str) -> Result<(WorkMeta, String), anyhow::Error> {
    let (raw_work, content) = parse_front_matter_and_fetch_contents::<RawWorkMeta>(file)?;

    let coi = match &raw_work.cover_image {
        Some(coverimg) => CoverOrImage::Cover(coverimg.to_string()),
        None => match &raw_work.link {
            Some(wlnl) => CoverOrImage::Link(wlnl.clone()),
            None => match &raw_work.file {
                Some(f) => CoverOrImage::AudioFile(f.to_string()),
                None => {
                    return Err(anyhow::Error::msg(
                        "Could not find a suitable display. Please ensure one of the following is set: `link`, `cover`, `file`.",
                    ))?;
                }
            },
        },
    };

    Ok((
        WorkMeta {
            title: raw_work.title,
            author: raw_work.author,
            collaborators: raw_work.collaborators,
            date: raw_work.date,
            short: raw_work.short,
            display: coi,
            cover_image: raw_work.cover_image,
            link: raw_work.link,
            file: raw_work.file,
            remix_original_work: raw_work.remix_original_work,
            featured: raw_work.featured,
            streaming: raw_work.streaming,
            duration_seconds: raw_work.duration_seconds,
        },
        content,
    ))
}

pub fn robots_txt() -> Result<Page, RuntimeError> {
    let mut robots = String::new();
    File::open("robots.txt")
        .unwrap()
        .read_to_string(&mut robots)
        .unwrap();
    Ok(Page::text(
        camino::Utf8PathBuf::from_str("robots.txt")?,
        robots,
    ))
}
