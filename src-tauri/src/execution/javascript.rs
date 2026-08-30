// ============================================================
// CodeFloat — JavaScript Executor
// Runs directly with `node`
// ============================================================

use super::language::{LanguageExecutor, ExecutionResult, create_temp_workspace, run_process};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::process::Command;

pub struct JavaScriptExecutor;

#[async_trait]
impl LanguageExecutor for JavaScriptExecutor {
    async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel: Arc<Mutex<bool>>,
    ) -> Result<ExecutionResult, String> {
        let workspace = create_temp_workspace()?;
        let src = workspace.path().join("main.js");
        std::fs::write(&src, code).map_err(|e| e.to_string())?;

        let mut cmd = Command::new("node");
        cmd.arg(&src);
        Ok(run_process(cmd, timeout_secs, cancel).await)
    }
}
