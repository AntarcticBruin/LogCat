use super::error::{AppError, AppResult};
use super::parsing::{
    is_probably_binary_file, is_probably_text_file, should_probe_text_file, sort_entries,
};
use super::session::get_conn;
use super::state::AppState;
use super::types::{DirEntry, EntryKind, TransferProgressEvent};
use russh::ChannelMsg;
use russh_sftp::client::SftpSession;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const MAX_PROBED_FILES_PER_DIR: usize = 8;
const MAX_PROBED_FILE_SIZE: u64 = 64 * 1024;

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn perm_to_kind(perm: Option<u32>) -> EntryKind {
    if let Some(perm) = perm {
        let file_type = perm & 0o170000;
        if file_type == 0o040000 {
            return EntryKind::Dir;
        }
        if file_type == 0o100000 {
            return EntryKind::File;
        }
        if file_type == 0o120000 {
            return EntryKind::Symlink;
        }
    }
    EntryKind::Other
}

async fn resolve_entry_kind(sftp: &SftpSession, path: &str, kind: EntryKind) -> EntryKind {
    if kind != EntryKind::Symlink {
        return kind;
    }

    match sftp.metadata(path).await {
        Ok(metadata) => {
            let resolved_kind = perm_to_kind(metadata.permissions);
            if resolved_kind == EntryKind::Other || resolved_kind == EntryKind::Symlink {
                EntryKind::Symlink
            } else {
                resolved_kind
            }
        }
        Err(_) => EntryKind::Symlink,
    }
}

async fn is_text_file(sftp: &SftpSession, path: &str) -> bool {
    if is_probably_text_file(path) {
        return true;
    }

    if is_probably_binary_file(path) {
        return false;
    }

    if !should_probe_text_file(path) {
        return false;
    }

    if let Ok(mut file) = sftp.open(path).await {
        let mut buffer = [0u8; 1024];
        // Read into buffer using tokio ReadExt since SftpFile implements tokio::io::AsyncRead
        return match file.read(&mut buffer).await {
            Ok(0) => true,
            Ok(read_bytes) => !buffer[..read_bytes].contains(&0),
            Err(_) => false,
        };
    }

    true
}

fn should_probe_file_contents(path: &str, size: Option<u64>, probed_files: usize) -> bool {
    should_probe_text_file(path)
        && probed_files < MAX_PROBED_FILES_PER_DIR
        && size.unwrap_or(0) <= MAX_PROBED_FILE_SIZE
}

pub async fn list_directory(
    state: &AppState,
    session_id: String,
    path: String,
) -> AppResult<Vec<DirEntry>> {
    let conn = get_conn(state, &session_id)?;

    // We must acquire the session to open a new channel
    let handle = conn.handle.lock().await;
    let channel = handle.channel_open_session().await?;
    channel.request_subsystem(true, "sftp").await?;
    let sftp = SftpSession::new(channel.into_stream()).await?;
    drop(handle);

    let mut items = Vec::new();
    let mut probed_files = 0usize;

    let entries = sftp.read_dir(&path).await?;

    for entry in entries {
        let name = entry.file_name().to_string();
        if name.is_empty() || name == "." || name == ".." {
            continue;
        }

        let stat = entry.metadata();
        let raw_kind = perm_to_kind(stat.permissions);
        
        // 拼接成全路径：处理好 path 末尾的斜杠
        let full_path = if path.ends_with('/') {
            format!("{}{}", path, name)
        } else {
            format!("{}/{}", path, name)
        };
        
        let kind = resolve_entry_kind(&sftp, &full_path, raw_kind.clone()).await;
        let size_val = stat.size;
        let is_text = matches!(kind, EntryKind::File)
            && if is_probably_text_file(&full_path) {
                true
            } else if is_probably_binary_file(&full_path) {
                false
            } else if should_probe_file_contents(&full_path, size_val, probed_files) {
                probed_files += 1;
                is_text_file(&sftp, &full_path).await
            } else {
                false
            };

        items.push(DirEntry {
            name,
            path: full_path,
            kind,
            is_symlink: raw_kind == EntryKind::Symlink,
            is_text,
            mode: stat.permissions.map(|perm| perm & 0o7777),
            size: size_val,
            mtime: stat.mtime.map(u64::from),
        });
    }

    sort_entries(&mut items);
    Ok(items)
}

