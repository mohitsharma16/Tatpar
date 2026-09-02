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
use tauri::{command, AppHandle, State};

// (Types are defined in language.rs and re-exported above)

// ─── Tauri Commands ───────────────────────────────────────────

/// Run code in the given language and return the result.
#[command]
pub async fn execute_code(
    app: AppHandle,
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
    let compiler_path = resolve_path_for_lang(&app, &request.language);

    let result = match request.language.as_str() {
        "kotlin"     => kotlin::KotlinExecutor.execute(&request.code, timeout, cancel, compiler_path).await,
        "python"     => python::PythonExecutor.execute(&request.code, timeout, cancel, compiler_path).await,
        "java"       => java::JavaExecutor.execute(&request.code, timeout, cancel, compiler_path).await,
        "javascript" => javascript::JavaScriptExecutor.execute(&request.code, timeout, cancel, compiler_path).await,
        "typescript" => typescript::TypeScriptExecutor.execute(&request.code, timeout, cancel, compiler_path).await,
        "cpp"        => cpp::CppExecutor.execute(&request.code, timeout, cancel, compiler_path).await,
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

/// Check which language runtimes are available on PATH or custom settings.
#[command]
pub async fn check_languages(app: AppHandle) -> Result<HashMap<String, bool>, String> {
    let languages = ["kotlin", "python", "java", "javascript", "typescript", "cpp"];
    let mut map = HashMap::new();

    for lang in languages {
        let is_available = resolve_path_for_lang(&app, lang).is_some();
        map.insert(lang.to_string(), is_available);
    }
    Ok(map)
}

/// Get the resolved path to the compiler/interpreter for a language.
#[command]
pub async fn get_compiler_path(app: AppHandle, language: String) -> Result<Option<String>, String> {
    Ok(resolve_path_for_lang(&app, &language))
}

pub fn resolve_path_for_lang(app: &AppHandle, language: &str) -> Option<String> {
    let settings = crate::settings::read_settings_sync(app);
    if let Some(ls) = settings.language_settings.get(language) {
        if let Some(ref custom_path) = ls.compiler_path {
            let trimmed = custom_path.trim();
            if !trimmed.is_empty() && std::path::Path::new(trimmed).exists() {
                return Some(trimmed.to_string());
            }
        }
    }

    let candidates: &[&str] = match language {
        "kotlin"     => &["kotlinc"],
        "python"     => &["python", "python3", "py"],
        "java"       => &["javac"],
        "javascript" => &["node", "bun"],
        "typescript" => &["tsc"],
        "cpp"        => &["g++", "clang++", "cl"],
        _            => return None,
    };

    for cmd in candidates {
        if let Ok(path) = which::which(cmd) {
            return Some(path.to_string_lossy().to_string());
        }
    }

    // Special fallback check for TypeScript via npx if tsc isn't directly on PATH
    if language == "typescript" && which::which("npx").is_ok() {
        if let Ok(npx_path) = which::which("npx") {
            return Some(format!("{} tsc", npx_path.to_string_lossy()));
        }
    }

    None
}
