import { useState } from "react";
import { History as HistoryIcon, RotateCcw, Trash2, Copy, Check, Clock } from "lucide-react";
import { useAppStore } from "../../store/app";
import { LANGUAGES } from "../../types";
import type { HistoryEntry } from "../../types";

// ============================================================
// Tatpar — Execution History Panel
// Phase 5 Step 3: Browse and restore previous runs
// ============================================================

export function HistoryPanel() {
  const history = useAppStore((s) => s.history);
  const clearHistory = useAppStore((s) => s.clearHistory);
  const setActiveLanguage = useAppStore((s) => s.setActiveLanguage);
  const setCode = useAppStore((s) => s.setCode);
  const setExecutionResult = useAppStore((s) => s.setExecutionResult);
  const setPanel = useAppStore((s) => s.setPanel);

  const [copiedId, setCopiedId] = useState<string | null>(null);

  const handleRestore = (entry: HistoryEntry) => {
    setActiveLanguage(entry.language);
    setCode(entry.code);
    setExecutionResult(entry.result);
    setPanel("editor");
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
                        onClick={() => handleRestore(entry)}
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
    </div>
  );
}
