use anyhow::Error;
use camino::Utf8PathBuf;
use dashmap::DashMap;
use octocrab::{Octocrab, models::PullRequestId};
use poise::serenity_prelude::{ChannelId, GenericChannelId, MessageId, ThreadId};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    bot::{sql::Sql, work::WorkData},
    site::namemap::NameMap,
};
mod commands;
mod editcmd;
mod event;
mod git_actions;
mod preview;
mod sql;
mod ticket;
pub mod utils;
mod work;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GitConfig {
    pub repo_url: String,
    pub owner: String,
    pub repo: String,
    pub main_branch: String,
    pub ssh_pubkey_file: PathBuf,
    pub ssh_privkey_file: PathBuf,
    pub github_token_file: PathBuf,
    pub git_name: String,
    pub git_email: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub server_id: u64,
    pub ping_on_pr_role: Vec<u64>,
    pub admin_role: Vec<u64>,
    pub ticket_channel: Option<u64>,
    pub site_prefix: String, // where the working dirs for each channel live
    pub zvezdochka_cmd: String,
    pub public_url_base: String,

    pub git: GitConfig,
}

pub struct Secrets {
    pub octocrab: Octocrab,
    pub git_public_key: String,
    pub git_private_key: String,
}

pub struct Work {
    pub directory: Utf8PathBuf,
    pub name_map: NameMap,
    pub work_state: Arc<Mutex<WorkState>>,
}

#[must_use]
pub struct StateChangeToken {
    old_state: WorkStateInt,
}

impl StateChangeToken {
    fn new(old: WorkStateInt) -> Self {
        Self { old_state: old }
    }

    pub fn old_state(&self) -> WorkStateInt {
        self.old_state
    }
}

pub enum WorkState {
    Initializing,
    Idle,
    Working(WorkWorkingData),
    Closed,
    Submitted(PullRequestId),
}

impl WorkState {
    pub fn state_int(&self) -> WorkStateInt {
        self.into()
    }

    #[must_use]
    pub fn complete_init(&mut self) -> Result<StateChangeToken, Error> {
        match self {
            WorkState::Initializing => {
                let old_state = self.state_int();
                *self = WorkState::Idle;
                return Ok(StateChangeToken::new(old_state));
            }
            _ => {
                return Err(Error::msg("（不正事態）この作業はまだ準備中であります。"));
            }
        }
    }

    #[must_use]
    pub fn to_idle(&mut self) -> Result<StateChangeToken, Error> {
        match self {
            WorkState::Initializing => {
                return Err(Error::msg("（不正事態）この作業はまだ準備中であります。"));
            }
            WorkState::Idle | WorkState::Working(_) => {
                let old_state = self.state_int();
                *self = WorkState::Idle;
                return Ok(StateChangeToken::new(old_state));
            }
            WorkState::Closed | WorkState::Submitted(_) => {
                return Err(Error::msg("（不正事態）この作業は終了であります。"));
            }
        }
    }

    #[must_use]
    pub fn to_working(&mut self, working_data: WorkWorkingData) -> Result<StateChangeToken, Error> {
        match self {
            WorkState::Initializing => {
                return Err(Error::msg("（不正事態）この作業はまだ準備中であります。"));
            }
            WorkState::Idle => {
                let old_state = self.state_int();
                *self = WorkState::Working(working_data);
                return Ok(StateChangeToken::new(old_state));
            }
            WorkState::Working(_) => {
                return Err(Error::msg("今進行中作業があります、不正事態。"));
            }
            WorkState::Closed | WorkState::Submitted(_) => {
                return Err(Error::msg("（不正事態）この作業は終了であります。"));
            }
        }
    }

    #[must_use]
    pub fn to_close(&mut self) -> Result<StateChangeToken, Error> {
        match self {
            WorkState::Initializing => {
                return Err(Error::msg("（不正事態）この作業はまだ準備中であります。"));
            }
            WorkState::Submitted(_) | WorkState::Closed => {
                return Err(Error::msg("（不正事態）この作業は終了であります。"));
            }
            WorkState::Working(_) => {
                return Err(Error::msg("今進行中作業があります、不正事態。"));
            }
            _ => {
                let old_state = self.state_int();
                *self = WorkState::Closed;
                return Ok(StateChangeToken::new(old_state));
            }
        }
    }

