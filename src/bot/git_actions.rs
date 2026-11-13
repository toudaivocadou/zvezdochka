use anyhow::Error;
use camino::Utf8PathBuf;
use flume::Sender;
use git2::{
    Cred, Direction, FetchOptions, IndexAddOption, PushOptions, RemoteCallbacks, Repository,
    Signature, build::RepoBuilder,
};
use humansize::{DECIMAL, format_size};
use octocrab::{Octocrab, models::pulls::PullRequest};
use std::{fmt::Display, sync::Arc};

use crate::bot::{Data, TicketId, utils::ticket_work_path};

#[derive(Clone, Debug)]
pub enum FromRemoteProgress {
    Counting(String),
    Transfer(Box<GitRecvProgress>),
}

impl Display for FromRemoteProgress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FromRemoteProgress::Counting(counting) => write!(f, "Counting Objects: {counting}"),
            FromRemoteProgress::Transfer(git_recv_progress) => {
                write!(
                    f,
                    "Objects: {} Total ({} Indexed, {} Received, {} Local)\nDeltas: {} Total ({} Indexed)\nBytes: {} Received.",
                    git_recv_progress.total_objects,
                    git_recv_progress.indexed_objects,
                    git_recv_progress.received_objects,
                    git_recv_progress.local_objects,
                    git_recv_progress.total_deltas,
                    git_recv_progress.indexed_deltas,
                    format_size(git_recv_progress.received_bytes, DECIMAL)
                )
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GitRecvProgress {
    pub total_objects: u64,
    pub indexed_objects: u64,
    pub received_objects: u64,
    pub local_objects: u64,
    pub total_deltas: u64,
    pub indexed_deltas: u64,
    pub received_bytes: u64,
}

#[derive(Clone, Debug)]
pub enum ToRemoteProgress {
    UpdateRefspecFail { refspec: String, server_msg: String },
    Transfer(Box<GitSendProgress>),
}

#[derive(Copy, Clone, Debug)]
pub struct GitSendProgress {
    pub current: u64,
    pub total: u64,
    pub bytes: u64,
}

pub struct CreatedGit {
    pub commit_sha: Vec<u8>,
    pub commit_message: String,
    pub path: Utf8PathBuf,
    pub repository: Repository,
}

pub fn clone_git(
    data: Arc<Data>,
    ticket_id: TicketId,
    progress: Arc<Sender<FromRemoteProgress>>,
) -> Result<CreatedGit, Error> {
    let sb_prog = progress.clone();
    let recv_progress = progress.clone();

    let mut callbacks = RemoteCallbacks::new();
    let data2 = data.clone();
    callbacks.credentials(move |_, username, _| {
        Cred::ssh_key_from_memory(
            username.unwrap(),
            Some(&data2.shhh.git_public_key),
            &data2.shhh.git_private_key,
            None,
        )
    });
    callbacks.sideband_progress(|progress| {
        let progress = String::from_utf8_lossy(progress).to_string();
        let _ = sb_prog.send(FromRemoteProgress::Counting(progress));
        true
    });
    callbacks.transfer_progress(|progress| {
        let git_prog = GitRecvProgress {
            total_objects: progress.total_objects() as u64,
            indexed_objects: progress.indexed_objects() as u64,
            received_objects: progress.received_objects() as u64,
            local_objects: progress.local_objects() as u64,
            total_deltas: progress.total_deltas() as u64,
            indexed_deltas: progress.indexed_deltas() as u64,
            received_bytes: progress.received_bytes() as u64,
        };
        let _ = recv_progress.send(FromRemoteProgress::Transfer(Box::new(git_prog)));
        true
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    let path = ticket_work_path(data.clone(), ticket_id);

    let data3 = data.clone();
    let repository = RepoBuilder::new()
        .fetch_options(fetch_opts)
        .clone(&data3.config.git.repo_url, path.as_std_path())?;

    let (commit_message, commit_sha) = {
        let head_commit = repository.head()?.peel_to_commit()?;
        let commit_message = head_commit.message()?.to_string();
        let commit_sha = head_commit.id().as_bytes().to_vec();
        (commit_message, commit_sha)
    };

    Ok(CreatedGit {
        commit_sha,
        commit_message,
        path,
        repository,
    })
}

pub fn import_existing_repository(path: &str) -> Result<Repository, Error> {
    Ok(Repository::open(path)?)
}

pub fn branch_git_repo(repository: &Repository, branch_name: &str) -> Result<(), Error> {
    let head_commit = repository.head()?.peel_to_commit()?;
    repository.branch(branch_name, &head_commit, false)?;
    Ok(())
}

pub fn stage_and_commit(
    repository: &Repository,
    on_behalf_of: &str,
    commit_email_address: &str,
    message: &str,
) -> Result<(), Error> {
    let mut index = repository.index()?;
    index.add_all(&["."], IndexAddOption::DEFAULT, None)?;
    let oid = index.write_tree()?;
    let head_commit = repository.head()?.peel_to_commit()?;
    let tree = repository.find_tree(oid)?;

    let author_signature = Signature::now(on_behalf_of, commit_email_address)?;
    let committer_signature = Signature::now("toudaivocadou-zvezdochka", commit_email_address)?; // us, aka zvezdochka

    repository.commit(
        Some("HEAD"),
        &author_signature,
        &committer_signature,
        message,
        &tree,
        &[&head_commit],
    )?;

    Ok(())
}

pub fn push(
    data: Arc<Data>,
    progress: Arc<Sender<ToRemoteProgress>>,
    repository: Repository,
) -> Result<(), Error> {
    let update_ref_prog = progress.clone();
    let push_progress = progress.clone();

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_, username, _| {
        Cred::ssh_key_from_memory(
            username.unwrap(),
            Some(&data.shhh.git_public_key),
            &data.shhh.git_private_key,
            None,
        )
    });

    callbacks.push_update_reference(|refspec, status| match status {
        Some(server_msg) => {
            let refspec = refspec.to_string();
            let server_msg = server_msg.to_string();

            let _ = update_ref_prog.try_send(ToRemoteProgress::UpdateRefspecFail {
                refspec,
                server_msg,
            });
            return Ok(());
        }
        None => return Ok(()),
    });

    callbacks.push_transfer_progress(|current, total, bytes| {
        let transfer = GitSendProgress {
            current: current as u64,
            total: total as u64,
            bytes: bytes as u64,
        };
        let _ = push_progress.try_send(ToRemoteProgress::Transfer(Box::new(transfer)));
    });

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(callbacks);

    let mut remote = repository.find_remote("origin")?;
    remote.connect(Direction::Push)?;
    remote.push::<&str>(&[], Some(&mut push_options));
    Ok(())
}

pub async fn create_github_pr(
    octocrab: Arc<Octocrab>,
    owner: &str,
    repo: &str,
    title: &str,
    from_branch: &str,
    body_str: &str,
) -> Result<PullRequest, Error> {
    let pr_handler = octocrab
        .pulls(owner, repo)
        .create(title, from_branch, "origin")
        .body(body_str)
        .send()
        .await?;
    Ok(pr_handler)
}
