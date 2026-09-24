use anyhow::Error;
use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use chrono::Utc;
use flume::{Receiver, Sender};
use git2::{
    Cred, FetchOptions, IndexAddOption, Oid, PushOptions, RemoteCallbacks, Repository, Signature,
    build::RepoBuilder,
};
use humansize::{FormatSizeOptions, format_size};
use poise::serenity_prelude::{
    CacheHttp, ChannelId, ChannelType, CreateChannel, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage, GuildChannel, GuildId,
    all::{ComponentInteraction, Context},
};
use poise::serenity_prelude::{EditMessage, Http};
use snowboard::tokio;
use std::time::Duration;
use std::{fmt::Display, path::Path, str::FromStr, sync::Arc};

use crate::bot::Data;
use crate::site::buildsite;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    #[default]
    Initalizing,
    Open,
    Closed,
    Error,
    Pushed,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Status::Open => "Open",
            Status::Closed => "Closed",
            Status::Error => "Error",
            Status::Pushed => "Pushed",
            Status::Initalizing => "Init",
        };
        write!(f, "{s}")
    }
}

impl FromStr for Status {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let status = match s {
            "Init" => Self::Initalizing,
            "Open" => Self::Open,
            "Closed" => Self::Closed,
            "Error" => Self::Error,
            "Pushed" => Self::Pushed,
            other => return Err(Error::msg(format!("Invalid Status: {other}"))),
        };
        Ok(status)
    }
}

pub async fn create_new_edit_channel(
    context: &Context,
    data: Arc<Data>,
    interaction: &ComponentInteraction,
) -> Result<(), Error> {
    let category_id = data.config.new_ticket_category;
    let guild = context
        .http
        .get_guild(GuildId::new(data.config.server_id))
        .await?;
    let now = Utc::now().to_rfc3339();
    let created_channel = CreateChannel::new(format!("編集-{}-{}", &interaction.user.name, now))
        .kind(ChannelType::Text)
        .category(ChannelId::new(category_id))
        .execute(context.http(), guild.id)
        .await?;

    data.database
        .new_ticket(created_channel.id, interaction.user.id)
        .await?;

    interaction
        .create_response(
            context.http(),
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .content(format!(
                        "<#{}>に編集用チャネルを作りました。",
                        created_channel.id.get()
                    )),
            ),
        )
        .await?;

    if let Err(_) = created_channel
        .send_message(
            context.http(),
            CreateMessage::new().content(format!(
                "<@{}>新しい編集環境を作っています。少々お待ち下さい。",
                interaction.user.id.get()
            )),
        )
        .await
    {
        created_channel
            .send_message(
                context.http(),
                CreateMessage::new().content(format!(
                    "<@{}>新しい編集環境を作っています。少々お待ち下さい。",
                    interaction.user.id.get()
                )),
            )
            .await?;
    }

    return Ok(());
}

pub async fn send_message_in_channel_and_prepare_git(
    data: Arc<Data>,
    context: &Context,
    channel: &GuildChannel,
) -> Result<(), Error> {
    channel
        .send_message(
            context.http(),
            CreateMessage::new().content("作業環境を準備しています..."),
        )
        .await?;

    let (sender, receiver) = flume::unbounded::<GitCloneProgress>();
    let ch_id = channel.id.clone();
    let ch_id_str = channel.id.to_string();
    let data2 = data.clone();
    let new_dir_join = tokio::task::spawn_blocking(move || {
        create_new_site_dir_and_git_clone(data2, ch_id, ch_id_str, sender)
    });

    let http = context.http.clone();
    let track_progress_join =
        tokio::task::spawn(track_clone_progress(http, channel.id.clone(), receiver));

    let (new_dir, _track_progress) = tokio::join!(new_dir_join, track_progress_join);

    if let Ok(Ok(created)) = new_dir {
        data.database
            .update_ticket_after_init(
                channel.id,
                &created.commit_sha,
                &created.new_repo_clone_path,
            )
            .await?;

        let sha = BASE64_STANDARD_NO_PAD.encode(&created.commit_sha);
        let short_msg = &created.commit_message[..100];

        channel
            .send_message(
                context.http(),
                CreateMessage::new().content(format!(
                    r#"
作業環境成功: `{}`に作りました。
commit `{}`
```
{}
```
"#,
                    created.new_repo_clone_path, sha, short_msg
                )),
            )
            .await?;
    } else {
        data.database.set_ticket_state_error(channel.id).await?;
        channel
            .send_message(
                context.http(),
                CreateMessage::new()
                    .content("エラーが発生しました: `create_new_site_dir_and_git_clone` error!"),
            )
            .await?;
        return Err(Error::msg("create_new_site_dir_and_git_clone error!"));
    }

    Ok(())
}

