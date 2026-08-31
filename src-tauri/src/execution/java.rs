// ============================================================
// Tatpar — Java Executor
// Compiles with `javac` and runs with `java`
// NOTE: filename must be Main.java (class must be named Main)
// ============================================================

use super::language::{LanguageExecutor, ExecutionResult, create_temp_workspace, run_process};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::process::Command;
use chrono::Utc;

pub struct JavaExecutor;

#[async_trait]
impl LanguageExecutor for JavaExecutor {
    async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel: Arc<Mutex<bool>>,
    ) -> Result<ExecutionResult, String> {
        if which::which("javac").is_err() {
            return Ok(missing_runtime_result(
                "javac",
                "Install the JDK: https://adoptium.net/ or `winget install Microsoft.OpenJDK.21`",
            ));
        }

        let workspace = create_temp_workspace()?;
        // Java requires the filename to match the public class name
        let src = workspace.path().join("Main.java");
        std::fs::write(&src, code).map_err(|e| e.to_string())?;

        // ── Step 1: Compile ──────────────────────────────────────
        let mut compile = Command::new("javac");
        compile.arg(&src);
        let compile_result = run_process(compile, timeout_secs, Arc::clone(&cancel)).await;

        if compile_result.status != "success" {
            return Ok(ExecutionResult {
                stderr: format!("[Compile error]\n{}", compile_result.stderr),
                ..compile_result
            });
        }

        // ── Cancellation check ───────────────────────────────────
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

        // ── Step 2: Run (classpath = temp dir, class = Main) ─────
        let remaining = timeout_secs.saturating_sub(compile_result.duration_ms / 1000).max(2);
        let mut run_cmd = Command::new("java");
        run_cmd.arg("-cp").arg(workspace.path()).arg("Main");
        let run_result = run_process(run_cmd, remaining, cancel).await;

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
