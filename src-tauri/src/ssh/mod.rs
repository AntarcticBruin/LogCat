mod error;
mod files;
mod parsing;
mod session;
mod state;
mod tail;
mod terminal;
mod types;

pub use state::AppState;
pub use types::*;

use std::sync::atomic::Ordering;
use tauri::{Manager, State};

#[tauri::command]
pub async fn connect_ssh(
    app: tauri::AppHandle,
    _state: State<'_, AppState>,
    opts: ConnectOptions,
) -> Result<ConnectResult, String> {
    let state = app.state::<AppState>();
    let conn = session::establish_for_state(opts).await.map_err(|error| error.to_string())?;
    Ok(session::connect_result_with_conn(conn, state.inner()))
}

#[tauri::command]
pub async fn disconnect_ssh(app: tauri::AppHandle, session_id: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    if let Some(conn) = state.inner().remove_session(&session_id) {
        {
            let mut watchers = conn.watchers.lock().unwrap();
            for (_, watcher) in watchers.drain() {
                watcher.stop.store(true, Ordering::SeqCst);
            }
        }
        if let Some(terminal) = conn.terminal.lock().await.take() {
            terminal.stop.store(true, Ordering::SeqCst);
            let writer = terminal.writer.lock().await;
            let _ = writer.close().await;
        }
        let handle = conn.handle.lock().await;
        let _ = handle.disconnect(russh::Disconnect::ByApplication, "bye", "en-US").await;
    }
    Ok(())
}

#[tauri::command]
pub async fn list_dir(
    app: tauri::AppHandle,
    _state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<Vec<DirEntry>, String> {
    let state = app.state::<AppState>();
    files::list_directory(state.inner(), session_id, path).await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn start_tail(
    app: tauri::AppHandle,
    _state: State<'_, AppState>,
    session_id: String,
    path: String,
    lines: Option<usize>,
) -> Result<String, String> {
    let state = app.state::<AppState>();
    tail::start(state.inner(), app.clone(), session_id, path, lines).await.map_err(|error| error.to_string())
}

#[tauri::command]
pub fn stop_tail(
    state: State<'_, AppState>,
    session_id: String,
    token: String,
) -> Result<(), String> {
    tail::stop(state, session_id, token).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn start_terminal(
    app: tauri::AppHandle,
    _state: State<'_, AppState>,
    session_id: String,
    cols: Option<u32>,
    rows: Option<u32>,
) -> Result<String, String> {
    let state = app.state::<AppState>();
    terminal::start(state.inner(), app.clone(), session_id, cols, rows)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn stop_terminal(
    state: State<'_, AppState>,
    session_id: String,
    token: String,
) -> Result<(), String> {
    terminal::stop(state, session_id, token).await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn write_terminal(
    state: State<'_, AppState>,
    session_id: String,
    token: String,
    data: String,
) -> Result<(), String> {
    terminal::write(state, session_id, token, data)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn resize_terminal(
    state: State<'_, AppState>,
    session_id: String,
    token: String,
    cols: u32,
    rows: u32,
) -> Result<(), String> {
    terminal::resize(state, session_id, token, cols, rows)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn upload_file(
    app: tauri::AppHandle,
    _state: State<'_, AppState>,
    session_id: String,
    local_path: String,
    remote_path: String,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    files::upload_file(state.inner(), app.clone(), session_id, local_path, remote_path)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn download_file(
    app: tauri::AppHandle,
    _state: State<'_, AppState>,
    session_id: String,
    remote_path: String,
    local_path: String,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    files::download_file(state.inner(), app.clone(), session_id, remote_path, local_path)
        .await
        .map_err(|error| error.to_string())
}
