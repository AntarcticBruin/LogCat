use super::error::AppResult;
use super::session::get_conn;
use super::state::{AppState, Watcher};
use super::types::TailEvent;
use russh::ChannelMsg;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

fn emit_tail_chunk(app: &AppHandle, token: &str, session_id: &str, path: &str, chunk: String) {
    let payload = TailEvent {
        token: token.to_string(),
        session_id: session_id.to_string(),
        path: path.to_string(),
        chunk,
    };
    let _ = app.emit("tail_data", payload);
}

fn cleanup_watcher(state: &AppState, session_id: &str, token: &str) {
    if let Some(conn) = state.get_session(session_id) {
        if let Ok(mut watchers) = conn.watchers.lock() {
            watchers.remove(token);
        }
    }
}

// Minimal shell escape just in case system_ssh is deleted
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

pub async fn spawn_tail(
    state: &AppState,
    app: AppHandle,
    session_id: String,
    path: String,
    lines: usize,
) -> AppResult<String> {
    let conn = get_conn(state, &session_id)?;
    let token = Uuid::new_v4().to_string();
    let watcher = Arc::new(Watcher {
        stop: std::sync::atomic::AtomicBool::new(false),
    });
    let thread_watcher = watcher.clone();
    let thread_token = token.clone();
    let thread_session_id = session_id.clone();
    let thread_path = path.clone();

    let handle = conn.handle.lock().await;
    let mut channel = handle.channel_open_session().await?;
    drop(handle);

    let command = format!("tail -n {} -F {}", lines, shell_escape(&path));
    channel.exec(true, command).await?;

    tauri::async_runtime::spawn(async move {
        let mut batch_buffer = String::with_capacity(16384);
        let mut interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            if thread_watcher.stop.load(Ordering::SeqCst) {
                let _ = channel.close().await;
                break;
            }

            tokio::select! {
                msg = channel.wait() => {
                    match msg {
                        Some(ChannelMsg::Data { ref data }) => {
                            batch_buffer.push_str(&String::from_utf8_lossy(data));
                        }
                        Some(ChannelMsg::ExtendedData { ref data, .. }) => {
                            batch_buffer.push_str(&String::from_utf8_lossy(data));
                        }
                        Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | None => {
                            break;
                        }
                        _ => {}
                    }
                    if batch_buffer.len() >= 8192 {
                        emit_tail_chunk(
                            &app,
                            &thread_token,
                            &thread_session_id,
                            &thread_path,
                            std::mem::take(&mut batch_buffer),
                        );
                    }
                }
                _ = interval.tick() => {
                    if !batch_buffer.is_empty() {
                        emit_tail_chunk(
                            &app,
                            &thread_token,
                            &thread_session_id,
                            &thread_path,
                            std::mem::take(&mut batch_buffer),
                        );
                    }
                }
            }
        }

        if !batch_buffer.is_empty() {
             emit_tail_chunk(
                 &app,
                 &thread_token,
                 &thread_session_id,
                 &thread_path,
                 std::mem::take(&mut batch_buffer),
             );
        }

        let state = app.state::<AppState>();
        cleanup_watcher(state.inner(), &thread_session_id, &thread_token);
    });

    let mut watchers = conn.watchers.lock().unwrap();
    watchers.insert(token.clone(), watcher);
    Ok(token)
}

pub async fn start(
    state: &AppState,
    app: AppHandle,
    session_id: String,
    path: String,
    lines: Option<usize>,
) -> AppResult<String> {
    let count = lines.unwrap_or(200);
    spawn_tail(state, app, session_id, path, count).await
}

pub fn stop(state: State<'_, AppState>, session_id: String, token: String) -> AppResult<()> {
    let conn = get_conn(state.inner(), &session_id)?;
    let mut watchers = conn.watchers.lock().unwrap();
    if let Some(watcher) = watchers.remove(&token) {
        watcher.stop.store(true, Ordering::SeqCst);
    }
    Ok(())
}
