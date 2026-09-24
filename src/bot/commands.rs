use crate::bot::{Data, event::TICKET_OPEN_BUTTON_INTERACTION_ID};
use anyhow::Error;
use poise::{
    Context, CreateReply,
    serenity_prelude::{
        CreateActionRow, CreateButton, CreateComponent, CreateContainer, CreateContainerComponent,
        CreateMessage, CreateTextDisplay, User,
    },
};

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn set_user_sitename(
    ctx: Context<'_, Data, Error>,
    #[description = "(必須) 活動名（英語字幕のみ）"] ascii_name: String,
    #[description = "(必須) discordアカウント"] user: User,
    #[description = "(任意) 重ね書き"] overwrite: Option<bool>,
) -> Result<(), Error> {
    if !ascii_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(Error::msg(
            "`ascii_name` フィールドで英語字幕しか入れません。",
        ));
    }

    if ascii_name.is_empty() {
        return Err(Error::msg("`ascii_name`　フィールドは必須です。"));
    }

    let existing_userid = ctx.data().database.user_link_by_ascii(&ascii_name).await?;
    let existing_asciiname = ctx.data().database.user_link_by_discord(user.id).await?;

    let overwrite = match overwrite {
        Some(ow) => ow,
        None => false,
    };

    if existing_userid.is_some() && !overwrite {
        return Err(Error::msg(
            "このdiscordユーザーはもう登録されています。重ね書きほしい場合、`overwrite`を`true`で設定して、もう一度実行をおねがいします。",
        ));
    }

    if existing_asciiname.is_some() && !overwrite {
        return Err(Error::msg(
            "このsitenameはもう登録されています。重ね書きほしい場合、`overwrite`を`true`で設定して、もう一度実行をおねがいします。",
        ));
    }

    ctx.data()
        .database
        .new_user_link(user.id, &ascii_name)
        .await?;

    ctx.reply("登録成功。").await?;

    Ok(())
}

#[poise::command(slash_command, guild_only, required_permissions = "ADMINISTRATOR")]
pub async fn create_ticket_message(ctx: Context<'_, Data, Error>) -> Result<(), Error> {
    let channel = ctx
        .channel()
        .await
        .ok_or(Error::msg("needs to be run in a channel!"))?
        .guild()
        .ok_or(Error::msg("needs to be run in a guild channel!"))?;

    let button = CreateButton::new(TICKET_OPEN_BUTTON_INTERACTION_ID)
        .label("新しいサイト編集チャネルを開く");

    let container_components = [CreateContainerComponent::ActionRow(
        CreateActionRow::Buttons(Cow::Borrowed(&[button])),
    )];

    let components = [
        CreateComponent::TextDisplay(CreateTextDisplay::new(
            "サイト編集をご希望の場合は下のボタンを押してください。",
        )),
        CreateComponent::Container(CreateContainer::new(&container_components)),
    ];

    channel
        .send_message(ctx.http(), CreateMessage::new().components(&components))
        .await?;
    ctx.send(
        CreateReply::new()
            .ephemeral(true)
            .content("編集開きボタンを作りました。"),
    )
    .await?;
    Ok(())
}

// #[poise::command(slash_command, prefix_command, guild_only)]
// pub async fn site_edits(ctx: Context<'_, Data, Error>) -> Result<(), Error> {
//     // rowid, channel id, created by discord, created by asciiname, created on : commit : status
//     let mut table =
//         Table::new("{:>} {:>} {:>} {:>} {:>} {:<} {:<}").with_heading("現在サイト編集目録");
//     table.add_heading("`UserID = 0`は削除されたユーザーの編集");
//     table.add_heading("```");
//     table.add_row(row!(
//         "ID",
//         "Channel ID",
//         "DSC-User",
//         "asciiname",
//         "Date",
//         "Commit",
//         "Status"
//     ));

// }
