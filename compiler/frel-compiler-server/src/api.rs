// HTTP API endpoint handlers

use std::path::PathBuf;

use actix_web::{web, HttpResponse, Responder};
use frel_compiler_core::source::{LineIndex, Span};
use serde::{Deserialize, Serialize};

use crate::compiler;
use crate::state::SharedState;

/// Helper to compute line/column from a span using source content
fn span_to_line_col(span: &Span, source: &str) -> (Option<usize>, Option<usize>) {
    if span.start == 0 && span.end == 0 {
        return (None, None);
    }
    let line_index = LineIndex::new(source);
    let loc = line_index.line_col(span.start);
    (Some(loc.line as usize), Some(loc.col as usize))
}

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

#[derive(Serialize, Deserialize, Clone, PartialEq)]
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
    pub dump: String,
}

#[derive(Serialize)]
pub struct GeneratedResponse {
    pub module: String,
    pub javascript: String,
}

// === Scope dump types ===

#[derive(Serialize)]
pub struct ScopeInfo {
    pub id: u32,
    pub kind: String,
    pub parent: Option<u32>,
    pub name: Option<String>,
    pub children: Vec<u32>,
    pub symbols: Vec<SymbolInfo>,
}

#[derive(Serialize)]
pub struct SymbolInfo {
    pub id: u32,
    pub name: String,
    pub kind: String,
    pub body_scope: Option<u32>,
    pub source_module: Option<String>,
}

#[derive(Serialize)]
pub struct ScopeResponse {
    pub module: String,
    pub scopes: Vec<ScopeInfo>,
}

#[derive(Serialize)]
pub struct SourceResponse {
    pub path: String,
    pub content: String,
    pub module: Option<String>,
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

#[derive(Deserialize)]
pub struct WriteRequest {
    pub path: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct WriteResponse {
    pub success: bool,
    pub modules_rebuilt: Vec<String>,
    pub duration_ms: u64,
    pub error_count: usize,
}

// === Expectations types (for compiler development mode) ===

#[derive(Serialize, Deserialize, Clone)]
pub struct ModuleExpectations {
    pub module: String,
    pub ast: Option<serde_json::Value>,
    pub diagnostics: Vec<DiagnosticInfo>,
    pub generated_js: Option<String>,
}

#[derive(Serialize)]
pub struct ExpectationsResponse {
    pub module: String,
    pub exists: bool,
    pub expectations: Option<ModuleExpectations>,
}

#[derive(Serialize)]
pub struct SaveExpectationsResponse {
    pub success: bool,
    pub module: String,
}

#[derive(Serialize)]
pub struct CompareResponse {
    pub module: String,
    pub has_differences: bool,
    pub ast_matches: bool,
    pub diagnostics_match: bool,
    pub generated_js_matches: bool,
    pub current: ModuleExpectations,
    pub expected: Option<ModuleExpectations>,
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
        // Get source for line/column computation
        let source = state.sources.get(path).map(|s| s.content.as_str());

        for diag in entry.diagnostics.iter() {
            let severity = format!("{:?}", diag.severity).to_lowercase();
            if diag.severity == frel_compiler_core::Severity::Error {
                total_errors += 1;
            } else if diag.severity == frel_compiler_core::Severity::Warning {
                total_warnings += 1;
            }

            let (line, column) = source
                .map(|s| span_to_line_col(&diag.span, s))
                .unwrap_or((None, None));

            all_diagnostics.push(DiagnosticInfo {
                severity,
                code: diag.code.clone(),
                message: diag.message.clone(),
                file: Some(path.display().to_string()),
                line,
                column,
            });
        }
    }

