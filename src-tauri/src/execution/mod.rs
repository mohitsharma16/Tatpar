// ============================================================
// Tatpar — Execution Engine (mod.rs)
// Dispatches execution requests to per-language executors
// ============================================================

pub mod language;
pub mod kotlin;
pub mod python;
pub mod java;
pub mod javascript;
pub mod typescript;
pub mod cpp;

// Re-export shared types so lib.rs can reference them
pub use language::{ExecutionRequest, ExecutionResult, ExecutionState};

use language::LanguageExecutor;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{command, State};

// (Types are defined in language.rs and re-exported above)

// ─── Tauri Commands ───────────────────────────────────────────

/// Run code in the given language and return the result.
#[command]
pub async fn execute_code(
    request: ExecutionRequest,
    state: State<'_, ExecutionState>,
) -> Result<ExecutionResult, String> {
    // Reset cancel flag before each run
    {
        let mut flag = state.cancel_flag.lock().unwrap();
        *flag = false;
    }

    let cancel = Arc::clone(&state.cancel_flag);
    let timeout = request.timeout_secs.unwrap_or(10);

    let result = match request.language.as_str() {
        "kotlin"     => kotlin::KotlinExecutor.execute(&request.code, timeout, cancel).await,
        "python"     => python::PythonExecutor.execute(&request.code, timeout, cancel).await,
        "java"       => java::JavaExecutor.execute(&request.code, timeout, cancel).await,
        "javascript" => javascript::JavaScriptExecutor.execute(&request.code, timeout, cancel).await,
        "typescript" => typescript::TypeScriptExecutor.execute(&request.code, timeout, cancel).await,
        "cpp"        => cpp::CppExecutor.execute(&request.code, timeout, cancel).await,
        other        => Err(format!("Unsupported language: {other}")),
    };

    result.map_err(|e| e.to_string())
}

/// Signal the currently running execution to be cancelled.
#[command]
pub async fn cancel_execution(
    state: State<'_, ExecutionState>,
) -> Result<(), String> {
    let mut flag = state.cancel_flag.lock().unwrap();
    *flag = true;
    Ok(())
}

/// Check which language runtimes are available on PATH.
#[command]
pub async fn check_languages() -> Result<HashMap<String, bool>, String> {
    let mut map = HashMap::new();
    map.insert("kotlin".to_string(),     which::which("kotlinc").is_ok());
    map.insert("python".to_string(),     which::which("python").is_ok() || which::which("python3").is_ok());
    map.insert("java".to_string(),       which::which("javac").is_ok());
    map.insert("javascript".to_string(), which::which("node").is_ok());
    map.insert("typescript".to_string(), which::which("tsc").is_ok() || which::which("npx").is_ok());
    map.insert("cpp".to_string(),        which::which("g++").is_ok() || which::which("clang++").is_ok());
    Ok(map)
}

/// Get the resolved path to the compiler/interpreter for a language.
#[command]
pub async fn get_compiler_path(language: String) -> Result<Option<String>, String> {
    let candidates: &[&str] = match language.as_str() {
        "kotlin"     => &["kotlinc"],
        "python"     => &["python3", "python"],
        "java"       => &["javac"],
        "javascript" => &["node"],
        "typescript" => &["tsc"],
        "cpp"        => &["g++", "clang++"],
        _            => return Ok(None),
    };

    for cmd in candidates {
        if let Ok(path) = which::which(cmd) {
            return Ok(Some(path.to_string_lossy().to_string()));
        }
    }
    Ok(None)
}
