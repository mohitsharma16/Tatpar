// ============================================================
// Tatpar — Settings Persistence (Phase 3 Step 5)
// Loads and saves user preferences in a SQLite database.
// Transparently migrates a legacy settings.json if one exists.
// ============================================================

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
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

const LANGUAGE_IDS: [&str; 6] = ["kotlin", "python", "java", "javascript", "typescript", "cpp"];

impl Default for Settings {
    fn default() -> Self {
        let mut lang_settings = HashMap::new();
        for lang in LANGUAGE_IDS {
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

// ─── Managed State ────────────────────────────────────────────

pub struct SettingsDb(pub Mutex<Connection>);

// ─── Init ─────────────────────────────────────────────────────

/// Open (creating if needed) the settings database, migrating a legacy
/// `settings.json` into it on first run. Call once from lib.rs setup(),
/// before anything else reads settings.
///
/// Falls back to an in-memory database on failure so the app can still
/// run — settings just won't survive a restart — rather than crashing.
pub fn init(app: &AppHandle) -> SettingsDb {
    match open_and_migrate(app) {
        Ok(conn) => SettingsDb(Mutex::new(conn)),
        Err(e) => {
            eprintln!("[Tatpar] Failed to open settings database ({e}); using in-memory settings");
            let conn = Connection::open_in_memory().expect("failed to open in-memory sqlite db");
            create_schema(&conn).expect("failed to create in-memory schema");
            seed_defaults(&conn).expect("failed to seed in-memory defaults");
            SettingsDb(Mutex::new(conn))
        }
    }
}

fn db_path(app: &AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .expect("No app data dir")
        .join("settings.db")
}

fn legacy_json_path(app: &AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .expect("No app data dir")
        .join("settings.json")
}

fn open_and_migrate(app: &AppHandle) -> rusqlite::Result<Connection> {
    let path = db_path(app);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let is_new = !path.exists();
    let conn = Connection::open(&path)?;
    create_schema(&conn)?;

    if is_new {
        let legacy = legacy_json_path(app);
        match std::fs::read_to_string(&legacy) {
            Ok(json) => match serde_json::from_str::<Settings>(&json) {
                Ok(settings) => {
                    write_settings(&conn, &settings)?;
                    // Keep the old file as a backup rather than deleting user data.
                    let _ = std::fs::rename(&legacy, legacy.with_extension("json.migrated"));
                    println!("[Tatpar] Migrated settings.json into settings.db");
                }
                Err(e) => {
                    eprintln!(
                        "[Tatpar] Found settings.json but couldn't parse it ({e}); leaving it \
                         in place and starting settings.db from defaults"
                    );
                    seed_defaults(&conn)?;
                }
            },
            Err(_) => {
                seed_defaults(&conn)?;
                println!("[Tatpar] Created default settings database at {}", path.display());
            }
        }
    }

    Ok(conn)
}

fn create_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS settings (
            id                    INTEGER PRIMARY KEY CHECK (id = 1),
            hotkey                TEXT NOT NULL,
            theme                 TEXT NOT NULL,
            editor_font_size      INTEGER NOT NULL,
            launch_on_startup     INTEGER NOT NULL,
            window_width          INTEGER NOT NULL,
            window_height         INTEGER NOT NULL,
            window_x              INTEGER,
            window_y              INTEGER,
            window_always_on_top  INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS language_settings (
            language        TEXT PRIMARY KEY,
            timeout_secs    INTEGER NOT NULL,
            network_enabled INTEGER NOT NULL,
            compiler_path   TEXT
        );",
    )
}

fn seed_defaults(conn: &Connection) -> rusqlite::Result<()> {
    write_settings(conn, &Settings::default())
}

// ─── Read / Write ──────────────────────────────────────────────

/// Read the full settings row (+ per-language rows) synchronously.
/// Returns defaults if the app state isn't ready yet.
pub fn read_settings_sync(app: &AppHandle) -> Settings {
    match app.try_state::<SettingsDb>() {
        Some(db) => match db.0.lock() {
            Ok(conn) => read_settings(&conn).unwrap_or_default(),
            Err(_) => Settings::default(),
        },
        None => Settings::default(),
    }
}

fn read_settings(conn: &Connection) -> rusqlite::Result<Settings> {
    let mut settings = conn.query_row(
        "SELECT hotkey, theme, editor_font_size, launch_on_startup,
                window_width, window_height, window_x, window_y, window_always_on_top
         FROM settings WHERE id = 1",
        [],
        |row| {
            Ok(Settings {
                hotkey: row.get(0)?,
                theme: row.get(1)?,
                editor_font_size: row.get::<_, i64>(2)? as u32,
                launch_on_startup: row.get::<_, i64>(3)? != 0,
                language_settings: HashMap::new(),
                window: WindowSettings {
                    width: row.get::<_, i64>(4)? as u32,
                    height: row.get::<_, i64>(5)? as u32,
                    x: row.get::<_, Option<i64>>(6)?.map(|v| v as i32),
                    y: row.get::<_, Option<i64>>(7)?.map(|v| v as i32),
                    always_on_top: row.get::<_, i64>(8)? != 0,
                },
            })
        },
    )?;

    let mut stmt = conn
        .prepare("SELECT language, timeout_secs, network_enabled, compiler_path FROM language_settings")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            LanguageSettings {
                timeout_secs: row.get::<_, i64>(1)? as u64,
                network_enabled: row.get::<_, i64>(2)? != 0,
                compiler_path: row.get(3)?,
            },
        ))
    })?;
    for r in rows.flatten() {
        settings.language_settings.insert(r.0, r.1);
    }

    // Backfill any language that never got a row (e.g. added after the DB was created).
    for lang in LANGUAGE_IDS {
        settings
            .language_settings
            .entry(lang.to_string())
            .or_insert_with(LanguageSettings::default);
    }

    Ok(settings)
}

