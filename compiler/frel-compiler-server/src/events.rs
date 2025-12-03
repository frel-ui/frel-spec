// Compilation events for SSE/WebSocket notifications

use serde::Serialize;

/// Events broadcast to connected clients
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CompilationEvent {
    /// Build started
    BuildStarted {
        modules: Vec<String>,
    },
    /// Build completed
    BuildCompleted {
        duration_ms: u64,
        modules_built: usize,
        error_count: usize,
    },
    /// File changed detected
    FileChanged {
        path: String,
    },
    /// Module updated (recompiled)
    ModuleUpdated {
        module: String,
        has_errors: bool,
    },
    /// Diagnostics updated for a module
    DiagnosticsUpdated {
        module: String,
        error_count: usize,
        warning_count: usize,
    },
}
