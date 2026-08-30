// ============================================================
// Tatpar — Window Management
// Tauri commands for always-on-top, tray, position persistence
// ============================================================

use tauri::{command, AppHandle, Manager, WebviewWindow};

/// Set the always-on-top property of the main window.
#[command]
pub async fn set_always_on_top(
    app: AppHandle,
    always_on_top: bool,
) -> Result<(), String> {
    let window = get_main_window(&app)?;
    window
        .set_always_on_top(always_on_top)
        .map_err(|e| e.to_string())
}

/// Minimize the window (hide it; system tray integration in Phase 3).
#[command]
pub async fn minimize_to_tray(app: AppHandle) -> Result<(), String> {
    let window = get_main_window(&app)?;
    window.hide().map_err(|e| e.to_string())
}

/// Save current window position and size to be restored on next launch.
/// The values are persisted via the settings module.
#[command]
pub async fn save_window_state(
    app: AppHandle,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    // Persist via settings (Phase 3 will wire this to SQLite)
    // For now we just log — full impl comes in Phase 3
    let _ = (x, y, width, height);
    let _ = app;
    println!("[Tatpar] Window state: x={x} y={y} w={width} h={height}");
    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────

fn get_main_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())
}
