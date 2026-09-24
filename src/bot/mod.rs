use dashmap::DashMap;
use octocrab::Octocrab;
use poise::serenity_prelude::ChannelId;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::{path::PathBuf, sync::Arc};

use crate::bot::{editcmd::WorkState, sql::Sql};

mod commands;
mod editcmd;
mod event;
mod git_actions;
mod preview;
mod sql;
mod ticket;

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
    pub ping_on_pr_role: Option<u64>,
    pub admin_role: Vec<u64>,
    pub new_ticket_category: u64,
    pub site_prefix: String,
    pub zvezdochka_cmd: String,
    pub public_url_base: String,

    pub git: GitConfig,
}

pub struct Secrets {
    pub octocrab: Octocrab,
    pub public_key: String,
    pub private_key: String,
}

pub struct Data {
    pub config: Config,
    pub database: Sql,
    pub shhh: Secrets,
    pub work_states: DashMap<ChannelId, Arc<Mutex<WorkState>>>,
}

pub async fn start_bot() {}