    // Collect from analysis cache
    for (module, entry) in &state.analysis_cache {
        // Get source from first file in module for line/column computation
        let source = state
            .module_index
            .files_for_module(module)
            .first()
            .and_then(|p| state.sources.get(p))
            .map(|s| s.content.as_str());

        for diag in entry.result.diagnostics.iter() {
            let severity = format!("{:?}", diag.severity).to_lowercase();
            if diag.severity == frel_compiler_core::Severity::Error {
                total_errors += 1;
            } else if diag.severity == frel_compiler_core::Severity::Warning {
                total_warnings += 1;
            }

            let (line, column) = source
                .map(|s| span_to_line_col(&diag.span, s))
                .unwrap_or((None, None));

            all_diagnostics.push(DiagnosticInfo {
                severity,
                code: diag.code.clone(),
                message: diag.message.clone(),
                file: Some(module.clone()),
                line,
                column,
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

    // Get first file in module for file path and line/column computation
    let first_file = state
        .module_index
        .files_for_module(&module_path)
        .first()
        .cloned();
    let module_source = first_file
        .as_ref()
        .and_then(|p| state.sources.get(p))
        .map(|s| s.content.as_str());
    let file_display = first_file.as_ref().map(|p| p.display().to_string());

    // Get from analysis cache
    if let Some(entry) = state.analysis_cache.get(&module_path) {
        for diag in entry.result.diagnostics.iter() {
            let severity = format!("{:?}", diag.severity).to_lowercase();
            if diag.severity == frel_compiler_core::Severity::Error {
                error_count += 1;
            } else if diag.severity == frel_compiler_core::Severity::Warning {
                warning_count += 1;
            }

            let (line, column) = module_source
                .map(|s| span_to_line_col(&diag.span, s))
                .unwrap_or((None, None));

            diagnostics.push(DiagnosticInfo {
                severity,
                code: diag.code.clone(),
                message: diag.message.clone(),
                file: file_display.clone(),
                line,
                column,
            });
        }
    }

    // Also get parse diagnostics for files in this module
    for file_path in state.module_index.files_for_module(&module_path) {
        let source = state.sources.get(file_path).map(|s| s.content.as_str());

        if let Some(entry) = state.parse_cache.get(file_path) {
            for diag in entry.diagnostics.iter() {
                let severity = format!("{:?}", diag.severity).to_lowercase();
                if diag.severity == frel_compiler_core::Severity::Error {
                    error_count += 1;
                } else if diag.severity == frel_compiler_core::Severity::Warning {
                    warning_count += 1;
                }

                let (line, column) = source
                    .map(|s| span_to_line_col(&diag.span, s))
                    .unwrap_or((None, None));

                diagnostics.push(DiagnosticInfo {
                    severity,
                    code: diag.code.clone(),
                    message: diag.message.clone(),
                    file: Some(file_path.display().to_string()),
                    line,
                    column,
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
            // Generate human-readable dump format
            let dump = frel_compiler_core::ast::DumpVisitor::dump(&entry.file);
            return HttpResponse::Ok().json(AstResponse {
                module: module_path,
                ast: ast_json,
                dump,
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

/// GET /scope/{module} - Get scope dump for a module
pub async fn get_module_scope(
    state: web::Data<SharedState>,
    path: web::Path<String>,
) -> impl Responder {
    let module_path = path.into_inner();
    let state = state.read().await;

    if let Some(entry) = state.analysis_cache.get(&module_path) {
        let mut scopes = Vec::new();

        // Iterate through all scopes in the scope graph
        for scope in entry.result.scopes.iter() {
            // Get symbols defined in this scope
            let symbols: Vec<SymbolInfo> = entry
                .result
                .symbols
                .symbols_in_scope(scope.id)
                .map(|sym| SymbolInfo {
                    id: sym.id.0,
                    name: sym.name.clone(),
                    kind: sym.kind.as_str().to_string(),
                    body_scope: sym.body_scope.map(|s| s.0),
                    source_module: sym.source_module.clone(),
                })
                .collect();

            scopes.push(ScopeInfo {
                id: scope.id.0,
                kind: scope.kind.as_str().to_string(),
                parent: scope.parent.map(|p| p.0),
                name: scope.name.clone(),
                children: scope.children.iter().map(|c| c.0).collect(),
                symbols,
            });
        }

        return HttpResponse::Ok().json(ScopeResponse {
            module: module_path,
            scopes,
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

    println!("File changed: {}", path.display());

    let result = {
        let mut state = state.write().await;
        compiler::handle_file_change(&mut state, &path)
    };

    if !result.modules_rebuilt.is_empty() {
        println!(
            "  Rebuilt {} module(s) in {:?}, {} error(s)",
            result.modules_rebuilt.len(),
            result.duration,
            result.error_count
        );
    }

    HttpResponse::Ok().json(NotifyResponse {
        success: true,
        modules_rebuilt: result.modules_rebuilt,
        duration_ms: result.duration.as_millis() as u64,
        error_count: result.error_count,
    })
}

/// POST /write - Write content to a file and trigger recompilation
pub async fn post_write(
    state: web::Data<SharedState>,
    body: web::Json<WriteRequest>,
) -> impl Responder {
    let path = PathBuf::from(&body.path);

    // Write file to disk
    if let Err(e) = std::fs::write(&path, &body.content) {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to write file: {}", e),
            "path": path.display().to_string()
        }));
    }

    println!("File written: {}", path.display());

    // Trigger recompilation
    let result = {
        let mut state = state.write().await;
        compiler::handle_file_change(&mut state, &path)
    };

    if !result.modules_rebuilt.is_empty() {
        println!(
            "  Rebuilt {} module(s) in {:?}, {} error(s)",
            result.modules_rebuilt.len(),
            result.duration,
            result.error_count
        );
    }

    HttpResponse::Ok().json(WriteResponse {
        success: true,
        modules_rebuilt: result.modules_rebuilt,
        duration_ms: result.duration.as_millis() as u64,
        error_count: result.error_count,
    })
}

/// GET /source/{path} - Get source file content
pub async fn get_source(
    state: web::Data<SharedState>,
    path: web::Path<String>,
) -> impl Responder {
    let file_path = PathBuf::from(path.into_inner());
    let state = state.read().await;

    // Check if we have this file in cache
    if let Some(entry) = state.sources.get(&file_path) {
        let module = state.module_index.module_for_file(&file_path).map(|s| s.to_string());
        return HttpResponse::Ok().json(SourceResponse {
            path: file_path.display().to_string(),
            content: entry.content.clone(),
            module,
        });
    }

    // Try to read from disk if not in cache
    match std::fs::read_to_string(&file_path) {
        Ok(content) => {
            let module = state.module_index.module_for_file(&file_path).map(|s| s.to_string());
            HttpResponse::Ok().json(SourceResponse {
                path: file_path.display().to_string(),
                content,
                module,
            })
        }
        Err(_) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "File not found",
            "path": file_path.display().to_string()
        })),
    }
}

/// GET /events - SSE endpoint for compilation events
pub async fn get_events() -> impl Responder {
    // TODO: Implement SSE stream
    // For now, return a placeholder response
    HttpResponse::Ok()
        .content_type("text/event-stream")
        .body("data: {\"type\": \"connected\"}\n\n")
}

// === Expectations handlers (for compiler development mode) ===

fn expectations_dir(state: &crate::state::ProjectState) -> PathBuf {
    state.root.join(".grove-expectations")
}

fn expectations_file(state: &crate::state::ProjectState, module: &str) -> PathBuf {
    let sanitized = module.replace('/', "_").replace('\\', "_");
    expectations_dir(state).join(format!("{}.json", sanitized))
}

fn get_current_module_state(
    state: &crate::state::ProjectState,
    module_path: &str,
) -> ModuleExpectations {
    // Get AST
    let ast = state
        .module_index
        .files_for_module(module_path)
        .first()
        .and_then(|file_path| state.parse_cache.get(file_path))
        .map(|entry| serde_json::to_value(&entry.file).unwrap_or(serde_json::Value::Null));

    // Get first file for file path and line/column computation
    let first_file = state
        .module_index
        .files_for_module(module_path)
        .first()
        .cloned();
    let module_source = first_file
        .as_ref()
        .and_then(|p| state.sources.get(p))
        .map(|s| s.content.as_str());
    let file_display = first_file.as_ref().map(|p| p.display().to_string());

    // Get diagnostics
    let mut diagnostics = Vec::new();
    if let Some(entry) = state.analysis_cache.get(module_path) {
        for diag in entry.result.diagnostics.iter() {
            let (line, column) = module_source
                .map(|s| span_to_line_col(&diag.span, s))
                .unwrap_or((None, None));

            diagnostics.push(DiagnosticInfo {
                severity: format!("{:?}", diag.severity).to_lowercase(),
                code: diag.code.clone(),
                message: diag.message.clone(),
                file: file_display.clone(),
                line,
                column,
            });
        }
    }
    for file_path in state.module_index.files_for_module(module_path) {
        let source = state.sources.get(file_path).map(|s| s.content.as_str());

        if let Some(entry) = state.parse_cache.get(file_path) {
            for diag in entry.diagnostics.iter() {
                let (line, column) = source
                    .map(|s| span_to_line_col(&diag.span, s))
                    .unwrap_or((None, None));

                diagnostics.push(DiagnosticInfo {
                    severity: format!("{:?}", diag.severity).to_lowercase(),
                    code: diag.code.clone(),
                    message: diag.message.clone(),
                    file: Some(file_path.display().to_string()),
                    line,
                    column,
                });
            }
        }
    }

    // Get generated JS
    let generated_js = state
        .analysis_cache
        .get(module_path)
        .map(|entry| entry.generated_js.clone());

    ModuleExpectations {
        module: module_path.to_string(),
        ast,
        diagnostics,
        generated_js,
    }
}

/// GET /expectations/{module} - Get expected results for a module
pub async fn get_expectations(
    state: web::Data<SharedState>,
    path: web::Path<String>,
) -> impl Responder {
    let module_path = path.into_inner();
    let state = state.read().await;

    let exp_file = expectations_file(&state, &module_path);

    if exp_file.exists() {
        match std::fs::read_to_string(&exp_file) {
            Ok(content) => match serde_json::from_str::<ModuleExpectations>(&content) {
                Ok(expectations) => {
                    return HttpResponse::Ok().json(ExpectationsResponse {
                        module: module_path,
                        exists: true,
                        expectations: Some(expectations),
                    });
                }
                Err(_) => {}
            },
            Err(_) => {}
        }
    }

    HttpResponse::Ok().json(ExpectationsResponse {
        module: module_path,
        exists: false,
        expectations: None,
    })
}

/// POST /expectations/{module}/save - Save current results as expected
pub async fn save_expectations(
    state: web::Data<SharedState>,
    path: web::Path<String>,
) -> impl Responder {
    let module_path = path.into_inner();
    let state = state.read().await;

    let current = get_current_module_state(&state, &module_path);
    let exp_dir = expectations_dir(&state);
    let exp_file = expectations_file(&state, &module_path);

    // Create expectations directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&exp_dir) {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create expectations directory: {}", e)
        }));
    }

    // Write expectations file
    match serde_json::to_string_pretty(&current) {
        Ok(json) => match std::fs::write(&exp_file, json) {
            Ok(_) => HttpResponse::Ok().json(SaveExpectationsResponse {
                success: true,
                module: module_path,
            }),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to write expectations file: {}", e)
            })),
        },
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to serialize expectations: {}", e)
        })),
    }
}

/// GET /compare/{module} - Compare current results with expected
pub async fn compare_expectations(
    state: web::Data<SharedState>,
    path: web::Path<String>,
) -> impl Responder {
    let module_path = path.into_inner();
    let state = state.read().await;

    let current = get_current_module_state(&state, &module_path);
    let exp_file = expectations_file(&state, &module_path);

    let expected = if exp_file.exists() {
        std::fs::read_to_string(&exp_file)
            .ok()
            .and_then(|content| serde_json::from_str::<ModuleExpectations>(&content).ok())
    } else {
        None
    };

    let (ast_matches, diagnostics_match, generated_js_matches) = if let Some(ref exp) = expected {
        let ast_matches = current.ast == exp.ast;
        let diagnostics_match = current.diagnostics == exp.diagnostics;
        let generated_js_matches = current.generated_js == exp.generated_js;
        (ast_matches, diagnostics_match, generated_js_matches)
    } else {
        (true, true, true) // No expectations = no differences
    };

    let has_differences = !ast_matches || !diagnostics_match || !generated_js_matches;

    HttpResponse::Ok().json(CompareResponse {
        module: module_path,
        has_differences,
        ast_matches,
        diagnostics_match,
        generated_js_matches,
        current,
        expected,
    })
}
