use std::path::{Path, PathBuf};

use crate::gpg;
use crate::platform::store_dir;
use crate::types::TreeNode;

pub fn build_tree(dir: &Path, prefix: &str) -> Vec<TreeNode> {
    let mut nodes = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return nodes;
    };

    let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    sorted.sort_by_key(|e| e.file_name());

    for entry in sorted {
        let name = entry.file_name().to_string_lossy().to_string();
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

pub fn list() -> Result<Vec<TreeNode>, String> {
    let store = store_dir();
    if !store.exists() {
        return Err("Password store not found".into());
    }
    Ok(build_tree(&store, ""))
}

fn entry_file_path(entry_path: &str) -> PathBuf {
    store_dir().join(format!("{}.gpg", entry_path))
}

pub fn show(path: &str, passphrase: Option<&str>) -> Result<String, String> {
    let file = entry_file_path(path);
    if !file.exists() {
        return Err(format!("Entry not found: {}", path));
    }
    gpg::decrypt_file(&file, passphrase)
}

pub fn insert(path: &str, content: &str) -> Result<(), String> {
    let store = store_dir();
    let gpg_id = gpg::read_gpg_id(&store)
        .ok_or_else(|| "No .gpg-id found in password store".to_string())?;

    insert_with_gpg_id(path, content, &gpg_id)
}

pub fn insert_with_gpg_id(path: &str, content: &str, gpg_id: &str) -> Result<(), String> {
    let file = entry_file_path(path);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    gpg::encrypt_to_file(content, gpg_id, &file)
}

pub fn remove(path: &str) -> Result<(), String> {
    let file = entry_file_path(path);
    if file.exists() {
        std::fs::remove_file(&file)
            .map_err(|e| format!("Failed to remove {}: {}", path, e))?;
    }

    // Also try removing as directory (for `pass rm -rf` equivalent)
    let dir = store_dir().join(path);
    if dir.is_dir() {
        std::fs::remove_dir_all(&dir)
            .map_err(|e| format!("Failed to remove directory {}: {}", path, e))?;
    }

    // Clean up empty parent directories
    if let Some(parent) = file.parent() {
        cleanup_empty_dirs(parent, &store_dir());
    }

    Ok(())
}

fn cleanup_empty_dirs(dir: &Path, stop_at: &Path) {
    let mut current = dir.to_path_buf();
    while current != *stop_at {
        if current.is_dir() {
            if std::fs::read_dir(&current)
                .map(|mut d| d.next().is_none())
                .unwrap_or(false)
            {
                let _ = std::fs::remove_dir(&current);
            } else {
                break;
            }
        }
        match current.parent() {
            Some(p) => current = p.to_path_buf(),
            None => break,
        }
    }
}

pub fn remove_batch(paths: &[String]) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    let mut sorted = paths.to_vec();
    sorted.sort_by(|a, b| b.len().cmp(&a.len()));

    for path in &sorted {
        if let Err(e) = remove(path) {
            errors.push(format!("{}: {}", path, e));
        }
    }

    if errors.is_empty() {
        Ok(vec![])
    } else {
        Err(errors.join("\n"))
    }
}

pub fn move_entry(old_path: &str, new_path: &str) -> Result<(), String> {
    let content = show(old_path, None)?;
    insert(new_path, &content)?;
    remove(old_path)?;
    Ok(())
}

pub fn copy_entry(old_path: &str, new_path: &str) -> Result<(), String> {
    let content = show(old_path, None)?;
    insert(new_path, &content)?;
    Ok(())
}

pub fn generate_password(length: u32, no_symbols: bool) -> String {
    use rand::Rng;

    let chars_with_symbols: Vec<char> =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+[]{}|;:,.<>?"
            .chars()
            .collect();
    let chars_no_symbols: Vec<char> =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
            .chars()
            .collect();

    let charset = if no_symbols {
        &chars_no_symbols
    } else {
        &chars_with_symbols
    };

    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| charset[rng.gen_range(0..charset.len())])
        .collect()
}

pub fn grep(pattern: &str) -> Result<String, String> {
    let store = store_dir();
    let mut results = Vec::new();

    fn walk_and_grep(
        dir: &Path,
        prefix: &str,
        pattern: &str,
        results: &mut Vec<String>,
    ) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };

        let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        sorted.sort_by_key(|e| e.file_name());

        for entry in sorted {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }

            let ft = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            if ft.is_dir() {
                let child_prefix = if prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", prefix, name)
                };
                walk_and_grep(&entry.path(), &child_prefix, pattern, results);
            } else if name.ends_with(".gpg") {
                let entry_name = name.trim_end_matches(".gpg");
                let entry_path = if prefix.is_empty() {
                    entry_name.to_string()
                } else {
                    format!("{}/{}", prefix, entry_name)
                };

                if let Ok(content) = gpg::decrypt_file(&entry.path(), None) {
                    let pattern_lower = pattern.to_lowercase();
                    for line in content.lines() {
                        if line.to_lowercase().contains(&pattern_lower) {
                            results.push(format!("{}:{}", entry_path, line));
                        }
                    }
                }
            }
        }
    }

    walk_and_grep(&store, "", pattern, &mut results);

    if results.is_empty() {
        Ok(String::new())
    } else {
        Ok(results.join("\n"))
    }
}

pub fn find(pattern: &str) -> Result<String, String> {
    let store = store_dir();
    let mut results = Vec::new();

    fn walk_and_find(dir: &Path, prefix: &str, pattern: &str, results: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };

        let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        sorted.sort_by_key(|e| e.file_name());

        for entry in sorted {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }

            let ft = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            if ft.is_dir() {
                let child_prefix = if prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", prefix, name)
                };
                walk_and_find(&entry.path(), &child_prefix, pattern, results);
            } else if name.ends_with(".gpg") {
                let entry_name = name.trim_end_matches(".gpg");
                let entry_path = if prefix.is_empty() {
                    entry_name.to_string()
                } else {
                    format!("{}/{}", prefix, entry_name)
                };

                if entry_path.to_lowercase().contains(&pattern.to_lowercase()) {
                    results.push(entry_path);
                }
            }
        }
    }

    walk_and_find(&store, "", pattern, &mut results);

    if results.is_empty() {
        Ok(String::new())
    } else {
        Ok(results.join("\n"))
    }
}

pub fn init_store(gpg_id: &str) -> Result<(), String> {
    let store = store_dir();
    std::fs::create_dir_all(&store)
        .map_err(|e| format!("Failed to create password store: {}", e))?;

    std::fs::write(store.join(".gpg-id"), format!("{}\n", gpg_id))
        .map_err(|e| format!("Failed to write .gpg-id: {}", e))?;

    Ok(())
}
