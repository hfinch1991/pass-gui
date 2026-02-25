use crate::entry;
use crate::git;
use crate::gpg;
use crate::platform::store_dir;
use crate::store;
use crate::types::{EntryFields, ImportFormat};

/// Import passwords from a CSV file
pub fn import_csv(file_path: &str, format: &ImportFormat) -> Result<u32, String> {
    let mut reader = csv::Reader::from_path(file_path)
        .map_err(|e| format!("Failed to open CSV: {}", e))?;

    let headers = reader
        .headers()
        .map_err(|e| format!("Failed to read CSV headers: {}", e))?
        .clone();

    let mapping = column_mapping(format, &headers)?;
    let mut count = 0u32;
    let store_path = store_dir();
    let gpg_id = gpg::read_gpg_id(&store_path)
        .ok_or_else(|| "No .gpg-id found in password store".to_string())?;

    for result in reader.records() {
        let record = result.map_err(|e| format!("Failed to read CSV row: {}", e))?;

        let name = get_field(&record, mapping.name)
            .unwrap_or_default();
        let password = get_field(&record, mapping.password)
            .unwrap_or_default();

        if name.is_empty() || password.is_empty() {
            continue;
        }

        let url_val = get_field(&record, mapping.url);
        let username = get_field(&record, mapping.username);
        let notes = get_field(&record, mapping.notes);
        let folder = get_field(&record, mapping.folder);

        // Derive path: folder/name or domain/name
        let entry_path = derive_path(&name, folder.as_deref(), url_val.as_deref());

        let fields = EntryFields {
            password,
            username,
            url: url_val,
            notes,
            totp: None,
            tags: vec![],
            extra: vec![],
        };

        let content = entry::serialize(&fields);
        if store::insert_with_gpg_id(&entry_path, &content, &gpg_id).is_ok() {
            count += 1;
        }
    }

    if count > 0 {
        git::auto_commit_all(&format!("Import {} entries from CSV", count));
    }

    Ok(count)
}

/// Export all passwords to a CSV file
pub fn export_csv(file_path: &str) -> Result<u32, String> {
    let store = store_dir();
    let mut writer = csv::Writer::from_path(file_path)
        .map_err(|e| format!("Failed to create CSV: {}", e))?;

    writer
        .write_record(["name", "url", "username", "password", "notes", "totp", "tags"])
        .map_err(|e| format!("Failed to write CSV header: {}", e))?;

    let mut count = 0u32;

    fn walk_and_export(
        dir: &std::path::Path,
        prefix: &str,
        writer: &mut csv::Writer<std::fs::File>,
        count: &mut u32,
    ) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };

        let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        sorted.sort_by_key(|e| e.file_name());

        for dir_entry in sorted {
            let name = dir_entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }

            let ft = match dir_entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            if ft.is_dir() {
                let child_prefix = if prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", prefix, name)
                };
                walk_and_export(&dir_entry.path(), &child_prefix, writer, count);
            } else if name.ends_with(".gpg") {
                let entry_name = name.trim_end_matches(".gpg");
                let entry_path = if prefix.is_empty() {
                    entry_name.to_string()
                } else {
                    format!("{}/{}", prefix, entry_name)
                };

                if let Ok(content) = gpg::decrypt_file(&dir_entry.path(), None) {
                    let fields = entry::parse(&content);
                    let tags_str = fields.tags.join(", ");
                    let _ = writer.write_record([
                        &entry_path,
                        fields.url.as_deref().unwrap_or(""),
                        fields.username.as_deref().unwrap_or(""),
                        &fields.password,
                        fields.notes.as_deref().unwrap_or(""),
                        fields.totp.as_deref().unwrap_or(""),
                        &tags_str,
                    ]);
                    *count += 1;
                }
            }
        }
    }

    walk_and_export(&store, "", &mut writer, &mut count);

    writer
        .flush()
        .map_err(|e| format!("Failed to flush CSV: {}", e))?;

    Ok(count)
}

struct ColumnMapping {
    name: Option<usize>,
    url: Option<usize>,
    username: Option<usize>,
    password: Option<usize>,
    notes: Option<usize>,
    folder: Option<usize>,
}

fn column_mapping(format: &ImportFormat, headers: &csv::StringRecord) -> Result<ColumnMapping, String> {
    let find = |names: &[&str]| -> Option<usize> {
        for name in names {
            for (i, header) in headers.iter().enumerate() {
                if header.to_lowercase() == name.to_lowercase() {
                    return Some(i);
                }
            }
        }
        None
    };

    let mapping = match format {
        ImportFormat::Chrome => ColumnMapping {
            name: find(&["name"]),
            url: find(&["url"]),
            username: find(&["username"]),
            password: find(&["password"]),
            notes: find(&["note", "notes"]),
            folder: None,
        },
        ImportFormat::Firefox => ColumnMapping {
            name: find(&["hostname", "url"]),
            url: find(&["hostname", "url"]),
            username: find(&["username"]),
            password: find(&["password"]),
            notes: None,
            folder: None,
        },
        ImportFormat::OnePassword => ColumnMapping {
            name: find(&["title"]),
            url: find(&["url", "urls"]),
            username: find(&["username"]),
            password: find(&["password"]),
            notes: find(&["notes", "notesplain"]),
            folder: find(&["vault", "folder"]),
        },
        ImportFormat::Bitwarden => ColumnMapping {
            name: find(&["name"]),
            url: find(&["login_uri", "url"]),
            username: find(&["login_username", "username"]),
            password: find(&["login_password", "password"]),
            notes: find(&["notes"]),
            folder: find(&["folder"]),
        },
        ImportFormat::KeePass => ColumnMapping {
            name: find(&["title", "account"]),
            url: find(&["url", "web site"]),
            username: find(&["username", "user name"]),
            password: find(&["password"]),
            notes: find(&["notes"]),
            folder: find(&["group"]),
        },
        ImportFormat::LastPass => ColumnMapping {
            name: find(&["name"]),
            url: find(&["url"]),
            username: find(&["username"]),
            password: find(&["password"]),
            notes: find(&["extra", "notes"]),
            folder: find(&["grouping", "folder"]),
        },
    };

    if mapping.password.is_none() {
        return Err("Could not find password column in CSV".to_string());
    }

    Ok(mapping)
}

fn get_field(record: &csv::StringRecord, index: Option<usize>) -> Option<String> {
    index.and_then(|i| record.get(i)).map(|s| s.to_string()).filter(|s| !s.is_empty())
}

fn derive_path(name: &str, folder: Option<&str>, url: Option<&str>) -> String {
    // Sanitize name: replace path-unfriendly characters
    let sanitized_name = name
        .replace(['/', '\\'], "-")
        .replace([':', '*', '?', '"', '<', '>', '|'], "");

    let sanitized_name = if sanitized_name.is_empty() {
        "unnamed".to_string()
    } else {
        sanitized_name
    };

    if let Some(folder) = folder {
        if !folder.is_empty() {
            let sanitized_folder = folder
                .replace('\\', "/")
                .trim_matches('/')
                .to_string();
            if !sanitized_folder.is_empty() {
                return format!("{}/{}", sanitized_folder, sanitized_name);
            }
        }
    }

    // Try to derive folder from URL domain
    if let Some(url_str) = url {
        if let Ok(parsed) = url::Url::parse(url_str) {
            if let Some(domain) = parsed.host_str() {
                // Strip common prefixes
                let domain = domain
                    .strip_prefix("www.")
                    .unwrap_or(domain);
                return format!("{}/{}", domain, sanitized_name);
            }
        }
    }

    sanitized_name
}
