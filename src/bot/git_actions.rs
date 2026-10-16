use anyhow::Error;
use git2::{
    Cred, Direction, FetchOptions, IndexAddOption, PushOptions, RemoteCallbacks, Repository,
    Signature, build::RepoBuilder,
};
use octocrab::{Octocrab, models::pulls::PullRequest};
use std::{path::Path, sync::Arc};
use tokio::sync::mpsc::Sender;

pub struct GitShhh {
    pub pubkey: String,
    pub privkey: String,
}

pub struct GHApiKey {}

#[derive(Clone, Debug)]
pub enum FromRemoteProgress {
    Counting(String),
    Transfer(Box<GitRecvProgress>),
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

pub fn clone_git(
    progress: Arc<Sender<FromRemoteProgress>>,
    git_secrets: &GitShhh,
    git_url: &str,
    to_directory: &str,
) -> Result<Repository, Error> {
    let sb_prog = progress.clone();
    let recv_progress = progress.clone();

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_, username, _| {
        Cred::ssh_key_from_memory(
            username.unwrap(),
            Some(&git_secrets.pubkey),
            &git_secrets.privkey,
            None,
        )
    });
    callbacks.sideband_progress(|progress| {
        let data = String::from_utf8_lossy(progress).to_string();
        let _ = sb_prog.try_send(FromRemoteProgress::Counting(data));
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
        let _ = recv_progress.try_send(FromRemoteProgress::Transfer(Box::new(git_prog)));
        true
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    let repo = RepoBuilder::new()
        .fetch_options(fetch_opts)
        .clone(git_url, Path::new(to_directory))?;

    Ok(repo)
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
    repository: &Repository,
    progress: Arc<Sender<ToRemoteProgress>>,
    git_secrets: &GitShhh,
) -> Result<(), Error> {
    let update_ref_prog = progress.clone();
    let push_progress = progress.clone();

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_, username, _| {
        Cred::ssh_key_from_memory(
            username.unwrap(),
            Some(&git_secrets.pubkey),
            &git_secrets.privkey,
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
