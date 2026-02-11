mod commands;
mod state;
mod types;
mod history;
mod nlbn;
mod converter_impl;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    log::info!("Starting NLBN GUI application");

    // Create application state
    let app_state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::convert_component,
            commands::batch_convert,
            commands::select_output_directory,
            commands::get_history,
            commands::clear_history,
            commands::import_ids_from_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
