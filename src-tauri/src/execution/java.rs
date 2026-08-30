// ============================================================
// CodeFloat — Java Executor
// Compiles with `javac` and runs with `java`
// Wraps top-level code in a Main class if needed
// ============================================================

use super::language::{LanguageExecutor, ExecutionResult, create_temp_workspace, run_process};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::process::Command;

pub struct JavaExecutor;

#[async_trait]
impl LanguageExecutor for JavaExecutor {
    async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel: Arc<Mutex<bool>>,
    ) -> Result<ExecutionResult, String> {
        let workspace = create_temp_workspace()?;
        // Java requires the filename to match the public class name
        let src = workspace.path().join("Main.java");
        std::fs::write(&src, code).map_err(|e| e.to_string())?;

        // Step 1: Compile
        let mut compile = Command::new("javac");
        compile.arg(&src);
        let compile_result = run_process(compile, timeout_secs, Arc::clone(&cancel)).await;
        if compile_result.status != "success" {
            return Ok(compile_result);
        }

        if *cancel.lock().unwrap() {
            return Ok(ExecutionResult {
                stdout: String::new(),
                stderr: "[Cancelled]".to_string(),
                exit_code: None,
                duration_ms: 0,
                status: "error".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
        }

        // Step 2: Run (classpath = temp dir, class = Main)
        let mut run = Command::new("java");
        run.arg("-cp").arg(workspace.path()).arg("Main");
        Ok(run_process(run, timeout_secs, cancel).await)
    }
}
