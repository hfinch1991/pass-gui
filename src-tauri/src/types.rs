use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EntryFields {
    pub password: String,
    pub username: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub totp: Option<String>,
    pub tags: Vec<String>,
    pub extra: Vec<(String, String)>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TotpCode {
    pub code: String,
    pub seconds_remaining: u64,
    pub period: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub enum ImportFormat {
    Chrome,
    Firefox,
    OnePassword,
    Bitwarden,
    KeePass,
    LastPass,
}
