use super::resolve;
use super::scope::{ScopeGraph, ScopeId};
use super::signature::SignatureRegistry;
use super::symbol::{SymbolId, SymbolTable};
use super::typecheck;
use super::types::Type;
use super::Module;
use crate::diagnostic::Diagnostics;
use crate::source::Span;
use std::collections::HashMap;

/// Result of Phase 2 module analysis
#[derive(Debug)]
pub struct ModuleAnalysisResult {
    /// The scope graph
    pub scopes: ScopeGraph,
    /// The symbol table
    pub symbols: SymbolTable,
    /// Diagnostics generated during analysis
    pub diagnostics: Diagnostics,
    /// Name resolutions (span -> symbol)
    pub resolutions: HashMap<Span, SymbolId>,
    /// Expression types (span -> type)
    pub expr_types: HashMap<Span, Type>,
    /// Resolved type expressions (span -> type)
    pub type_resolutions: HashMap<Span, Type>,
}

impl ModuleAnalysisResult {
    /// Check if analysis succeeded (no errors)
    pub fn success(&self) -> bool {
        !self.diagnostics.has_errors()
    }

    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.diagnostics.error_count()
    }
}

/// Analyze a module with access to external module signatures (Phase 2 compilation)
///
/// This performs full semantic analysis including:
/// - Name resolution with import validation
/// - Type resolution using the registry for imported types
/// - Type checking
///
/// The registry should contain signatures for all modules that this module imports.
pub fn analyze_module(module: &Module, registry: &SignatureRegistry) -> ModuleAnalysisResult {
    let mut combined_diagnostics = Diagnostics::new();
    let mut combined_resolutions = HashMap::new();
    let mut combined_scopes = ScopeGraph::new();
    let mut combined_symbols = SymbolTable::new();
    let mut combined_expr_types = HashMap::new();
    let mut combined_type_resolutions = HashMap::new();

    // Process each file in the module
    for file in &module.files {
        // Phase 1a: Name resolution with registry validation
        let resolve_result = resolve::resolve_with_registry(file, registry);

        // Phase 1b: Type resolution and checking with registry
        let typecheck_result = typecheck::typecheck_with_registry(
            file,
            &resolve_result.scopes,
            &resolve_result.symbols,
            &resolve_result.imports,
            registry,
        );

        // Merge results
        if combined_symbols.is_empty() {
            combined_scopes = resolve_result.scopes;
            combined_symbols = resolve_result.symbols;
            combined_resolutions = resolve_result.resolutions;
        } else {
            // Check for conflicts between files
            for symbol in resolve_result.symbols.iter() {
                if symbol.scope == ScopeId::ROOT {
                    if combined_symbols.lookup_local(ScopeId::ROOT, &symbol.name).is_some() {
                        combined_diagnostics.error(
                            format!(
                                "duplicate definition of '{}' (also defined in another file)",
                                symbol.name
                            ),
                            symbol.def_span,
                        );
                    }
                }
            }
            // Merge resolutions (spans are unique per file so no conflicts)
            combined_resolutions.extend(resolve_result.resolutions);
        }

        // Merge diagnostics
        combined_diagnostics.merge(resolve_result.diagnostics);
        combined_diagnostics.merge(typecheck_result.diagnostics);

        // Merge type information
        combined_expr_types.extend(typecheck_result.expr_types);
        combined_type_resolutions.extend(typecheck_result.type_resolutions);
    }

    ModuleAnalysisResult {
        scopes: combined_scopes,
        symbols: combined_symbols,
        diagnostics: combined_diagnostics,
        resolutions: combined_resolutions,
        expr_types: combined_expr_types,
        type_resolutions: combined_type_resolutions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use super::super::signature_builder::build_signature;

    #[test]
    fn test_analyze_module_with_import() {
        // Build signature for test.data module
        let data_source = r#"
module test.data

scheme User {
    id: i64
    name: String
}

enum Status {
    Active
    Inactive
}
"#;
        let parse_result = parser::parse(data_source);
        let file = parse_result.file.unwrap();
        let data_module = Module::from_file(file);
        let sig_result = build_signature(&data_module);
        assert!(!sig_result.has_errors());

        // Register the signature
        let mut registry = SignatureRegistry::new();
        registry.register(sig_result.signature);

        // Now compile a module that imports from test.data
        let app_source = r#"
module test.app

import test.data.User
import test.data.Status

scheme UserInfo {
    user: User
    status: Status
}
"#;
        let parse_result = parser::parse(app_source);
        assert!(!parse_result.diagnostics.has_errors());
        let file = parse_result.file.unwrap();
        let app_module = Module::from_file(file);

        // Analyze with registry
        let result = analyze_module(&app_module, &registry);

        // Should succeed with no errors
        assert!(
            result.success(),
            "Expected no errors, got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_analyze_module_missing_import_module() {
        // Create an empty registry (no modules registered)
        let registry = SignatureRegistry::new();

        // Compile a module that imports from a non-existent module
        let source = r#"
module test.app

import test.missing.User

scheme Item {
    user: User
}
"#;
        let parse_result = parser::parse(source);
        assert!(!parse_result.diagnostics.has_errors());
        let file = parse_result.file.unwrap();
        let module = Module::from_file(file);

        // Analyze with registry
        let result = analyze_module(&module, &registry);

        // Should have an error about missing module
        assert!(!result.success());
        assert!(
            result.diagnostics.iter().any(|d| d.message.contains("module 'test.missing' not found")),
            "Expected 'module not found' error, got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_analyze_module_missing_export() {
        // Build signature for test.data module (with only User)
        let data_source = r#"
module test.data

scheme User {
    id: i64
}
"#;
        let parse_result = parser::parse(data_source);
        let file = parse_result.file.unwrap();
        let data_module = Module::from_file(file);
        let sig_result = build_signature(&data_module);

        let mut registry = SignatureRegistry::new();
        registry.register(sig_result.signature);

        // Try to import something that doesn't exist
        let source = r#"
module test.app

import test.data.Missing

scheme Item {
    data: Missing
}
"#;
        let parse_result = parser::parse(source);
        assert!(!parse_result.diagnostics.has_errors());
        let file = parse_result.file.unwrap();
        let module = Module::from_file(file);

        // Analyze with registry
        let result = analyze_module(&module, &registry);

        // Should have an error about missing export
        assert!(!result.success());
        assert!(
            result.diagnostics.iter().any(|d| d.message.contains("'Missing' is not exported")),
            "Expected 'not exported' error, got: {:?}",
            result.diagnostics
        );
    }
}