fn write_settings(conn: &Connection, settings: &Settings) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO settings
            (id, hotkey, theme, editor_font_size, launch_on_startup,
             window_width, window_height, window_x, window_y, window_always_on_top)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
            hotkey = excluded.hotkey,
            theme = excluded.theme,
            editor_font_size = excluded.editor_font_size,
            launch_on_startup = excluded.launch_on_startup,
            window_width = excluded.window_width,
            window_height = excluded.window_height,
            window_x = excluded.window_x,
            window_y = excluded.window_y,
            window_always_on_top = excluded.window_always_on_top",
        params![
            settings.hotkey,
            settings.theme,
            settings.editor_font_size,
            settings.launch_on_startup as i64,
            settings.window.width,
            settings.window.height,
            settings.window.x,
            settings.window.y,
            settings.window.always_on_top as i64,
        ],
    )?;

    for (lang, ls) in &settings.language_settings {
        conn.execute(
            "INSERT INTO language_settings (language, timeout_secs, network_enabled, compiler_path)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(language) DO UPDATE SET
                timeout_secs = excluded.timeout_secs,
                network_enabled = excluded.network_enabled,
                compiler_path = excluded.compiler_path",
            params![lang, ls.timeout_secs as i64, ls.network_enabled as i64, ls.compiler_path],
        )?;
    }

    Ok(())
}

/// Read just the window settings — used to restore window geometry at
/// startup before the frontend has loaded.
pub fn read_window_settings(app: &AppHandle) -> WindowSettings {
    let Some(db) = app.try_state::<SettingsDb>() else {
        return WindowSettings::default();
    };
    let Ok(conn) = db.0.lock() else {
        return WindowSettings::default();
    };

    conn.query_row(
        "SELECT window_width, window_height, window_x, window_y, window_always_on_top FROM settings WHERE id = 1",
        [],
        |row| {
            Ok(WindowSettings {
                width: row.get::<_, i64>(0)? as u32,
                height: row.get::<_, i64>(1)? as u32,
                x: row.get::<_, Option<i64>>(2)?.map(|v| v as i32),
                y: row.get::<_, Option<i64>>(3)?.map(|v| v as i32),
                always_on_top: row.get::<_, i64>(4)? != 0,
            })
        },
    )
    .unwrap_or_default()
}

/// Read the global hotkey string — used at startup before the frontend
/// has loaded, and by `update_hotkey`.
pub fn read_hotkey(app: &AppHandle) -> String {
    read_settings_sync(app).hotkey
}

/// Update only the window geometry (x, y, width, height), leaving every
/// other setting — including `window.always_on_top` — untouched.
///
/// Geometry is owned by the native window-event tracker in `window.rs`, so
/// this is the only writer for those four columns; `save_settings` (driven
/// by the frontend) deliberately never touches them, which avoids a stale
/// in-memory copy of the window rect clobbering a position the user just
/// dragged the window to.
pub fn persist_window_geometry(
    app: &AppHandle,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let db = app
        .try_state::<SettingsDb>()
        .ok_or_else(|| "Settings database not initialized".to_string())?;
    let conn = db.0.lock().map_err(|_| "Settings database lock poisoned".to_string())?;
    conn.execute(
        "UPDATE settings SET window_x = ?1, window_y = ?2, window_width = ?3, window_height = ?4 WHERE id = 1",
        params![x, y, width, height],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Tauri Commands ───────────────────────────────────────────

/// Load settings from the database. Returns defaults if unavailable.
#[command]
pub async fn load_settings(app: AppHandle) -> Result<Settings, String> {
    Ok(read_settings_sync(&app))
}

/// Save settings to the database. Window geometry (x/y/width/height) is
/// preserved from the existing row regardless of what the caller sends —
/// see `persist_window_geometry` for why.
#[command]
pub async fn save_settings(app: AppHandle, mut settings: Settings) -> Result<(), String> {
    let db = app
        .try_state::<SettingsDb>()
        .ok_or_else(|| "Settings database not initialized".to_string())?;
    let conn = db.0.lock().map_err(|_| "Settings database lock poisoned".to_string())?;

    let on_disk_window = read_settings(&conn).map(|s| s.window).unwrap_or_default();
    settings.window.x = on_disk_window.x;
    settings.window.y = on_disk_window.y;
    settings.window.width = on_disk_window.width;
    settings.window.height = on_disk_window.height;

    write_settings(&conn, &settings).map_err(|e| e.to_string())
}
