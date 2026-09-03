import { useRef, useEffect } from "react";
import MonacoEditor, { type OnMount, type OnChange } from "@monaco-editor/react";
import { useAppStore, useActiveLanguage, useSettings } from "../../store/app";
import { saveSettings } from "../../api/tauri";

type EditorInstance = Parameters<OnMount>[0];

// ============================================================
// Tatpar — Editor Component
// Monaco editor wrapper with language-aware configuration
// ============================================================

interface EditorProps {
  /** Called when the user types in the editor */
  onChange?: (value: string) => void;
}

export function Editor({ onChange }: EditorProps) {
  const editorRef = useRef<EditorInstance | null>(null);
  const activeLanguage = useActiveLanguage();
  const code = useAppStore((s) => s.codePerLanguage[s.activeLanguage]);
  const setCode = useAppStore((s) => s.setCode);
  const settings = useSettings();

  useEffect(() => {
    const handleFocus = () => {
      editorRef.current?.focus();
    };
    window.addEventListener("tatpar:focus-editor", handleFocus);
    return () => {
      window.removeEventListener("tatpar:focus-editor", handleFocus);
    };
  }, []);

  const handleChange: OnChange = (value) => {
    const newCode = value ?? "";
    setCode(newCode);
    onChange?.(newCode);
  };

  const handleMount: OnMount = (editor, monaco) => {
    editorRef.current = editor;
    // Ctrl+Enter → trigger run
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => {
      window.dispatchEvent(new CustomEvent("tatpar:run"));
    });

    // Ctrl+K → clear terminal
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyK, () => {
      window.dispatchEvent(new CustomEvent("tatpar:clear"));
    });

    // Ctrl+, → toggle settings
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Comma, () => {
      window.dispatchEvent(new CustomEvent("tatpar:settings"));
    });

    // Ctrl+0 → reset zoom / font size to default (14px)
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Digit0, () => {
      const defaultFontSize = 14;
      editor.updateOptions({ fontSize: defaultFontSize });
      useAppStore.getState().setSettings({ editorFontSize: defaultFontSize });
      saveSettings({
        ...useAppStore.getState().settings,
        editorFontSize: defaultFontSize,
      }).catch(() => {});
    });

    // Ctrl+1 through Ctrl+6 → switch language
    const digitKeyCodes = [
      monaco.KeyCode.Digit1,
      monaco.KeyCode.Digit2,
      monaco.KeyCode.Digit3,
      monaco.KeyCode.Digit4,
      monaco.KeyCode.Digit5,
      monaco.KeyCode.Digit6,
    ];

    digitKeyCodes.forEach((keyCode, index) => {
      editor.addCommand(monaco.KeyMod.CtrlCmd | keyCode, () => {
        window.dispatchEvent(
          new CustomEvent("tatpar:switch-lang", { detail: { index } })
        );
      });
    });

    // Sync font size changes from Ctrl + mouse wheel zoom to state & persistent settings
    editor.onDidChangeConfiguration((e) => {
      if (e.hasChanged(monaco.editor.EditorOption.fontSize)) {
        const newFontSize = Math.round(
          editor.getOption(monaco.editor.EditorOption.fontSize)
        );
        const currentFontSize = useAppStore.getState().settings.editorFontSize;
        if (newFontSize !== currentFontSize && newFontSize >= 10 && newFontSize <= 32) {
          const updatedSettings = {
            ...useAppStore.getState().settings,
            editorFontSize: newFontSize,
          };
          useAppStore.getState().setSettings({ editorFontSize: newFontSize });
          saveSettings(updatedSettings).catch(() => {});
        }
      }
    });

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
          mouseWheelZoom: true,
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
