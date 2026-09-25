import { useRef, useEffect } from "react";
import { Terminal as TerminalIcon, X, ChevronDown, ChevronUp } from "lucide-react";
import { useAppStore } from "../../store/app";

// ============================================================
// Tatpar — Stdin Drawer (Issue #8)
// A collapsible panel above the terminal that lets users type
// mock stdin text to be piped to the running process.
// Toggle: Ctrl+I or the keyboard icon in the terminal header.
// ============================================================

interface StdinDrawerProps {
  open: boolean;
  onToggle: () => void;
}

export function StdinDrawer({ open, onToggle }: StdinDrawerProps) {
  const stdinPerLanguage = useAppStore((s) => s.stdinPerLanguage);
  const setStdin        = useAppStore((s) => s.setStdin);
  const activeLanguage  = useAppStore((s) => s.activeLanguage);
  const textareaRef     = useRef<HTMLTextAreaElement | null>(null);

  const value = stdinPerLanguage[activeLanguage] ?? "";

  // Auto-focus the textarea when the drawer opens
  useEffect(() => {
    if (open && textareaRef.current) {
      textareaRef.current.focus();
    }
  }, [open]);

  return (
    <div className={`stdin-drawer${open ? " stdin-drawer--open" : ""}`} aria-expanded={open}>
      {/* ── Drawer header / toggle bar ── */}
      <button
        id="stdin-toggle-btn"
        className="stdin-toggle-bar"
        onClick={onToggle}
        aria-label={open ? "Close stdin drawer" : "Open stdin drawer"}
        title="Toggle stdin drawer (Ctrl+I)"
      >
        <div className="stdin-toggle-left">
          <TerminalIcon size={12} className="stdin-icon" />
          <span className="stdin-label">Stdin Input</span>
          {value.trim().length > 0 && (
            <span className="stdin-badge" title="Stdin is active">
              {value.trim().split("\n").length} line{value.trim().split("\n").length !== 1 ? "s" : ""}
            </span>
          )}
        </div>
        <div className="stdin-toggle-right">
          <kbd className="stdin-kbd">Ctrl+I</kbd>
          {open ? <ChevronDown size={12} /> : <ChevronUp size={12} />}
        </div>
      </button>

      {/* ── Textarea body (only rendered when open) ── */}
      {open && (
        <div className="stdin-body">
          <textarea
            ref={textareaRef}
            id="stdin-textarea"
            className="stdin-textarea"
            placeholder={"Type stdin here — one value per line.\nThis text is piped to the process when you press Run."}
            value={value}
            onChange={(e) => setStdin(e.target.value)}
            spellCheck={false}
            autoCorrect="off"
            autoCapitalize="off"
          />
          {value.trim().length > 0 && (
            <button
              id="stdin-clear-btn"
              className="stdin-clear-btn"
              onClick={() => setStdin("")}
              title="Clear stdin"
              aria-label="Clear stdin"
            >
              <X size={11} />
              <span>Clear</span>
            </button>
          )}
        </div>
      )}
    </div>
  );
}
