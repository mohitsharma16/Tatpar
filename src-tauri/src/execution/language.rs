// ============================================================
// Tatpar — Language Trait + Shared Types
// All language executors implement LanguageExecutor.
// ExecutionResult, ExecutionRequest, and ExecutionState are
// defined here and re-exported via execution/mod.rs.
// ============================================================

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::process::Command;

// ─── Shared Types ─────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionRequest {
    pub language: String,
    pub code: String,
    pub timeout_secs: Option<u64>,
    /// Optional mock stdin piped to the process before it starts reading.
    #[serde(default)]
    pub stdin: Option<String>,
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
        compiler_path: Option<String>,
        stdin: Option<String>,
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

/// Kill an entire process tree by PID.
///
/// Fix for issue #19:
///
/// On Windows, `child.kill()` only terminates the top-level process (usually
/// `cmd.exe`). The real compiler or runner (kotlinc JVM, tsc Node process, etc.)
/// is a child of cmd.exe and keeps running as an orphan — burning CPU and holding
/// locks on temp files indefinitely.
///
/// Solution: On Windows we run `taskkill /PID <pid> /T /F` which kills the
/// process AND all of its descendants atomically before falling through to the
/// regular kill. On non-Windows platforms we just do the normal kill.
async fn kill_process_tree(child: &mut tokio::process::Child) {
    #[cfg(target_os = "windows")]
    {
        if let Some(pid) = child.id() {
            // /T = kill tree (all children recursively)
            // /F = force (no graceful shutdown dialog)
            // CREATE_NO_WINDOW = no console flash in the windowless Tauri build
            use std::os::windows::process::CommandExt;
            let _ = std::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .creation_flags(0x08000000)
                .output();
        }
    }
    // Always attempt the Tokio native kill as a safety net (handles the case
    // where taskkill was unavailable or the process had already exited).
    let _ = child.kill().await;
}

/// Create a Command for the given program, correctly handling Windows
/// `.bat` / `.cmd` wrappers (kotlinc, tsc, npx, etc.) by routing them
/// through `cmd /C`. On Windows these scripts cannot be spawned directly
/// by a Rust process — they need the shell interpreter.
///
/// In the production (windowless) Tauri build we also set CREATE_NO_WINDOW
/// so that cmd.exe never flashes a console or triggers Windows shell
/// file-association dialogs (e.g. the `.ts` -> MPEG-2 handler).
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
            // Must run Windows batch/cmd scripts via the shell.
            // CREATE_NO_WINDOW (0x08000000) prevents console flash and
            // shell-association dialogs in the windowless production build.
            let mut cmd = Command::new("cmd");
            cmd.arg("/C").arg(resolved);
            #[cfg(target_os = "windows")]
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            return cmd;
        }
        // Real executable (.exe or no extension on Unix) — spawn directly.
        let mut cmd = Command::new(resolved);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        return cmd;
    }
    // which() failed — return a Command that will produce a clear OS error.
    let mut cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    cmd
}

/// Run a subprocess with timeout and cancellation support, ensuring process cleanup on exit.
pub async fn run_process(
    cmd: Command,
    timeout_secs: u64,
    cancel: Arc<Mutex<bool>>,
) -> ExecutionResult {
    run_process_with_stdin(cmd, timeout_secs, cancel, None).await
}

/// Like run_process but pipes `stdin_input` bytes into the child process before
/// it starts blocking on reads. Used for Issue #8 (interactive stdin drawer).
pub async fn run_process_with_stdin(
    mut cmd: Command,
    timeout_secs: u64,
    cancel: Arc<Mutex<bool>>,
    stdin_input: Option<String>,
) -> ExecutionResult {
    let start = Instant::now();
    let now = Utc::now().to_rfc3339();

    cmd.kill_on_drop(true);
    // Pipe stdout/stderr always. For stdin: if the caller provided mock
    // input, open a pipe so we can write it; otherwise null it out so
    // no child process can block waiting for interactive input.
    if stdin_input.is_some() {
        cmd.stdin(std::process::Stdio::piped());
    } else {
        cmd.stdin(std::process::Stdio::null());
    }
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            return ExecutionResult {
                stdout: String::new(),
                stderr: format!("Failed to spawn process: {e}"),
                exit_code: None,
                duration_ms: start.elapsed().as_millis() as u64,
                status: "error".to_string(),
                timestamp: now,
            };
        }
    };

    // Write mock stdin bytes then close the pipe so the child sees EOF.
    if let Some(input) = stdin_input {
        if let Some(mut stdin_pipe) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            let _ = stdin_pipe.write_all(input.as_bytes()).await;
            // Drop closes the pipe — child gets EOF on its stdin.
        }
    }

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let stdout_handle = tokio::spawn(async move {
        let mut buf = Vec::new();
        if let Some(mut stream) = stdout {
            let _ = tokio::io::AsyncReadExt::read_to_end(&mut stream, &mut buf).await;
        }
        buf
    });

    let stderr_handle = tokio::spawn(async move {
        let mut buf = Vec::new();
        if let Some(mut stream) = stderr {
            let _ = tokio::io::AsyncReadExt::read_to_end(&mut stream, &mut buf).await;
        }
        buf
    });

    let timeout_duration = Duration::from_secs(timeout_secs);
    let check_interval = Duration::from_millis(40);

    loop {
        // 1. Check if user cancelled execution
        if *cancel.lock().unwrap() {
            stdout_handle.abort();
            stderr_handle.abort();
            // Fix #19: kill the entire process tree so kotlinc JVM / tsc node
            // don't survive as orphans after cmd.exe is terminated.
            kill_process_tree(&mut child).await;
            return ExecutionResult {
                stdout: String::new(),
                stderr: "[Process cancelled by user]".to_string(),
                exit_code: None,
                duration_ms: start.elapsed().as_millis() as u64,
                status: "error".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            };
        }

        // 2. Check if process finished
        match child.try_wait() {
            Ok(Some(status)) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let stdout_buf = stdout_handle.await.unwrap_or_default();
                let stderr_buf = stderr_handle.await.unwrap_or_default();

                let stdout = String::from_utf8_lossy(&stdout_buf).to_string();
                let stderr = String::from_utf8_lossy(&stderr_buf).to_string();
                let exit_code = status.code();
                let exec_status = if exit_code == Some(0) { "success" } else { "error" };

                return ExecutionResult {
                    stdout,
                    stderr,
                    exit_code,
                    duration_ms,
                    status: exec_status.to_string(),
                    timestamp: now,
                };
            }
            Ok(None) => {
                // Process is still running — continue polling
            }
            Err(e) => {
                stdout_handle.abort();
                stderr_handle.abort();
                // Fix #19: kill the entire process tree on unexpected wait error.
                kill_process_tree(&mut child).await;
                return ExecutionResult {
                    stdout: String::new(),
                    stderr: format!("Error waiting for process: {e}"),
                    exit_code: None,
                    duration_ms: start.elapsed().as_millis() as u64,
                    status: "error".to_string(),
                    timestamp: now,
                };
            }
        }

        // 3. Check for timeout
        if start.elapsed() >= timeout_duration {
            stdout_handle.abort();
            stderr_handle.abort();
            // Fix #19: kill the entire process tree on timeout so no orphan
            // compiler processes keep running after the user's timeout.
            kill_process_tree(&mut child).await;
            return ExecutionResult {
                stdout: String::new(),
                stderr: format!("[Process timed out after {}s]", timeout_secs),
                exit_code: None,
                duration_ms: timeout_secs * 1000,
                status: "timeout".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            };
        }

        tokio::time::sleep(check_interval).await;
    }
}
