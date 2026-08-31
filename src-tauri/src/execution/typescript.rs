// ============================================================
// Tatpar — TypeScript Executor
// Compiles with `tsc` (or npx tsc) then runs with `node`
// On Windows, tsc and npx are .cmd files — use new_command()
// ============================================================

use super::language::{LanguageExecutor, ExecutionResult, create_temp_workspace, run_process, new_command};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use chrono::Utc;

pub struct TypeScriptExecutor;

#[async_trait]
impl LanguageExecutor for TypeScriptExecutor {
    async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel: Arc<Mutex<bool>>,
    ) -> Result<ExecutionResult, String> {
        // Need node to run the compiled output
        if which::which("node").is_err() {
            return Ok(missing_runtime_result(
                "node",
                "Install Node.js (includes npx): https://nodejs.org/",
            ));
        }

        // Prefer global tsc, fall back to npx tsc
        let use_tsc = which::which("tsc").is_ok();
        let use_npx = !use_tsc && which::which("npx").is_ok();

        if !use_tsc && !use_npx {
            return Ok(missing_runtime_result(
                "tsc / npx",
                "Install TypeScript globally: `npm install -g typescript`",
            ));
        }

        let workspace = create_temp_workspace()?;
        let src = workspace.path().join("main.ts");
        let out = workspace.path().join("main.js");
        std::fs::write(&src, code).map_err(|e| e.to_string())?;

        // ── Step 1: Compile ──────────────────────────────────────
        // new_command wraps tsc.cmd / npx.cmd in `cmd /C` on Windows
        let mut compile = if use_npx {
            let mut c = new_command("npx");
            c.arg("tsc");
            c
        } else {
            new_command("tsc")
        };

        compile
            .arg(&src)
            .arg("--outDir").arg(workspace.path())
            .arg("--target").arg("ES2020")
            .arg("--module").arg("commonjs")
            .arg("--skipLibCheck");

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

        // ── Step 2: Run with node ────────────────────────────────
        let remaining = timeout_secs.saturating_sub(compile_result.duration_ms / 1000).max(2);
        let mut run_cmd = new_command("node");
        run_cmd.arg(&out);
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
