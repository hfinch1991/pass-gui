use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use crate::entry;
use crate::gpg;
use crate::platform::{store_dir, home_dir};
use crate::types::{EntryFields, TreeNode};
use crate::store;

#[derive(Deserialize, Debug)]
struct BrowserRequest {
    action: String,
    settings: Option<serde_json::Value>,
    entry: Option<String>,
    domain: Option<String>,
    url: Option<String>,
    path: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum BrowserResponse {
    Version { version: u32 },
    List(std::collections::HashMap<String, Vec<String>>),
    Fetch {
        #[serde(rename = "action")]
        action_name: String,
        entry: String,
        #[serde(rename = "rawEntry")]
        raw_entry: String,
    },
    Status {
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    Error {
        status: String,
        error: String,
    },
}

fn log_msg(msg: &str) {
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/pass-gui-native.log")
    {
        let _ = writeln!(file, "[{}] {}", chrono::Local::now(), msg);
    }
}

pub fn handle_native_messaging() {
    log_msg("Native session started");
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    loop {
        let mut len_buf = [0u8; 4];
        if stdin.read_exact(&mut len_buf).is_err() { break; }
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut msg_buf = vec![0u8; len];
        if stdin.read_exact(&mut msg_buf).is_err() { break; }

        let request: Result<BrowserRequest, _> = serde_json::from_slice(&msg_buf);
        let response = match request {
            Ok(req) => {
                match req.action.as_str() {
                    "version" => BrowserResponse::Version { version: 3 },
                    "list" => {
                        let mut files = Vec::new();
                        fn collect_files(nodes: Vec<TreeNode>, files: &mut Vec<String>) {
                            for node in nodes {
                                if node.is_dir { collect_files(node.children.clone(), files); }
                                else { files.push(node.path.clone()); }
                            }
                        }
                        if let Ok(tree) = store::list() { collect_files(tree, &mut files); }
                        let mut map = std::collections::HashMap::new();
                        map.insert("files".to_string(), files);
                        BrowserResponse::List(map)
                    }
                    "fetch" => {
                        if let Some(path) = req.entry {
                            match store::show(&path, None) {
                                Ok(raw) => BrowserResponse::Fetch { action_name: "fetch".into(), entry: path, raw_entry: raw },
                                Err(e) => BrowserResponse::Error { status: "error".into(), error: e },
                            }
                        } else {
                            BrowserResponse::Error { status: "error".into(), error: "No path".into() }
                        }
                    }
                    "save" => {
                        let path = req.path.unwrap_or_default();
                        let user = req.username.unwrap_or_default();
                        let pass = req.password.unwrap_or_default();
                        if path.is_empty() || pass.is_empty() {
                            BrowserResponse::Error { status: "error".into(), error: "Required fields missing".into() }
                        } else {
                            let fields = EntryFields {
                                password: pass,
                                username: if user.is_empty() { None } else { Some(user) },
                                url: req.url,
                                ..Default::default()
                            };
                            let content = entry::serialize(&fields);
                            match store::insert(&path, &content) {
                                Ok(_) => BrowserResponse::Status { status: "ok".into(), message: Some("Saved".into()) },
                                Err(e) => BrowserResponse::Error { status: "error".into(), error: e },
                            }
                        }
                    }
                    "search" => {
                        let term = req.domain.or(req.url).unwrap_or_default();
                        match search_by_url(&term) {
                            Ok(results) => {
                                let res = serde_json::json!({ "status": "ok", "results": results });
                                let res_json = serde_json::to_vec(&res).unwrap_or_default();
                                let _ = stdout.write_all(&(res_json.len() as u32).to_le_bytes());
                                let _ = stdout.write_all(&res_json);
                                let _ = stdout.flush();
                                continue;
                            }
                            Err(e) => BrowserResponse::Error { status: "error".into(), error: e }
                        }
                    }
                    _ => BrowserResponse::Version { version: 3 }
                }
            }
            Err(e) => BrowserResponse::Error { status: "error".into(), error: format!("JSON: {}", e) },
        };

        let res_json = serde_json::to_vec(&response).unwrap_or_default();
        let _ = stdout.write_all(&(res_json.len() as u32).to_le_bytes());
        let _ = stdout.write_all(&res_json);
        let _ = stdout.flush();
    }
}

pub fn install_manifest() -> Result<String, String> {
    let home = home_dir();
    let exe_path = "/Applications/Pass GUI.app/Contents/MacOS/pass-gui";
    
    // Create absolute path wrapper
    let bin_dir = home.join(".local/bin");
    std::fs::create_dir_all(&bin_dir).ok();
    let wrapper_path = bin_dir.join("pass-gui-native-wrapper");
    let script = format!("#!/bin/bash\nexport PATH=\"/usr/local/bin:/opt/homebrew/bin:$PATH\"\n\"{}\" --native-messaging \"$@\"\n", exe_path);
    std::fs::write(&wrapper_path, script).ok();
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(&wrapper_path, std::fs::Permissions::from_mode(0o755));

    let wrapper_path_str = wrapper_path.to_string_lossy();

    // Chrome Manifest
    let chrome_manifest = serde_json::json!({
        "name": "com.github.browserpass.native",
        "description": "Pass GUI Native Messaging Host",
        "path": wrapper_path_str,
        "type": "stdio",
        "allowed_origins": [
            "chrome-extension://eiffdhkmjeghngolhgmcfgbggemokfml/",
            "chrome-extension://naepdomgkenhinolocfifgehidddafch/",
            "chrome-extension://naepdomlvcaocididbedllndadhbuien/",
            "chrome-extension://mcbpblocgmgfnpjjppndjneeidokajeg/"
        ]
    });

    // Firefox Manifest (MUST be distinct)
    let firefox_manifest = serde_json::json!({
        "name": "com.github.browserpass.native",
        "description": "Pass GUI Native Messaging Host",
        "path": wrapper_path_str,
        "type": "stdio",
        "allowed_extensions": [
            "pass-gui-filler@harold.local"
        ]
    });

    #[cfg(target_os = "macos")]
    {
        let chrome_paths = [
            "Library/Application Support/Google/Chrome/NativeMessagingHosts",
            "Library/Application Support/Microsoft Edge/NativeMessagingHosts",
        ];
        let firefox_paths = [
            "Library/Application Support/Mozilla/NativeMessagingHosts",
            "Library/Application Support/Firefox/NativeMessagingHosts",
        ];

        for p in chrome_paths {
            let dir = home.join(p);
            std::fs::create_dir_all(&dir).ok();
            let _ = std::fs::write(dir.join("com.github.browserpass.native.json"), serde_json::to_string_pretty(&chrome_manifest).unwrap());
        }
        for p in firefox_paths {
            let dir = home.join(p);
            std::fs::create_dir_all(&dir).ok();
            let _ = std::fs::write(dir.join("com.github.browserpass.native.json"), serde_json::to_string_pretty(&firefox_manifest).unwrap());
        }
        Ok("All manifests re-installed with distinct configs. Please RESTART all browsers.".into())
    }

    #[cfg(target_os = "windows")]
    {
        let config_dir = crate::platform::config_dir();
        std::fs::create_dir_all(&config_dir).ok();
        let json_path = config_dir.join("com.github.browserpass.native.json");
        std::fs::write(&json_path, serde_json::to_string_pretty(&chrome_manifest).unwrap()).ok();
        Ok(format!("Windows JSON created at {}. Use registry to register.", json_path.display()))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { Err("Unsupported OS".into()) }
}

pub fn search_by_url(search_url: &str) -> Result<Vec<(String, EntryFields)>, String> {
    let domain = extract_domain(search_url).ok_or("Invalid URL")?;
    let domain_lower = domain.to_lowercase();
    let store = store_dir();
    let mut results = Vec::new();
    walk_and_search(&store, "", &domain_lower, &mut results);
    Ok(results)
}

fn extract_domain(url_str: &str) -> Option<String> {
    if url_str == "localhost" { return Some("localhost".to_string()); }
    if let Ok(parsed) = url::Url::parse(url_str) {
        if let Some(host) = parsed.host_str() {
            return Some(host.strip_prefix("www.").unwrap_or(host).to_string());
        }
    }
    Some(url_str.to_string())
}

fn walk_and_search(dir: &std::path::Path, prefix: &str, domain: &str, results: &mut Vec<(String, EntryFields)>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return; };
    for dir_entry in entries.filter_map(|e| e.ok()) {
        let name = dir_entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if dir_entry.file_type().map(|f| f.is_dir()).unwrap_or(false) {
            let child_prefix = if prefix.is_empty() { name } else { format!("{}/{}", prefix, name) };
            walk_and_search(&dir_entry.path(), &child_prefix, domain, results);
        } else if name.ends_with(".gpg") {
            let entry_name = name.trim_end_matches(".gpg");
            let entry_path = if prefix.is_empty() { entry_name.to_string() } else { format!("{}/{}", prefix, entry_name) };
            if entry_path.to_lowercase().contains(domain) {
                if let Ok(content) = gpg::decrypt_file(&dir_entry.path(), None) {
                    results.push((entry_path, entry::parse(&content)));
                }
            }
        }
    }
}