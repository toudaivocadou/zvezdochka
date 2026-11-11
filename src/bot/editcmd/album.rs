use anyhow::Error;
use fancy_duration::FancyDuration;
use poise::{
    Context, CreateReply,
    serenity_prelude::{ChannelId, MessageFlags},
};
use std::{collections::HashSet, default::Default, time::Duration};
use tokio::{
    fs::{File, remove_file},
    io::{AsyncReadExt, AsyncWriteExt},
};
use url::Url;

use crate::{
    bot::{
        Data, WorkWorkingData,
        editcmd::common::{
            ERR_MSG_ONLY_WORK_CHANNEL, ERR_WORK_MUTEX_LOCK, ensure_new_work, glob_files,
            split_file_str,
        },
        utils::{PathSection, verify_and_canonicalise_path},
        work::{
            AlbumTitleAndTrack, AlbumWorkStatePage, Authors, BasicInfo, MainLink, ModalListWrapper,
            ModalWrapper, PageContent, SnsLink, Thumbnail, TitleAndIllustraion, WorkData,
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
    let channel_id = ChannelId::new(ctx.channel_id().get());

    match ctx.data().work_states.get(&channel_id) {
        Some(work) => {
            let file_path =
                verify_and_canonicalise_path(&work.directory, PathSection::Album, &album_file)?;

            let mut file_str = String::new();
            File::open(&file_path).await?.read_to_string(&mut file_str);
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

#[poise::command(slash_command, guild_only)]
pub async fn new_album(ctx: Context<'_, Data, Error>) -> Result<(), Error> {
    ctx.reply("作業準備中...").await?;

    let channel_id = ChannelId::new(ctx.channel_id().get());
    ensure_new_work(ctx.data(), channel_id).await?;

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

                    let token = workstate.to_working(WorkWorkingData {
                        message_id,
                        data: Box::new(workdata),
                    })?;

                    ctx.data()
                        .database
                        .update_ticket_state(channel_id.into(), &workstate, token)
                        .await?;

                    return Ok(());
                }
                Err(why) => return Err(Error::msg(format!("{ERR_WORK_MUTEX_LOCK}: {why}"))),
            }
        }
        None => return Err(Error::msg(ERR_MSG_ONLY_WORK_CHANNEL)),
    }
}

#[poise::command(slash_command, guild_only)]
pub async fn from_bandcamp(
    ctx: Context<'_, Data, Error>,
    bandcamp_link: String,
) -> Result<(), Error> {
    ctx.reply("データ獲得準備中...").await?;

    let channel_id = ChannelId::new(ctx.channel_id().get());
    ensure_new_work(ctx.data(), channel_id).await?;

    match ctx.data().work_states.get(&channel_id) {
        Some(work) => {
            let album_link = Url::parse(&bandcamp_link)?;
            match album_link.domain() {
                Some(domain) => {
                    if !domain.ends_with("bandcamp.com") {
                        return Err(Error::msg("バンドキャンプのリンクではありません。"));
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

            let thumbnail = match album_data.image.get_url() {
                Some(url) => {
                    let parsed_url = Url::parse(&url)?;
                    let url_path = parsed_url.to_file_path().map_err(|_| {
                        Error::msg("あ。。。。　あの。。。　あ。。。　あの。。。　ヤバすぎ。。。")
                    })?;
                    let destionation_file_name = match url_path.into_iter().last() {
                        Some(a) => a.to_string_lossy(),
                        None => return Err(Error::msg("No filename in path - this is bad!")),
                    };
                    let destination_file_path =
                        format!("{}/images/{}", &work.directory, &destionation_file_name);

                    match reqwest::get(url).await?.error_for_status() {
                        Ok(req) => {
                            let mut new_file =
                                tokio::fs::File::create(destination_file_path).await?;
                            new_file.write_all(&req.bytes().await?).await?;
                        }
                        Err(why) => {
                            return Err(Error::msg(
                                "バンドキャンプからサムネイルのダウンロードが失敗しました",
                            )
                            .context(why));
                        }
                    }

                    Illustration {
                        image: destionation_file_name.to_string(),
                        illustrators: vec![],
                        additional_illustrators: vec![],
                        description: None,
                    }
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

                    let token = workstate.to_working(WorkWorkingData {
                        message_id,
                        data: Box::new(workdata),
                    })?;

                    ctx.data()
                        .database
                        .update_ticket_state(channel_id.into(), &workstate, token)
                        .await?;

                    return Ok(());
                }
                Err(why) => return Err(Error::msg(format!("{ERR_WORK_MUTEX_LOCK}: {why}"))),
            }
        }
        None => return Err(Error::msg(ERR_MSG_ONLY_WORK_CHANNEL)),
    }
}

#[poise::command(slash_command, guild_only)]
pub async fn edit_album(ctx: Context<'_, Data, Error>, album_file: String) -> Result<(), Error> {
    let channel_id = ChannelId::new(ctx.channel_id().get());
    ensure_new_work(ctx.data(), channel_id).await?;

    match ctx.data().work_states.get(&channel_id) {
        Some(work) => {
            let editing_path =
                verify_and_canonicalise_path(&work.directory, PathSection::Album, &album_file)?;
            match work.work_state.try_lock() {
                Ok(mut workstate) => {
                    let mut file_str = String::new();
                    File::open(&editing_path)
                        .await?
                        .read_to_string(&mut file_str);
                    let (front_matter, page_content) = split_file_str(&file_str)?;

                    let album = serde_yaml::from_str::<AlbumMeta>(front_matter)?;

                    if album.sns_links.len() > 8 {
                        return Err(Error::msg("Too many SNS Links to edit!"));
                    }
                    if album.illustrations.len() > 5 {
                        return Err(Error::msg("Too many Illustrations to edit!"));
                    }
                    if album.tracks.len() > 20 {
                        return Err(Error::msg("Too many Tracks to edit!"));
                    }

                    let link = match album.link {
                        Some(l) => ModalWrapper::InitWithValue(MainLink { url: l }),
                        None => ModalWrapper::Unset,
                    };

                    let sns_links = album
                        .sns_links
                        .into_iter()
                        .map(|url| SnsLink { url })
                        .collect();

                    let illustrations = album
                        .illustrations
                        .into_iter()
                        .map(|(title, illustration)| TitleAndIllustraion {
                            title,
                            illustration,
                        })
                        .collect();

                    let tracks = album
                        .tracks
                        .into_iter()
                        .map(|(title, track)| AlbumTitleAndTrack { title, track })
                        .collect();

                    let workdata = WorkData::Album {
                        page: AlbumWorkStatePage::BasicInfo,
                        basic_info: ModalWrapper::InitWithValue(BasicInfo {
                            title: album.title,
                            short_description: album.short.unwrap_or_default(),
                            date: album.date,
                        }),
                        authors: ModalWrapper::InitWithValue(Authors {
                            authors: album.authors,
                            additional_authors: album.additional_authors,
                        }),
                        content: ModalWrapper::InitWithValue(PageContent {
                            content: page_content.to_string(),
                        }),
                        link,
                        sns_links: ModalListWrapper::new(sns_links)?,
                        thumbnail: ModalWrapper::InitWithValue(Thumbnail {
                            illustration: album.thumbnail,
                        }),
                        illustrations: ModalListWrapper::new(illustrations)?,
                        tracks: ModalListWrapper::new(tracks)?,
                    };

                    let msg_components = workdata.create_components(None);

                    let reply = ctx
                        .send(
                            CreateReply::new()
                                .components(msg_components)
                                .flags(MessageFlags::IS_COMPONENTS_V2),
                        )
                        .await?;

                    let message_id = reply.message().await?.id;

                    let token = workstate.to_working(WorkWorkingData {
                        message_id,
                        data: Box::new(workdata),
                    })?;

                    ctx.data()
                        .database
                        .update_ticket_state(channel_id.into(), &workstate, token)
                        .await?;

                    return Ok(());
                }
                Err(why) => return Err(Error::msg(format!("{ERR_WORK_MUTEX_LOCK}: {why}"))),
            }
        }
        None => return Err(Error::msg(ERR_MSG_ONLY_WORK_CHANNEL)),
    }
}

#[poise::command(slash_command, guild_only)]
pub async fn delete_album(ctx: Context<'_, Data, Error>, album_file: String) -> Result<(), Error> {
    let channel_id = ChannelId::new(ctx.channel_id().get());

    match ctx.data().work_states.get(&channel_id) {
        Some(work) => {
            let file_path =
                verify_and_canonicalise_path(&work.directory, PathSection::Album, &album_file)?;
            remove_file(&file_path).await?;
            ctx.reply("削除成功").await?;
        }
        None => return Err(Error::msg(ERR_MSG_ONLY_WORK_CHANNEL)),
    }
    Ok(())
}
