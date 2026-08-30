import { useEffect } from "react";
import { Editor } from "./components/editor/Editor";
import { Header } from "./components/ui/Header";
import { useAppStore, useSettings } from "./store/app";
import "./App.css";

// Temporary Phase 1 scaffold — Terminal added in Step 3
function App() {
  const isRunning = useAppStore((s) => s.isRunning);
  const settings = useSettings();

  // Handle run — will call Tauri execute_code in Phase 2
  const handleRun = () => {
    console.log("[Tatpar] Run triggered — execution engine coming in Phase 2");
  };

  // Listen for Ctrl+Enter dispatched by Monaco editor
  useEffect(() => {
    const handler = () => handleRun();
    window.addEventListener("tatpar:run", handler);
    return () => window.removeEventListener("tatpar:run", handler);
  }, []);

  return (
    <div className={`app-root ${settings.theme}`}>
      <Header onRun={handleRun} isRunning={isRunning} />
      <main className="app-main">
        <Editor />
      </main>
    </div>
  );
}

export default App;