pub async fn upload_file(
    state: &AppState,
    app: AppHandle,
    session_id: String,
    local_path: String,
    remote_path: String,
) -> AppResult<()> {
    let conn = get_conn(state, &session_id)?;

    let  handle = conn.handle.lock().await;
    let channel = handle.channel_open_session().await?;
    channel.request_subsystem(true, "sftp").await?;
    let sftp = SftpSession::new(channel.into_stream()).await?;
    drop(handle);

    let mut local_file = tokio::fs::File::open(&local_path).await?;
    let file_meta = local_file.metadata().await?;
    let total_size = file_meta.len();
    
    let mut remote_file = sftp.create(remote_path).await?;

    let file_name = std::path::Path::new(&local_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut buffer = vec![0; 512 * 1024];
    let mut transferred = 0u64;
    let mut last_emitted = 0u64;

    loop {
        let n = local_file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        
        remote_file.write_all(&buffer[..n]).await?;
        transferred += n as u64;

        // emit progress every 1% or if finished
        if transferred == total_size || transferred - last_emitted >= (total_size / 100).max(1024 * 1024) {
            let _ = app.emit("transfer_progress", TransferProgressEvent {
                session_id: session_id.clone(),
                file_name: file_name.clone(),
                transferred,
                total: total_size,
            });
            last_emitted = transferred;
        }
    }

    Ok(())
}

pub async fn download_file(
    state: &AppState,
    app: AppHandle,
    session_id: String,
    remote_path: String,
    local_path: String,
) -> AppResult<()> {
    let conn = get_conn(state, &session_id)?;

    let handle = conn.handle.lock().await;
    let channel = handle.channel_open_session().await?;
    channel.request_subsystem(true, "sftp").await?;
    let sftp = SftpSession::new(channel.into_stream()).await?;
    drop(handle);

    let mut remote_file = sftp.open(&remote_path).await?;
    let file_meta = remote_file.metadata().await?;
    let total_size = file_meta.size.unwrap_or(0);
    
    let mut local_file = tokio::fs::File::create(&local_path).await?;

    let file_name = remote_path.rsplit('/').next().unwrap_or(&remote_path).to_string();

    let mut buffer = vec![0; 512 * 1024];
    let mut transferred = 0u64;
    let mut last_emitted = 0u64;

    loop {
        let n = remote_file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        
        local_file.write_all(&buffer[..n]).await?;
        transferred += n as u64;

        if total_size > 0 && (transferred == total_size || transferred - last_emitted >= (total_size / 100).max(1024 * 1024)) {
            let _ = app.emit("transfer_progress", TransferProgressEvent {
                session_id: session_id.clone(),
                file_name: file_name.clone(),
                transferred,
                total: total_size,
            });
            last_emitted = transferred;
        } else if total_size == 0 && transferred - last_emitted >= 1024 * 1024 {
            // If total size is unknown, emit every 1MB
             let _ = app.emit("transfer_progress", TransferProgressEvent {
                session_id: session_id.clone(),
                file_name: file_name.clone(),
                transferred,
                total: 0,
            });
            last_emitted = transferred;
        }
    }

    Ok(())
}

pub async fn rename_entry(
    state: &AppState,
    session_id: String,
    old_path: String,
    new_path: String,
) -> AppResult<()> {
    let conn = get_conn(state, &session_id)?;

    let handle = conn.handle.lock().await;
    let channel = handle.channel_open_session().await?;
    channel.request_subsystem(true, "sftp").await?;
    let sftp = SftpSession::new(channel.into_stream()).await?;
    drop(handle);

    sftp.rename(&old_path, &new_path).await?;
    Ok(())
}

async fn exec_remote_command(state: &AppState, session_id: String, command: String) -> AppResult<()> {
    let conn = get_conn(state, &session_id)?;
    let handle = conn.handle.lock().await;
    let mut channel = handle.channel_open_session().await?;
    drop(handle);

    channel.exec(true, command.clone()).await?;

    let mut stderr = String::new();
    let mut exit_code = None;

    while let Some(message) = channel.wait().await {
        match message {
            ChannelMsg::ExitStatus { exit_status } => {
                exit_code = Some(exit_status);
            }
            ChannelMsg::ExtendedData { data, .. } => {
                stderr.push_str(&String::from_utf8_lossy(&data));
            }
            ChannelMsg::Close | ChannelMsg::Eof => break,
            _ => {}
        }
    }

    if exit_code.unwrap_or_default() == 0 {
        Ok(())
    } else {
        let message = if stderr.trim().is_empty() {
            format!("remote command failed: {command}")
        } else {
            stderr.trim().to_string()
        };
        Err(AppError::Message(message))
    }
}

pub async fn chmod_entry(
    state: &AppState,
    session_id: String,
    path: String,
    mode: String,
) -> AppResult<()> {
    let command = format!("chmod {} {}", mode, shell_escape(&path));
    exec_remote_command(state, session_id, command).await
}

pub async fn read_text_file(
    state: &AppState,
    session_id: String,
    path: String,
) -> AppResult<String> {
    let conn = get_conn(state, &session_id)?;

    let handle = conn.handle.lock().await;
    let channel = handle.channel_open_session().await?;
    channel.request_subsystem(true, "sftp").await?;
    let sftp = SftpSession::new(channel.into_stream()).await?;
    drop(handle);

    let mut remote_file = sftp.open(&path).await?;
    let mut buffer = Vec::new();
    remote_file.read_to_end(&mut buffer).await?;
    Ok(String::from_utf8(buffer)?)
}

pub async fn write_text_file(
    state: &AppState,
    session_id: String,
    path: String,
    content: String,
) -> AppResult<()> {
    let conn = get_conn(state, &session_id)?;

    let handle = conn.handle.lock().await;
    let channel = handle.channel_open_session().await?;
    channel.request_subsystem(true, "sftp").await?;
    let sftp = SftpSession::new(channel.into_stream()).await?;
    drop(handle);

    let mut remote_file = sftp.create(path).await?;
    remote_file.write_all(content.as_bytes()).await?;
    remote_file.shutdown().await?;
    Ok(())
}

pub async fn create_file(
    state: &AppState,
    session_id: String,
    path: String,
) -> AppResult<()> {
    let conn = get_conn(state, &session_id)?;

    let handle = conn.handle.lock().await;
    let channel = handle.channel_open_session().await?;
    channel.request_subsystem(true, "sftp").await?;
    let sftp = SftpSession::new(channel.into_stream()).await?;
    drop(handle);

    let mut remote_file = sftp.create(path).await?;
    remote_file.shutdown().await?;
    Ok(())
}

pub async fn create_dir(
    state: &AppState,
    session_id: String,
    path: String,
) -> AppResult<()> {
    let conn = get_conn(state, &session_id)?;

    let handle = conn.handle.lock().await;
    let channel = handle.channel_open_session().await?;
    channel.request_subsystem(true, "sftp").await?;
    let sftp = SftpSession::new(channel.into_stream()).await?;
    drop(handle);

    sftp.create_dir(path).await?;
    Ok(())
}

async fn remove_dir_recursive(sftp: &SftpSession, root_path: &str) -> AppResult<()> {
    let mut stack = vec![(root_path.to_string(), false)];

    while let Some((path, visited)) = stack.pop() {
        if visited {
            sftp.remove_dir(&path).await?;
            continue;
        }

        stack.push((path.clone(), true));

        for entry in sftp.read_dir(&path).await? {
            let name = entry.file_name().to_string();
            if name.is_empty() || name == "." || name == ".." {
                continue;
            }

            let child_path = if path.ends_with('/') {
                format!("{}{}", path, name)
            } else {
                format!("{}/{}", path, name)
            };

            match perm_to_kind(entry.metadata().permissions) {
                EntryKind::Dir => stack.push((child_path, false)),
                _ => {
                    sftp.remove_file(&child_path).await?;
                }
            }
        }
    }

    Ok(())
}

pub async fn delete_entry(
    state: &AppState,
    session_id: String,
    path: String,
    kind: EntryKind,
    is_symlink: bool,
) -> AppResult<()> {
    let conn = get_conn(state, &session_id)?;

    let handle = conn.handle.lock().await;
    let channel = handle.channel_open_session().await?;
    channel.request_subsystem(true, "sftp").await?;
    let sftp = SftpSession::new(channel.into_stream()).await?;
    drop(handle);

    match (kind, is_symlink) {
        (_, true) => sftp.remove_file(&path).await?,
        (EntryKind::Dir, false) => remove_dir_recursive(&sftp, &path).await?,
        _ => sftp.remove_file(&path).await?,
    }

    Ok(())
}
