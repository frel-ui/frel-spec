// HTTP API endpoint handlers

use std::path::PathBuf;

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::compiler;
use crate::state::SharedState;

// === Response types ===

#[derive(Serialize)]
pub struct StatusResponse {
    pub initialized: bool,
    pub error_count: usize,
    pub module_count: usize,
}

#[derive(Serialize)]
pub struct ModuleInfo {
    pub path: String,
    pub source_files: Vec<String>,
    pub has_errors: bool,
    pub error_count: usize,
    pub warning_count: usize,
}

#[derive(Serialize)]
pub struct ModulesResponse {
    pub modules: Vec<ModuleInfo>,
}

#[derive(Serialize)]
pub struct DiagnosticInfo {
    pub severity: String,
    pub code: Option<String>,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Serialize)]
pub struct DiagnosticsResponse {
    pub module: Option<String>,
    pub diagnostics: Vec<DiagnosticInfo>,
    pub error_count: usize,
    pub warning_count: usize,
}

#[derive(Serialize)]
pub struct AstResponse {
    pub module: String,
    pub ast: serde_json::Value,
}

#[derive(Serialize)]
pub struct GeneratedResponse {
    pub module: String,
    pub javascript: String,
}

#[derive(Deserialize)]
pub struct NotifyRequest {
    pub path: String,
}

#[derive(Serialize)]
pub struct NotifyResponse {
    pub success: bool,
    pub modules_rebuilt: Vec<String>,
    pub duration_ms: u64,
    pub error_count: usize,
}

// === Handlers ===

/// GET /status - Get server status
pub async fn get_status(state: web::Data<SharedState>) -> impl Responder {
    let state = state.read().await;
    HttpResponse::Ok().json(StatusResponse {
        initialized: state.initialized,
        error_count: state.error_count(),
        module_count: state.modules().len(),
    })
}

/// GET /modules - List all modules
pub async fn get_modules(state: web::Data<SharedState>) -> impl Responder {
    let state = state.read().await;
    let mut modules = Vec::new();

    for module_path in state.modules() {
        let source_files: Vec<String> = state
            .module_index
            .files_for_module(module_path)
            .iter()
            .map(|p| p.display().to_string())
            .collect();

        let (error_count, warning_count) = if let Some(entry) = state.analysis_cache.get(module_path) {
            (
                entry.result.diagnostics.error_count(),
                entry.result.diagnostics.warning_count(),
            )
        } else if let Some(entry) = state.signature_cache.get(module_path) {
            (
                entry.result.diagnostics.error_count(),
                entry.result.diagnostics.warning_count(),
            )
        } else {
            (0, 0)
        };

        modules.push(ModuleInfo {
            path: module_path.to_string(),
            source_files,
            has_errors: error_count > 0,
            error_count,
            warning_count,
        });
    }

    HttpResponse::Ok().json(ModulesResponse { modules })
}

/// GET /diagnostics - Get all diagnostics
pub async fn get_all_diagnostics(state: web::Data<SharedState>) -> impl Responder {
    let state = state.read().await;
    let mut all_diagnostics = Vec::new();
    let mut total_errors = 0;
    let mut total_warnings = 0;

    // Collect from parse cache
    for (path, entry) in &state.parse_cache {
        for diag in entry.diagnostics.iter() {
            let severity = format!("{:?}", diag.severity).to_lowercase();
            if diag.severity == frel_compiler_core::Severity::Error {
                total_errors += 1;
            } else if diag.severity == frel_compiler_core::Severity::Warning {
                total_warnings += 1;
            }
            all_diagnostics.push(DiagnosticInfo {
                severity,
                code: diag.code.clone(),
                message: diag.message.clone(),
                file: Some(path.display().to_string()),
                line: None, // Would need LineIndex to compute
                column: None,
            });
        }
    }

    // Collect from analysis cache
    for (module, entry) in &state.analysis_cache {
        for diag in entry.result.diagnostics.iter() {
            let severity = format!("{:?}", diag.severity).to_lowercase();
            if diag.severity == frel_compiler_core::Severity::Error {
                total_errors += 1;
            } else if diag.severity == frel_compiler_core::Severity::Warning {
                total_warnings += 1;
            }
            all_diagnostics.push(DiagnosticInfo {
                severity,
                code: diag.code.clone(),
                message: diag.message.clone(),
                file: Some(module.clone()),
                line: None,
                column: None,
            });
        }
    }

    HttpResponse::Ok().json(DiagnosticsResponse {
        module: None,
        diagnostics: all_diagnostics,
        error_count: total_errors,
        warning_count: total_warnings,
    })
}

