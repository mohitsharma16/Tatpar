// ============================================================
// CodeFloat — TypeScript Executor
// Compiles with `tsc` (or npx tsc) then runs with `node`
// ============================================================

use super::language::{LanguageExecutor, ExecutionResult, create_temp_workspace, run_process};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::process::Command;

pub struct TypeScriptExecutor;

#[async_trait]
impl LanguageExecutor for TypeScriptExecutor {
    async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel: Arc<Mutex<bool>>,
    ) -> Result<ExecutionResult, String> {
        let workspace = create_temp_workspace()?;
        let src = workspace.path().join("main.ts");
        let out = workspace.path().join("main.js");
        std::fs::write(&src, code).map_err(|e| e.to_string())?;

        // Prefer global tsc, fall back to npx
        let (tsc_bin, tsc_args): (&str, Vec<&str>) = if which::which("tsc").is_ok() {
            ("tsc", vec![])
        } else {
            ("npx", vec!["tsc"])
        };

        // Step 1: Compile
        let mut compile = Command::new(tsc_bin);
        for arg in &tsc_args { compile.arg(arg); }
        compile
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

        // Step 2: Run with node
        let mut run = Command::new("node");
        run.arg(&out);
        Ok(run_process(run, timeout_secs, cancel).await)
    }
}
