//! desktop-ui-tauri — Tauri application entry point

mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_libsql::Builder::default().build())
        .plugin(tauri_plugin_deep_link::init())
        .invoke_handler(tauri::generate_handler![
            commands::auth::start_oauth,
            commands::auth::handle_oauth_callback,
            commands::auth::get_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
