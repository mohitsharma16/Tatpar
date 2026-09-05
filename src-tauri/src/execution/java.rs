// ============================================================
// Tatpar — Java Executor
// Compiles with `javac` and runs with `java`
// NOTE: filename must be Main.java (class must be named Main)
// ============================================================

use super::language::{LanguageExecutor, ExecutionResult, create_temp_workspace, run_process, new_command};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use chrono::Utc;

pub struct JavaExecutor;

#[async_trait]
impl LanguageExecutor for JavaExecutor {
    async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel: Arc<Mutex<bool>>,
        compiler_path: Option<String>,
    ) -> Result<ExecutionResult, String> {
        let javac_cmd = match compiler_path {
            Some(ref path) => path.clone(),
            None => {
                if which::which("javac").is_err() {
                    return Ok(missing_runtime_result(
                        "javac",
                        "Install the JDK: https://adoptium.net/ or `winget install Microsoft.OpenJDK.21`",
                    ));
                }
                "javac".to_string()
            }
        };

        let workspace = create_temp_workspace()?;
        let class_name = extract_java_class_name(code);
        let src = workspace.path().join(format!("{class_name}.java"));
        std::fs::write(&src, code).map_err(|e| e.to_string())?;

        // ── Step 1: Compile ──────────────────────────────────────
        let mut compile = new_command(&javac_cmd);
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

        // ── Step 2: Run (classpath = temp dir, class = class_name) ─────
        let remaining = timeout_secs.saturating_sub(compile_result.duration_ms / 1000).max(2);
        let mut run_cmd = new_command("java");
        run_cmd.arg("-cp").arg(workspace.path()).arg(&class_name);
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

/// Extracts the main class name from Java source code.
/// Prioritizes `public class <Name>`, then falls back to any `class <Name>`.
/// Defaults to "Main" if no class definition is found.
fn extract_java_class_name(code: &str) -> String {
    let mut fallback_class = None;

    for line in code.lines() {
        let line = if let Some((before, _)) = line.split_once("//") {
            before.trim()
        } else {
            line.trim()
        };

        if line.is_empty() {
            continue;
        }

        let words: Vec<&str> = line.split_whitespace().collect();
        for i in 0..words.len() {
            if words[i] == "class" && i + 1 < words.len() {
                let candidate = words[i + 1]
                    .trim_end_matches('{')
                    .trim_end_matches('(')
                    .trim();

                let candidate = candidate.split('<').next().unwrap_or("").trim();

                if !candidate.is_empty()
                    && candidate
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
                {
                    if i > 0 && words[i - 1] == "public" {
                        return candidate.to_string();
                    }
                    if fallback_class.is_none() {
                        fallback_class = Some(candidate.to_string());
                    }
                }
            }
        }
    }

    fallback_class.unwrap_or_else(|| "Main".to_string())
}
