// ============================================================
// Tatpar — Rust Backend Entry Point
// Registers all Tauri commands and configures plugins
// ============================================================

mod execution;
mod hotkey;
mod settings;
mod tray;
mod window;

use tauri::{Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // ── System tray ───────────────────────────────
            if let Err(e) = tray::setup_tray(app.handle()) {
                eprintln!("[Tatpar] Tray setup failed: {e}");
            }

            // ── Global hotkey ─────────────────────────────
            if let Err(e) = hotkey::register_global_hotkey(app.handle()) {
                eprintln!("[Tatpar] Hotkey registration failed: {e}");
            }

            // ── Intercept window close → hide instead ─────
            // This keeps the process alive in the tray so the
            // global hotkey keeps working after ✕ is clicked.
            let main_window = app.get_webview_window("main")
                .expect("main window not found");
            let win_clone = main_window.clone();
            main_window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = win_clone.hide();
                }
            });

            // ── Load persisted settings ───────────────────
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
            // Hotkey
            hotkey::update_hotkey,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tatpar");
}
