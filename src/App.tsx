import { useEffect } from "react";
import { Editor } from "./components/editor/Editor";
import { Header } from "./components/ui/Header";
import { Terminal } from "./components/terminal/Terminal";
import { useAppStore, useSettings } from "./store/app";
import "./App.css";

// Phase 1 scaffold — execution engine wired in Phase 2
function App() {
  const isRunning = useAppStore((s) => s.isRunning);
  const setExecutionResult = useAppStore((s) => s.setExecutionResult);
  const settings = useSettings();

  const handleRun = () => {
    console.log("[Tatpar] Run triggered — execution engine coming in Phase 2");
  };

  const handleClear = () => {
    setExecutionResult(null);
  };

  // Listen for Ctrl+Enter dispatched by Monaco
  useEffect(() => {
    const handler = () => handleRun();
    window.addEventListener("tatpar:run", handler);
    return () => window.removeEventListener("tatpar:run", handler);
  }, []);

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
