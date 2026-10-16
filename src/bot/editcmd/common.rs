use std::path::PathBuf;

use anyhow::Error;
use camino::{Utf8Path, Utf8PathBuf};
use glob::{GlobError, glob};
use indexmap::IndexMap;
use tokio::io::AsyncReadExt;

use crate::site::{FRONT_MATTER_SPLIT, MEMBER_GLOB, member::MemberMeta, namemap::NameMap};

pub const ERR_MSG_ONLY_WORK_CHANNEL: &'static str =
    "このコマンドは作業チャネルでのみ実行ができます。";
pub const ERR_WORK_MUTEX_LOCK: &'static str =
    "作業内容の複数読み・書きはできません。先のコマンドの実行の完了まで少々お待ち下さい。";
pub const ERR_FILE_BAD: &'static str = "このファイルは開けません。";

pub async fn construct_name_map(working_directory: Utf8PathBuf) -> Result<NameMap, anyhow::Error> {
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

pub fn split_file_str(file_str: &str) -> Result<(&str, &str), anyhow::Error> {
    let (_, second) = file_str
        .split_once(FRONT_MATTER_SPLIT)
        .ok_or(Error::msg(ERR_FILE_BAD))?;
    let (front_matter, content) = second
        .split_once(FRONT_MATTER_SPLIT)
        .ok_or(Error::msg(ERR_FILE_BAD))?;
    Ok((front_matter, content))
}

pub fn glob_files(base_dir: &Utf8Path, glob_str: &str) -> Result<Vec<PathBuf>, anyhow::Error> {
    let pattern = format!("{}/{}", base_dir, glob_str);
    let files = glob(&pattern)?.collect::<Result<Vec<PathBuf>, GlobError>>();
    files.map_err(|why| Error::msg(why.to_string()))
}
