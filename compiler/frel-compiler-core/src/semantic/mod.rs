// Semantic analysis for Frel compiler
//
// This module provides semantic analysis passes including:
// - Scope graph construction
// - Symbol table population
// - Name resolution
// - Type checking (Phase 1b)
//
// The analysis is organized in layers that produce immutable output,
// enabling incremental compilation and IDE support.

pub mod dump;
pub mod resolve;
pub mod scope;
pub mod signature;
pub mod signature_builder;
pub mod symbol;
pub mod typecheck;
pub mod types;
pub mod module_analysis;

pub use dump::dump as dump_semantic;
pub use resolve::{resolve, resolve_with_registry, ResolveResult, Resolver};
pub use scope::{Scope, ScopeGraph, ScopeId, ScopeKind};
pub use signature::{
    ExportedDecl, ModuleSignature, SerializableScope, SerializableScopeGraph,
    SerializableSymbol, SerializableSymbolTable, SignatureRegistry, SIGNATURE_VERSION,
};
pub use signature_builder::{build_signature, SignatureResult};
pub use module_analysis::{analyze_module, ModuleAnalysisResult};
pub use symbol::{LookupResult, Symbol, SymbolId, SymbolKind, SymbolTable};
pub use typecheck::{typecheck, typecheck_with_registry, TypeCheckResult, TypeChecker};
pub use types::{ResolvedType, Type};

use crate::ast;
use crate::diagnostic::Diagnostics;
use crate::source::Span;

/// Result of semantic analysis
#[derive(Debug)]
pub struct SemanticResult {
    /// The scope graph
    pub scopes: ScopeGraph,
    /// The symbol table
    pub symbols: SymbolTable,
    /// Diagnostics generated during analysis
    pub diagnostics: Diagnostics,
    /// Name resolutions (span -> symbol)
    pub resolutions: std::collections::HashMap<Span, SymbolId>,
    /// Expression types (span -> type)
    pub expr_types: std::collections::HashMap<Span, Type>,
    /// Resolved type expressions (span -> type)
    pub type_resolutions: std::collections::HashMap<Span, Type>,
}

impl SemanticResult {
    /// Check if semantic analysis succeeded (no errors)
    pub fn success(&self) -> bool {
        !self.diagnostics.has_errors()
    }

    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.diagnostics.error_count()
    }

    /// Get the type of an expression at a span
    pub fn expr_type(&self, span: Span) -> Option<&Type> {
        self.expr_types.get(&span)
    }

    /// Get the resolved type for a type expression at a span
    pub fn resolved_type(&self, span: Span) -> Option<&Type> {
        self.type_resolutions.get(&span)
    }
}

/// Perform semantic analysis on a parsed file
///
/// Runs name resolution and type checking.
pub fn analyze(file: &ast::File) -> SemanticResult {
    // Phase 1a: Name resolution
    let resolve_result = resolve::resolve(file);

    // Phase 1b: Type resolution and checking
    let typecheck_result = typecheck::typecheck(file, &resolve_result.scopes, &resolve_result.symbols, &resolve_result.imports);

    // Merge diagnostics
    let mut diagnostics = resolve_result.diagnostics;
    diagnostics.merge(typecheck_result.diagnostics);

    SemanticResult {
        scopes: resolve_result.scopes,
        symbols: resolve_result.symbols,
        diagnostics,
        resolutions: resolve_result.resolutions,
        expr_types: typecheck_result.expr_types,
        type_resolutions: typecheck_result.type_resolutions,
    }
}

/// A module compilation unit (one or more files with the same module declaration)
#[derive(Debug)]
pub struct Module {
    /// Module path (e.g., "test.data")
    pub path: String,
    /// Files that make up this module (each has source_path and spans)
    pub files: Vec<ast::File>,
}

impl Module {
    /// Create a new module from a single file
    pub fn from_file(file: ast::File) -> Self {
        let path = file.module.clone();
        Self {
            path,
            files: vec![file],
        }
    }

    /// Create a new module from multiple files
    /// All files must have the same module path
    pub fn from_files(path: String, files: Vec<ast::File>) -> Self {
        Self { path, files }
    }

    /// Add a file to this module
    pub fn add_file(&mut self, file: ast::File) {
        self.files.push(file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn analyze_source(source: &str) -> SemanticResult {
        let parse_result = parser::parse(source);
        assert!(
            !parse_result.diagnostics.has_errors(),
            "Parse errors: {:?}",
            parse_result.diagnostics
        );
        analyze(&parse_result.file.unwrap())
    }

    #[test]
    fn test_analyze_complete_example() {
        let source = r#"
module test.counter

backend Counter {
    count: i32 = 0
    command increment()
    command decrement()
}

scheme Item {
    id: i64
    name: String
}

enum Status {
    Active
    Inactive
}

blueprint CounterView {
    with Counter
}
"#;
        let result = analyze_source(source);
        assert!(result.success(), "Errors: {:?}", result.diagnostics);

        // Verify symbols were collected
        // Counter (1) + count/increment/decrement (3) + Item (1) + id/name (2) + Status (1) + Active/Inactive (2) + CounterView (1) = 11
        assert!(
            result.symbols.len() >= 11,
            "Expected at least 11 symbols, got {}",
            result.symbols.len()
        );

        // Verify scope graph was built (module + Counter + Item + Status + CounterView = 5)
        assert!(
            result.scopes.len() >= 5,
            "Expected at least 5 scopes, got {}",
            result.scopes.len()
        );
    }

    #[test]
    fn test_analyze_with_errors() {
        let source = r#"
module test

backend A { }
backend A { }
"#;
        let result = analyze_source(source);
        assert!(!result.success());
        assert_eq!(result.error_count(), 1);
    }

}
