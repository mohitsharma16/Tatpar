// ============================================================
// Tatpar — C++ Executor
// Compiles with `g++` (or `clang++`) and runs the binary
// ============================================================

use super::language::{LanguageExecutor, ExecutionResult, create_temp_workspace, run_process, new_command};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::process::Command;
use chrono::Utc;

pub struct CppExecutor;

#[async_trait]
impl LanguageExecutor for CppExecutor {
    async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel: Arc<Mutex<bool>>,
    ) -> Result<ExecutionResult, String> {
        // Prefer g++, fall back to clang++
        let compiler = if which::which("g++").is_ok() {
            "g++"
        } else if which::which("clang++").is_ok() {
            "clang++"
        } else {
            return Ok(missing_runtime_result(
                "g++ / clang++",
                "Install MinGW-w64: https://www.mingw-w64.org/ or MSVC via Visual Studio",
            ));
        };

        let workspace = create_temp_workspace()?;
        let src = workspace.path().join("main.cpp");
        let bin = workspace.path().join("main.exe");
        std::fs::write(&src, code).map_err(|e| e.to_string())?;

        // ── Step 1: Compile ──────────────────────────────────────
        let mut compile = new_command(compiler);
        compile
            .arg(&src)
            .arg("-o").arg(&bin)
            .arg("-std=c++17")
            .arg("-Wall");

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

        // ── Step 2: Run binary ───────────────────────────────────
        let remaining = timeout_secs.saturating_sub(compile_result.duration_ms / 1000).max(2);
        let run_cmd = Command::new(&bin);
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
