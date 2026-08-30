import { Editor } from "./components/editor/Editor";
import "./App.css";

// Temporary Phase 1 scaffold — will be replaced with full layout in next steps
function App() {
  return (
    <div style={{ height: "100vh", display: "flex", flexDirection: "column" }}>
      <Editor />
    </div>
  );
}

export default App;
