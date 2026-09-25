import { useState } from "react";
import { History as HistoryIcon, RotateCcw, Trash2, Copy, Check, Clock, AlertTriangle } from "lucide-react";
import { useAppStore } from "../../store/app";
import { LANGUAGES } from "../../types";
import type { HistoryEntry } from "../../types";

// ============================================================
// Tatpar — Execution History Panel
// Fix #21: Guard against silent code loss when restoring from history.
// If the current editor buffer differs from the history entry's code,
// show a confirmation dialog before overwriting.
// ============================================================

export function HistoryPanel() {
  const history       = useAppStore((s) => s.history);
  const clearHistory  = useAppStore((s) => s.clearHistory);
  const setActiveLanguage = useAppStore((s) => s.setActiveLanguage);
  const setCode       = useAppStore((s) => s.setCode);
  const setExecutionResult = useAppStore((s) => s.setExecutionResult);
  const setPanel      = useAppStore((s) => s.setPanel);
  const codePerLanguage = useAppStore((s) => s.codePerLanguage);

  const [copiedId, setCopiedId] = useState<string | null>(null);
  // #21: pending entry waiting for confirmation
  const [pendingRestore, setPendingRestore] = useState<HistoryEntry | null>(null);

  // Commit the restore — called either directly (no conflict) or after confirm
  const commitRestore = (entry: HistoryEntry) => {
    setActiveLanguage(entry.language);
    setCode(entry.code);
    setExecutionResult(entry.result);
    setPanel("editor");
    setPendingRestore(null);
  };

  // #21: Check whether the current buffer for the entry's language has
  // unsaved/different content before overwriting it.
  const handleRestore = (entry: HistoryEntry) => {
    const currentCode = codePerLanguage[entry.language] ?? "";
    const isSameCode  = currentCode.trim() === entry.code.trim();

    if (isSameCode || currentCode.trim() === "") {
      // No conflict — restore immediately
      commitRestore(entry);
    } else {
      // Conflict — show confirmation dialog
      setPendingRestore(entry);
    }
  };

  const handleCopyCode = async (id: string, code: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await navigator.clipboard.writeText(code);
      setCopiedId(id);
      setTimeout(() => setCopiedId(null), 1600);
    } catch {
      console.warn("[Tatpar] Clipboard write failed");
    }
  };

  return (
    <div className="history-panel" role="region" aria-label="Execution History">
      <div className="history-inner">
        {/* Header */}
        <div className="history-header">
          <div className="history-title-group">
            <HistoryIcon size={16} className="history-header-icon" />
            <h2 className="history-title">Execution History</h2>
            <span className="history-count">({history.length})</span>
          </div>

          {history.length > 0 && (
            <button
              id="clear-history-btn"
              className="history-clear-btn"
              onClick={clearHistory}
              title="Clear all execution history"
            >
              <Trash2 size={12} />
              <span>Clear History</span>
            </button>
          )}
        </div>

        {/* List or Empty State */}
        {history.length === 0 ? (
          <div className="history-empty">
            <Clock size={28} className="history-empty-icon" />
            <p className="history-empty-title">No execution history yet</p>
            <p className="history-empty-desc">
              Run code in the editor to record recent execution results and snippets here.
            </p>
          </div>
        ) : (
          <div className="history-list">
            {history.map((entry) => {
              const langInfo = LANGUAGES[entry.language];
              const isCopied = copiedId === entry.id;
              const hasOutput = entry.result.stdout || entry.result.stderr;

              return (
                <div
                  key={entry.id}
                  className={`history-card history-card--${entry.result.status}`}
                  onClick={() => handleRestore(entry)}
                  title="Click to restore this snippet into the editor"
                  role="button"
                  tabIndex={0}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      handleRestore(entry);
                    }
                  }}
                >
                  <div className="history-card-header">
                    <div className="history-card-left">
                      <span className="history-lang-badge">
                        {langInfo?.name ?? entry.language}
                      </span>
                      <span className={`history-status-badge history-status-badge--${entry.result.status}`}>
                        {entry.result.status}
                      </span>
                      <span className="history-duration">
                        {entry.result.durationMs}ms
                      </span>
                    </div>

                    <div className="history-card-actions">
                      <span className="history-time">
                        {new Date(entry.result.timestamp).toLocaleTimeString()}
                      </span>
                      <button
                        className="history-action-icon-btn"
                        onClick={(e) => handleCopyCode(entry.id, entry.code, e)}
                        title="Copy code"
                        aria-label="Copy code"
                      >
                        {isCopied ? <Check size={12} className="text-success" /> : <Copy size={12} />}
                      </button>
                      <button
                        className="history-restore-btn"
                        onClick={(e) => { e.stopPropagation(); handleRestore(entry); }}
                        title="Restore into editor"
                      >
                        <RotateCcw size={11} />
                        <span>Restore</span>
                      </button>
                    </div>
                  </div>

                  {/* Code Snippet Preview */}
                  <pre className="history-code-preview">
                    {entry.code.trim()}
                  </pre>

                  {/* Output Preview */}
                  {hasOutput && (
                    <div className="history-output-preview">
                      <span className="history-output-label">Output:</span>
                      <span className="history-output-text">
                        {(entry.result.stdout || entry.result.stderr).trim().slice(0, 120)}
                      </span>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* ── Fix #21: Restore Confirmation Dialog ─────────────────── */}
      {pendingRestore && (
        <div
          className="restore-overlay"
          role="dialog"
          aria-modal="true"
          aria-labelledby="restore-dialog-title"
          onClick={() => setPendingRestore(null)}
        >
          <div
            className="restore-dialog"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="restore-dialog-icon">
              <AlertTriangle size={20} />
            </div>

            <div className="restore-dialog-body">
              <h3 id="restore-dialog-title" className="restore-dialog-title">
                Overwrite current code?
              </h3>
              <p className="restore-dialog-desc">
                Your current{" "}
                <strong>{LANGUAGES[pendingRestore.language]?.name ?? pendingRestore.language}</strong>{" "}
                buffer has unsaved changes. Restoring this snippet will replace it permanently.
              </p>
            </div>

            <div className="restore-dialog-actions">
              <button
                id="restore-cancel-btn"
                className="restore-dialog-btn restore-dialog-btn--cancel"
                onClick={() => setPendingRestore(null)}
              >
                Keep current
              </button>
              <button
                id="restore-confirm-btn"
                className="restore-dialog-btn restore-dialog-btn--confirm"
                onClick={() => commitRestore(pendingRestore)}
                autoFocus
              >
                <RotateCcw size={12} />
                Restore anyway
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
