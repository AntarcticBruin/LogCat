use super::error::AppResult;
use super::parsing::{
    is_probably_binary_file, is_probably_text_file, should_probe_text_file, sort_entries,
};
use super::session::get_conn;
use super::state::AppState;
use super::types::{DirEntry, EntryKind, TransferProgressEvent};
use russh_sftp::client::SftpSession;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const MAX_PROBED_FILES_PER_DIR: usize = 8;
const MAX_PROBED_FILE_SIZE: u64 = 64 * 1024;

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
    let mut handle = conn.handle.lock().await;
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
        let kind = perm_to_kind(stat.permissions);
        
        // 拼接成全路径：处理好 path 末尾的斜杠
        let full_path = if path.ends_with('/') {
            format!("{}{}", path, name)
        } else {
            format!("{}/{}", path, name)
        };
        
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
            is_text,
            size: size_val,
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

    let mut handle = conn.handle.lock().await;
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

    let mut handle = conn.handle.lock().await;
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
