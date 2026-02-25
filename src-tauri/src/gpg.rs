use std::path::Path;

use crate::platform::{augmented_path, gpg_binary, run_command, run_command_stdin};
use crate::types::GpgKey;

pub fn decrypt_file(path: &Path, passphrase: Option<&str>) -> Result<String, String> {
    let gpg = gpg_binary();

    let mut cmd = std::process::Command::new(gpg);
    cmd.args(["--batch", "--no-tty", "--quiet", "--yes", "--decrypt"]);

    if passphrase.is_some() {
        cmd.args(["--pinentry-mode", "loopback", "--passphrase-fd", "0"]);
        cmd.stdin(std::process::Stdio::piped());
    } else {
        cmd.stdin(std::process::Stdio::null());
    }

    cmd.arg(path)
        .env("PATH", augmented_path())
        .env("LC_ALL", "C")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn gpg: {}", e))?;

    if let Some(pw) = passphrase {
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(pw.as_bytes())
                .map_err(|e| format!("Failed to write passphrase to gpg: {}", e))?;
            stdin.write_all(b"\n").map_err(|e| format!("Failed to write newline to gpg: {}", e))?;
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for gpg: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(if stderr.trim().is_empty() {
            "GPG decryption failed".to_string()
        } else {
            stderr
        })
    }
}

pub fn encrypt_to_file(content: &str, gpg_id: &str, dest: &Path) -> Result<(), String> {
    use std::io::Write;

    let gpg = gpg_binary();
    let dest_str = dest.to_string_lossy();

    let mut child = std::process::Command::new(gpg)
        .args([
            "--batch", "--no-tty", "--quiet", "--yes", "-r", gpg_id, "--encrypt", "--output", &dest_str,
        ])
        .env("PATH", augmented_path())
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run gpg encrypt: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write to gpg stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for gpg: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(if stderr.trim().is_empty() {
            "GPG encryption failed".to_string()
        } else {
            stderr
        })
    }
}

pub fn list_secret_keys() -> Result<Vec<GpgKey>, String> {
    let gpg = gpg_binary();
    let output = run_command(gpg, &["--list-secret-keys", "--with-colons"])
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
                    current_key_id = String::new();
                    current_fingerprint = String::new();
                }
            }
            _ => {}
        }
    }

    Ok(keys)
}

pub fn generate_key(name: &str, email: &str, passphrase: &str) -> Result<GpgKey, String> {
    let gpg = gpg_binary();

    let key_params = if passphrase.is_empty() {
        format!(
            "%no-protection\nKey-Type: RSA\nKey-Length: 4096\nSubkey-Type: RSA\nSubkey-Length: 4096\nName-Real: {}\nName-Email: {}\nExpire-Date: 0\n%commit\n",
            name, email
        )
    } else {
        format!(
            "Key-Type: RSA\nKey-Length: 4096\nSubkey-Type: RSA\nSubkey-Length: 4096\nName-Real: {}\nName-Email: {}\nExpire-Date: 0\nPassphrase: {}\n%commit\n",
            name, email, passphrase
        )
    };

    run_command_stdin(gpg, &["--batch", "--gen-key"], &key_params)
        .map_err(|e| format!("Failed to generate GPG key: {}", e))?;

    let keys = list_secret_keys()?;
    keys.into_iter()
        .find(|k| k.uid.contains(email))
        .ok_or_else(|| "Key was generated but could not be found".to_string())
}

pub fn read_gpg_id(store_path: &Path) -> Option<String> {
    std::fs::read_to_string(store_path.join(".gpg-id"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
