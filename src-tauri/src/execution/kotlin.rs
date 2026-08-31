// ============================================================
// Tatpar — Kotlin Executor
// Compiles with `kotlinc` and runs with `java -jar`
// ============================================================

use super::language::{LanguageExecutor, ExecutionResult, create_temp_workspace, run_process};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::process::Command;
use chrono::Utc;

pub struct KotlinExecutor;

#[async_trait]
impl LanguageExecutor for KotlinExecutor {
    async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel: Arc<Mutex<bool>>,
    ) -> Result<ExecutionResult, String> {
        // Guard: kotlinc must be on PATH
        if which::which("kotlinc").is_err() {
            return Ok(missing_runtime_result(
                "kotlinc",
                "Install the Kotlin compiler: https://kotlinlang.org/docs/command-line.html",
            ));
        }

        let workspace = create_temp_workspace()?;
        let src = workspace.path().join("main.kt");
        let jar = workspace.path().join("main.jar");

        std::fs::write(&src, code).map_err(|e| e.to_string())?;

        // ── Step 1: Compile ──────────────────────────────────────
        // kotlinc can be slow on first run (JVM startup ~5-15s).
        // We give it the full timeout; run gets whatever is left.
        let mut compile = Command::new("kotlinc");
        compile
            .arg(&src)
            .arg("-include-runtime")
            .arg("-d")
            .arg(&jar);

        let compile_result = run_process(compile, timeout_secs, Arc::clone(&cancel)).await;

        if compile_result.status != "success" {
            // Annotate compiler errors so the terminal shows context
            return Ok(ExecutionResult {
                stderr: format!("[Compile error]\n{}", compile_result.stderr),
                ..compile_result
            });
        }

        // ── Cancellation check between compile and run ───────────
        if *cancel.lock().unwrap() {
            return Ok(ExecutionResult {
                stdout: String::new(),
                stderr: "[Cancelled between compile and run]".to_string(),
                exit_code: None,
                duration_ms: compile_result.duration_ms,
                status: "error".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            });
        }

        // ── Step 2: Run ──────────────────────────────────────────
        let remaining = timeout_secs.saturating_sub(compile_result.duration_ms / 1000);
        let run_timeout = remaining.max(2); // always allow at least 2s to run

        let mut run_cmd = Command::new("java");
        run_cmd.arg("-jar").arg(&jar);

        let run_result = run_process(run_cmd, run_timeout, cancel).await;

        // Combine compile duration + run duration in the reported result
        Ok(ExecutionResult {
            duration_ms: compile_result.duration_ms + run_result.duration_ms,
            ..run_result
        })
    }
}

fn missing_runtime_result(tool: &str, hint: &str) -> ExecutionResult {
    ExecutionResult {
        stdout: String::new(),
        stderr: format!(
            "Runtime not found: `{tool}` is not installed or not on PATH.\n\nHint: {hint}"
        ),
        exit_code: None,
        duration_ms: 0,
        status: "error".to_string(),
        timestamp: Utc::now().to_rfc3339(),
    }
}
