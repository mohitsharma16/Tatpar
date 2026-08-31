// ============================================================
// Tatpar — JavaScript Executor
// Runs directly with `node`
// ============================================================

use super::language::{LanguageExecutor, ExecutionResult, create_temp_workspace, run_process};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::process::Command;
use chrono::Utc;

pub struct JavaScriptExecutor;

#[async_trait]
impl LanguageExecutor for JavaScriptExecutor {
    async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel: Arc<Mutex<bool>>,
    ) -> Result<ExecutionResult, String> {
        if which::which("node").is_err() {
            return Ok(missing_runtime_result(
                "node",
                "Install Node.js: https://nodejs.org/",
            ));
        }

        let workspace = create_temp_workspace()?;
        let src = workspace.path().join("main.js");
        std::fs::write(&src, code).map_err(|e| e.to_string())?;

        let mut cmd = Command::new("node");
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
