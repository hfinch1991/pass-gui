use crate::entry;
use crate::git;
use crate::gpg;
use crate::import_export;
use crate::native_messaging;
use crate::platform;
use crate::store;
use crate::totp;
use crate::types::*;

// ── Existing commands (same signatures, API-compatible) ──

#[tauri::command]
pub fn check_setup_status() -> SetupStatus {
    let deps = vec![
        platform::check_dependency("gpg", &["--version"]),
        platform::check_dependency("git", &["--version"]),
    ];
    let dependencies_ok = deps.iter().all(|d| d.installed);
    let store_exists = platform::store_dir().join(".gpg-id").exists();
    let first_run = !platform::is_wizard_done();
    SetupStatus {
        dependencies_ok,
        store_exists,
        needs_setup: first_run || !dependencies_ok || !store_exists,
        first_run,
        dependencies: deps,
    }
}

#[tauri::command]
pub fn mark_setup_complete() -> Result<(), String> {
    let dir = platform::config_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;
    std::fs::write(dir.join("configured"), "")
        .map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn get_store_info() -> StoreInfo {
    let store = platform::store_dir();
    let store_path = store.to_string_lossy().to_string();
    let gpg_id = gpg::read_gpg_id(&store);
    let has_git = git::has_repo();
    let git_remote = git::get_remote_url();

    StoreInfo {
        store_path,
        gpg_id,
        has_git,
        git_remote,
    }
}

#[tauri::command]
pub fn install_dependencies() -> Result<SetupStatus, String> {
    #[cfg(target_os = "macos")]
    {
        platform::run_command("brew", &["--version"])
            .map_err(|_| "Homebrew is not installed. Please install it from https://brew.sh first.".to_string())?;

        platform::run_command("brew", &["install", "gnupg", "git"])
            .map_err(|e| format!("Failed to install dependencies: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        return Err("Please install gpg and git using your system package manager:\n\
            Ubuntu/Debian: sudo apt install gnupg git\n\
            Fedora: sudo dnf install gnupg2 git\n\
            Arch: sudo pacman -S gnupg git".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        return Err("Please install the following manually:\n\
            1. Gpg4win: https://www.gpg4win.org/\n\
            2. Git for Windows: https://git-scm.com/download/win".to_string());
    }

    Ok(check_setup_status())
}

#[tauri::command]
pub fn list_gpg_keys() -> Result<Vec<GpgKey>, String> {
    gpg::list_secret_keys()
}

#[tauri::command]
pub fn generate_gpg_key(name: String, email: String, passphrase: String) -> Result<GpgKey, String> {
    gpg::generate_key(&name, &email, &passphrase)
}

#[tauri::command]
pub fn init_password_store(gpg_id: String) -> Result<String, String> {
    store::init_store(&gpg_id)?;
    Ok("Password store initialized".to_string())
}

#[tauri::command]
pub fn init_git_repo() -> Result<String, String> {
    git::init()?;
    Ok("Git repository initialized".to_string())
}

#[tauri::command]
pub fn add_git_remote(url: String) -> Result<String, String> {
    git::add_remote(&url)?;
    Ok("Remote added".to_string())
}

#[tauri::command]
pub fn list_store() -> Result<Vec<TreeNode>, String> {
    store::list()
}

#[tauri::command]
pub fn show_password(path: String, passphrase: Option<String>) -> Result<String, String> {
    store::show(&path, passphrase.as_deref())
}

#[tauri::command]
pub fn insert_password(path: String, content: String) -> Result<String, String> {
    store::insert(&path, &content)?;
    let gpg_path = format!("{}.gpg", path);
    git::auto_commit(&[&gpg_path], &format!("Add/update {} with pass-gui", path));
    Ok(String::new())
}

#[tauri::command]
pub fn generate_password(path: String, length: u32, no_symbols: bool) -> Result<String, String> {
    let password = store::generate_password(length, no_symbols);
    store::insert(&path, &password)?;
    let gpg_path = format!("{}.gpg", path);
    git::auto_commit(
        &[&gpg_path],
        &format!("Generate password for {} with pass-gui", path),
    );
    Ok(format!("Generated password for {}\n{}", path, password))
}

#[tauri::command]
pub fn remove_password(path: String) -> Result<String, String> {
    let gpg_path = format!("{}.gpg", path);
    store::remove(&path)?;
    git::auto_commit(&[&gpg_path], &format!("Remove {} with pass-gui", path));
    Ok(String::new())
}

#[tauri::command]
pub fn remove_batch(paths: Vec<String>) -> Result<Vec<String>, String> {
    let result = store::remove_batch(&paths)?;
    git::auto_commit_all(&format!(
        "Remove {} entries with pass-gui",
        paths.len()
    ));
    Ok(result)
}

#[tauri::command]
pub fn move_password(old_path: String, new_path: String) -> Result<String, String> {
    store::move_entry(&old_path, &new_path)?;
    git::auto_commit_all(&format!(
        "Move {} to {} with pass-gui",
        old_path, new_path
    ));
    Ok(String::new())
}

#[tauri::command]
pub fn copy_password(old_path: String, new_path: String) -> Result<String, String> {
    store::copy_entry(&old_path, &new_path)?;
    let gpg_path = format!("{}.gpg", new_path);
    git::auto_commit(
        &[&gpg_path],
        &format!("Copy {} to {} with pass-gui", old_path, new_path),
    );
    Ok(String::new())
}

#[tauri::command]
pub fn find_passwords(pattern: String) -> Result<String, String> {
    store::find(&pattern)
}

#[tauri::command]
pub fn grep_passwords(pattern: String) -> Result<String, String> {
    store::grep(&pattern)
}

#[tauri::command]
pub fn git_push() -> Result<String, String> {
    git::push()
}

#[tauri::command]
pub fn git_pull() -> Result<String, String> {
    git::pull()
}

#[tauri::command]
pub fn git_log() -> Result<String, String> {
    git::log(20)
}

// ── Phase 2: Structured entries + TOTP ──

#[tauri::command]
pub fn get_entry_fields(path: String, passphrase: Option<String>) -> Result<EntryFields, String> {
    let raw = store::show(&path, passphrase.as_deref())?;
    Ok(entry::parse(&raw))
}

#[tauri::command]
pub fn save_entry_fields(path: String, fields: EntryFields) -> Result<(), String> {
    let content = entry::serialize(&fields);
    store::insert(&path, &content)?;
    let gpg_path = format!("{}.gpg", path);
    git::auto_commit(&[&gpg_path], &format!("Update {} with pass-gui", path));
    Ok(())
}

#[tauri::command]
pub fn get_totp_code(path: String, passphrase: Option<String>) -> Result<TotpCode, String> {
    let raw = store::show(&path, passphrase.as_deref())?;
    let fields = entry::parse(&raw);
    let totp_uri = fields
        .totp
        .ok_or_else(|| "No TOTP configured for this entry".to_string())?;
    totp::generate_code(&totp_uri)
}

// ── Phase 3: Import/Export ──

#[tauri::command]
pub fn import_csv(file_path: String, format: ImportFormat) -> Result<u32, String> {
    import_export::import_csv(&file_path, &format)
}

#[tauri::command]
pub fn export_csv(file_path: String) -> Result<u32, String> {
    import_export::export_csv(&file_path)
}

// ── Phase 4: Browser extension search ──

#[tauri::command]
pub fn search_by_url(url: String) -> Result<Vec<(String, EntryFields)>, String> {
    native_messaging::search_by_url(&url, None)
}

#[tauri::command]
pub fn install_browser_extension() -> Result<String, String> {
    native_messaging::install_manifest()
}
