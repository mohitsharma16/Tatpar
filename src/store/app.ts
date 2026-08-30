import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import type {
  AppState,
  ExecutionResult,
  HistoryEntry,
  LanguageId,
  PanelView,
  Settings,
} from "../types";
import {
  LANGUAGES,
  LANGUAGE_LIST,
  DEFAULT_SETTINGS,
} from "../types";
import { nanoid } from "../utils/nanoid";

// ============================================================
// Tatpar — Zustand App Store
// Central state management for the application
// ============================================================

interface AppActions {
  // Language
  setActiveLanguage: (language: LanguageId) => void;

  // Editor
  setCode: (code: string) => void;

  // Execution
  setRunning: (running: boolean) => void;
  setExecutionResult: (result: ExecutionResult | null) => void;

  // History
  addToHistory: (entry: Omit<HistoryEntry, "id">) => void;
  clearHistory: () => void;

  // Panel
  setPanel: (panel: PanelView) => void;

  // Settings
  setSettings: (settings: Partial<Settings>) => void;
  resetSettings: () => void;
}

type AppStore = AppState & AppActions;

/** Build initial code map — one default snippet per language */
const buildInitialCodeMap = (): Record<LanguageId, string> => {
  return Object.fromEntries(
    LANGUAGE_LIST.map((lang) => [lang.id, lang.defaultCode])
  ) as Record<LanguageId, string>;
};

export const useAppStore = create<AppStore>()(
  persist(
    (set, get) => ({
      // ─── Initial State ────────────────────────────────────
      activeLanguage: "kotlin",
      codePerLanguage: buildInitialCodeMap(),
      executionResult: null,
      isRunning: false,
      history: [],
      panel: "editor",
      settings: DEFAULT_SETTINGS,

      // ─── Language ─────────────────────────────────────────
      setActiveLanguage: (language) => {
        set({ activeLanguage: language, executionResult: null });
      },

      // ─── Editor ───────────────────────────────────────────
      setCode: (code) => {
        const { activeLanguage } = get();
        set((state) => ({
          codePerLanguage: {
            ...state.codePerLanguage,
            [activeLanguage]: code,
          },
        }));
      },

      // ─── Execution ────────────────────────────────────────
      setRunning: (running) => set({ isRunning: running }),

      setExecutionResult: (result) => set({ executionResult: result }),

      // ─── History ──────────────────────────────────────────
      addToHistory: (entry) => {
        const newEntry: HistoryEntry = { id: nanoid(), ...entry };
        set((state) => ({
          history: [newEntry, ...state.history].slice(0, 10), // keep max 10
        }));
      },

      clearHistory: () => set({ history: [] }),

      // ─── Panel ────────────────────────────────────────────
      setPanel: (panel) => set({ panel }),

      // ─── Settings ─────────────────────────────────────────
      setSettings: (partial) =>
        set((state) => ({
          settings: { ...state.settings, ...partial },
        })),

      resetSettings: () => set({ settings: DEFAULT_SETTINGS }),
    }),
    {
      name: "Tatpar-app-state",
      storage: createJSONStorage(() => localStorage),
      // Only persist the parts that make sense to keep across sessions
      partialize: (state) => ({
        activeLanguage: state.activeLanguage,
        codePerLanguage: state.codePerLanguage,
        history: state.history,
        settings: state.settings,
      }),
    }
  )
);

// ─── Selectors (convenience hooks) ──────────────────────────

/** Returns the code for the currently active language */
export const useCurrentCode = () =>
  useAppStore((s) => s.codePerLanguage[s.activeLanguage]);

/** Returns the Language definition for the active language */
export const useActiveLanguage = () =>
  useAppStore((s) => LANGUAGES[s.activeLanguage]);

/** Returns whether a run is in progress */
export const useIsRunning = () => useAppStore((s) => s.isRunning);

/** Returns the latest execution result */
export const useExecutionResult = () => useAppStore((s) => s.executionResult);

/** Returns user settings */
export const useSettings = () => useAppStore((s) => s.settings);
