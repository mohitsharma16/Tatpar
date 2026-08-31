import { useAppStore, useActiveLanguage, useSettings, useLanguageAvailability } from "../../store/app";
import { LANGUAGE_LIST } from "../../types";
import type { LanguageId } from "../../types";

// ============================================================
// Tatpar — Header / Toolbar Component
// Language picker · Run button · Theme toggle · Status dots
// ============================================================

interface HeaderProps {
  onRun: () => void;
  isRunning: boolean;
}

export function Header({ onRun, isRunning }: HeaderProps) {
  const activeLanguage = useActiveLanguage();
  const setActiveLanguage = useAppStore((s) => s.setActiveLanguage);
  const settings = useSettings();
  const setSettings = useAppStore((s) => s.setSettings);
  const availability = useLanguageAvailability();

  const isDark = settings.theme === "dark";
  const isAvailable = availability[activeLanguage.id as LanguageId];
  // undefined = not yet checked (optimistic); false = confirmed missing
  const showWarning = isAvailable === false;

  const toggleTheme = () => {
    setSettings({ theme: isDark ? "light" : "dark" });
  };

  const handleLanguageChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setActiveLanguage(e.target.value as LanguageId);
  };

  return (
    <>
      <header className="tatpar-header">
        {/* App branding */}
        <div className="header-brand">
          <span className="header-logo">⚡</span>
          <span className="header-title">tatpar</span>
        </div>

        {/* Controls */}
        <div className="header-controls">
          {/* Language picker */}
          <div className="lang-picker-wrapper">
            <select
              id="language-picker"
              className="lang-picker"
              value={activeLanguage.id}
              onChange={handleLanguageChange}
              title="Select language"
            >
              {LANGUAGE_LIST.map((lang) => {
                const avail = availability[lang.id as LanguageId];
                const dot = avail === true ? "●" : avail === false ? "○" : "";
                return (
                  <option key={lang.id} value={lang.id}>
                    {dot ? `${dot} ${lang.name}` : lang.name}
                  </option>
                );
              })}
            </select>
            <span className="lang-picker-arrow">▾</span>
          </div>

          {/* Theme toggle */}
          <button
            id="theme-toggle"
            className="icon-btn"
            onClick={toggleTheme}
            title={isDark ? "Switch to light mode" : "Switch to dark mode"}
            aria-label="Toggle theme"
          >
            {isDark ? "☀️" : "🌙"}
          </button>

          {/* Run button */}
          <button
            id="run-btn"
            className={`run-btn ${isRunning ? "run-btn--running" : ""}`}
            onClick={onRun}
            disabled={isRunning}
            title="Run code (Ctrl+Enter)"
            aria-label={isRunning ? "Running…" : "Run code"}
          >
            {isRunning ? (
              <>
                <span className="run-spinner" aria-hidden="true" />
                Running…
              </>
            ) : (
              <>
                <span aria-hidden="true">▶</span>
                Run
              </>
            )}
          </button>
        </div>
      </header>

      {/* Runtime not found warning bar */}
      {showWarning && (
        <div className="runtime-warning" role="alert">
          <span className="runtime-warning-icon">⚠</span>
          <span>
            <strong>{activeLanguage.name}</strong> runtime not found on PATH.
            Run anyway to see the install hint.
          </span>
        </div>
      )}
    </>
  );
}
