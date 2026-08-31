import MonacoEditor, { type OnMount, type OnChange } from "@monaco-editor/react";
import { useAppStore, useActiveLanguage, useSettings } from "../../store/app";

// ============================================================
// Tatpar — Editor Component
// Monaco editor wrapper with language-aware configuration
// ============================================================

interface EditorProps {
  /** Called when the user types in the editor */
  onChange?: (value: string) => void;
}

export function Editor({ onChange }: EditorProps) {
  const activeLanguage = useActiveLanguage();
  const code = useAppStore((s) => s.codePerLanguage[s.activeLanguage]);
  const setCode = useAppStore((s) => s.setCode);
  const settings = useSettings();

  const handleChange: OnChange = (value) => {
    const newCode = value ?? "";
    setCode(newCode);
    onChange?.(newCode);
  };

  const handleMount: OnMount = (editor, monaco) => {
    // Ctrl+Enter → trigger run (dispatches a custom event the parent listens to)
    editor.addCommand(
      monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter,
      () => {
        window.dispatchEvent(new CustomEvent("tatpar:run"));
      }
    );

    // Focus the editor immediately
    editor.focus();
  };

  return (
    <div className="editor-wrapper">
      <MonacoEditor
        height="100%"
        language={activeLanguage.monacoLanguage}
        value={code}
        theme={settings.theme === "dark" ? "vs-dark" : "vs"}
        onChange={handleChange}
        onMount={handleMount}
        options={{
          fontSize: settings.editorFontSize,
          fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', Consolas, monospace",
          fontLigatures: true,
          minimap: { enabled: false },
          scrollBeyondLastLine: false,
          lineNumbers: "on",
          renderLineHighlight: "line",
          cursorBlinking: "smooth",
          cursorSmoothCaretAnimation: "on",
          smoothScrolling: true,
          automaticLayout: true,
          tabSize: 4,
          insertSpaces: true,
          wordWrap: "on",
          padding: { top: 12, bottom: 12 },
          overviewRulerLanes: 0,
          hideCursorInOverviewRuler: true,
          scrollbar: {
            vertical: "auto",
            horizontal: "auto",
            verticalScrollbarSize: 6,
            horizontalScrollbarSize: 6,
          },
        }}
      />
      <div className="editor-watermark" aria-hidden="true">
        <span className="editor-watermark-en">TATPAR</span>
        <span className="editor-watermark-hi">तत्पर</span>
      </div>
    </div>
  );
}
