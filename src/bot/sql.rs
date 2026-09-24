use anyhow::Error;
use chrono::Utc;
use poise::serenity_prelude::{ChannelId, UserId};
use sqlx::{SqlitePool, query};

use crate::bot::ticket::Status;

pub struct Sql {
    database: SqlitePool,
}

impl Sql {
    pub async fn user_link_by_ascii(&self, ascii_name: &str) -> Result<Option<UserId>, Error> {
        let data = query!(
            "SELECT (discord_id) FROM discord_to_artist WHERE artist_ascii_name = $1 LIMIT 1",
            ascii_name
        )
        .fetch_optional(&self.database)
        .await?;

        return Ok(data.map(|dsc| UserId::new(dsc.discord_id as u64)));
    }

    pub async fn user_link_by_discord(&self, discord_id: UserId) -> Result<Option<String>, Error> {
        let discord_id = discord_id.get() as i64;
        let data = query!(
            "SELECT (artist_ascii_name) FROM discord_to_artist WHERE discord_id = $1 LIMIT 1",
            discord_id
        )
        .fetch_optional(&self.database)
        .await?;

        return Ok(data.map(|name| name.artist_ascii_name));
    }

    pub async fn new_user_link(
        &self,
        new_discord_id: UserId,
        new_ascii_name: &str,
    ) -> Result<(), Error> {
        let new_discord_id = new_discord_id.get() as i64;
        let _ = query!(
            "INSERT INTO discord_to_artist VALUES ($1, $2)",
            new_discord_id,
            new_ascii_name
        )
        .execute(&self.database)
        .await?;

        return Ok(());
    }

    pub async fn update_user_discord_id(
        &self,
        new_discord_id: UserId,
        existing_ascii_name: &str,
    ) -> Result<(), Error> {
        let new_discord_id = new_discord_id.get() as i64;
        let _ = query!(
            "UPDATE discord_to_artist SET discord_id = $1 WHERE artist_ascii_name = $2",
            new_discord_id,
            existing_ascii_name
        )
        .execute(&self.database)
        .await?;

        Ok(())
    }

    pub async fn update_user_artist_ascii_name(
        &self,
        existing_discord_id: UserId,
        new_ascii_name: &str,
    ) -> Result<(), Error> {
        let existing_discord_id = existing_discord_id.get() as i64;
        let _ = query!(
            "UPDATE discord_to_artist SET artist_ascii_name = $1 WHERE discord_id = $2",
            new_ascii_name,
            existing_discord_id
        )
        .execute(&self.database)
        .await?;

        Ok(())
    }

    pub async fn delete_user_by_discord_id(&self, discord_id: UserId) -> Result<(), Error> {
        let discord_id = discord_id.get() as i64;
        let _ = query!(
            "DELETE FROM discord_to_artist WHERE discord_id = $1",
            discord_id
        )
        .execute(&self.database)
        .await?;

        Ok(())
    }

    pub async fn delete_user_by_ascii_name(&self, ascii_name: &str) -> Result<(), Error> {
        let _ = query!(
            "DELETE FROM discord_to_artist WHERE artist_ascii_name = $1",
            ascii_name
        )
        .execute(&self.database)
        .await?;

        Ok(())
    }

    // pub async fn tickets(&self) -> Result<Vec<(id, )>, Error> {}
    pub async fn new_ticket(&self, channel_id: ChannelId, user: UserId) -> Result<(), Error> {
        let channel_id_i64 = channel_id.get() as i64;
        let user_id_i64 = user.get() as i64;
        let now = Utc::now().timestamp();
        let open = Status::Initalizing.to_string();
        let _ = query!(
            "INSERT INTO tickets(ticket_channel_id, created_by, created_on_tsec, status) VALUES ($1, $2, $3, $4)",
            channel_id_i64,
            user_id_i64,
            now,
            open
        ).execute(&self.database).await?;

        return Ok(());
    }

    pub async fn is_channel_id_a_ticket_channel(
        &self,
        channel_id: ChannelId,
    ) -> Result<bool, Error> {
        let channel = channel_id.get() as i64;
        let count = query!(
            "SELECT * FROM tickets WHERE ticket_channel_id = $1 LIMIT 1",
            channel
        )
        .fetch_optional(&self.database)
        .await?
        .is_some();

        return Ok(count);
    }

    pub async fn update_ticket_after_init(
        &self,
        channel_id: ChannelId,
        commit: &[u8],
        path: &str,
    ) -> Result<(), Error> {
        let channel = channel_id.get() as i64;
        let open = Status::Open.to_string();
        let _ = query!("UPDATE tickets SET start_commit = $1, path = $2, status = $3 WHERE ticket_channel_id = $4", commit, path, open, channel).execute(&self.database).await?;
        Ok(())
    }

    pub async fn set_ticket_state_error(&self, channel_id: ChannelId) -> Result<(), Error> {
        let channel = channel_id.get() as i64;
        let open = Status::Error.to_string();
        let _ = query!(
            "UPDATE tickets SET status = $1 WHERE ticket_channel_id = $2",
            open,
            channel
        )
        .execute(&self.database)
        .await?;
        Ok(())
    }
}
