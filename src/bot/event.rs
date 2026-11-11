use std::borrow::Cow;
use std::str::FromStr;
use std::sync::Arc;

use crate::bot::Data;
use crate::bot::utils::{ComponentInteractionTypes, TicketComponents};
use anyhow::Error;
use chrono::Utc;
use poise::FrameworkContext;
use poise::serenity_prelude::all::{Context, FullEvent, Interaction};
use poise::serenity_prelude::{
    ChannelId, ChannelType, ComponentInteractionDataKind, CreateMessage, CreateThread,
    GenericChannelId, GuildThread, Http, User, UserId,
};

pub async fn watch_for_open_ticket(
    context: &Context,
    event: &FullEvent,
    framework: FrameworkContext<'_, Arc<Data>, Error>,
    data: &Arc<Data>,
) -> Result<(), Error> {
    match event {
        FullEvent::InteractionCreate { interaction, .. } => match interaction {
            Interaction::Component(component_interaction) => {
                component_interaction.defer(&context.http).await?;
                let component_interaction_type =
                    ComponentInteractionTypes::from_str(&component_interaction.data.custom_id)?;

                match component_interaction_type {
                    ComponentInteractionTypes::Ticket(ticket_components) => match ticket_components
                    {
                        TicketComponents::TicketOpen => {}
                        TicketComponents::TicketClose => todo!(),
                        TicketComponents::TicketSubmit => todo!(),
                    },
                    ComponentInteractionTypes::Singular(singular_edit_components) => todo!(),
                    ComponentInteractionTypes::Multiple(multiple_edit_components) => todo!(),
                    ComponentInteractionTypes::PageTurners(page_turner) => todo!(),
                    ComponentInteractionTypes::EditConfirmCancel(edit_confirm_cancel) => todo!(),
                }
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
        FullEvent::ThreadCreate {
            thread,
            newly_created,
            ..
        } => {
            let newly_created = newly_created.unwrap_or(false);

            if !newly_created {
                return Ok(());
            }

            return Ok(());
        }
        _ => return Ok(()),
    }
}
