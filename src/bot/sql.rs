use std::str::FromStr;

use anyhow::Error;
use camino::{Utf8Path, Utf8PathBuf};
use chrono::Utc;
use octocrab::models::PullRequestId;
use poise::serenity_prelude::UserId;
use sqlx::{SqlitePool, query};

use crate::bot::{StateChangeToken, TicketId, WorkState, WorkStateInt, WorkWorkingData};

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

    pub async fn new_ticket(
        &self,
        channel_id: TicketId,
        user: UserId,
        path: &Utf8Path,
    ) -> Result<i64, Error> {
        let channel_id_i64 = channel_id.get() as i64;
        let mut txn = self.database.begin().await?;

        if let Some(_) = query!(
            "SELECT * FROM tickets WHERE ticket_channel_id = $1 LIMIT 1",
            channel_id_i64
        )
        .fetch_optional(&mut *txn)
        .await?
        {
            return Err(Error::msg("この作業はもう存在であります。"));
        }

        let user_id_i64 = user.get() as i64;
        let time = Utc::now().timestamp();
        let open = WorkState::Initializing.state_int().to_state_int();
        let result = query!(
            "INSERT INTO tickets(ticket_channel_id, created_by, created_on, state, path) VALUES ($1, $2, $3, $4, $5)",
            channel_id_i64,
            user_id_i64,
            time,
            open,
            &path.to_string()
        ).execute(&mut *txn).await?;

        txn.commit().await?;

        return Ok(result.last_insert_rowid());
    }

    pub async fn is_channel_id_a_ticket_channel(
        &self,
        channel_id: TicketId,
    ) -> Result<bool, Error> {
        let channel = channel_id.get() as i64;
        let count = query!(
            "SELECT rowid FROM tickets WHERE ticket_channel_id = $1 LIMIT 1",
            channel
        )
        .fetch_optional(&self.database)
        .await?
        .is_some();

        return Ok(count);
    }

    pub async fn initalise_ticket(
        &self,
        ticket: TicketId,
        commit: &[u8],
        change_token: StateChangeToken,
    ) -> Result<(), Error> {
        self.is_channel_id_a_ticket_channel(ticket).await?;

        let idle_state = WorkState::Idle.state_int().to_state_int();
        if change_token.old_state() != WorkState::Initializing.state_int() {
            return Err(Error::msg("Bad state!"));
        }
        let channel = ticket.get() as i64;

        let _ = query!(
            "
        UPDATE tickets SET state = $1, start_commit = $2 WHERE ticket_channel_id = $4
            ",
            idle_state,
            commit,
            channel
        )
        .execute(&self.database)
        .await?;

        Ok(())
    }

    pub async fn update_ticket_state(
        &self,
        ticket: TicketId,
        work_state: &WorkState,
        _change_token: StateChangeToken,
    ) -> Result<(), Error> {
        self.is_channel_id_a_ticket_channel(ticket).await?;

        let mut txn = self.database.begin().await?;

        let work_state_int = work_state.state_int().to_state_int();
        let channel = ticket.get() as i64;

        let _ = query!(
            "
        UPDATE tickets SET state = $1 WHERE ticket_channel_id = $2
            ",
            work_state_int,
            channel
        )
        .execute(&mut *txn)
        .await?;

        let _ = query!(
            "DELETE FROM tickets_checkpoint WHERE ticket_channel_id = $1",
            channel
        )
        .execute(&mut *txn)
        .await?;

        let time = Utc::now().timestamp();

        match work_state {
            WorkState::Working(work_working_data) => {
                let json = serde_sqlite_jsonb::to_vec(work_working_data)?;
                let _ = query!(
                    "INSERT INTO tickets_checkpoint VALUES ($1, $2)",
                    channel,
                    json
                )
                .execute(&mut *txn)
                .await?;

                let _ = query!(
                    "UPDATE tickets SET closed_on = $1 WHERE ticket_channel_id = $2",
                    time,
                    channel
                )
                .execute(&mut *txn)
                .await?;
            }
            WorkState::Submitted(pr_number) => {
                let _ = query!(
                    "INSERT INTO tickets_submitted VALUES ($1, $2)",
                    channel,
                    pr_number.0 as i64
                )
                .execute(&mut *txn)
                .await?;

                let _ = query!(
                    "UPDATE tickets SET closed_on = $1 WHERE ticket_channel_id = $2",
                    time,
                    channel
                )
                .execute(&mut *txn)
                .await?;
            }
            _ => {}
        }

        txn.commit().await?;

        Ok(())
    }

    pub async fn tickets_by_state(&self, state: WorkStateInt) -> Result<Vec<TicketId>, Error> {
        Ok(query!(
            "SELECT ticket_channel_id FROM tickets WHERE state = $1",
            &state.to_state_int()
        )
        .fetch_all(&self.database)
        .await?
        .into_iter()
        .map(|rec| TicketId(rec.ticket_channel_id))
        .collect())
    }

    pub async fn ticket_by_id(&self, id: TicketId) -> Result<(Utf8PathBuf, WorkState), Error> {
        let channel_id = id.get() as i64;
        let mut txn = self.database.begin().await?;

        let ticket = query!(
            "SELECT state, path FROM tickets WHERE ticket_channel_id = $1",
            channel_id
        )
        .fetch_one(&mut *txn)
        .await?;

        let state = WorkStateInt::try_from(ticket.state as i32)?;
        let work_state = match state {
            WorkStateInt::Initializing => WorkState::Initializing,
            WorkStateInt::Idle => WorkState::Idle,
            WorkStateInt::Working => {
                let state_data = query!(
                    "SELECT work_state_json FROM tickets_checkpoint WHERE ticket_channel_id = $1",
                    channel_id
                )
                .fetch_one(&mut *txn)
                .await?
                .work_state_json;
                let state_deser = serde_sqlite_jsonb::from_slice::<WorkWorkingData>(&state_data)?;
                WorkState::Working(state_deser)
            }
            WorkStateInt::Closed => WorkState::Closed,
            WorkStateInt::Submitted => {
                let pr_number = query!(
                    "SELECT pr_number FROM tickets_submitted WHERE ticket_channel_id = $1",
                    channel_id
                )
                .fetch_one(&mut *txn)
                .await?
                .pr_number;
                WorkState::Submitted(PullRequestId(pr_number as u64))
            }
        };

        let path = Utf8PathBuf::from_str(&ticket.path)?;

        txn.commit().await?;

        Ok((path, work_state))
    }
}
