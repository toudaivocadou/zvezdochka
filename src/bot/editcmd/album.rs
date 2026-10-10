use anyhow::Error;
use camino::Utf8Path;
use glob::{GlobError, glob};
use poise::{Context, serenity_prelude::ChannelId};
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncReadExt};

use crate::{
    bot::{
        Data,
        editcmd::common::{ERR_FILE_BAD, ERR_MSG_ONLY_WORK_CHANNEL, ERR_WORK_MUTEX_LOCK},
    },
    site::{ALBUM_GLOB, FRONT_MATTER_SPLIT, album::AlbumMeta},
};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn list_albums(ctx: Context<'_, Data, Error>, page: Option<u32>) -> Result<(), Error> {
    let page = page.unwrap_or(1);
    if page == 0 {
        ctx.reply("不正引数: 第0ページは存在しません。").await?;
        return Ok(());
    }
    let channel_id = ChannelId::from(ctx.channel_id().get());
    if let Some(work) = ctx.data().work_states.get(&channel_id) {
        match work.try_lock() {
            Ok(work) => {
                let pattern = format!("{}/{}", work.directory, ALBUM_GLOB);
                match glob(&pattern)?
                    .collect::<Result<Vec<PathBuf>, GlobError>>()?
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
            Err(why) => return Err(Error::msg(format!("{ERR_WORK_MUTEX_LOCK}: {why}"))),
        }
    } else {
        return Err(Error::msg(ERR_MSG_ONLY_WORK_CHANNEL));
    }
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn about_album(ctx: Context<'_, Data, Error>, album_file: String) -> Result<(), Error> {
    if album_file.contains("..") {
        ctx.reply("lmao no").await?;
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

    if let Some(work) = ctx.data().work_states.get(&channel_id) {
        match work.try_lock() {
            Ok(work) => {
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
                let (_, second) = file_str
                    .split_once(FRONT_MATTER_SPLIT)
                    .ok_or(Error::msg(ERR_FILE_BAD))?;
                let (front_matter, content) = file_str
                    .split_once(FRONT_MATTER_SPLIT)
                    .ok_or(Error::msg(ERR_FILE_BAD))?;

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
                    album.link.map(|x| x.as_str()).unwrap_or("なし"),
                    album.demonstration.map(|x| x.as_str()).unwrap_or("なし"),
                    album
                        .sns_links
                        .iter()
                        .map(|link| { format!(" - {}\n", link.as_str()) })
                        .collect::<String>(),
                    tracks,
                    album.thumbnail.image,
                    illustrations
                );

                return Ok(());
            }
            Err(why) => return Err(Error::msg(format!("{ERR_WORK_MUTEX_LOCK}: {why}"))),
        }
    } else {
        return Err(Error::msg(ERR_MSG_ONLY_WORK_CHANNEL));
    }
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn new_album() {}

pub async fn from_bandcamp() {}

pub async fn edit_album() {}

pub async fn delete_album() {}
