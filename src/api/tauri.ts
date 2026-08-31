import { invoke } from "@tauri-apps/api/core";
import type {
  ExecutionRequest,
  ExecutionResult,
  Settings,
  LanguageId,
} from "../types";

// ============================================================
// Tatpar — Tauri Command Bindings
// Type-safe wrappers around all Tauri invoke() calls
// ============================================================

// ------------------------------------------------------------
// Execution Commands
// ------------------------------------------------------------

/**
 * Execute code in the given language.
 * Corresponds to Rust command: `execute_code`
 */
export async function executeCode(
  request: ExecutionRequest
): Promise<ExecutionResult> {
  return invoke<ExecutionResult>("execute_code", { request });
}

/**
 * Cancel a currently running execution.
 * Corresponds to Rust command: `cancel_execution`
 */
export async function cancelExecution(): Promise<void> {
  return invoke<void>("cancel_execution");
}

// ------------------------------------------------------------
// Language Discovery Commands
// ------------------------------------------------------------

/**
 * Check which languages are available (compiler/interpreter found).
 * Returns a map of languageId -> bool.
 * Corresponds to Rust command: `check_languages`
 */
export async function checkLanguages(): Promise<Record<LanguageId, boolean>> {
  return invoke<Record<LanguageId, boolean>>("check_languages");
}

/**
 * Get the detected compiler/interpreter path for a language.
 * Corresponds to Rust command: `get_compiler_path`
 */
export async function getCompilerPath(
  language: LanguageId
): Promise<string | null> {
  return invoke<string | null>("get_compiler_path", { language });
}

// ------------------------------------------------------------
// Settings Commands
// ------------------------------------------------------------

/**
 * Load settings from persistent storage.
 * Corresponds to Rust command: `load_settings`
 */
export async function loadSettings(): Promise<Settings> {
  return invoke<Settings>("load_settings");
}

/**
 * Save settings to persistent storage.
 * Corresponds to Rust command: `save_settings`
 */
export async function saveSettings(settings: Settings): Promise<void> {
  return invoke<void>("save_settings", { settings });
}

// ------------------------------------------------------------
// Window Commands
// ------------------------------------------------------------

/**
 * Set always-on-top state for the window.
 * Corresponds to Rust command: `set_always_on_top`
 */
export async function setAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
  return invoke<void>("set_always_on_top", { alwaysOnTop });
}

/**
 * Minimize window to system tray.
 * Corresponds to Rust command: `minimize_to_tray`
 */
export async function minimizeToTray(): Promise<void> {
  return invoke<void>("minimize_to_tray");
}

/**
 * Save window position and size.
 * Corresponds to Rust command: `save_window_state`
 */
export async function saveWindowState(
  x: number,
  y: number,
  width: number,
  height: number
): Promise<void> {
  return invoke<void>("save_window_state", { x, y, width, height });
}

// ------------------------------------------------------------
// Hotkey Commands
// ------------------------------------------------------------

/**
 * Register a global hotkey.
 * Corresponds to Rust command: `register_hotkey`
 */
export async function registerHotkey(hotkey: string): Promise<boolean> {
  return invoke<boolean>("register_hotkey", { hotkey });
}

/**
 * Check if a hotkey string conflicts with any system shortcuts.
 * Corresponds to Rust command: `check_hotkey_conflict`
 */
export async function checkHotkeyConflict(hotkey: string): Promise<boolean> {
  return invoke<boolean>("check_hotkey_conflict", { hotkey });
}
