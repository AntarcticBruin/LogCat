mod ssh;
mod window;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ssh::AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            ssh::connect_ssh,
            ssh::disconnect_ssh,
            ssh::list_dir,
            ssh::start_tail,
            ssh::stop_tail,
            window::show_main_window,
            window::default_ssh_key_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