pub async fn track_clone_progress(
    http: Arc<Http>,
    channel_id: ChannelId,
    receiver: Receiver<GitCloneProgress>,
) -> Result<(), Error> {
    let mut message = CreateMessage::new()
        .content(
            r#"
遠隔リポジトリーから読込中...
複製しています...
Objects: 0/0 (Received/Total)
Indexed Objects: 0
Local Objects: 0
Deltas: 0/0 (Indexed/Total)
Received Bytes: N/A
        "#,
        )
        .execute(&http, channel_id.into())
        .await?;

    let mut timeout_counter = 0;

    while !receiver.is_disconnected() || timeout_counter >= 300 {
        if let Some(item) = receiver.drain().last() {
            let obj_recv = item.recv_objects;
            let obv_total = item.total_objects;
            let obj_idx = item.indexed_objects;
            let obj_local = item.local_objects;
            let delt_idx = item.indexed_deltas;
            let del_total = item.total_deltas;
            let bytes = format_size(item.received_bytes, FormatSizeOptions::default());
            message
                .edit(
                    &http,
                    EditMessage::new().content(format!(
                        r#"
遠隔リポジトリーから読込中...
複製しています...
Objects: {obj_recv}/{obv_total} (Received/Total)
Indexed Objects: {obj_idx}
Local Objects: {obj_local}
Deltas: {delt_idx}/{del_total} (Indexed/Total)
Received Bytes: {bytes}
        "#
                    )),
                )
                .await?;
        }
        timeout_counter += 1;
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    if timeout_counter == 300 {
        CreateMessage::new()
            .content("エラーが発生しました: 追跡スレッドが自動タイムアウトされました。")
            .execute(&http, channel_id.into())
            .await?;
    }

    Ok(())
}

struct CreatedGit {
    pub new_repo_clone_path: String,
    pub commit_sha: Vec<u8>,
    pub commit_message: String,
}

struct GitCloneProgress {
    pub total_objects: usize,
    pub indexed_objects: usize,
    pub recv_objects: usize,
    pub local_objects: usize,
    pub total_deltas: usize,
    pub indexed_deltas: usize,
    pub received_bytes: usize,
}

pub fn create_new_site_dir_and_git_clone(
    data: Arc<Data>,
    channel_id: ChannelId,
    new_branch_name: String,
    progress_watcher: Sender<GitCloneProgress>,
) -> Result<CreatedGit, Error> {
    let new_repo_clone_path = format!("{}/{}", data.config.site_prefix, channel_id.get());
    let mut remote_cb = RemoteCallbacks::new();
    remote_cb.credentials(|_, _, _| {
        Cred::ssh_key_from_memory(
            "zvezdochka",
            Some(&data.shhh.public_key),
            &data.shhh.private_key,
            None,
        )
    });
    remote_cb.transfer_progress(|progress| {
        let _ = progress_watcher.send(GitCloneProgress {
            total_objects: progress.total_objects(),
            indexed_objects: progress.indexed_objects(),
            recv_objects: progress.received_objects(),
            local_objects: progress.local_objects(),
            total_deltas: progress.total_deltas(),
            indexed_deltas: progress.indexed_deltas(),
            received_bytes: progress.received_bytes(),
        });
        true
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(remote_cb);

    let repository = RepoBuilder::new()
        .fetch_options(fetch_options)
        .clone(&data.config.git.repo_url, Path::new(&new_repo_clone_path))?;

    let head_commit = repository.head()?.peel_to_commit()?;
    repository.branch(&new_branch_name, &head_commit, false)?;

    let commit_sha = head_commit.id().as_bytes().to_owned();
    let commit_message = head_commit.message().unwrap_or_default().to_string();

    let created = CreatedGit {
        new_repo_clone_path,
        commit_sha,
        commit_message,
    };
    Ok(created)
}

pub fn stage_and_commit_git(
    data: Arc<Data>,
    repo_path: &str,
    commit_msg: &str,
    username: &str,
) -> Result<Oid, Error> {
    let repository = Repository::open(&Path::new(repo_path))?;

    let mut index = repository.index()?;
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    index.write();

    let head_commit = repository.head()?.peel_to_commit()?;
    let head_tree = repository.head()?.peel_to_tree()?;

    let author = Signature::now(&username, &data.config.git.git_email)?;
    let committer = Signature::now(&data.config.git.git_name, &data.config.git.git_email)?;
    let new_commit = repository.commit(
        Some("HEAD"),
        &author,
        &committer,
        commit_msg,
        &head_tree,
        &[&head_commit],
    )?;

    return Ok(new_commit);
}

fn push_git(
    data: Arc<Data>,
    repo_path: &str,
    reference_update_watcher: Sender<(String, Option<String>)>,
    transfer_watcher: Sender<(usize, usize, usize)>,
) -> Result<(), Error> {
    let repository = Repository::open(&Path::new(repo_path))?;

    let mut remote_callback = RemoteCallbacks::new();
    remote_callback.push_update_reference(|reference, status| {
        let _ =
            reference_update_watcher.send((reference.to_string(), status.map(ToString::to_string)));
        Ok(())
    });
    remote_callback.push_transfer_progress(|current, total, bytes| {
        let _ = transfer_watcher.send((current, total, bytes));
    });
    remote_callback.credentials(|_, _, _| {
        Cred::ssh_key_from_memory(
            "zvezdochka",
            Some(&data.shhh.public_key),
            &data.shhh.private_key,
            None,
        )
    });

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(remote_callback);

    let mut remote = repository.find_remote("origin")?;
    remote.push::<&str>(&[], Some(&mut push_options))?;

    Ok(())
}

async fn create_gh_pr(data: Arc<Data>, title: &str, branch_name: &str) -> Result<u64, Error> {
    let pr_number = data
        .shhh
        .octocrab
        .pulls(&data.config.git.owner, &data.config.git.repo)
        .create(title, branch_name, &data.config.git.main_branch)
        .maintainer_can_modify(Some(true))
        .send()
        .await?
        .number;
    Ok(pr_number)
}

pub async fn run_site_build(data: Arc<Data>, ticket_id: u64) -> Result<(), Error> {
    let branch_path = format!("{}/{}", &data.config.site_prefix, ticket_id);
    tokio::task::spawn_blocking(move || {
        buildsite(
            Some(ticket_id),
            data.config.site_prefix.clone(),
            branch_path.clone(),
            Some(branch_path),
            false,
            false,
            false,
        )
    })
    .await??;
    Ok(())
}
