import { ChevronDown, Sun, Moon, Play, TriangleAlert, Settings as SettingsIcon, History as HistoryIcon } from "lucide-react";
import { useAppStore, useActiveLanguage, useSettings, useLanguageAvailability } from "../../store/app";
import { LANGUAGE_LIST } from "../../types";
import type { LanguageId } from "../../types";
import { Logo } from "./Logo";

// ============================================================
// Tatpar — Header / Toolbar Component
// Language picker · Run/Stop button · Theme toggle · History · Settings
// Availability indicator: colored dot beside the picker
// ============================================================

interface HeaderProps {
  onRun: () => void;
  onCancel: () => void;
  isRunning: boolean;
}

export function Header({ onRun, onCancel, isRunning }: HeaderProps) {
  const activeLanguage = useActiveLanguage();
  const setActiveLanguage = useAppStore((s) => s.setActiveLanguage);
  const settings = useSettings();
  const setSettings = useAppStore((s) => s.setSettings);
  const availability = useLanguageAvailability();
  const panel = useAppStore((s) => s.panel);
  const setPanel = useAppStore((s) => s.setPanel);
  const isSettingsOpen = panel === "settings";
  const isHistoryOpen = panel === "history";

  const toggleSettings = () => setPanel(isSettingsOpen ? "editor" : "settings");
  const toggleHistory = () => setPanel(isHistoryOpen ? "editor" : "history");

  const isDark = settings.theme === "dark";
  const avail = availability[activeLanguage.id as LanguageId];
  // undefined = not yet checked (optimistic); true = ok; false = missing
  const showWarning = avail === false;

  // Status dot state for the current language
  const dotState: "ok" | "missing" | "checking" =
    avail === true ? "ok" : avail === false ? "missing" : "checking";

  const dotTitle =
    dotState === "ok"
      ? `${activeLanguage.name} runtime found on PATH`
      : dotState === "missing"
        ? `${activeLanguage.name} runtime not found — run to see install hint`
        : "Checking runtime availability…";

  const toggleTheme = () => setSettings({ theme: isDark ? "light" : "dark" });

  const handleLanguageChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setActiveLanguage(e.target.value as LanguageId);
  };

  return (
    <>
      <header className="tatpar-header">
        {/* App branding */}
        <div className="header-brand">
          <Logo size={20} />
          <span className="header-title">Tatpar</span>
        </div>

        {/* Controls */}
        <div className="header-controls">
          {/* Language picker + availability dot */}
          <div className="lang-picker-group">
            <div className="lang-picker-wrapper">
              <select
                id="language-picker"
                className="lang-picker"
                value={activeLanguage.id}
                onChange={handleLanguageChange}
                disabled={isRunning}
                title="Select language (Ctrl+1 through Ctrl+6)"
              >
                {LANGUAGE_LIST.map((lang, index) => (
                  <option key={lang.id} value={lang.id}>
                    {lang.name} (Ctrl+{index + 1})
                  </option>
                ))}
              </select>
              <ChevronDown className="lang-picker-arrow" size={13} aria-hidden="true" />
            </div>
            {/* Colored dot shows runtime status for the SELECTED language */}
            <span
              className={`lang-status-dot lang-status-dot--${dotState}`}
              title={dotTitle}
              aria-label={dotTitle}
            />
          </div>

          {/* Theme toggle */}
          <button
            id="theme-toggle"
            className="icon-btn"
            onClick={toggleTheme}
            title={isDark ? "Switch to light mode" : "Switch to dark mode"}
            aria-label="Toggle theme"
          >
            {isDark ? <Sun size={15} /> : <Moon size={15} />}
          </button>

          {/* History toggle */}
          <button
            id="history-toggle"
            className={`icon-btn${isHistoryOpen ? " icon-btn--active" : ""}`}
            onClick={toggleHistory}
            title={isHistoryOpen ? "Close history (Esc or Ctrl+H)" : "View execution history (Ctrl+H)"}
            aria-label="Toggle execution history"
            aria-pressed={isHistoryOpen}
          >
            <HistoryIcon size={15} />
          </button>

          {/* Settings toggle */}
          <button
            id="settings-toggle"
            className={`icon-btn${isSettingsOpen ? " icon-btn--active" : ""}`}
            onClick={toggleSettings}
            title={isSettingsOpen ? "Close settings (Esc or Ctrl+,)" : "Open settings (Ctrl+,)"}
            aria-label="Toggle settings"
            aria-pressed={isSettingsOpen}
          >
            <SettingsIcon size={15} />
          </button>

          {/* Run / Stop toggle */}
          {isRunning ? (
            <button
              id="stop-btn"
              className="stop-btn"
              onClick={onCancel}
              title="Stop execution"
              aria-label="Stop execution"
            >
              <span className="run-spinner" aria-hidden="true" />
              Stop
            </button>
          ) : (
            <button
              id="run-btn"
              className="run-btn"
              onClick={onRun}
              title="Run code (Ctrl+Enter)"
              aria-label="Run code"
            >
              <Play size={13} fill="currentColor" aria-hidden="true" />
              Run
            </button>
          )}
        </div>
      </header>

      {/* Runtime not found warning bar — only shown when runtime is confirmed absent */}
      {showWarning && !isRunning && (
        <div className="runtime-warning" role="alert">
          <TriangleAlert className="runtime-warning-icon" size={14} aria-hidden="true" />
          <span>
            <strong>{activeLanguage.name}</strong> runtime not found on PATH.
            Click Run to see the install instructions.
          </span>
        </div>
      )}
    </>
  );
}
