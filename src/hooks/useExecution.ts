import { useCallback } from "react";
import { useAppStore } from "../store/app";
import { executeCode } from "../api/tauri";
import type { LanguageId } from "../types";

// ============================================================
// Tatpar — useExecution Hook
// Encapsulates all code-execution logic.
// Phase 2: calls the real Tauri execute_code Rust command.
// Phase 3: will add cancellation + streaming output.
// ============================================================

export interface UseExecutionReturn {
  run: (language: LanguageId, code: string) => Promise<void>;
  cancel: () => void;
  isRunning: boolean;
}

export function useExecution(): UseExecutionReturn {
  const setRunning = useAppStore((s) => s.setRunning);
  const setExecutionResult = useAppStore((s) => s.setExecutionResult);
  const addToHistory = useAppStore((s) => s.addToHistory);
  const isRunning = useAppStore((s) => s.isRunning);

  const run = useCallback(
    async (language: LanguageId, code: string) => {
      if (isRunning) return;

      setRunning(true);
      setExecutionResult(null);

      try {
        // ── Real Tauri invocation ──────────────────────────────
        const result = await executeCode({
          language,
          code,
          timeoutSecs: 10,
        });

        setExecutionResult(result);
        addToHistory({ language, code, result });
      } catch (err) {
        // Surface Rust-side errors (e.g. compiler not found)
        const errorResult = {
          stdout: "",
          stderr: String(err),
          exitCode: null,
          durationMs: 0,
          status: "error" as const,
          timestamp: new Date().toISOString(),
        };
        setExecutionResult(errorResult);
      } finally {
        setRunning(false);
      }
    },
    [isRunning, setRunning, setExecutionResult, addToHistory]
  );

  const cancel = useCallback(() => {
    // Phase 3: will call cancelExecution() Tauri command
    setRunning(false);
  }, [setRunning]);

  return { run, cancel, isRunning };
}
