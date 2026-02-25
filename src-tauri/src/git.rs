use std::path::Path;

use crate::platform::store_dir;

pub fn has_repo() -> bool {
    store_dir().join(".git").exists()
}

pub fn init() -> Result<(), String> {
    let store = store_dir();
    git2::Repository::init(&store)
        .map_err(|e| format!("Failed to initialize git repo: {}", e))?;

    // Create initial commit
    add_all_and_commit("Initialize password store")?;

    Ok(())
}

pub fn add_and_commit(paths: &[&str], message: &str) -> Result<(), String> {
    let store = store_dir();
    let repo = open_repo(&store)?;

    let mut index = repo
        .index()
        .map_err(|e| format!("Failed to get index: {}", e))?;

    for path in paths {
        // Convert to relative path from store root
        let rel = Path::new(path);
        // Try to add; if file was deleted, remove from index
        let full_path = store.join(rel);
        if full_path.exists() {
            index
                .add_path(rel)
                .map_err(|e| format!("Failed to add {}: {}", path, e))?;
        } else {
            let _ = index.remove_path(rel);
        }
    }

    index
        .write()
        .map_err(|e| format!("Failed to write index: {}", e))?;

    commit_index(&repo, &mut index, message)?;
    Ok(())
}

pub fn add_all_and_commit(message: &str) -> Result<(), String> {
    let store = store_dir();
    let repo = open_repo(&store)?;

    let mut index = repo
        .index()
        .map_err(|e| format!("Failed to get index: {}", e))?;

    index
        .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .map_err(|e| format!("Failed to add all: {}", e))?;

    index
        .write()
        .map_err(|e| format!("Failed to write index: {}", e))?;

    commit_index(&repo, &mut index, message)?;
    Ok(())
}

fn commit_index(
    repo: &git2::Repository,
    index: &mut git2::Index,
    message: &str,
) -> Result<git2::Oid, String> {
    let tree_oid = index
        .write_tree()
        .map_err(|e| format!("Failed to write tree: {}", e))?;

    let tree = repo
        .find_tree(tree_oid)
        .map_err(|e| format!("Failed to find tree: {}", e))?;

    let sig = repo
        .signature()
        .or_else(|_| git2::Signature::now("Pass GUI", "pass-gui@localhost"))
        .map_err(|e| format!("Failed to create signature: {}", e))?;

    let parent_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());

    let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .map_err(|e| format!("Failed to commit: {}", e))
}

pub fn push() -> Result<String, String> {
    let store = store_dir();
    let repo = open_repo(&store)?;

    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| format!("Failed to find remote 'origin': {}", e))?;

    // Determine the current branch name
    let head = repo
        .head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?;
    let branch_name = head
        .shorthand()
        .unwrap_or("master");
    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
    });

    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    remote
        .push(&[&refspec], Some(&mut push_options))
        .map_err(|e| format!("Failed to push: {}", e))?;

    Ok("Push completed successfully".to_string())
}

pub fn pull() -> Result<String, String> {
    let store = store_dir();
    let repo = open_repo(&store)?;

    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| format!("Failed to find remote 'origin': {}", e))?;

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
    });

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let head = repo.head().ok();
    let branch_name = head
        .as_ref()
        .and_then(|h| h.shorthand())
        .unwrap_or("master");

    remote
        .fetch(&[branch_name], Some(&mut fetch_options), None)
        .map_err(|e| format!("Failed to fetch: {}", e))?;

    // Merge the fetched branch
    let fetch_head = repo
        .find_reference("FETCH_HEAD")
        .map_err(|e| format!("Failed to find FETCH_HEAD: {}", e))?;

    let fetch_commit = repo
        .reference_to_annotated_commit(&fetch_head)
        .map_err(|e| format!("Failed to get annotated commit: {}", e))?;

    let (analysis, _) = repo
        .merge_analysis(&[&fetch_commit])
        .map_err(|e| format!("Failed to analyze merge: {}", e))?;

    if analysis.is_up_to_date() {
        Ok("Already up to date".to_string())
    } else if analysis.is_fast_forward() {
        let refname = format!("refs/heads/{}", branch_name);
        let mut reference = repo
            .find_reference(&refname)
            .map_err(|e| format!("Failed to find reference: {}", e))?;

        reference
            .set_target(fetch_commit.id(), "Fast-forward pull")
            .map_err(|e| format!("Failed to fast-forward: {}", e))?;

        repo.set_head(&refname)
            .map_err(|e| format!("Failed to set HEAD: {}", e))?;

        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|e| format!("Failed to checkout: {}", e))?;

        Ok("Fast-forward pull completed".to_string())
    } else {
        Err("Pull requires a merge, which is not supported. Please resolve manually.".to_string())
    }
}

pub fn log(count: usize) -> Result<String, String> {
    let store = store_dir();
    let repo = open_repo(&store)?;

    let mut revwalk = repo
        .revwalk()
        .map_err(|e| format!("Failed to create revwalk: {}", e))?;

    revwalk
        .push_head()
        .map_err(|e| format!("Failed to push HEAD: {}", e))?;

    let mut lines = Vec::new();
    for (i, oid) in revwalk.enumerate() {
        if i >= count {
            break;
        }
        let oid = oid.map_err(|e| format!("Failed to get oid: {}", e))?;
        let commit = repo
            .find_commit(oid)
            .map_err(|e| format!("Failed to find commit: {}", e))?;

        let short_id = &oid.to_string()[..7];
        let message = commit.summary().unwrap_or("(no message)");
        lines.push(format!("{} {}", short_id, message));
    }

    Ok(lines.join("\n"))
}

pub fn add_remote(url: &str) -> Result<(), String> {
    let store = store_dir();
    let repo = open_repo(&store)?;

    let result = repo.remote("origin", url);
    match result {
        Ok(_remote) => Ok(()),
        Err(_) => {
            repo.remote_set_url("origin", url)
                .map_err(|e| format!("Failed to set remote URL: {}", e))
        }
    }
}

pub fn get_remote_url() -> Option<String> {
    let store = store_dir();
    let repo = open_repo(&store).ok()?;
    let remote = repo.find_remote("origin").ok()?;
    remote.url().map(|s| s.to_string())
}

pub fn has_remote() -> bool {
    get_remote_url().is_some()
}

fn open_repo(path: &Path) -> Result<git2::Repository, String> {
    git2::Repository::open(path).map_err(|e| format!("Failed to open git repo: {}", e))
}

/// Auto-commit after a write operation if git is initialized
pub fn auto_commit(paths: &[&str], message: &str) {
    if !has_repo() {
        return;
    }
    let _ = add_and_commit(paths, message);
}

/// Auto-commit all changes after a write operation if git is initialized
pub fn auto_commit_all(message: &str) {
    if !has_repo() {
        return;
    }
    let _ = add_all_and_commit(message);
}
