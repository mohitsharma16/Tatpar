// ============================================================
// Tatpar — Settings Persistence
// Loads and saves user preferences using a JSON file.
// Phase 3 will upgrade this to SQLite.
// ============================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{command, AppHandle, Manager};

// ─── Types (mirror src/types/index.ts) ───────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageSettings {
    pub timeout_secs: u64,
    pub network_enabled: bool,
    pub compiler_path: Option<String>,
}

impl Default for LanguageSettings {
    fn default() -> Self {
        Self {
            timeout_secs: 10,
            network_enabled: false,
            compiler_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSettings {
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub always_on_top: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            width: 600,
            height: 700,
            x: None,
            y: None,
            always_on_top: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub hotkey: String,
    pub theme: String,
    pub editor_font_size: u32,
    pub launch_on_startup: bool,
    pub language_settings: HashMap<String, LanguageSettings>,
    pub window: WindowSettings,
}

impl Default for Settings {
    fn default() -> Self {
        let mut lang_settings = HashMap::new();
        for lang in ["kotlin", "python", "java", "javascript", "typescript", "cpp"] {
            lang_settings.insert(lang.to_string(), LanguageSettings::default());
        }
        Self {
            hotkey: "ctrl+shift+space".to_string(),
            theme: "dark".to_string(),
            editor_font_size: 14,
            launch_on_startup: false,
            language_settings: lang_settings,
            window: WindowSettings::default(),
        }
    }
}

// ─── File Path Helper ─────────────────────────────────────────

fn settings_path(app: &AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .expect("No app data dir")
        .join("settings.json")
}

// ─── Init ─────────────────────────────────────────────────────

/// Called once on app startup (synchronously, before window setup) to
/// ensure the settings file exists.
pub fn init_settings(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings_path(app);
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let defaults = Settings::default();
        let json = serde_json::to_string_pretty(&defaults)?;
        std::fs::write(&path, json)?;
        println!("[Tatpar] Created default settings at {}", path.display());
    }
    Ok(())
}

/// Read settings from disk synchronously. Returns defaults if missing/corrupt.
pub fn read_settings_sync(app: &AppHandle) -> Settings {
    let path = settings_path(app);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

/// Read just the window settings synchronously — used to restore window
/// geometry at startup before the frontend has loaded.
pub fn read_window_settings(app: &AppHandle) -> WindowSettings {
    read_settings_sync(app).window
}

/// Update only the window geometry (x, y, width, height) on disk, leaving
/// every other setting — including `window.always_on_top` — untouched.
///
/// Geometry is owned by the native window-event tracker in `window.rs`, so
/// this is the only writer for those four fields; `save_settings` (driven by
/// the frontend) deliberately never touches them, which avoids a stale
/// in-memory copy of the window rect clobbering a position the user just
/// dragged the window to.
pub fn persist_window_geometry(
    app: &AppHandle,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let mut settings = read_settings_sync(app);
    settings.window.x = Some(x);
    settings.window.y = Some(y);
    settings.window.width = width;
    settings.window.height = height;
    write_settings_sync(app, &settings)
}

fn write_settings_sync(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let path = settings_path(app);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

// ─── Tauri Commands ───────────────────────────────────────────

/// Load settings from disk. Returns defaults if file is missing/corrupt.
#[command]
pub async fn load_settings(app: AppHandle) -> Result<Settings, String> {
    Ok(read_settings_sync(&app))
}

/// Save settings to disk. Window geometry (x/y/width/height) is preserved
/// from the existing file regardless of what the caller sends — see
/// `persist_window_geometry` for why.
#[command]
pub async fn save_settings(app: AppHandle, mut settings: Settings) -> Result<(), String> {
    let on_disk = read_settings_sync(&app);
    settings.window.x = on_disk.window.x;
    settings.window.y = on_disk.window.y;
    settings.window.width = on_disk.window.width;
    settings.window.height = on_disk.window.height;
    write_settings_sync(&app, &settings)
}
