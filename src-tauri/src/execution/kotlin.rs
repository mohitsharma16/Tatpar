// ============================================================
// CodeFloat — Kotlin Executor
// Compiles with `kotlinc` and runs with `java -jar`
// ============================================================

use super::language::{LanguageExecutor, ExecutionResult, create_temp_workspace, run_process};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::process::Command;

pub struct KotlinExecutor;

#[async_trait]
impl LanguageExecutor for KotlinExecutor {
    async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel: Arc<Mutex<bool>>,
    ) -> Result<ExecutionResult, String> {
        let workspace = create_temp_workspace()?;
        let src = workspace.path().join("main.kt");
        let jar = workspace.path().join("main.jar");

        std::fs::write(&src, code).map_err(|e| e.to_string())?;

        // Step 1: Compile
        let mut compile = Command::new("kotlinc");
        compile.arg(&src).arg("-include-runtime").arg("-d").arg(&jar);
        let compile_result = run_process(compile, timeout_secs, Arc::clone(&cancel)).await;

        if compile_result.status != "success" {
            return Ok(compile_result);
        }

        // Check cancel flag between compile and run
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

        // Step 2: Run
        let mut run = Command::new("java");
        run.arg("-jar").arg(&jar);
        Ok(run_process(run, timeout_secs, cancel).await)
    }
}
