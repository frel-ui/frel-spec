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
                    if let Some(existing_id) =
                        combined_symbols.lookup_local(ScopeId::ROOT, &symbol.name)
                    {
                        let existing = combined_symbols.get(existing_id);
                        // Allow duplicate imports from the same source module
                        let is_same_import = match (&existing, &symbol.source_module) {
                            (Some(e), Some(new_source)) => {
                                e.source_module.as_ref() == Some(new_source)
                            }
                            _ => false,
                        };

                        if !is_same_import {
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

    #[test]
    fn test_self_import_module_not_in_registry() {
        // Test what happens when a module tries to import from itself
        // and its signature is NOT in the registry (fresh compile)
        let source = r#"
module test.app

import test.app.User

scheme User {
    id: i64
}
"#;
        let parse_result = parser::parse(source);
        assert!(!parse_result.diagnostics.has_errors());
        let file = parse_result.file.unwrap();
        let module = Module::from_file(file);

        // Empty registry - the module's own signature is NOT registered
        let registry = SignatureRegistry::new();

        let result = analyze_module(&module, &registry);

        // Should fail because the module can't find itself in the registry
        assert!(!result.success());
        assert!(
            result.diagnostics.iter().any(|d| d.message.contains("module 'test.app' not found")),
            "Expected 'module not found' error for self-import, got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_self_import_module_in_registry() {
        // Test what happens when a module tries to import from itself
        // and its signature IS in the registry (e.g., recompilation in server)
        let source = r#"
module test.app

import test.app.User

scheme User {
    id: i64
}
"#;
        let parse_result = parser::parse(source);
        assert!(!parse_result.diagnostics.has_errors());
        let file = parse_result.file.unwrap();
        let module = Module::from_file(file);

        // First, build and register the module's signature (simulating previous compile)
        let sig_result = build_signature(&module);
        let mut registry = SignatureRegistry::new();
        registry.register(sig_result.signature);

        // Now recompile the same module with itself in the registry
        let result = analyze_module(&module, &registry);

        // This should fail with duplicate definition since User is both
        // imported from registry AND defined locally
        assert!(
            !result.success(),
            "Self-import should cause an error, but got success"
        );
    }

    #[test]
    fn test_circular_imports() {
        // Test circular imports: module_a imports module_b, module_b imports module_a
        // This is legal and should work if we compile in the right order

        // First compile module_a (without module_b available yet - import will fail)
        let source_a = r#"
module module_a

scheme UserA {
    id: i64
}
"#;
        // First compile module_b (without module_a available yet)
        let source_b = r#"
module module_b

scheme UserB {
    name: String
}
"#;

        // Build signatures for both (no imports yet)
        let parse_a = parser::parse(source_a);
        let module_a = Module::from_file(parse_a.file.unwrap());
        let sig_a = build_signature(&module_a);

        let parse_b = parser::parse(source_b);
        let module_b = Module::from_file(parse_b.file.unwrap());
        let sig_b = build_signature(&module_b);

        // Register both signatures
        let mut registry = SignatureRegistry::new();
        registry.register(sig_a.signature);
        registry.register(sig_b.signature);

        // Now compile module_a with import from module_b
        let source_a_with_import = r#"
module module_a

import module_b.UserB

scheme UserA {
    id: i64
    friend: UserB
}
"#;
        // And module_b with import from module_a
        let source_b_with_import = r#"
module module_b

import module_a.UserA

scheme UserB {
    name: String
    owner: UserA
}
"#;

        let parse_a2 = parser::parse(source_a_with_import);
        let module_a2 = Module::from_file(parse_a2.file.unwrap());
        let result_a = analyze_module(&module_a2, &registry);

        let parse_b2 = parser::parse(source_b_with_import);
        let module_b2 = Module::from_file(parse_b2.file.unwrap());
        let result_b = analyze_module(&module_b2, &registry);

        println!("Module A success: {}", result_a.success());
        for diag in result_a.diagnostics.iter() {
            println!("  A: {}", diag.message);
        }

        println!("Module B success: {}", result_b.success());
        for diag in result_b.diagnostics.iter() {
            println!("  B: {}", diag.message);
        }

        // Both should succeed - circular imports are allowed
        // as long as signatures are in the registry
        assert!(result_a.success(), "Module A should compile");
        assert!(result_b.success(), "Module B should compile");
    }

    #[test]
    fn test_duplicate_import_across_files() {
        // Test: two files in the same module both import the same thing
        // File B: module b, import a.User
        // File C: module b, import a.User

        // First, create module_a with User
        let source_a = r#"
module module_a

scheme User {
    id: i64
}
"#;
        let parse_a = parser::parse(source_a);
        let module_a = Module::from_file(parse_a.file.unwrap());
        let sig_a = build_signature(&module_a);

        let mut registry = SignatureRegistry::new();
        registry.register(sig_a.signature);

        // Now create module_b with TWO files, both importing User from module_a
        let source_b1 = r#"
module module_b

import module_a.User

scheme Profile {
    user: User
}
"#;
        let source_b2 = r#"
module module_b

import module_a.User

scheme Account {
    owner: User
}
"#;

        let parse_b1 = parser::parse(source_b1);
        let parse_b2 = parser::parse(source_b2);

        let module_b = Module::from_files(
            "module_b".to_string(),
            vec![parse_b1.file.unwrap(), parse_b2.file.unwrap()],
        );

        let result = analyze_module(&module_b, &registry);

        println!("Module B (2 files) success: {}", result.success());
        for diag in result.diagnostics.iter() {
            println!("  - {}", diag.message);
        }

        // Same import in multiple files should be allowed
        assert!(
            result.success(),
            "Same import in multiple files should be allowed, got errors: {:?}",
            result.diagnostics
        );
    }
}
