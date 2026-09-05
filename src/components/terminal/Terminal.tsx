import { useState, useRef, useEffect } from "react";
import { Copy, Check, Trash2 } from "lucide-react";
import { useAppStore } from "../../store/app";
import type { ExecutionResult } from "../../types";

// ============================================================
// Tatpar — Terminal / Output Panel
// Displays stdout, stderr, exit code, and execution metadata
// ============================================================

interface TerminalProps {
  onClear: () => void;
}

export function Terminal({ onClear }: TerminalProps) {
  const result = useAppStore((s) => s.executionResult);
  const isRunning = useAppStore((s) => s.isRunning);

  return (
    <div className="terminal-root">
      <TerminalHeader result={result} isRunning={isRunning} onClear={onClear} />
      <TerminalBody result={result} isRunning={isRunning} />
    </div>
  );
}

// ─── Sub-components ────────────────────────────────────────────

function TerminalHeader({
  result,
  isRunning,
  onClear,
}: {
  result: ExecutionResult | null;
  isRunning: boolean;
  onClear: () => void;
}) {
  const [copied, setCopied] = useState(false);
  const statusBadge = getStatusBadge(result, isRunning);

  const handleCopy = async () => {
    if (!result) return;
    await copyOutput(result);
    setCopied(true);
    setTimeout(() => setCopied(false), 1800);
  };

  return (
    <div className="terminal-header">
      <div className="terminal-header-left">
        <span className="terminal-label">Output</span>
        {statusBadge && (
          <span className={`terminal-badge terminal-badge--${statusBadge.type}`}>
            {statusBadge.text}
          </span>
        )}
        {result && (
          <span className="terminal-meta">
            {result.durationMs}ms
          </span>
        )}
      </div>

      <div className="terminal-header-right">
        {result && (
          <>
            <button
              id="copy-output-btn"
              className={`terminal-action-btn${copied ? " terminal-action-btn--copied" : ""}`}
              title="Copy output (Ctrl+Shift+C)"
              onClick={handleCopy}
              aria-label="Copy output"
            >
              {copied ? (
                <>
                  <Check size={11} aria-hidden="true" />
                  <span>Copied!</span>
                </>
              ) : (
                <>
                  <Copy size={11} aria-hidden="true" />
                  <span>Copy</span>
                </>
              )}
            </button>
            <button
              id="clear-output-btn"
              className="terminal-action-btn"
              title="Clear output (Ctrl+K)"
              onClick={onClear}
              aria-label="Clear output"
            >
              <Trash2 size={11} aria-hidden="true" />
              <span>Clear</span>
            </button>
          </>
        )}
      </div>
    </div>
  );
}

function TerminalBody({
  result,
  isRunning,
}: {
  result: ExecutionResult | null;
  isRunning: boolean;
}) {
  const bodyRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (result && bodyRef.current) {
      bodyRef.current.scrollTop = bodyRef.current.scrollHeight;
    }
  }, [result]);

  if (isRunning) {
    return (
      <div className="terminal-body terminal-body--idle">
        <span className="terminal-spinner" aria-hidden="true" />
        <span className="terminal-idle-text">Running…</span>
      </div>
    );
  }

  if (!result) {
    return (
      <div className="terminal-body terminal-body--idle">
        <span className="terminal-idle-text">
          Press <kbd>▶ Run</kbd> or <kbd>Ctrl+Enter</kbd> to execute
        </span>
      </div>
    );
  }

  const hasStdout = result.stdout.trim().length > 0;
  const hasStderr = result.stderr.trim().length > 0;

  return (
    <div ref={bodyRef} className="terminal-body terminal-body--output">
      {/* stdout */}
      {hasStdout && (
        <pre className="terminal-output terminal-output--stdout">
          {result.stdout}
        </pre>
      )}

      {/* stderr */}
      {hasStderr && (
        <pre className={`terminal-output terminal-output--stderr ${result.status === "timeout" ? "terminal-output--timeout" : ""}`}>
          {result.stderr}
        </pre>
      )}

      {/* Empty success */}
      {!hasStdout && !hasStderr && result.status === "success" && (
        <span className="terminal-idle-text terminal-idle-text--success">
          ✓ Process exited with code 0 — no output
        </span>
      )}

      {/* Exit code footer */}
      <div className="terminal-footer">
        <span className={`terminal-exit-code terminal-exit-code--${result.status}`}>
          exit {result.exitCode ?? "—"}
        </span>
        <span className="terminal-timestamp">
          {new Date(result.timestamp).toLocaleTimeString()}
        </span>
      </div>
    </div>
  );
}

// ─── Helpers ───────────────────────────────────────────────────

function getStatusBadge(
  result: ExecutionResult | null,
  isRunning: boolean
): { text: string; type: string } | null {
  if (isRunning) return { text: "running", type: "running" };
  if (!result) return null;
  switch (result.status) {
    case "success": return { text: "success", type: "success" };
    case "error":   return { text: "error",   type: "error"   };
    case "timeout": return { text: "timeout", type: "timeout" };
    default:        return null;
  }
}

async function copyOutput(result: ExecutionResult) {
  const text = [result.stdout, result.stderr].filter(Boolean).join("\n").trim();
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    console.warn("[Tatpar] Clipboard write failed");
  }
}
