// ============================================================
// Tatpar — Window Management (Phase 3 Step 4)
// Always-on-top toggle, plus window position/size persistence:
// geometry is restored on startup and continuously tracked via
// native Moved/Resized events so it survives quit/relaunch.
// ============================================================

use std::sync::Mutex;
use std::time::Duration;
use tauri::{command, AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

/// Minimum pixels of the saved top-left corner that must land on some
/// connected monitor before we trust it — guards against restoring a
/// position from a monitor that's since been unplugged.
const VISIBILITY_MARGIN: i32 = 80;

/// How often the autosave loop flushes a pending geometry change to disk.
const AUTOSAVE_INTERVAL: Duration = Duration::from_millis(700);

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

/// Persist window position and size immediately.
#[command]
pub async fn save_window_state(
    app: AppHandle,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    crate::settings::persist_window_geometry(&app, x, y, width, height)
}

// ─── Geometry Tracker ───────────────────────────────────────────
//
// Moved/Resized events fire continuously while the user drags or resizes
// the window. Writing to disk on every single event would hammer the
// filesystem, so the event handler just records the latest geometry here,
// and a background loop (started once from lib.rs) flushes it periodically.

#[derive(Default)]
pub struct WindowStateTracker(Mutex<Option<PendingGeometry>>);

struct PendingGeometry {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

/// Call from the window's Moved/Resized event handlers.
///
/// Windows reports a minimized window's rect as the sentinel (-32000, -32000)
/// with a 0×0 size — if we recorded that verbatim, minimizing to the taskbar
/// (a normal user action, distinct from our hide-to-tray) would silently
/// overwrite the last good position with garbage. Skip anything that looks
/// like that sentinel, or an actually-minimized window.
pub fn record_geometry_change(
    window: &WebviewWindow,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) {
    if width == 0 || height == 0 || x <= -30000 || y <= -30000 {
        return;
    }
    if window.is_minimized().unwrap_or(false) {
        return;
    }

    if let Some(tracker) = window.try_state::<WindowStateTracker>() {
        if let Ok(mut pending) = tracker.0.lock() {
            *pending = Some(PendingGeometry { x, y, width, height });
        }
    }
}

/// Write any pending geometry change to disk now. Safe to call often —
/// it's a no-op when nothing changed since the last flush.
pub fn flush_pending_geometry(app: &AppHandle) {
    let pending = app
        .try_state::<WindowStateTracker>()
        .and_then(|tracker| tracker.0.lock().ok().and_then(|mut g| g.take()));

    if let Some(g) = pending {
        if let Err(e) = crate::settings::persist_window_geometry(app, g.x, g.y, g.width, g.height)
        {
            eprintln!("[Tatpar] Failed to persist window geometry: {e}");
        }
    }
}

/// Start the periodic autosave loop. Call once from lib.rs setup().
pub fn start_autosave_loop(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(AUTOSAVE_INTERVAL).await;
            flush_pending_geometry(&app);
        }
    });
}

// ─── Startup Restore ─────────────────────────────────────────────

/// Apply the persisted window geometry/always-on-top state and reveal the
/// window. Call once from lib.rs setup(), before the window is shown.
pub fn restore_window_state(app: &AppHandle) {
    let window = match get_main_window(app) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[Tatpar] Cannot restore window state: {e}");
            return;
        }
    };

    let saved = crate::settings::read_window_settings(app);

    let _ = window.set_size(PhysicalSize::new(saved.width, saved.height));

    if let (Some(x), Some(y)) = (saved.x, saved.y) {
        if is_position_reachable(&window, x, y) {
            let _ = window.set_position(PhysicalPosition::new(x, y));
        } else {
            println!(
                "[Tatpar] Saved window position ({x}, {y}) is off-screen; keeping default position"
            );
        }
    }

    let _ = window.set_always_on_top(saved.always_on_top);
    let _ = window.show();
}

/// Whether (x, y) — the window's top-left corner — lands close enough to a
/// connected monitor's bounds to be reachable by the user.
fn is_position_reachable(window: &WebviewWindow, x: i32, y: i32) -> bool {
    let monitors = match window.available_monitors() {
        Ok(m) => m,
        Err(_) => return false,
    };

    monitors.iter().any(|m| {
        let pos = m.position();
        let size = m.size();
        let min_x = pos.x;
        let max_x = pos.x + size.width as i32;
        let min_y = pos.y;
        let max_y = pos.y + size.height as i32;
        x + VISIBILITY_MARGIN > min_x && x < max_x && y + VISIBILITY_MARGIN > min_y && y < max_y
    })
}

// ─── Visibility Helpers ────────────────────────────────────────
//
// Shared by the global hotkey and the tray icon so both agree on what
// "toggle"/"show" mean for a minimized window.

/// Toggle the main window between hidden and shown.
///
/// A minimized window still reports `is_visible() == true` on Windows —
/// naively treating that as "already shown" and calling `hide()` would
/// vanish it into the tray instead of restoring it, leaving the hotkey/tray
/// click looking like it did nothing while the window sits minimized.
pub fn toggle_visibility(app: &AppHandle) {
    let Ok(window) = get_main_window(app) else { return };

    if window.is_minimized().unwrap_or(false) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    } else if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Unconditionally bring the main window to the front, restoring it first
/// if it's minimized. Used by the tray's "Show" menu item.
pub fn show_and_focus(app: &AppHandle) {
    let Ok(window) = get_main_window(app) else { return };

    if window.is_minimized().unwrap_or(false) {
        let _ = window.unminimize();
    }
    let _ = window.show();
    let _ = window.set_focus();
}

// ─── Helpers ─────────────────────────────────────────────────

fn get_main_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())
}
