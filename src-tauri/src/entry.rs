use crate::types::EntryFields;

/// Parse a raw pass entry into structured fields.
/// Format (browserpass/passff compatible):
/// ```text
/// thepassword
/// url: https://example.com
/// username: user@example.com
/// notes: some notes
/// totp: otpauth://totp/...
/// tags: email, personal
/// ```
pub fn parse(raw: &str) -> EntryFields {
    let mut fields = EntryFields::default();
    let mut lines = raw.lines();

    // First line is always the password
    if let Some(first) = lines.next() {
        fields.password = first.to_string();
    }

    let mut notes_lines: Vec<String> = Vec::new();
    let mut in_notes = false;

    for line in lines {
        if in_notes {
            // Once we're in notes mode, everything is notes unless we hit a recognized key
            if let Some((key, _)) = parse_key_value(line) {
                if is_recognized_key(&key) {
                    in_notes = false;
                    // Fall through to key processing below
                } else {
                    notes_lines.push(line.to_string());
                    continue;
                }
            } else {
                notes_lines.push(line.to_string());
                continue;
            }
        }

        if let Some((key, value)) = parse_key_value(line) {
            match normalize_key(&key) {
                Some(NormalizedKey::Username) => fields.username = Some(value),
                Some(NormalizedKey::Url) => fields.url = Some(value),
                Some(NormalizedKey::Totp) => fields.totp = Some(value),
                Some(NormalizedKey::Tags) => {
                    fields.tags = value
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                Some(NormalizedKey::Notes) => {
                    in_notes = true;
                    if !value.is_empty() {
                        notes_lines.push(value);
                    }
                }
                None => {
                    fields.extra.push((key, value));
                }
            }
        } else if !line.is_empty() {
            // Lines without key: value format go to extra
            fields.extra.push((String::new(), line.to_string()));
        }
    }

    if !notes_lines.is_empty() {
        fields.notes = Some(notes_lines.join("\n"));
    }

    fields
}

/// Serialize structured fields back to raw pass format
pub fn serialize(fields: &EntryFields) -> String {
    let mut lines = Vec::new();

    lines.push(fields.password.clone());

    if let Some(ref username) = fields.username {
        if !username.is_empty() {
            lines.push(format!("username: {}", username));
        }
    }

    if let Some(ref url) = fields.url {
        if !url.is_empty() {
            lines.push(format!("url: {}", url));
        }
    }

    if let Some(ref totp) = fields.totp {
        if !totp.is_empty() {
            lines.push(format!("totp: {}", totp));
        }
    }

    if !fields.tags.is_empty() {
        lines.push(format!("tags: {}", fields.tags.join(", ")));
    }

    for (key, value) in &fields.extra {
        if key.is_empty() {
            lines.push(value.clone());
        } else {
            lines.push(format!("{}: {}", key, value));
        }
    }

    if let Some(ref notes) = fields.notes {
        if !notes.is_empty() {
            lines.push(format!("notes: {}", notes));
        }
    }

    lines.join("\n")
}

fn parse_key_value(line: &str) -> Option<(String, String)> {
    let idx = line.find(':')?;
    let key = line[..idx].trim().to_string();
    let value = line[idx + 1..].trim().to_string();
    if key.is_empty() {
        return None;
    }
    Some((key, value))
}

enum NormalizedKey {
    Username,
    Url,
    Totp,
    Tags,
    Notes,
}

fn normalize_key(key: &str) -> Option<NormalizedKey> {
    match key.to_lowercase().as_str() {
        "username" | "user" | "login" | "email" | "account" => Some(NormalizedKey::Username),
        "url" | "uri" | "site" | "website" | "link" | "homepage" => Some(NormalizedKey::Url),
        "totp" | "otpauth" | "otp" | "2fa" => Some(NormalizedKey::Totp),
        "tags" | "tag" | "labels" | "label" | "categories" | "category" => Some(NormalizedKey::Tags),
        "notes" | "note" | "comment" | "comments" | "description" => Some(NormalizedKey::Notes),
        _ => None,
    }
}

fn is_recognized_key(key: &str) -> bool {
    normalize_key(key).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let raw = "mysecretpassword\nusername: john@example.com\nurl: https://example.com";
        let fields = parse(raw);
        assert_eq!(fields.password, "mysecretpassword");
        assert_eq!(fields.username.as_deref(), Some("john@example.com"));
        assert_eq!(fields.url.as_deref(), Some("https://example.com"));
    }

    #[test]
    fn test_parse_password_only() {
        let fields = parse("justpassword");
        assert_eq!(fields.password, "justpassword");
        assert!(fields.username.is_none());
        assert!(fields.url.is_none());
    }

    #[test]
    fn test_parse_with_aliases() {
        let raw = "pass123\nlogin: user\nsite: https://test.com\ntag: work, dev";
        let fields = parse(raw);
        assert_eq!(fields.username.as_deref(), Some("user"));
        assert_eq!(fields.url.as_deref(), Some("https://test.com"));
        assert_eq!(fields.tags, vec!["work", "dev"]);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let fields = EntryFields {
            password: "secret".to_string(),
            username: Some("user@test.com".to_string()),
            url: Some("https://test.com".to_string()),
            notes: Some("my notes".to_string()),
            totp: None,
            tags: vec!["work".to_string()],
            extra: vec![],
        };
        let serialized = serialize(&fields);
        let parsed = parse(&serialized);
        assert_eq!(parsed.password, "secret");
        assert_eq!(parsed.username.as_deref(), Some("user@test.com"));
        assert_eq!(parsed.url.as_deref(), Some("https://test.com"));
        assert_eq!(parsed.notes.as_deref(), Some("my notes"));
        assert_eq!(parsed.tags, vec!["work"]);
    }
}
