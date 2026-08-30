import { useCallback } from "react";
import { useAppStore } from "../store/app";
import type { LanguageId } from "../types";

// ============================================================
// Tatpar — useExecution Hook
// Encapsulates all code-execution logic.
//
// Phase 1: stub that logs and marks isRunning = false.
// Phase 2: will call the Tauri execute_code command and handle
//          streaming output, timeout, and cancellation.
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
        // ── Phase 2: replace this block with the real Tauri invoke ──
        // const result = await executeCode({ language, code, timeoutSecs: 10 });
        //
        // For now, simulate a short delay so the UI running state is visible
        await new Promise((resolve) => setTimeout(resolve, 600));

        const stub = {
          stdout: "",
          stderr: "[Phase 2] Execution engine not yet connected.",
          exitCode: null,
          durationMs: 600,
          status: "error" as const,
          timestamp: new Date().toISOString(),
        };

        setExecutionResult(stub);
        addToHistory({ language, code, result: stub });
      } catch (err) {
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
    // Phase 2: will call cancelExecution() Tauri command
    setRunning(false);
  }, [setRunning]);

  return { run, cancel, isRunning };
}
