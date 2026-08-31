// ============================================================
// Tatpar — C++ Executor
// Compiles with `g++` (or `clang++`) and runs the binary
// ============================================================

use super::language::{LanguageExecutor, ExecutionResult, create_temp_workspace, run_process};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::process::Command;

pub struct CppExecutor;

#[async_trait]
impl LanguageExecutor for CppExecutor {
    async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel: Arc<Mutex<bool>>,
    ) -> Result<ExecutionResult, String> {
        let workspace = create_temp_workspace()?;
        let src = workspace.path().join("main.cpp");
        let bin = workspace.path().join("main.exe");
        std::fs::write(&src, code).map_err(|e| e.to_string())?;

        // Prefer g++, fall back to clang++
        let compiler = if which::which("g++").is_ok() { "g++" } else { "clang++" };

        // Step 1: Compile
        let mut compile = Command::new(compiler);
        compile
            .arg(&src)
            .arg("-o")
            .arg(&bin)
            .arg("-std=c++17")
            .arg("-Wall");

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

        // Step 2: Run
        let run = Command::new(&bin);
        Ok(run_process(run, timeout_secs, cancel).await)
    }
}
