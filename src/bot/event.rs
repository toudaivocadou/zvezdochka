use std::sync::Arc;

use anyhow::Error;
use poise::FrameworkContext;
use poise::serenity_prelude::all::{ComponentInteraction, Context, FullEvent, Interaction};
use poise::serenity_prelude::{CacheHttp, CreateMessage, EditMessage};

use crate::bot::Data;
use crate::bot::ticket::{run_site_build, send_message_in_channel_and_prepare_git};

pub const TICKET_OPEN_BUTTON_INTERACTION_ID: &'static str = "ZVEZDOCHKA_OPEN_TICKET";
pub const TICKET_COMPONENT_PREFIX: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT";
pub const ZVEZDOCHKA_TICKET_COMPONENT_BASICINFO: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_BASICINFO";
pub const ZVEZDOCHKA_TICKET_COMPONENT_AUTHORS: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_AUTHORS";
pub const ZVEZDOCHKA_TICKET_COMPONENT_CONTENT: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_CONTENT";
pub const ZVEZDOCHKA_TICKET_COMPONENT_LINK: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_LINK";
pub const ZVEZDOCHKA_TICKET_COMPONENT_THUMBNAIL: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_THUMBNAIL";
pub const ZVEZDOCHKA_TICKET_COMPONENT_ARTISTINFO: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ARTISTINFO";
pub const ZVEZDOCHKA_TICKET_COMPONENT_EXT_ARTISTINFO: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_EXT_ARTISTINFO";
pub const ZVEZDOCHKA_TICKET_COMPONENT_DURATION: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_DURATION";

// SNS Links
pub const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_1: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_1";
pub const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_2: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_2";
pub const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_3: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_3";
pub const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_4: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_4";
pub const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_5: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_5";
pub const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_6: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_6";
pub const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_7: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_7";
pub const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_8: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_8";
pub const ZVEZDOCHKA_TICKET_COMPONENT_DELETE_SNSLINK: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_DELETE_SNSLINK";
pub const ZVEZDOCHKA_TICKET_COMPONENT_NEW_SNSLINK: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_NEW_SNSLINK";

// illustrations
pub const ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_1: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_1";
pub const ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_2: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_2";
pub const ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_3: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_3";
pub const ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_4: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_4";
pub const ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_5: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_5";
pub const ZVEZDOCHKA_TICKET_COMPONENT_NEW_ILLUSTRATION: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_NEW_ILLUSTRATION";
pub const ZVEZDOCHKA_TICKET_COMPONENT_DELETE_ILLUSTRATION: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_DELETE_ILLUSTRATION";
// tracks
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_1: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_1";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_2: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_2";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_3: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_3";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_4: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_4";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_5: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_5";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_6: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_6";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_7: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_7";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_8: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_8";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_9: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_9";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_10: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_10";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_11: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_11";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_12: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_12";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_13: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_13";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_14: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_14";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_15: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_15";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_16: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_16";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_17: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_17";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_18: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_18";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_19: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_19";
pub const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_20: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_20";
pub const ZVEZDOCHKA_TICKET_COMPONENT_DELETE_TRACK: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_DELETE_TRACK";
pub const ZVEZDOCHKA_TICKET_COMPONENT_NEW_TRACK: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_NEW_TRACK";
// cancel + confirm
pub const ZVEZDOCHKA_TICKET_COMPONENT_CANCEL_WORK: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_CANCEL_WORK";
pub const ZVEZDOCHKA_TICKET_COMPONENT_CONFIRM_WORK: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_CONFIRM_WORK";
pub const ZVEZDOCHKA_TICKET_COMPONENT_ALBUM_INFOPAGE: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ALBUM_INFOPAGE";
pub const ZVEZDOCHKA_TICKET_COMPONENT_ALBUM_TRACKSPAGE: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ALBUM_TRACKSPAGE";

pub async fn watch_for_open_ticket(
    context: &Context,
    event: &FullEvent,
    framework: FrameworkContext<'_, Arc<Data>, Error>,
    data: &Arc<Data>,
) -> Result<(), Error> {
    match event {
        FullEvent::InteractionCreate { interaction, .. } => match interaction {
            Interaction::Component(component_interaction) => {
                if component_interaction.data.custom_id == TICKET_OPEN_INTERACTION {}
            }
            Interaction::Modal(modal_interaction) => {
                if modal_interaction
                    .data
                    .custom_id
                    .starts_with(TICKET_COMPONENT_PREFIX)
                {}
            }
            _ => return Ok(()),
        },
        FullEvent::ChannelCreate { channel, .. } => {
            if data
                .database
                .is_channel_id_a_ticket_channel(channel.id)
                .await?
            {
                send_message_in_channel_and_prepare_git(data.clone(), context, channel).await?;
                let message = channel
                    .send_message(
                        context.http(),
                        CreateMessage::new().content("サイトプレビューをけんちくしています。"),
                    )
                    .await?;
                run_site_build(data.clone(), channel.id.get()).await?;
                let preview_site_url =
                    format!("{}/{}", &data.config.public_url_base, channel.id.get());
                message
                    .edit(
                        context.http(),
                        EditMessage::new()
                            .content(format!("プレビュー準備完了: {}", preview_site_url)),
                    )
                    .await?;
            }
            return Ok(());
        }
        _ => return Ok(()),
    }
}
