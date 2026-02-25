use std::path::PathBuf;
use std::process::Command;

use crate::types::DependencyStatus;

pub fn home_dir() -> PathBuf {
    dirs::home_dir()
        .or_else(|| std::env::var("HOME").ok().map(PathBuf::from))
        .unwrap_or_else(|| {
            #[cfg(windows)]
            {
                std::env::var("USERPROFILE")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default"))
            }
            #[cfg(not(windows))]
            {
                PathBuf::from("/tmp")
            }
        })
}

pub fn store_dir() -> PathBuf {
    std::env::var("PASSWORD_STORE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().join(".password-store"))
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| home_dir().join(".config"))
        .join("pass-gui")
}

pub fn augmented_path() -> String {
    let current = std::env::var("PATH").unwrap_or_default();

    #[cfg(target_os = "macos")]
    {
        format!("/opt/homebrew/bin:/usr/local/bin:{}", current)
    }

    #[cfg(target_os = "windows")]
    {
        // Add common Gpg4win and Git for Windows paths
        let program_files = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".into());
        let program_files_x86 = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".into());
        format!(
            "{}\\GnuPG\\bin;{}\\Git\\cmd;{}\\GnuPG\\bin;{}",
            program_files, program_files, program_files_x86, current
        )
    }

    #[cfg(target_os = "linux")]
    {
        current
    }
}

pub fn gpg_binary() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "gpg"
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Prefer gpg2 if available, fall back to gpg
        if Command::new("gpg2")
            .arg("--version")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok()
        {
            "gpg2"
        } else {
            "gpg"
        }
    }
}

pub fn is_wizard_done() -> bool {
    config_dir().join("configured").exists()
}

pub fn run_command(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .env("PATH", augmented_path())
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("Failed to run {}: {}", program, e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Err(if stderr.trim().is_empty() {
            stdout
        } else {
            stderr
        })
    }
}

pub fn run_command_stdin(program: &str, args: &[&str], input: &str) -> Result<String, String> {
    use std::io::Write;
    let mut child = Command::new(program)
        .args(args)
        .env("PATH", augmented_path())
        .env("LC_ALL", "C")
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

pub fn check_dependency(name: &str, version_args: &[&str]) -> DependencyStatus {
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
