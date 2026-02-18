use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Serialize, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DependencyStatus {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SetupStatus {
    pub dependencies_ok: bool,
    pub store_exists: bool,
    pub needs_setup: bool,
    pub first_run: bool,
    pub dependencies: Vec<DependencyStatus>,
}

#[derive(Debug, Serialize, Clone)]
pub struct StoreInfo {
    pub store_path: String,
    pub gpg_id: Option<String>,
    pub has_git: bool,
    pub git_remote: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpgKey {
    pub key_id: String,
    pub uid: String,
    pub fingerprint: String,
}

fn store_dir() -> PathBuf {
    dirs_home().join(".password-store")
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("~"))
}

fn build_tree(dir: &std::path::Path, prefix: &str) -> Vec<TreeNode> {
    let mut nodes = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return nodes;
    };

    let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    sorted.sort_by_key(|e| e.file_name());

    for entry in sorted {
        let name = entry.file_name().to_string_lossy().to_string();
        // Skip hidden files/dirs
        if name.starts_with('.') {
            continue;
        }

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            let child_prefix = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };
            let children = build_tree(&entry.path(), &child_prefix);
            nodes.push(TreeNode {
                name: name.clone(),
                path: child_prefix,
                is_dir: true,
                children,
            });
        } else if name.ends_with(".gpg") {
            let entry_name = name.trim_end_matches(".gpg").to_string();
            let entry_path = if prefix.is_empty() {
                entry_name.clone()
            } else {
                format!("{}/{}", prefix, entry_name)
            };
            nodes.push(TreeNode {
                name: entry_name,
                path: entry_path,
                is_dir: false,
                children: vec![],
            });
        }
    }

    nodes
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip ESC [ ... (final byte 0x40-0x7E)
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&nc) = chars.peek() {
                    chars.next();
                    if nc.is_ascii() && (nc as u8) >= 0x40 && (nc as u8) <= 0x7E {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn run_pass(args: &[&str]) -> Result<String, String> {
    let output = Command::new("pass")
        .args(args)
        .env("PASSWORD_STORE_CLIP_TIME", "45")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("PATH", augmented_path())
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("Failed to run pass: {}", e))?;

    if output.status.success() {
        Ok(strip_ansi(&String::from_utf8_lossy(&output.stdout)))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Err(if stderr.is_empty() { stdout } else { stderr })
    }
}

fn run_pass_stdin(args: &[&str], input: &str) -> Result<String, String> {
    use std::io::Write;
    let mut child = Command::new("pass")
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("PATH", augmented_path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run pass: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for pass: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn augmented_path() -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    format!("/opt/homebrew/bin:/usr/local/bin:{}", current)
}

fn run_command(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .env("PATH", augmented_path())
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("Failed to run {}: {}", program, e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Err(if stderr.trim().is_empty() { stdout } else { stderr })
    }
}

fn run_command_stdin(program: &str, args: &[&str], input: &str) -> Result<String, String> {
    use std::io::Write;
    let mut child = Command::new(program)
        .args(args)
        .env("PATH", augmented_path())
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run {}: {}", program, e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for {}: {}", program, e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn config_dir() -> PathBuf {
    dirs_home().join(".config").join("pass-gui")
}

fn is_wizard_done() -> bool {
    config_dir().join("configured").exists()
}

fn check_dependency(name: &str, version_args: &[&str]) -> DependencyStatus {
    match run_command(name, version_args) {
        Ok(output) => {
            let version = output.lines().next().unwrap_or("").to_string();
            DependencyStatus {
                name: name.to_string(),
                installed: true,
                version: Some(version),
            }
        }
        Err(_) => DependencyStatus {
            name: name.to_string(),
            installed: false,
            version: None,
        },
    }
}

#[tauri::command]
pub fn check_setup_status() -> SetupStatus {
    let deps = vec![
        check_dependency("pass", &["version"]),
        check_dependency("gpg", &["--version"]),
        check_dependency("git", &["--version"]),
    ];
    let dependencies_ok = deps.iter().all(|d| d.installed);
    let store_exists = store_dir().join(".gpg-id").exists();
    let first_run = !is_wizard_done();
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
    let dir = config_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;
    std::fs::write(dir.join("configured"), "")
        .map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn get_store_info() -> StoreInfo {
    let store = store_dir();
    let store_path = store.to_string_lossy().to_string();

    let gpg_id = std::fs::read_to_string(store.join(".gpg-id"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let has_git = store.join(".git").exists();

    let git_remote = if has_git {
        run_command("git", &["-C", &store_path, "remote", "get-url", "origin"]).ok()
    } else {
        None
    };

    StoreInfo {
        store_path,
        gpg_id,
        has_git,
        git_remote,
    }
}

#[tauri::command]
pub fn install_dependencies() -> Result<SetupStatus, String> {
    // Check if brew is available
    run_command("brew", &["--version"])
        .map_err(|_| "Homebrew is not installed. Please install it from https://brew.sh first.".to_string())?;

    run_command("brew", &["install", "pass", "gnupg", "git"])
        .map_err(|e| format!("Failed to install dependencies: {}", e))?;

    Ok(check_setup_status())
}

#[tauri::command]
pub fn list_gpg_keys() -> Result<Vec<GpgKey>, String> {
    let output = run_command("gpg", &["--list-secret-keys", "--with-colons"])
        .map_err(|e| format!("Failed to list GPG keys: {}", e))?;

    let mut keys = Vec::new();
    let mut current_fingerprint = String::new();
    let mut current_key_id = String::new();

    for line in output.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.is_empty() {
            continue;
        }
        match fields[0] {
            "sec" => {
                if fields.len() > 4 {
                    current_key_id = fields[4].to_string();
                }
            }
            "fpr" => {
                if fields.len() > 9 {
                    current_fingerprint = fields[9].to_string();
                }
            }
            "uid" => {
                if fields.len() > 9 && !current_key_id.is_empty() {
                    keys.push(GpgKey {
                        key_id: current_key_id.clone(),
                        uid: fields[9].to_string(),
                        fingerprint: current_fingerprint.clone(),
                    });
                    // Reset so we only take the first uid per key
                    current_key_id = String::new();
                    current_fingerprint = String::new();
                }
            }
            _ => {}
        }
    }

    Ok(keys)
}

#[tauri::command]
pub fn generate_gpg_key(name: String, email: String, passphrase: String) -> Result<GpgKey, String> {
    let key_params = format!(
        "%no-protection\nKey-Type: RSA\nKey-Length: 4096\nSubkey-Type: RSA\nSubkey-Length: 4096\nName-Real: {}\nName-Email: {}\nExpire-Date: 0\n{}%commit\n",
        name,
        email,
        if passphrase.is_empty() {
            String::new()
        } else {
            format!("Passphrase: {}\n", passphrase)
        }
    );

    // Remove %no-protection if passphrase is set
    let key_params = if !passphrase.is_empty() {
        key_params.replace("%no-protection\n", "")
    } else {
        key_params
    };

    run_command_stdin("gpg", &["--batch", "--gen-key"], &key_params)
        .map_err(|e| format!("Failed to generate GPG key: {}", e))?;

    // Get the newly created key
    let keys = list_gpg_keys()?;
    keys.into_iter()
        .find(|k| k.uid.contains(&email))
        .ok_or_else(|| "Key was generated but could not be found".to_string())
}

#[tauri::command]
pub fn init_password_store(gpg_id: String) -> Result<String, String> {
    run_command("pass", &["init", &gpg_id])
        .map_err(|e| format!("Failed to initialize password store: {}", e))
}

#[tauri::command]
pub fn init_git_repo() -> Result<String, String> {
    run_command("pass", &["git", "init"])
        .map_err(|e| format!("Failed to initialize git repo: {}", e))
}

#[tauri::command]
pub fn add_git_remote(url: String) -> Result<String, String> {
    // Try adding; if it already exists, use set-url
    match run_command("pass", &["git", "remote", "add", "origin", &url]) {
        Ok(output) => Ok(output),
        Err(_) => run_command("pass", &["git", "remote", "set-url", "origin", &url])
            .map_err(|e| format!("Failed to set git remote: {}", e)),
    }
}

#[tauri::command]
pub fn list_store() -> Result<Vec<TreeNode>, String> {
    let store = store_dir();
    if !store.exists() {
        return Err("Password store not found at ~/.password-store/".into());
    }
    Ok(build_tree(&store, ""))
}

#[tauri::command]
pub fn show_password(path: String) -> Result<String, String> {
    run_pass(&["show", &path])
}

#[tauri::command]
pub fn insert_password(path: String, content: String) -> Result<String, String> {
    run_pass_stdin(&["insert", "-m", "-f", &path], &content)
}

#[tauri::command]
pub fn generate_password(path: String, length: u32, no_symbols: bool) -> Result<String, String> {
    let len_str = length.to_string();
    if no_symbols {
        run_pass(&["generate", "-n", "-f", &path, &len_str])
    } else {
        run_pass(&["generate", "-f", &path, &len_str])
    }
}

#[tauri::command]
pub fn remove_password(path: String) -> Result<String, String> {
    run_pass(&["rm", "-f", &path])
}

#[tauri::command]
pub fn remove_batch(paths: Vec<String>) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    // Sort longest paths first to remove children before parents
    let mut sorted = paths;
    sorted.sort_by(|a, b| b.len().cmp(&a.len()));
    for path in &sorted {
        if let Err(e) = run_pass(&["rm", "-rf", path]) {
            errors.push(format!("{}: {}", path, e));
        }
    }
    if errors.is_empty() {
        Ok(vec![])
    } else {
        Err(errors.join("\n"))
    }
}

#[tauri::command]
pub fn move_password(old_path: String, new_path: String) -> Result<String, String> {
    run_pass(&["mv", "-f", &old_path, &new_path])
}

#[tauri::command]
pub fn copy_password(old_path: String, new_path: String) -> Result<String, String> {
    run_pass(&["cp", "-f", &old_path, &new_path])
}

#[tauri::command]
pub fn find_passwords(pattern: String) -> Result<String, String> {
    run_pass(&["find", &pattern])
}

#[tauri::command]
pub fn grep_passwords(pattern: String) -> Result<String, String> {
    run_pass(&["grep", &pattern])
}

#[tauri::command]
pub fn git_push() -> Result<String, String> {
    run_pass(&["git", "push"])
}

#[tauri::command]
pub fn git_pull() -> Result<String, String> {
    run_pass(&["git", "pull"])
}

#[tauri::command]
pub fn git_log() -> Result<String, String> {
    run_pass(&["git", "log", "--oneline", "-20"])
}
