use std::{path::PathBuf, sync::Arc};

use anyhow::Error;
use camino::Utf8Path;
use glob::{GlobError, glob};
use indexmap::IndexMap;
use poise::{
    Context,
    serenity_prelude::{Attachment, ChannelId},
};
use tokio::io::AsyncReadExt;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{
    bot::{
        Data, WorkState,
        utils::{
            PathSection, check_attachmemt_is_picture_and_under_flimit, verify_and_canonicalise_path,
        },
    },
    site::{FRONT_MATTER_SPLIT, MEMBER_GLOB, member::MemberMeta, namemap::NameMap},
};

pub const ERR_MSG_ONLY_WORK_CHANNEL: &'static str =
    "このコマンドは作業チャネルでのみ実行ができます。";
pub const ERR_WORK_MUTEX_LOCK: &'static str =
    "作業内容の複数読み・書きはできません。先のコマンドの実行の完了まで少々お待ち下さい。";
pub const ERR_FILE_BAD: &'static str = "このファイルは開けません。";

pub async fn construct_name_map(working_directory: &Utf8Path) -> Result<NameMap, Error> {
    let artist_files = glob_files(&working_directory, MEMBER_GLOB)?;

    let mut members = IndexMap::new();

    for file in artist_files {
        let mut file_string = String::new();
        tokio::fs::File::open(&file)
            .await?
            .read_to_string(&mut file_string)
            .await?;

        let (front_matter, _) = split_file_str(&file_string)?;

        let document = serde_yaml::from_str::<MemberMeta>(&front_matter)?;
        members.insert(document.name, document.ascii_name);
    }

    Ok(NameMap { members })
}

pub fn split_file_str(file_str: &str) -> Result<(&str, &str), Error> {
    let (_, second) = file_str
        .split_once(FRONT_MATTER_SPLIT)
        .ok_or(Error::msg(ERR_FILE_BAD))?;
    let (front_matter, content) = second
        .split_once(FRONT_MATTER_SPLIT)
        .ok_or(Error::msg(ERR_FILE_BAD))?;
    Ok((front_matter, content))
}

pub fn glob_files(base_dir: &Utf8Path, glob_str: &str) -> Result<Vec<PathBuf>, Error> {
    let pattern = format!("{}/{}", base_dir, glob_str);
    let files = glob(&pattern)?.collect::<Result<Vec<PathBuf>, GlobError>>();
    files.map_err(|why| Error::msg(why.to_string()))
}

pub async fn ensure_new_work(
    context: Arc<Data>,
    channel_ticket_id: ChannelId,
) -> Result<(), Error> {
    match context.work_states.get(&channel_ticket_id) {
        Some(work) => match &*work.work_state.try_lock()? {
            WorkState::Initializing => return Err(Error::msg("まだ作業を準備しています。")),
            WorkState::Idle => return Ok(()),
            WorkState::Working { .. } => {
                return Err(Error::msg(
                    "現在、まだ未完了の編集があります。編集を完了・キャンセルする後で新しい編集を作成してください。",
                ));
            }
            WorkState::Closed | WorkState::Submitted(_) => {
                return Err(Error::msg("このチャネルはもう終了られました。"));
            }
        },
        None => return Err(Error::msg("このチャネルに作業をみつかりませんでした。")),
    }
}

#[poise::command(slash_command, guild_only)]
pub async fn upload_image(
    ctx: Context<'_, Data, Error>,
    image: Attachment,
    directory: Option<String>,
    overwrite: Option<bool>,
) -> Result<(), Error> {
    ctx.defer().await?;
    let invocation_channel = ChannelId::new(ctx.channel_id().get());
    let overwrite = overwrite.unwrap_or(false);
    match ctx.data().work_states.get(&invocation_channel) {
        Some(work) => {
            check_attachmemt_is_picture_and_under_flimit(&image)?;
            let work_directory = work.directory.as_path();
            let attachment_name = &image.filename;
            let image_name = match directory {
                Some(fpath) => format!("{fpath}/{attachment_name}"),
                None => attachment_name.clone().into_string(),
            };
            let canonical_file_path =
                verify_and_canonicalise_path(work_directory, PathSection::Image, &image_name)?;

            if let Ok(_) = File::open(&canonical_file_path).await {
                if !overwrite {
                    return Err(Error::msg("EOVERWRITE: もうファイルが存在しています。"));
                }
            }

            let mut new_file_opts = File::options()
                .write(true)
                .create(true)
                .open(&canonical_file_path)
                .await?;

            let file_data = image.download().await?;
            new_file_opts.write_all(&file_data).await?;

            ctx.reply(format!(
                "アップロードできました。このファイルの名は`{}`です。",
                image_name
            ))
            .await?;
            return Ok(());
        }
        None => return Err(Error::msg("このチャネルに作業をみつかりませんでした。")),
    }
}
