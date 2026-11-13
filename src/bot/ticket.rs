use crate::bot::editcmd::common::construct_name_map;
use crate::bot::git_actions::{CreatedGit, clone_git};
use crate::bot::utils::ticket_work_path;
use crate::bot::{Data, TicketId, Work, WorkState};
use crate::site::buildsite;
use crate::site::namemap::NameMap;
use anyhow::Error;
use flume::unbounded;
use indexmap::IndexMap;
use poise::serenity_prelude::{
    ChannelType, CreateThread, EditThread, GenericChannelId, GuildThread, User,
};
use poise::serenity_prelude::{CreateMessage, Http};
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;

pub async fn create_new_ticket_channel(
    data: Arc<Data>,
    http: Arc<Http>,
    channel: GenericChannelId,
    user: User,
) -> Result<GuildThread, Error> {
    let ticket = TicketId::from(channel);
    let path = ticket_work_path(data.clone(), ticket);

    let this_ticket_internal_id = data.database.new_ticket(ticket, user.id, &path).await?;

    data.work_states.insert(
        ticket,
        Work {
            directory: path,
            work_state: Arc::new(Mutex::new(WorkState::Initializing)),
            name_map: NameMap {
                members: IndexMap::default(),
            },
        },
    );

    let guild_channel = http
        .get_channel(channel)
        .await?
        .guild()
        .ok_or(Error::msg("このボタンはサーバー限定であります"))?;

    let new_thread_name = format!("{}-{}", &user.name, this_ticket_internal_id);

    let created_thread = guild_channel
        .id
        .create_thread(
            &http,
            CreateThread::new(new_thread_name)
                .invitable(true)
                .audit_log_reason(Cow::Borrowed("新しい作業"))
                .kind(ChannelType::PrivateThread),
        )
        .await?;

    created_thread.id.add_thread_member(&http, user.id).await?;

    created_thread
        .send_message(
            &http,
            CreateMessage::new().content(Cow::Borrowed("作業環境を準備中")),
        )
        .await?;

    Ok(created_thread)
}

pub async fn clone_git_to_channel(
    data: Arc<Data>,
    http: Arc<Http>,
    thread: GuildThread,
) -> Result<CreatedGit, Error> {
    let (send, recv) = unbounded();

    let git_data = data.clone();
    let ticket_id: TicketId = thread.id.into();
    let git_repo = spawn_blocking(move || clone_git(git_data, ticket_id, Arc::new(send))).await??;

    let name_map = construct_name_map(&git_repo.path).await?;

    let state_change_token = match data.work_states.get_mut(&ticket_id) {
        Some(mut state) => {
            state.name_map = name_map;
            state.work_state.lock().await.to_idle()?
        }
        None => {
            return Err(Error::msg(
                "この作業は存在していません！ (clone git to channel)",
            ));
        }
    };

    data.database
        .initalise_ticket(ticket_id, &git_repo.commit_sha, state_change_token)
        .await?;

    match recv.drain().last() {
        Some(progress) => {
            thread
                .send_message(
                    &http,
                    CreateMessage::new().content(format!("{progress}\n複製完了。編集する準備完了")),
                )
                .await?;
        }
        None => {
            thread
                .send_message(
                    &http,
                    CreateMessage::new().content(format!("複製完了。編集する準備完了")),
                )
                .await?;
        }
    }

    Ok(git_repo)
}

pub async fn close_site_channel(
    data: Arc<Data>,
    http: Arc<Http>,
    mut thread: GuildThread,
) -> Result<(), Error> {
    let ticket_id: TicketId = thread.id.into();

    match data.work_states.get_mut(&ticket_id) {
        Some(state) => {
            let mut workstate = state.work_state.lock().await;
            let token = workstate.to_close()?;
            data.database
                .update_ticket_state(ticket_id, &workstate, token)
                .await?;
        }
        None => return Err(Error::msg("作業終了が失敗しました: 作業は存在していません")),
    };

    thread
        .send_message(
            &http,
            CreateMessage::new().content(Cow::Borrowed("作業を閉めます。")),
        )
        .await?;

    thread
        .edit(
            &http,
            EditThread::new()
                .locked(true)
                .archived(true)
                .audit_log_reason(Cow::Borrowed("使用者が作業をやめました。")),
        )
        .await?;

    Ok(())
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
