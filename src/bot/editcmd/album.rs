use anyhow::Error;
use camino::Utf8Path;
use fancy_duration::FancyDuration;
use poise::{
    Context, CreateReply,
    serenity_prelude::{ChannelId, MessageFlags},
};
use std::{collections::HashSet, time::Duration};
use tokio::{fs::File, io::AsyncReadExt};
use url::Url;

use crate::{
    bot::{
        Data,
        editcmd::common::{
            ERR_MSG_ONLY_WORK_CHANNEL, ERR_WORK_MUTEX_LOCK, glob_files, split_file_str,
        },
        work::{
            AlbumTitleAndTrack, AlbumWorkStatePage, AttachmentOrDownloadURL, Authors, BasicInfo,
            MainLink, ModalListWrapper, ModalWrapper, Thumbnail, WorkData, WorkState,
        },
    },
    site::{
        ALBUM_GLOB,
        album::{AlbumMeta, Illustration, Track},
    },
};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn list_albums(ctx: Context<'_, Data, Error>, page: Option<u32>) -> Result<(), Error> {
    let page = page.unwrap_or(1);
    if page == 0 {
        ctx.reply("不正引数: 第0ページは存在しません。").await?;
        return Ok(());
    }
    let channel_id = ChannelId::from(ctx.channel_id().get());
    match ctx.data().work_states.get(&channel_id) {
        Some(work) => {
            match glob_files(&work.directory, ALBUM_GLOB)?
                .chunks(15)
                .nth((page - 1) as usize)
            {
                Some(files) => {
                    let file_str = files
                        .iter()
                        .enumerate()
                        .map(|(idx, path)| {
                            let path_str = path.to_string_lossy();
                            let index_number = ((page - 1) * 15) + (idx as u32 + 1);
                            format!("{index_number}: {path_str}\n")
                        })
                        .collect::<String>();
                    ctx.reply(format!(
                        r#"
```
ページ {page}:

{file_str}
```
                            "#
                    ))
                    .await?;
                    return Ok(());
                }
                None => {
                    ctx.reply("このページはファイルがありません。").await?;
                    return Ok(());
                }
            };
        }
        None => return Err(Error::msg("{ERR_MSG_ONLY_WORK_CHANNEL}")),
    }
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn about_album(ctx: Context<'_, Data, Error>, album_file: String) -> Result<(), Error> {
    if album_file.contains("..") {
        ctx.reply("Hello, this is Schaeffer. I'm with the other team now, and we are heading to KDS Deep; meet us there. Bring the datacubes.\nファイル名に`..`は不法文字であります。").await?;
        return Ok(());
    }

    if album_file.starts_with("/") {
        ctx.reply("<insert joycat emoji here>").await?;
        return Ok(());
    }

    if album_file.starts_with("~/") {
        ctx.reply("really funny my guy can't wait for the next one")
            .await?;
        return Ok(());
    }

    let channel_id = ChannelId::new(ctx.channel_id().get());

    match ctx.data().work_states.get(&channel_id) {
        Some(work) => {
            let made_path = format!("{}/albums/{}", work.directory.as_str(), album_file);
            let constructed_canonicalised_path = Utf8Path::new(&made_path).canonicalize()?;

            if !constructed_canonicalised_path.starts_with(work.directory.as_std_path()) {
                ctx.reply(format!(
                    "You are the murderer, {}! You and you alone.",
                    ctx.author().name
                ))
                .await?;
                return Ok(());
            }

            let mut file_str = String::new();
            File::open(&constructed_canonicalised_path)
                .await?
                .read_to_string(&mut file_str);
            let (front_matter, _) = split_file_str(&file_str)?;

            let album = serde_yaml::from_str::<AlbumMeta>(front_matter)?;

            let tracks = album
                .tracks
                .iter()
                .enumerate()
                .map(|(idx, (title, trackinfo))| {
                    format!(
                        r#"

 - 第{}トラック: 「{}」
  - 作成者: {:?}
  - 追加作成者: {:?}
  - 長さ: {}

                        "#,
                        idx + 1,
                        title,
                        trackinfo.authors,
                        trackinfo.additional_authors,
                        trackinfo.duration
                    )
                })
                .collect::<String>();

            let illustrations = album
                .illustrations
                .iter()
                .enumerate()
                .map(|(idx, (title, illustinfo))| {
                    format!(
                        r#"
 - 第{}イラスト: 「{}」
  - 作成者: {:?}
  - 追加作成者: {:?}
  - ファイル名: {}
                        "#,
                        idx + 1,
                        title,
                        illustinfo.illustrators,
                        illustinfo.additional_illustrators,
                        illustinfo.image
                    )
                })
                .collect::<String>();

            let basic_album_info = format!(
                "
題目: {}
投稿者: {:?}
追加投稿者: {:?}
リリース日付: {}

主リンク: {}
XFD: {}
SNSリンク:
{}

トラック:
{}

サムネイル: {}
イラスト:
{}

                    ",
                album.title,
                album.authors,
                album.additional_authors,
                album.date,
                album
                    .link
                    .map(|x| x.to_string())
                    .unwrap_or("なし".to_string()),
                album
                    .demonstration
                    .map(|x| x.to_string())
                    .unwrap_or("なし".to_string()),
                album
                    .sns_links
                    .iter()
                    .map(|link| { format!(" - {}\n", link.as_str()) })
                    .collect::<String>(),
                tracks,
                album.thumbnail.image,
                illustrations
            );

            ctx.reply(basic_album_info).await?;

            return Ok(());
        }
        None => return Err(Error::msg("{ERR_MSG_ONLY_WORK_CHANNEL}: {why}")),
    }
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn new_album(ctx: Context<'_, Data, Error>) -> Result<(), Error> {
    ctx.reply("作業準備中...").await?;

    let channel_id = ChannelId::new(ctx.channel_id().get());
    match ctx.data().work_states.get(&channel_id) {
        Some(work) => {
            let workdata = WorkData::Album {
                page: AlbumWorkStatePage::BasicInfo,
                basic_info: ModalWrapper::default(),
                authors: ModalWrapper::default(),
                content: ModalWrapper::default(),
                link: ModalWrapper::default(),
                sns_links: ModalListWrapper::default(),
                thumbnail: ModalWrapper::default(),
                illustrations: ModalListWrapper::default(),
                tracks: ModalListWrapper::default(),
            };

            let msg_components = workdata.create_components(None);

            match work.work_state.try_lock() {
                Ok(mut workstate) => {
                    let reply = ctx
                        .send(
                            CreateReply::new()
                                .components(msg_components)
                                .flags(MessageFlags::IS_COMPONENTS_V2),
                        )
                        .await?;

                    let message_id = reply.message().await?.id;

                    *workstate = WorkState::Working {
                        message_id,
                        data: Box::new(workdata),
                    };
                    return Ok(());
                }
                Err(why) => return Err(Error::msg(format!("{ERR_WORK_MUTEX_LOCK}: {why}"))),
            }
        }
        None => return Err(Error::msg(ERR_MSG_ONLY_WORK_CHANNEL)),
    }
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn from_bandcamp(
    ctx: Context<'_, Data, Error>,
    bandcamp_link: String,
) -> Result<(), Error> {
    ctx.reply("データ獲得準備中...").await?;

    let channel_id = ChannelId::new(ctx.channel_id().get());
    match ctx.data().work_states.get(&channel_id) {
        Some(work) => {
            let mut is_vocadou = false;

            let album_link = Url::parse(&bandcamp_link)?;
            match album_link.domain() {
                Some(domain) => {
                    if !domain.ends_with("bandcamp.com") {
                        return Err(Error::msg("バンドキャンプのリンクではありません。"));
                    }
                    if domain == "toudaivocadou.bandcamp.com" {
                        is_vocadou = true;
                    }
                }
                None => return Err(Error::msg("正しいURLを入力してください")),
            }

            let album_data = bandcamp::album_from_url(&bandcamp_link)
                .await
                .map_err(|why| {
                    Error::msg(format!(
                        "バンドキャンプからデータを読み込む中、エーラが発生しました: {why}"
                    ))
                })?;

            ctx.reply("バンドキャンプからデータを読み込みました。")
                .await?;

            let mut authors = HashSet::new();
            let mut additional_authors = HashSet::new();

            if is_vocadou {
                if album_data.album_artist.contains(",") {
                    for split in album_data.album_artist.split(",") {
                        let name = split.trim();
                        match work.name_map.members.values().find(|i| name == i.as_str()) {
                            Some(ascii_name) => {
                                authors.insert(ascii_name.clone());
                            }
                            None => {
                                additional_authors.insert(name.to_string());
                            }
                        }
                    }
                }
            }

            let album_short = album_data.about.unwrap_or(album_data.title.clone());
            let album_title = album_data.title;
            let album_date = album_data.release_date.date_naive();

            let mut tracks = Vec::with_capacity(album_data.tracks.len());

            for trackdata in album_data.tracks.into_iter() {
                let duration = match trackdata.duration {
                    Some(d) => Duration::from_secs_f32(d),
                    None => {
                        return Err(Error::msg("このトラックは長さがありません。"));
                    }
                };

                let mut authors = HashSet::new();
                let mut additional_authors = HashSet::new();

                for split in trackdata.band_name.split(",") {
                    let aname = split.trim();

                    match work.name_map.members.values().find(|i| aname == i.as_str()) {
                        Some(name) => {
                            authors.insert(name.to_string());
                        }
                        None => {
                            additional_authors.insert(aname.to_string());
                        }
                    }
                }

                tracks.push(AlbumTitleAndTrack {
                    title: trackdata.title,
                    track: Track {
                        authors: authors.into_iter().collect(),
                        additional_authors: additional_authors.into_iter().collect(),
                        duration: FancyDuration::new(duration),
                    },
                });
            }

            // get thumbnail and save to work

            let (thumbnail, destname, url) = match album_data.image.get_url() {
                Some(url) => {
                    let parsed_url = Url::parse(&url)?;
                    let url_path = parsed_url.to_file_path().map_err(|_| {
                        Error::msg("あ。。。。　あの。。。　あ。。。　あの。。。　ヤバすぎ。。。")
                    })?;
                    let destionation_file_name = match url_path.into_iter().last() {
                        Some(a) => a.to_string_lossy(),
                        None => return Err(Error::msg("No filename in path - this is bad!")),
                    };

                    // match reqwest::get(url).await?.error_for_status() {
                    //     Ok(req) => {
                    //         let mut new_file =
                    //             tokio::fs::File::create(destination_file_path).await?;
                    //         new_file.write_all(&req.bytes().await?).await?;
                    //     }
                    //     Err(why) => {
                    //         return Err(Error::msg(
                    //             "バンドキャンプからサムネイルのダウンロードが失敗しました",
                    //         )
                    //         .context(why));
                    //     }
                    // }

                    (
                        Illustration {
                            image: destionation_file_name.to_string(),
                            illustrators: vec![],
                            additional_illustrators: vec![],
                            description: None,
                        },
                        destionation_file_name.to_string(),
                        parsed_url,
                    )
                }
                None => {
                    return Err(Error::msg(
                        "サムネイルを獲得がありません:　サムネイルは必要です",
                    ));
                }
            };

            let workdata = WorkData::Album {
                page: AlbumWorkStatePage::BasicInfo,
                basic_info: ModalWrapper::InitWithValue(BasicInfo {
                    title: album_title,
                    short_description: album_short,
                    date: album_date,
                }),
                authors: ModalWrapper::InitWithValue(Authors {
                    authors: authors.into_iter().collect(),
                    additional_authors: additional_authors.into_iter().collect(),
                }),
                content: ModalWrapper::Unset,
                link: ModalWrapper::InitWithValue(MainLink { url: album_link }),
                sns_links: ModalListWrapper::default(),
                thumbnail: ModalWrapper::InitWithValue(Thumbnail {
                    illustration: thumbnail,
                    attachment: AttachmentOrDownloadURL::DownloadURL {
                        destination_filename: destname,
                        source_url: url,
                    },
                }),
                illustrations: ModalListWrapper::default(),
                tracks: ModalListWrapper::new(tracks)?,
            };

            let msg_components = workdata.create_components(None);

            match work.work_state.try_lock() {
                Ok(mut workstate) => {
                    let reply = ctx
                        .send(
                            CreateReply::new()
                                .components(msg_components)
                                .flags(MessageFlags::IS_COMPONENTS_V2),
                        )
                        .await?;

                    let message_id = reply.message().await?.id;

                    *workstate = WorkState::Working {
                        message_id,
                        data: Box::new(workdata),
                    };
                    return Ok(());
                }
                Err(why) => return Err(Error::msg(format!("{ERR_WORK_MUTEX_LOCK}: {why}"))),
            }
        }
        None => return Err(Error::msg(ERR_MSG_ONLY_WORK_CHANNEL)),
    }
}

pub async fn edit_album() {}

pub async fn delete_album() {}
