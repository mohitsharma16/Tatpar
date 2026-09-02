// ============================================================
// Tatpar — Kotlin Executor
// Compiles with `kotlinc` and runs with `java -jar`
// On Windows, kotlinc is a .bat file — must use new_command()
// ============================================================

use super::language::{
    create_temp_workspace, new_command, run_process, ExecutionResult, LanguageExecutor,
};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::{Arc, Mutex};

pub struct KotlinExecutor;

#[async_trait]
impl LanguageExecutor for KotlinExecutor {
    async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel: Arc<Mutex<bool>>,
        compiler_path: Option<String>,
    ) -> Result<ExecutionResult, String> {
        let kotlinc_cmd = match compiler_path {
            Some(ref path) => path.clone(),
            None => {
                if which::which("kotlinc").is_err() {
                    return Ok(missing_runtime_result(
                        "kotlinc",
                        "Install the Kotlin compiler: https://kotlinlang.org/docs/command-line.html",
                    ));
                }
                "kotlinc".to_string()
            }
        };

        let workspace = create_temp_workspace()?;
        let src = workspace.path().join("main.kt");
        let jar = workspace.path().join("main.jar");

        std::fs::write(&src, code).map_err(|e| e.to_string())?;

        // ── Step 1: Compile ──────────────────────────────────────
        // new_command wraps kotlinc.bat in `cmd /C` on Windows
        let mut compile = new_command(&kotlinc_cmd);
        compile
            .arg(&src)
            .arg("-include-runtime")
            .arg("-d")
            .arg(&jar);

        let compile_result = run_process(compile, timeout_secs, Arc::clone(&cancel)).await;

        if compile_result.status != "success" {
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
        let remaining = timeout_secs
            .saturating_sub(compile_result.duration_ms / 1000)
            .max(2);
        let mut run_cmd = new_command("java");
        run_cmd.arg("-jar").arg(&jar);

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
