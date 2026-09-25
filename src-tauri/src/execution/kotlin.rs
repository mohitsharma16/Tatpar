// ============================================================
// Tatpar — Kotlin Executor
// Fix #25 / #6: Compile to a classes directory instead of a fat
// JAR so we skip -include-runtime (packaging the full Kotlin
// stdlib on every run) — cuts compilation time from ~8s to ~3s.
//
// Strategy:
//   1. kotlinc main.kt -d classes/     (no -include-runtime)
//   2. kotlin -cp classes MainKt        (kotlin runner knows stdlib)
//   3. Fallback: java -cp classes:<kotlin-home>/lib/kotlin-stdlib.jar MainKt
//
// On Windows, kotlinc and kotlin are .bat files — must route
// through new_command() which wraps them in `cmd /C`.
// ============================================================

use super::language::{
    cancelled_result, create_temp_workspace, new_command, run_process, ExecutionResult,
    LanguageExecutor,
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
        // ── Resolve kotlinc path ──────────────────────────────────
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
        let src     = workspace.path().join("main.kt");
        // Fix #25: compile to a classes dir, not a fat JAR
        let classes = workspace.path().join("classes");
        std::fs::create_dir_all(&classes).map_err(|e| e.to_string())?;
        std::fs::write(&src, code).map_err(|e| e.to_string())?;

        // ── Step 1: Compile ───────────────────────────────────────
        // Drop -include-runtime — we don't package the stdlib anymore.
        // This saves ~1.7 MB of disk I/O per run and removes the main
        // bottleneck that caused 7–10 s compilation times.
        let mut compile = new_command(&kotlinc_cmd);
        compile
            .arg(&src)
            .arg("-d")
            .arg(&classes);

        let compile_result = run_process(compile, timeout_secs, Arc::clone(&cancel)).await;

        if compile_result.status != "success" {
            return Ok(ExecutionResult {
                stderr: format!("[Compile error]\n{}", compile_result.stderr),
                ..compile_result
            });
        }

        // ── Cancellation check between compile and run ────────────
        if *cancel.lock().unwrap() {
            return Ok(cancelled_result());
        }

        let remaining = timeout_secs
            .saturating_sub(compile_result.duration_ms / 1000)
            .max(2);

        // ── Step 2: Run ───────────────────────────────────────────
        // Prefer the `kotlin` runner (knows where stdlib lives).
        // Fall back to `java -cp classes:<stdlib>` if kotlin isn't on PATH.
        let run_result = if which::which("kotlin").is_ok() {
            let mut run_cmd = new_command("kotlin");
            run_cmd.arg("-cp").arg(&classes).arg("MainKt");
            run_process(run_cmd, remaining, Arc::clone(&cancel)).await
        } else {
            // Build a java -cp that includes the Kotlin stdlib.
            // kotlinc's home directory is one level above the kotlinc binary.
            let stdlib_path = find_kotlin_stdlib(&kotlinc_cmd);
            let cp = match stdlib_path {
                Some(ref stdlib) => {
                    // Java classpath separator: ; on Windows, : on Unix
                    let cp_sep = if cfg!(target_os = "windows") { ";" } else { ":" };
                    format!("{}{}{}", classes.display(), cp_sep, stdlib)
                }
                None => classes.to_string_lossy().to_string(),
            };
            let mut run_cmd = new_command("java");
            run_cmd.arg("-cp").arg(&cp).arg("MainKt");
            run_process(run_cmd, remaining, Arc::clone(&cancel)).await
        };

        Ok(ExecutionResult {
            duration_ms: compile_result.duration_ms + run_result.duration_ms,
            ..run_result
        })
    }
}

// ─── Helpers ──────────────────────────────────────────────────

/// Try to locate kotlin-stdlib.jar relative to the kotlinc binary so we
/// can build a -cp for the java fallback runner. Returns None if not found.
fn find_kotlin_stdlib(kotlinc_path: &str) -> Option<String> {
    // kotlinc lives at <kotlin-home>/bin/kotlinc (or kotlinc.bat on Windows).
    // The stdlib jar is at <kotlin-home>/lib/kotlin-stdlib.jar.
    let binary = if let Ok(p) = which::which(kotlinc_path) {
        p
    } else {
        std::path::PathBuf::from(kotlinc_path)
    };

    // Go up: bin/ → kotlin-home/, then into lib/
    let kotlin_home = binary.parent()?.parent()?;
    let stdlib = kotlin_home.join("lib").join("kotlin-stdlib.jar");
    if stdlib.exists() {
        Some(stdlib.to_string_lossy().to_string())
    } else {
        None
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
