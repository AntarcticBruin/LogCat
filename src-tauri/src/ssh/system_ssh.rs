use super::error::{AppError, AppResult};
use super::types::ConnectOptions;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn shell_escape(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_alphanumeric() || ch == '/' || ch == '_' || ch == '-' || ch == '.')
    {
        return value.to_string();
    }

    let mut escaped = String::from("'");
    for ch in value.chars() {
        if ch == '\'' {
            escaped.push_str("'\\''");
        } else {
            escaped.push(ch);
        }
    }
    escaped.push('\'');
    escaped
}

fn ssh_target(opts: &ConnectOptions) -> String {
    format!("{}@{}", opts.username, opts.host)
}

fn ensure_key_mode_supported(opts: &ConnectOptions) -> AppResult<PathBuf> {
    match &opts.auth {
        super::types::Auth::Key {
            key_path,
            passphrase,
        } => {
            if passphrase.as_deref().is_some_and(|value| !value.is_empty()) {
                return Err(AppError::Message(
                    "Passphrase-protected SSH keys are not directly supported by the new system-ssh backend yet. Load the key into ssh-agent first, or use an unencrypted private key for this app.".into(),
                ));
            }
            Ok(PathBuf::from(key_path))
        }
        _ => Err(AppError::Message(
            "system ssh backend requires key authentication".into(),
        )),
    }
}

pub fn build_ssh_command(opts: &ConnectOptions, remote_command: &str) -> AppResult<Command> {
    let key_path = ensure_key_mode_supported(opts)?;
    let mut command = Command::new("ssh");
    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);
    command
        .arg("-o")
        .arg("BatchMode=yes")
        .arg("-o")
        .arg("StrictHostKeyChecking=accept-new")
        .arg("-o")
        .arg("IdentitiesOnly=yes")
        .arg("-o")
        .arg("ConnectTimeout=10")
        .arg("-p")
        .arg(opts.port.to_string())
        .arg("-i")
        .arg(key_path)
        .arg(ssh_target(opts))
        .arg(remote_command);
    Ok(command)
}

pub fn run_ssh_command(opts: &ConnectOptions, remote_command: &str) -> AppResult<Output> {
    let output = build_ssh_command(opts, remote_command)?.output()?;
    if output.status.success() {
        return Ok(output);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Err(AppError::Message(format!(
        "system ssh command failed with status {:?}. stderr={}, stdout={}",
        output.status.code(),
        stderr,
        stdout
    )))
}

pub fn spawn_ssh_command(opts: &ConnectOptions, remote_command: &str) -> AppResult<std::process::Child> {
    let child = build_ssh_command(opts, remote_command)?
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    Ok(child)
}