    #[must_use]
    pub fn to_submitted(&mut self, pr_number: PullRequestId) -> Result<StateChangeToken, Error> {
        match self {
            WorkState::Initializing => {
                return Err(Error::msg("（不正事態）この作業はまだ準備中であります。"));
            }
            WorkState::Closed | WorkState::Submitted(_) => {
                return Err(Error::msg("（不正事態）この作業は終了であります。"));
            }
            WorkState::Working(_) => {
                return Err(Error::msg("今進行中作業があります、不正事態。"));
            }
            WorkState::Idle => {
                let old_state = self.state_int();
                *self = WorkState::Submitted(pr_number);
                return Ok(StateChangeToken::new(old_state));
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WorkStateInt {
    Initializing,
    Idle,
    Working,
    Closed,
    Submitted,
}

impl WorkStateInt {
    pub const INITIALISING: i32 = 0;
    pub const IDLE: i32 = 1;
    pub const WORKING: i32 = 2;
    pub const CLOSED: i32 = 3;
    pub const SUBMITTED: i32 = 4;

    pub fn to_state_int(&self) -> i32 {
        match self {
            WorkStateInt::Initializing => Self::INITIALISING,
            WorkStateInt::Idle => Self::IDLE,
            WorkStateInt::Working => Self::WORKING,
            WorkStateInt::Closed => Self::CLOSED,
            WorkStateInt::Submitted => Self::SUBMITTED,
        }
    }
}

impl From<WorkState> for WorkStateInt {
    fn from(value: WorkState) -> Self {
        match value {
            WorkState::Initializing => WorkStateInt::Initializing,
            WorkState::Idle => WorkStateInt::Idle,
            WorkState::Working(_) => WorkStateInt::Working,
            WorkState::Closed => WorkStateInt::Closed,
            WorkState::Submitted(_) => WorkStateInt::Submitted,
        }
    }
}
impl From<&WorkState> for WorkStateInt {
    fn from(value: &WorkState) -> Self {
        match value {
            WorkState::Initializing => WorkStateInt::Initializing,
            WorkState::Idle => WorkStateInt::Idle,
            WorkState::Working(_) => WorkStateInt::Working,
            WorkState::Closed => WorkStateInt::Closed,
            WorkState::Submitted(_) => WorkStateInt::Submitted,
        }
    }
}

impl TryFrom<i32> for WorkStateInt {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let wsi = match value {
            Self::INITIALISING => Self::Initializing,
            Self::IDLE => Self::Idle,
            Self::WORKING => Self::Working,
            Self::CLOSED => Self::Closed,
            Self::SUBMITTED => Self::Submitted,
            _ => return Err(Error::msg("bad state int > 5 or < 0")),
        };
        Ok(wsi)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TicketId(pub i64);

impl TicketId {
    pub fn get(&self) -> i64 {
        self.0
    }
}

impl From<ChannelId> for TicketId {
    fn from(value: ChannelId) -> Self {
        TicketId(value.get() as i64)
    }
}

impl From<GenericChannelId> for TicketId {
    fn from(value: GenericChannelId) -> Self {
        TicketId(value.get() as i64)
    }
}

impl From<ThreadId> for TicketId {
    fn from(value: ThreadId) -> Self {
        TicketId(value.get() as i64)
    }
}

impl Into<ChannelId> for TicketId {
    fn into(self) -> ChannelId {
        ChannelId::new(self.get() as u64)
    }
}
impl Into<GenericChannelId> for TicketId {
    fn into(self) -> GenericChannelId {
        GenericChannelId::new(self.get() as u64)
    }
}
impl Into<ThreadId> for TicketId {
    fn into(self) -> ThreadId {
        ThreadId::new(self.get() as u64)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkWorkingData {
    message_id: MessageId,
    data: Box<WorkData>,
}

pub struct Data {
    pub config: Config,
    pub database: Sql,
    pub shhh: Secrets,
    pub work_states: DashMap<ChannelId, Work>,
}

pub async fn start_bot() {}
