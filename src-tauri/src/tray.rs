// ============================================================
// Tatpar — System Tray (Phase 3 Step 3)
// Keeps Tatpar alive in the background after the window is
// "closed". Left-click toggles visibility; menu has Show + Quit.
// ============================================================

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

// ─── Public API ───────────────────────────────────────────────

/// Build and register the system tray icon.
/// Call once from lib.rs setup().
pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, "show", "Show Tatpar", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit Tatpar", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_item, &separator, &quit_item])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Tatpar — press Ctrl+Shift+Space to open")
        .menu(&menu)
        .show_menu_on_left_click(false) // left-click = toggle, right-click = menu
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_window(app),
            "quit" => {
                println!("[Tatpar] Quit via tray menu");
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // Left-click the tray icon → toggle window visibility
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        show_window(app);
                    }
                }
            }
        })
        .build(app)?;

    println!("[Tatpar] System tray ready");
    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
