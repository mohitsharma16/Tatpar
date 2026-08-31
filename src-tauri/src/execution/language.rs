// ============================================================
// Tatpar — Language Trait + Shared Types
// All language executors implement LanguageExecutor.
// ExecutionResult, ExecutionRequest, and ExecutionState are
// defined here and re-exported via execution/mod.rs.
// ============================================================

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::time::timeout;
use chrono::Utc;

// ─── Shared Types ─────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionRequest {
    pub language: String,
    pub code: String,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    /// "success" | "error" | "timeout"
    pub status: String,
    pub timestamp: String,
}

/// Shared state that holds the per-run cancellation flag.
#[derive(Default)]
pub struct ExecutionState {
    pub cancel_flag: Arc<Mutex<bool>>,
}

// ─── Language Trait ───────────────────────────────────────────

/// Common interface all language executors must implement.
#[async_trait::async_trait]
pub trait LanguageExecutor: Send + Sync {
    async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel: Arc<Mutex<bool>>,
    ) -> Result<ExecutionResult, String>;
}

// ─── Shared Utilities ─────────────────────────────────────────

/// Create an isolated temporary directory for one execution.
pub fn create_temp_workspace() -> Result<tempfile::TempDir, String> {
    tempfile::Builder::new()
        .prefix("Tatpar_")
        .tempdir()
        .map_err(|e| format!("Failed to create temp workspace: {e}"))
}

/// Build a cancelled ExecutionResult. Used in Phase 3 cancellation flow.
#[allow(dead_code)]
pub fn cancelled_result() -> ExecutionResult {
    ExecutionResult {
        stdout: String::new(),
        stderr: "[Cancelled]".to_string(),
        exit_code: None,
        duration_ms: 0,
        status: "error".to_string(),
        timestamp: Utc::now().to_rfc3339(),
    }
}

/// Create a Command for the given program, correctly handling Windows
/// `.bat` / `.cmd` wrappers (kotlinc, tsc, npx, etc.) by routing them
/// through `cmd /C`. On Windows these scripts cannot be spawned directly
/// by a Rust process — they need the shell interpreter.
///
/// Additional arguments should be appended to the returned Command as
/// normal (they are passed after the script path to cmd /C).
pub fn new_command(program: &str) -> Command {
    // Try to resolve the full path first so we can inspect the extension.
    if let Ok(resolved) = which::which(program) {
        let ext = resolved
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_lowercase);

        if matches!(ext.as_deref(), Some("bat") | Some("cmd")) {
            // Must run Windows batch/cmd scripts via the shell
            let mut cmd = Command::new("cmd");
            cmd.arg("/C").arg(resolved);
            return cmd;
        }
        // Real executable (.exe or no extension on Unix)
        return Command::new(resolved);
    }
    // which() failed — return a Command that will produce a clear OS error
    Command::new(program)
}

/// Run a subprocess with a timeout; capture stdout/stderr.
pub async fn run_process(
    mut cmd: Command,
    timeout_secs: u64,
    _cancel: Arc<Mutex<bool>>,
) -> ExecutionResult {
    let start = Instant::now();
    let now = Utc::now().to_rfc3339();

    let run = async {
        match cmd.output().await {
            Ok(output) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code();
                let status = if exit_code == Some(0) { "success" } else { "error" };
                ExecutionResult {
                    stdout,
                    stderr,
                    exit_code,
                    duration_ms,
                    status: status.to_string(),
                    timestamp: now.clone(),
                }
            }
            Err(e) => ExecutionResult {
                stdout: String::new(),
                stderr: format!("Failed to spawn process: {e}"),
                exit_code: None,
                duration_ms: start.elapsed().as_millis() as u64,
                status: "error".to_string(),
                timestamp: now.clone(),
            },
        }
    };

    match timeout(Duration::from_secs(timeout_secs), run).await {
        Ok(result) => result,
        Err(_) => ExecutionResult {
            stdout: String::new(),
            stderr: format!("[Process timeout after {}s]", timeout_secs),
            exit_code: None,
            duration_ms: timeout_secs * 1000,
            status: "timeout".to_string(),
            timestamp: Utc::now().to_rfc3339(),
        },
    }
}
