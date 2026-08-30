import { useEffect } from "react";
import { Editor } from "./components/editor/Editor";
import { Header } from "./components/ui/Header";
import { Terminal } from "./components/terminal/Terminal";
import { useAppStore } from "./store/app";
import { useExecution } from "./hooks/useExecution";
import { useSettings } from "./hooks/useSettings";
import "./App.css";

function App() {
  const activeLanguage = useAppStore((s) => s.activeLanguage);
  const code = useAppStore((s) => s.codePerLanguage[s.activeLanguage]);
  const setExecutionResult = useAppStore((s) => s.setExecutionResult);

  const { run, isRunning } = useExecution();
  const { settings, load: loadSettings } = useSettings();

  // Load persisted settings from Rust backend on first mount
  useEffect(() => {
    loadSettings();
  }, []);

  const handleRun = () => {
    run(activeLanguage, code);
  };

  const handleClear = () => {
    setExecutionResult(null);
  };

  // Listen for Ctrl+Enter dispatched by Monaco editor
  useEffect(() => {
    const handler = () => handleRun();
    window.addEventListener("tatpar:run", handler);
    return () => window.removeEventListener("tatpar:run", handler);
  }, [activeLanguage, code]);   // re-bind when language/code changes

  return (
    <div className={`app-root ${settings.theme}`}>
      <Header onRun={handleRun} isRunning={isRunning} />
      <main className="app-main">
        <div className="app-editor-pane">
          <Editor />
        </div>
        <div className="app-terminal-pane">
          <Terminal onClear={handleClear} />
        </div>
      </main>
    </div>
  );
}

export default App;
