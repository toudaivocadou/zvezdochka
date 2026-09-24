use std::time::Duration;

use anyhow::Error;
use poise::{
    Context, execute_modal,
    serenity_prelude::{Action::Timeout, ChannelId},
};
use url::Url;

use crate::bot::{
    Data,
    editcmd::{
        AuthorsModal, BasicInfoModal, MainLinkModal, PageContentModal, SnsLinksModal,
        ThumbnailModal, WorkState, WorkStateData, check_channel_is_work_channel,
    },
};

async fn check_album(ctx: &Context<'_, Data, Error>) -> Result<(), Error> {
    let channel = ctx
        .channel()
        .await
        .ok_or(Error::msg("Must be run in a guild channel"))?;

    let channel_id = ChannelId::new(channel.id().get());

    check_channel_is_work_channel(ctx.data().clone(), channel_id).await?;

    if let Some(chs) = ctx.data().work_states.get(&channel_id) {
        match chs.as_ref() {
            Some(work_state) => {
                if let WorkStateData::Album { .. } = work_state.work_state {
                    return Ok(());
                }
                return Err(Error::msg(
                    "Different workstate! (make sure to cancel/finish existing work)",
                ));
            }
            None => {
                return Err(Error::msg(
                    "Locked! (make sure no one else is using a command right now in this channel)",
                ));
            }
        }
    };
    Ok(())
}

pub async fn album_create(ctx: Context<'_, Data, Error>, url: String) -> Result<(), Error> {
    let album_lock = match ctx
        .data()
        .work_states
        .try_get(&ChannelId::new(ctx.channel_id().get()))
        .try_unwrap()
        .ok_or(Error::msg(
            "Command/work already running. Abort! Abort! Abandon Ship!",
        ))?
        .lock()
    {
        Ok(lock) => lock,
        Err(why) => {
            let _ = ctx
                .data()
                .database
                .set_ticket_state_error(ChannelId::new(ctx.channel_id().get()))
                .await;
            return Err(Error::msg(format!("ロック獲得失敗: {why}")));
        }
    };

    check_album(&ctx).await?;
    if let Context::Application(app_ctx) = ctx {
        let basic_info = execute_modal::<Data, Error, BasicInfoModal>(
            app_ctx,
            None,
            Some(Duration::from_mins(5)),
        )
        .await?
        .ok_or(Error::msg("Modal cannot be empty!"));
        let basic_info = execute_modal::<Data, Error, BasicInfoModal>(
            app_ctx,
            None,
            Some(Duration::from_mins(5)),
        )
        .await?
        .ok_or(Error::msg("Modal cannot be empty, or timed out!"))?;
        let authors =
            execute_modal::<Data, Error, AuthorsModal>(app_ctx, None, Some(Duration::from_mins(5)))
                .await?
                .ok_or(Error::msg("Modal cannot be empty, or timed out!"))?;
        let content = execute_modal::<Data, Error, PageContentModal>(
            app_ctx,
            None,
            Some(Duration::from_mins(5)),
        )
        .await?
        .ok_or(Error::msg("Modal cannot be empty, or timed out!"))?;
        let link = execute_modal::<Data, Error, MainLinkModal>(
            app_ctx,
            None,
            Some(Duration::from_mins(5)),
        )
        .await?
        .ok_or(Error::msg("Modal cannot be empty, or timed out!"))?;
        let sns_links = execute_modal::<Data, Error, SnsLinksModal>(
            app_ctx,
            None,
            Some(Duration::from_mins(5)),
        )
        .await?
        .ok_or(Error::msg("Modal cannot be empty, or timed out!"))?;
        let thumbnail = execute_modal::<Data, Error, ThumbnailModal>(
            app_ctx,
            None,
            Some(Duration::from_mins(5)),
        )
        .await?
        .ok_or(Error::msg("Modal cannot be empty, or timed out!"))?;

        let album_workstate = WorkState {
            current_work_message: todo!(),
            work_state: todo!(),
        };
    } else {
        return Err(Error::msg("This is a bug!"));
    }
    return Ok(());
}

pub async fn album_import(ctx: Context<'_, Data, Error>, url: String) -> Result<(), Error> {
    ctx.defer().await?;
    let parsed_url = Url::parse(&url)?;
    let domain = match parsed_url.domain() {
        Some(u) => u,
        None => return Err(Error::msg("Bad URL.")),
    };

    if !domain.ends_with("bandcamp.com") {
        return Err(Error::msg("bandcampしか処理できません。"));
    };

    let album = bandcamp::album_from_url(parsed_url.as_str())
        .await
        .map_err(|why| Error::msg("bandcampアルバム読む込みが失敗しました: {why}"))?;
}

pub fn album_edit() {}

pub fn write_album_to_disk() {}

pub fn album_by_bandcamp() {}

pub fn album_by_youtube_playlist() {}
