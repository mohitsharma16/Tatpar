import { useCallback } from "react";
import { useAppStore, useSettings as useSettingsStore } from "../store/app";
import { loadSettings, saveSettings } from "../api/tauri";
import type { Settings } from "../types";

// ============================================================
// Tatpar — useSettings Hook
// Loads settings from Rust on mount, saves on change.
// ============================================================

export interface UseSettingsReturn {
  settings: Settings;
  update: (partial: Partial<Settings>) => void;
  save: () => Promise<void>;
  load: () => Promise<void>;
}

export function useSettings(): UseSettingsReturn {
  const settings = useSettingsStore();
  const setSettings = useAppStore((s) => s.setSettings);

  const update = useCallback(
    (partial: Partial<Settings>) => {
      setSettings(partial);
    },
    [setSettings]
  );

  /**
   * Persist current settings to the Rust backend (JSON file → SQLite in Phase 3).
   * Safe to call after any settings change.
   */
  const save = useCallback(async () => {
    try {
      await saveSettings(settings);
    } catch (err) {
      console.error("[Tatpar] Failed to save settings:", err);
    }
  }, [settings]);

  /**
   * Load settings from the Rust backend and merge into the Zustand store.
   * Call this on app startup to hydrate persisted settings.
   */
  const load = useCallback(async () => {
    try {
      const persisted = await loadSettings();
      // Map Rust snake_case fields to camelCase TypeScript types
      setSettings({
        hotkey: persisted.hotkey,
        theme: persisted.theme as Settings["theme"],
        editorFontSize: persisted.editorFontSize ?? 14,
        launchOnStartup: persisted.launchOnStartup ?? false,
      });
    } catch (err) {
      console.error("[Tatpar] Failed to load settings:", err);
    }
  }, [setSettings]);

  return { settings, update, save, load };
}
