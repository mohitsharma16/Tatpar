// ============================================================
// Tatpar — Core TypeScript Types
// All shared types for the frontend application
// ============================================================

// ------------------------------------------------------------
// Language System
// ------------------------------------------------------------

export type LanguageId =
  | "kotlin"
  | "python"
  | "java"
  | "javascript"
  | "typescript"
  | "cpp";

export interface Language {
  id: LanguageId;
  name: string;
  /** File extension used when creating temp files */
  extension: string;
  /** Monaco editor language identifier */
  monacoLanguage: string;
  /** Default starter code shown when switching to this language */
  defaultCode: string;
  /** Whether the language requires compilation before running */
  compiled: boolean;
}

export const LANGUAGES: Record<LanguageId, Language> = {
  kotlin: {
    id: "kotlin",
    name: "Kotlin",
    extension: "kt",
    monacoLanguage: "kotlin",
    compiled: true,
    defaultCode: `fun main() {\n    println("Hello, Tatpar!")\n}`,
  },
  python: {
    id: "python",
    name: "Python",
    extension: "py",
    monacoLanguage: "python",
    compiled: false,
    defaultCode: `print("Hello, Tatpar!")`,
  },
  java: {
    id: "java",
    name: "Java",
    extension: "java",
    monacoLanguage: "java",
    compiled: true,
    defaultCode: `public class Main {\n    public static void main(String[] args) {\n        System.out.println("Hello, Tatpar!");\n    }\n}`,
  },
  javascript: {
    id: "javascript",
    name: "JavaScript",
    extension: "js",
    monacoLanguage: "javascript",
    compiled: false,
    defaultCode: `console.log("Hello, Tatpar!");`,
  },
  typescript: {
    id: "typescript",
    name: "TypeScript",
    extension: "ts",
    monacoLanguage: "typescript",
    compiled: true,
    defaultCode: `const greet = (name: string): string => \`Hello, \${name}!\`;\nconsole.log(greet("Tatpar"));`,
  },
  cpp: {
    id: "cpp",
    name: "C++",
    extension: "cpp",
    monacoLanguage: "cpp",
    compiled: true,
    defaultCode: `#include <iostream>\n\nint main() {\n    std::cout << "Hello, Tatpar!" << std::endl;\n    return 0;\n}`,
  },
};

export const LANGUAGE_LIST = Object.values(LANGUAGES);

// ------------------------------------------------------------
// Execution
// ------------------------------------------------------------

export type ExecutionStatus = "idle" | "running" | "success" | "error" | "timeout";

export interface ExecutionResult {
  stdout: string;
  stderr: string;
  exitCode: number | null;
  /** Execution duration in milliseconds */
  durationMs: number;
  status: ExecutionStatus;
  /** ISO timestamp when this execution finished */
  timestamp: string;
}

export interface ExecutionRequest {
  language: LanguageId;
  code: string;
  /** Timeout in seconds (default: 10) */
  timeoutSecs?: number;
}

// ------------------------------------------------------------
// History
// ------------------------------------------------------------

export interface HistoryEntry {
  id: string;
  language: LanguageId;
  code: string;
  result: ExecutionResult;
}

// ------------------------------------------------------------
// Settings
// ------------------------------------------------------------

export interface LanguageSettings {
  /** Execution timeout in seconds */
  timeoutSecs: number;
  /** Whether network access is allowed (default: false) */
  networkEnabled: boolean;
  /** Custom compiler/interpreter path (empty = auto-detect from PATH) */
  compilerPath?: string;
}

export type Theme = "dark" | "light";

export interface WindowSettings {
  width: number;
  height: number;
  x: number | null;
  y: number | null;
  alwaysOnTop: boolean;
}

export interface Settings {
  /** Global hotkey string (e.g., "ctrl+shift+space") */
  hotkey: string;
  theme: Theme;
  /** Font size for the Monaco editor */
  editorFontSize: number;
  /** Launch app at Windows startup */
  launchOnStartup: boolean;
  /** Per-language configuration */
  languageSettings: Record<LanguageId, LanguageSettings>;
  /** Last remembered window position/size */
  window: WindowSettings;
}

export const DEFAULT_LANGUAGE_SETTINGS: LanguageSettings = {
  timeoutSecs: 10,
  networkEnabled: false,
};

export const DEFAULT_SETTINGS: Settings = {
  hotkey: "ctrl+shift+space",
  theme: "dark",
  editorFontSize: 14,
  launchOnStartup: false,
  languageSettings: {
    kotlin: { ...DEFAULT_LANGUAGE_SETTINGS },
    python: { ...DEFAULT_LANGUAGE_SETTINGS },
    java: { ...DEFAULT_LANGUAGE_SETTINGS },
    javascript: { ...DEFAULT_LANGUAGE_SETTINGS },
    typescript: { ...DEFAULT_LANGUAGE_SETTINGS },
    cpp: { ...DEFAULT_LANGUAGE_SETTINGS },
  },
  window: {
    width: 600,
    height: 700,
    x: null,
    y: null,
    alwaysOnTop: true,
  },
};

// ------------------------------------------------------------
// App State (UI)
// ------------------------------------------------------------

export type PanelView = "editor" | "settings";

export interface AppState {
  /** Currently selected language */
  activeLanguage: LanguageId;
  /** Code per language (persisted across language switches) */
  codePerLanguage: Record<LanguageId, string>;
  /** Current execution result */
  executionResult: ExecutionResult | null;
  /** Whether a run is in progress */
  isRunning: boolean;
  /** Execution history (most recent first, max 10) */
  history: HistoryEntry[];
  /** Active panel */
  panel: PanelView;
  /** User settings */
  settings: Settings;
}
