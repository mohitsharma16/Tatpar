// ============================================================
// Tatpar — TypeScript Executor
// Supports fast runners (bun, tsx, deno) and compiles via tsc/npx
// On Windows, batch/cmd files (tsc.cmd, npx.cmd) route via new_command()
// ============================================================

use super::language::{
    cancelled_result, create_temp_workspace, new_command, run_process, ExecutionResult,
    LanguageExecutor,
};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::{Arc, Mutex};

pub struct TypeScriptExecutor;

#[async_trait]
impl LanguageExecutor for TypeScriptExecutor {
    async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel: Arc<Mutex<bool>>,
        compiler_path: Option<String>,
    ) -> Result<ExecutionResult, String> {
        let workspace = create_temp_workspace()?;
        let src = workspace.path().join("main.ts");
        let out = workspace.path().join("main.js");
        std::fs::write(&src, code).map_err(|e| e.to_string())?;

        // ── 1. Custom runner or compiler override from Settings ──
        if let Some(ref path_str) = compiler_path {
            let trimmed = path_str.trim();
            if !trimmed.is_empty() {
                let lower = trimmed.to_lowercase();
                if lower.ends_with("tsc") || lower.ends_with("tsc.cmd") || lower.ends_with("tsc.exe") {
                    return compile_and_run_tsc(trimmed, &src, &out, workspace.path(), timeout_secs, cancel).await;
                } else {
                    // Direct runner (e.g. tsx, bun, deno)
                    let mut cmd = new_command(trimmed);
                    cmd.arg(&src);
                    return Ok(run_process(cmd, timeout_secs, cancel).await);
                }
            }
        }

        // ── 2. Direct fast runners on PATH (instant execution) ───
        if let Ok(bun) = which::which("bun") {
            let mut cmd = new_command(&bun.to_string_lossy());
            cmd.arg("run").arg(&src);
            return Ok(run_process(cmd, timeout_secs, cancel).await);
        }

        if let Ok(tsx) = which::which("tsx") {
            let mut cmd = new_command(&tsx.to_string_lossy());
            cmd.arg(&src);
            return Ok(run_process(cmd, timeout_secs, cancel).await);
        }

        if let Ok(deno) = which::which("deno") {
            let mut cmd = new_command(&deno.to_string_lossy());
            cmd.arg("run").arg(&src);
            return Ok(run_process(cmd, timeout_secs, cancel).await);
        }

        // ── 3. Standard `tsc` compiler on PATH ───────────────────
        if let Ok(tsc) = which::which("tsc") {
            return compile_and_run_tsc(
                &tsc.to_string_lossy(),
                &src,
                &out,
                workspace.path(),
                timeout_secs,
                cancel,
            )
            .await;
        }

        // ── 4. Fallback: npx -y tsc (non-interactive, never hangs) ───
        if let Ok(npx) = which::which("npx") {
            let mut compile = new_command(&npx.to_string_lossy());
            compile
                .arg("-y")
                .arg("tsc")
                .arg(&src)
                .arg("--outDir")
                .arg(workspace.path())
                .arg("--target")
                .arg("ES2020")
                .arg("--module")
                .arg("commonjs")
                .arg("--skipLibCheck");

            let compile_result = run_process(compile, timeout_secs, Arc::clone(&cancel)).await;
            if compile_result.status != "success" {
                return Ok(ExecutionResult {
                    stderr: format!("[Compile error]\n{}", compile_result.stderr),
                    ..compile_result
                });
            }

            if *cancel.lock().unwrap() {
                return Ok(cancelled_result());
            }

            let node_executable = which::which("node")
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "node".to_string());

            let remaining = timeout_secs
                .saturating_sub(compile_result.duration_ms / 1000)
                .max(2);
            let mut run_cmd = new_command(&node_executable);
            run_cmd.arg(&out);
            let run_result = run_process(run_cmd, remaining, cancel).await;

            return Ok(ExecutionResult {
                duration_ms: compile_result.duration_ms + run_result.duration_ms,
                ..run_result
            });
        }

        // ── 5. Neither tsc nor npx found ─────────────────────────
        Ok(missing_runtime_result(
            "tsc / node",
            "Install Node.js: https://nodejs.org/ and TypeScript globally: `npm install -g typescript`",
        ))
    }
}

async fn compile_and_run_tsc(
    tsc_executable: &str,
    src: &std::path::Path,
    out: &std::path::Path,
    out_dir: &std::path::Path,
    timeout_secs: u64,
    cancel: Arc<Mutex<bool>>,
) -> Result<ExecutionResult, String> {
    let mut compile = new_command(tsc_executable);
    compile
        .arg(src)
        .arg("--outDir")
        .arg(out_dir)
        .arg("--target")
        .arg("ES2020")
        .arg("--module")
        .arg("commonjs")
        .arg("--skipLibCheck");

    let compile_result = run_process(compile, timeout_secs, Arc::clone(&cancel)).await;
    if compile_result.status != "success" {
        return Ok(ExecutionResult {
            stderr: format!("[Compile error]\n{}", compile_result.stderr),
            ..compile_result
        });
    }

    if *cancel.lock().unwrap() {
        return Ok(cancelled_result());
    }

    let node_executable = which::which("node")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "node".to_string());

    let remaining = timeout_secs
        .saturating_sub(compile_result.duration_ms / 1000)
        .max(2);
    let mut run_cmd = new_command(&node_executable);
    run_cmd.arg(out);
    let run_result = run_process(run_cmd, remaining, cancel).await;

    Ok(ExecutionResult {
        duration_ms: compile_result.duration_ms + run_result.duration_ms,
        ..run_result
    })
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
