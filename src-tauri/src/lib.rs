// ============================================================
// Tatpar — Rust Backend Entry Point
// Registers all Tauri commands and configures plugins
// ============================================================

mod execution;
mod settings;
mod window;
// mod hotkey; // TODO: Phase 3

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Load persisted settings on startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = settings::init_settings(&app_handle).await {
                    eprintln!("[Tatpar] Failed to init settings: {e}");
                }
            });
            Ok(())
        })
        .manage(execution::ExecutionState::default())
        .invoke_handler(tauri::generate_handler![
            // Execution
            execution::execute_code,
            execution::cancel_execution,
            execution::check_languages,
            execution::get_compiler_path,
            // Settings
            settings::load_settings,
            settings::save_settings,
            // Window
            window::set_always_on_top,
            window::minimize_to_tray,
            window::save_window_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tatpar");
}
