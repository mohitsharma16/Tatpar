import { useEffect } from "react";
import { Editor } from "./components/editor/Editor";
import { Header } from "./components/ui/Header";
import { Terminal } from "./components/terminal/Terminal";
import { SettingsPanel } from "./components/ui/Settings";
import { HistoryPanel } from "./components/history/History";
import { useAppStore } from "./store/app";
import { useExecution } from "./hooks/useExecution";
import { useSettings } from "./hooks/useSettings";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useResizable } from "./hooks/useResizable";
import { checkLanguages } from "./api/tauri";
import "./App.css";

function App() {
  const activeLanguage = useAppStore((s) => s.activeLanguage);
  const code = useAppStore((s) => s.codePerLanguage[s.activeLanguage]);
  const setExecutionResult = useAppStore((s) => s.setExecutionResult);
  const setLanguageAvailability = useAppStore((s) => s.setLanguageAvailability);
  const panel = useAppStore((s) => s.panel);

  const { run, cancel, isRunning } = useExecution();
  const { settings, load: loadSettings } = useSettings();
  const {
    height: terminalHeight,
    isResizing,
    startResizing,
    toggleOrReset,
  } = useResizable();

  // On startup: load settings + check which runtimes are available
  useEffect(() => {
    loadSettings();

    checkLanguages()
      .then(setLanguageAvailability)
      .catch((err) => console.warn("[Tatpar] check_languages failed:", err));
  }, []);

  const handleRun = () => {
    run(activeLanguage, code);
  };

  const handleClear = () => {
    setExecutionResult(null);
  };

  // Global & Monaco keyboard shortcuts (Ctrl+Enter, Ctrl+1..6, Ctrl+K, Ctrl+,, Escape)
  useKeyboardShortcuts({
    onRun: handleRun,
    onClear: handleClear,
  });

  return (
    <div className={`app-root ${settings.theme}${isResizing ? " app-resizing" : ""}`}>
      <Header onRun={handleRun} onCancel={cancel} isRunning={isRunning} />
      <main className="app-main">
        {panel === "settings" ? (
          <SettingsPanel />
        ) : panel === "history" ? (
          <HistoryPanel />
        ) : (
          <>
            <div className="app-editor-pane">
              <Editor />
            </div>
            <div
              className={`app-resizer${isResizing ? " active" : ""}`}
              onPointerDown={startResizing}
              onDoubleClick={toggleOrReset}
              role="separator"
              aria-orientation="horizontal"
              title="Drag to resize terminal (Double click to toggle)"
            >
              <div className="app-resizer-line" />
            </div>
            <div
              className="app-terminal-pane"
              style={{
                height: `${terminalHeight}px`,
                flex: `0 0 ${terminalHeight}px`,
              }}
            >
              <Terminal onClear={handleClear} />
            </div>
          </>
        )}
      </main>
    </div>
  );
}

export default App;

