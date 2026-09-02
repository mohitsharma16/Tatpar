import { useCallback } from "react";
import { useAppStore } from "../store/app";
import { executeCode, cancelExecution } from "../api/tauri";
import type { LanguageId } from "../types";

// ============================================================
// Tatpar — useExecution Hook
// Phase 2: real Tauri execute_code invocation.
// Phase 3: cancel() now calls cancel_execution on the Rust side.
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
  const languageSettings = useAppStore((s) => s.settings.languageSettings);

  const run = useCallback(
    async (language: LanguageId, code: string) => {
      if (isRunning) return;

      setRunning(true);
      setExecutionResult(null);

      const timeoutSecs = languageSettings[language]?.timeoutSecs ?? 10;

      try {
        const result = await executeCode({
          language,
          code,
          timeoutSecs,
        });

        setExecutionResult(result);
        addToHistory({ language, code, result });
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
    [isRunning, setRunning, setExecutionResult, addToHistory, languageSettings]
  );

  const cancel = useCallback(() => {
    // Signal the Rust backend to set the cancel flag.
    // The executor checks this flag and terminates early.
    cancelExecution().catch((err) =>
      console.warn("[Tatpar] cancel_execution failed:", err)
    );
    // Optimistically mark as not running immediately in the UI
    setRunning(false);
  }, [setRunning]);

  return { run, cancel, isRunning };
}
