// ============================================================
// Tatpar — Python Executor
// Runs directly with `python` (Windows) or `python3` (Unix)
// NOTE: On Windows, `python3` is often a Microsoft Store stub
//       that opens a dialog instead of running code — always
//       use `python` on Windows.
// ============================================================

use super::language::{LanguageExecutor, ExecutionResult, create_temp_workspace, run_process, new_command};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use chrono::Utc;

pub struct PythonExecutor;

#[async_trait]
impl LanguageExecutor for PythonExecutor {
    async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel: Arc<Mutex<bool>>,
    ) -> Result<ExecutionResult, String> {
        // On Windows: `python3` is usually a Microsoft Store stub — skip it.
        // On Unix: prefer python3, fall back to python.
        #[cfg(target_os = "windows")]
        let python = {
            if which::which("python").is_ok() { "python" }
            else {
                return Ok(missing_runtime_result(
                    "python",
                    "Install Python 3: https://www.python.org/downloads/ (check 'Add to PATH')",
                ));
            }
        };

        #[cfg(not(target_os = "windows"))]
        let python = {
            if which::which("python3").is_ok() { "python3" }
            else if which::which("python").is_ok() { "python" }
            else {
                return Ok(missing_runtime_result(
                    "python",
                    "Install Python 3: https://www.python.org/downloads/",
                ));
            }
        };

        let workspace = create_temp_workspace()?;
        let src = workspace.path().join("main.py");
        std::fs::write(&src, code).map_err(|e| e.to_string())?;

        let mut cmd = new_command(python);
        cmd.arg(&src);
        Ok(run_process(cmd, timeout_secs, cancel).await)
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