/// GET /diagnostics/{module} - Get diagnostics for a specific module
pub async fn get_module_diagnostics(
    state: web::Data<SharedState>,
    path: web::Path<String>,
) -> impl Responder {
    let module_path = path.into_inner();
    let state = state.read().await;

    let mut diagnostics = Vec::new();
    let mut error_count = 0;
    let mut warning_count = 0;

    // Get from analysis cache
    if let Some(entry) = state.analysis_cache.get(&module_path) {
        for diag in entry.result.diagnostics.iter() {
            let severity = format!("{:?}", diag.severity).to_lowercase();
            if diag.severity == frel_compiler_core::Severity::Error {
                error_count += 1;
            } else if diag.severity == frel_compiler_core::Severity::Warning {
                warning_count += 1;
            }
            diagnostics.push(DiagnosticInfo {
                severity,
                code: diag.code.clone(),
                message: diag.message.clone(),
                file: None,
                line: None,
                column: None,
            });
        }
    }

    // Also get parse diagnostics for files in this module
    for file_path in state.module_index.files_for_module(&module_path) {
        if let Some(entry) = state.parse_cache.get(file_path) {
            for diag in entry.diagnostics.iter() {
                let severity = format!("{:?}", diag.severity).to_lowercase();
                if diag.severity == frel_compiler_core::Severity::Error {
                    error_count += 1;
                } else if diag.severity == frel_compiler_core::Severity::Warning {
                    warning_count += 1;
                }
                diagnostics.push(DiagnosticInfo {
                    severity,
                    code: diag.code.clone(),
                    message: diag.message.clone(),
                    file: Some(file_path.display().to_string()),
                    line: None,
                    column: None,
                });
            }
        }
    }

    HttpResponse::Ok().json(DiagnosticsResponse {
        module: Some(module_path),
        diagnostics,
        error_count,
        warning_count,
    })
}

/// GET /ast/{module} - Get AST for a module
pub async fn get_module_ast(
    state: web::Data<SharedState>,
    path: web::Path<String>,
) -> impl Responder {
    let module_path = path.into_inner();
    let state = state.read().await;

    // Get the first file's AST for this module
    if let Some(file_path) = state.module_index.files_for_module(&module_path).first() {
        if let Some(entry) = state.parse_cache.get(file_path) {
            // Serialize AST to JSON
            let ast_json = serde_json::to_value(&entry.file).unwrap_or(serde_json::Value::Null);
            return HttpResponse::Ok().json(AstResponse {
                module: module_path,
                ast: ast_json,
            });
        }
    }

    HttpResponse::NotFound().json(serde_json::json!({
        "error": "Module not found",
        "module": module_path
    }))
}

/// GET /generated/{module} - Get generated JavaScript for a module
pub async fn get_module_generated(
    state: web::Data<SharedState>,
    path: web::Path<String>,
) -> impl Responder {
    let module_path = path.into_inner();
    let state = state.read().await;

    if let Some(entry) = state.analysis_cache.get(&module_path) {
        return HttpResponse::Ok().json(GeneratedResponse {
            module: module_path,
            javascript: entry.generated_js.clone(),
        });
    }

    HttpResponse::NotFound().json(serde_json::json!({
        "error": "Module not found or not compiled",
        "module": module_path
    }))
}

/// POST /notify - Notify server of a file change
pub async fn post_notify(
    state: web::Data<SharedState>,
    body: web::Json<NotifyRequest>,
) -> impl Responder {
    let path = PathBuf::from(&body.path);

    let result = {
        let mut state = state.write().await;
        compiler::handle_file_change(&mut state, &path)
    };

    HttpResponse::Ok().json(NotifyResponse {
        success: true,
        modules_rebuilt: result.modules_rebuilt,
        duration_ms: result.duration.as_millis() as u64,
        error_count: result.error_count,
    })
}

/// GET /events - SSE endpoint for compilation events
pub async fn get_events() -> impl Responder {
    // TODO: Implement SSE stream
    // For now, return a placeholder response
    HttpResponse::Ok()
        .content_type("text/event-stream")
        .body("data: {\"type\": \"connected\"}\n\n")
}
