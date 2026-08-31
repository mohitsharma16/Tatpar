// ============================================================
// Tatpar — Global Hotkey (Phase 3 Step 3)
// Registers Ctrl+Shift+Space system-wide to toggle window
// visibility. Works even while the window is hidden.
// ============================================================

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

// ─── Public API ───────────────────────────────────────────────

/// Register the global hotkey read from settings.json.
/// Falls back to Ctrl+Shift+Space if the file is missing/corrupt.
/// Call this once from lib.rs setup().
pub fn register_global_hotkey(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let hotkey_str = read_hotkey_setting(app);
    println!("[Tatpar] Registering global hotkey: {hotkey_str}");

    let shortcut = parse_hotkey(&hotkey_str)
        .ok_or_else(|| format!("Cannot parse hotkey: {hotkey_str}"))?;

    app.global_shortcut().on_shortcut(shortcut, |app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            toggle_window(app);
        }
    })?;

    Ok(())
}

/// Tauri command: unregister the old hotkey and register a new one.
/// Called from the Settings panel (future customization).
#[tauri::command]
pub async fn update_hotkey(app: AppHandle, hotkey: String) -> Result<(), String> {
    // Unregister everything first
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;

    let shortcut = parse_hotkey(&hotkey)
        .ok_or_else(|| format!("Invalid hotkey string: {hotkey}"))?;

    app.global_shortcut()
        .on_shortcut(shortcut, |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                toggle_window(app);
            }
        })
        .map_err(|e| e.to_string())?;

    println!("[Tatpar] Hotkey updated to: {hotkey}");
    Ok(())
}

// ─── Toggle Helper ────────────────────────────────────────────

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let visible = window.is_visible().unwrap_or(false);
        if visible {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

// ─── Settings Reader ─────────────────────────────────────────

/// Read the hotkey string from the settings database.
fn read_hotkey_setting(app: &AppHandle) -> String {
    crate::settings::read_hotkey(app)
}

// ─── Hotkey String Parser ─────────────────────────────────────

/// Parse "ctrl+shift+space" → Shortcut { modifiers, key_code }.
/// Supports: ctrl, shift, alt, meta/win, and common key names.
fn parse_hotkey(s: &str) -> Option<Shortcut> {
    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    for token in s.split('+') {
        match token.trim().to_lowercase().as_str() {
            "ctrl" | "control"              => modifiers |= Modifiers::CONTROL,
            "shift"                         => modifiers |= Modifiers::SHIFT,
            "alt"                           => modifiers |= Modifiers::ALT,
            "meta" | "win" | "cmd" | "super" => modifiers |= Modifiers::META,
            // Common keys
            "space"                         => key_code = Some(Code::Space),
            "tab"                           => key_code = Some(Code::Tab),
            "enter" | "return"              => key_code = Some(Code::Enter),
            "escape" | "esc"               => key_code = Some(Code::Escape),
            "backspace"                     => key_code = Some(Code::Backspace),
            "delete" | "del"               => key_code = Some(Code::Delete),
            "insert" | "ins"               => key_code = Some(Code::Insert),
            "home"                          => key_code = Some(Code::Home),
            "end"                           => key_code = Some(Code::End),
            "pageup"                        => key_code = Some(Code::PageUp),
            "pagedown"                      => key_code = Some(Code::PageDown),
            // Single letter A–Z
            k if k.len() == 1 => {
                key_code = match k.chars().next()?.to_ascii_uppercase() {
                    'A' => Some(Code::KeyA), 'B' => Some(Code::KeyB),
                    'C' => Some(Code::KeyC), 'D' => Some(Code::KeyD),
                    'E' => Some(Code::KeyE), 'F' => Some(Code::KeyF),
                    'G' => Some(Code::KeyG), 'H' => Some(Code::KeyH),
                    'I' => Some(Code::KeyI), 'J' => Some(Code::KeyJ),
                    'K' => Some(Code::KeyK), 'L' => Some(Code::KeyL),
                    'M' => Some(Code::KeyM), 'N' => Some(Code::KeyN),
                    'O' => Some(Code::KeyO), 'P' => Some(Code::KeyP),
                    'Q' => Some(Code::KeyQ), 'R' => Some(Code::KeyR),
                    'S' => Some(Code::KeyS), 'T' => Some(Code::KeyT),
                    'U' => Some(Code::KeyU), 'V' => Some(Code::KeyV),
                    'W' => Some(Code::KeyW), 'X' => Some(Code::KeyX),
                    'Y' => Some(Code::KeyY), 'Z' => Some(Code::KeyZ),
                    _ => None,
                };
            }
            _ => {
                eprintln!("[Tatpar] Unknown hotkey token: {token}");
            }
        }
    }

    let code = key_code?;
    if modifiers.is_empty() {
        Some(Shortcut::new(None, code))
    } else {
        Some(Shortcut::new(Some(modifiers), code))
    }
}
