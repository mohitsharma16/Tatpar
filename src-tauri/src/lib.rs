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
            // ── Open the settings database (must happen before anything
            //    below reads hotkey/window settings) ────────────────────
            app.manage(settings::init(app.handle()));

            // ── System tray ───────────────────────────────
            if let Err(e) = tray::setup_tray(app.handle()) {
                eprintln!("[Tatpar] Tray setup failed: {e}");
            }

            // ── Global hotkey ─────────────────────────────
            if let Err(e) = hotkey::register_global_hotkey(app.handle()) {
                eprintln!("[Tatpar] Hotkey registration failed: {e}");
            }

            // ── Restore window position/size, then reveal it ──
            // (window is created hidden — see tauri.conf.json — so the
            // restore below happens before the user ever sees it move)
            window::restore_window_state(app.handle());
            window::start_autosave_loop(app.handle());

            // ── Track + intercept window events ───────────
            // Moved/Resized: record geometry for the autosave loop.
            // CloseRequested: hide instead of closing (keeps the process
            // alive in the tray) and flush geometry immediately.
            let main_window = app.get_webview_window("main")
                .expect("main window not found");
            let app_handle = app.handle().clone();
            let win_clone = main_window.clone();
            main_window.on_window_event(move |event| match event {
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    window::flush_pending_geometry(&app_handle);
                    let _ = win_clone.hide();
                }
                WindowEvent::Moved(position) => {
                    let size = win_clone.inner_size().unwrap_or_default();
                    window::record_geometry_change(
                        &win_clone,
                        position.x,
                        position.y,
                        size.width,
                        size.height,
                    );
                }
                WindowEvent::Resized(size) => {
                    let position = win_clone.outer_position().unwrap_or_default();
                    window::record_geometry_change(
                        &win_clone,
                        position.x,
                        position.y,
                        size.width,
                        size.height,
                    );
                }
                _ => {}
            });

            Ok(())
        })
        .manage(execution::ExecutionState::default())
        .manage(window::WindowStateTracker::default())
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
