import { useEffect } from "react";
import { useAppStore } from "../store/app";
import { LANGUAGE_LIST } from "../types";

// ============================================================
// Tatpar — useKeyboardShortcuts Hook
// Phase 5 Step 1: Global keyboard shortcuts & navigation
// ============================================================

export interface KeyboardShortcutsProps {
  onRun: () => void;
  onClear: () => void;
}

export function useKeyboardShortcuts({ onRun, onClear }: KeyboardShortcutsProps) {
  const panel = useAppStore((s) => s.panel);
  const setPanel = useAppStore((s) => s.setPanel);
  const setActiveLanguage = useAppStore((s) => s.setActiveLanguage);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const isInput =
        target?.tagName === "INPUT" ||
        target?.tagName === "TEXTAREA" ||
        target?.isContentEditable;

      // ── Escape: Close settings or history modal / Return to editor ──
      if (e.key === "Escape") {
        if (panel !== "editor") {
          e.preventDefault();
          setPanel("editor");
          return;
        }
      }

      // ── Ctrl+H : Toggle Execution History ─────────────────
      if ((e.ctrlKey || e.metaKey) && (e.key === "h" || e.key === "H")) {
        if (!isInput) {
          e.preventDefault();
          setPanel(panel === "history" ? "editor" : "history");
          return;
        }
      }

      // ── Ctrl+, : Toggle Settings ─────────────────────────
      if ((e.ctrlKey || e.metaKey) && e.key === ",") {
        e.preventDefault();
        setPanel(panel === "settings" ? "editor" : "settings");
        return;
      }

      // ── Ctrl+K : Clear Output ────────────────────────────
      if ((e.ctrlKey || e.metaKey) && (e.key === "k" || e.key === "K")) {
        if (!isInput) {
          e.preventDefault();
          onClear();
          return;
        }
      }

      // ── Ctrl+1 through Ctrl+6 : Instant Language Switching
      if ((e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey) {
        const digit = parseInt(e.key, 10);
        if (!isNaN(digit) && digit >= 1 && digit <= LANGUAGE_LIST.length) {
          e.preventDefault();
          const targetLang = LANGUAGE_LIST[digit - 1];
          if (targetLang) {
            setActiveLanguage(targetLang.id);
            if (panel === "settings") {
              setPanel("editor");
            }
          }
          return;
        }
      }

      // ── Ctrl+Enter : Run Code ────────────────────────────
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
        if (!isInput) {
          e.preventDefault();
          onRun();
        }
      }
    };

    // Listen for custom events dispatched by Monaco editor
    const handleCustomRun = () => onRun();
    const handleCustomClear = () => onClear();
    const handleCustomToggleSettings = () => {
      setPanel(panel === "settings" ? "editor" : "settings");
    };
    const handleCustomSwitchLang = (e: Event) => {
      const customEvent = e as CustomEvent<{ index: number }>;
      const idx = customEvent.detail?.index;
      if (typeof idx === "number" && LANGUAGE_LIST[idx]) {
        setActiveLanguage(LANGUAGE_LIST[idx].id);
        if (panel === "settings") {
          setPanel("editor");
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("tatpar:run", handleCustomRun);
    window.addEventListener("tatpar:clear", handleCustomClear);
    window.addEventListener("tatpar:settings", handleCustomToggleSettings);
    window.addEventListener("tatpar:switch-lang", handleCustomSwitchLang);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("tatpar:run", handleCustomRun);
      window.removeEventListener("tatpar:clear", handleCustomClear);
      window.removeEventListener("tatpar:settings", handleCustomToggleSettings);
      window.removeEventListener("tatpar:switch-lang", handleCustomSwitchLang);
    };
  }, [panel, setPanel, setActiveLanguage, onRun, onClear]);
}
