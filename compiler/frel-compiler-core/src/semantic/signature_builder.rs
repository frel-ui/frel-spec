use super::resolve;
use super::scope::{ScopeGraph, ScopeId};
use super::signature::{ExportedDecl, ModuleSignature};
use super::symbol::SymbolTable;
use super::Module;
use crate::diagnostic::Diagnostics;
use crate::source::Span;

/// Result of building a module signature (Phase 1)
#[derive(Debug)]
pub struct SignatureResult {
    /// The module signature
    pub signature: ModuleSignature,
    /// Diagnostics from Phase 1 (name clashes, duplicate definitions)
    pub diagnostics: Diagnostics,
}

impl SignatureResult {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }
}

/// Build a module signature (Phase 1 compilation)
///
/// This performs scope building and symbol collection without cross-module
/// type resolution. The resulting signature can be cached and used by other
/// modules that import from this one.
pub fn build_signature(module: &Module) -> SignatureResult {
    let mut diagnostics = Diagnostics::new();
    let mut combined_scopes = ScopeGraph::new();
    let mut combined_symbols = SymbolTable::new();

    // Create root module scope
    let root_span = module.files.first().map(|_| Span::default()).unwrap_or_default();
    combined_scopes.create_root(root_span);

    // Process each file in the module
    for file in &module.files {
        // Run name resolution on this file
        let resolve_result = resolve::resolve(file);

        // Merge scopes and symbols from this file
        merge_resolve_result(
            &mut combined_scopes,
            &mut combined_symbols,
            &mut diagnostics,
            resolve_result,
        );
    }

    // Extract exported declarations (top-level type definitions)
    let exports = extract_exports(&combined_symbols);

    let signature = ModuleSignature::new(
        module.path.clone(),
        exports,
        &combined_scopes,
        &combined_symbols,
    );

    SignatureResult {
        signature,
        diagnostics,
    }
}

/// Merge a resolve result into the combined scopes and symbols
fn merge_resolve_result(
    combined_scopes: &mut ScopeGraph,
    combined_symbols: &mut SymbolTable,
    diagnostics: &mut Diagnostics,
    resolve_result: resolve::ResolveResult,
) {
    // Merge diagnostics from this file
    diagnostics.merge(resolve_result.diagnostics);

    // If we have multiple files, we take the first file's scopes/symbols
    // and report conflicts.
    if combined_symbols.is_empty() {
        // First file - just take its scopes and symbols directly
        // We need to rebuild combined_scopes from resolve_result.scopes
        // For now, we'll just use the resolve result directly
        *combined_scopes = resolve_result.scopes;
        *combined_symbols = resolve_result.symbols;
    } else {
        // Additional file - check for conflicts at module level
        for symbol in resolve_result.symbols.iter() {
            if symbol.scope == ScopeId::ROOT {
                // Top-level declaration - check for conflict
                if combined_symbols.lookup_local(ScopeId::ROOT, &symbol.name).is_some() {
                    diagnostics.error(
                        format!(
                            "duplicate definition of '{}' (also defined in another file)",
                            symbol.name
                        ),
                        symbol.def_span,
                    );
                }
            }
        }
        // TODO: Merge non-conflicting symbols with ID remapping
    }
}

/// Extract exported declarations from the symbol table
fn extract_exports(symbols: &SymbolTable) -> Vec<ExportedDecl> {
    symbols
        .symbols_in_scope(ScopeId::ROOT)
        .filter(|s| s.kind.is_type_definition())
        .map(|s| ExportedDecl::new(s.name.clone(), s.kind, s.id, s.body_scope))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use super::super::signature::SignatureRegistry;
    use crate::SymbolKind;

    #[test]
    fn test_build_signature_single_file() {
        let source = r#"
module test.data

scheme User {
    id: i64
    name: String
}

scheme Order {
    id: i64
    total: f64
}

enum Status {
    Pending
    Complete
}
"#;
        let parse_result = parser::parse(source);
        assert!(!parse_result.diagnostics.has_errors());

        let file = parse_result.file.unwrap();
        let module = Module::from_file(file);

        let result = build_signature(&module);
        assert!(!result.has_errors(), "Errors: {:?}", result.diagnostics);

        let sig = &result.signature;
        assert_eq!(sig.path, "test.data");
        assert_eq!(sig.exports.len(), 3); // User, Order, Status

        // Check exports
        let user = sig.get_export("User");
        assert!(user.is_some());
        assert_eq!(user.unwrap().kind, SymbolKind::Scheme);

        let order = sig.get_export("Order");
        assert!(order.is_some());

        let status = sig.get_export("Status");
        assert!(status.is_some());
        assert_eq!(status.unwrap().kind, SymbolKind::Enum);

        // Verify signature is serializable
        let json = serde_json::to_string(&sig).unwrap();
        assert!(json.contains("test.data"));
        assert!(json.contains("User"));
    }

    #[test]
    fn test_build_signature_with_registry() {
        // Build signature for test.data module
        let data_source = r#"
module test.data

scheme User {
    id: i64
    name: String
}
"#;
        let parse_result = parser::parse(data_source);
        let file = parse_result.file.unwrap();
        let module = Module::from_file(file);
        let result = build_signature(&module);

        // Register the signature
        let mut registry = SignatureRegistry::new();
        registry.register(result.signature);

        // Verify we can look up imports
        let user = registry.resolve_import("test.data", "User");
        assert!(user.is_some());
        assert_eq!(user.unwrap().name, "User");

        let missing = registry.resolve_import("test.data", "Missing");
        assert!(missing.is_none());

        let wrong_module = registry.resolve_import("test.other", "User");
        assert!(wrong_module.is_none());
    }

    #[test]
    fn test_backend_members_in_signature() {
        let source = r#"
module test.backend

backend EditorBackend {
    content: String
    command save()
}
"#;
        let parse_result = parser::parse(source);
        assert!(!parse_result.diagnostics.has_errors());

        let file = parse_result.file.unwrap();
        let module = Module::from_file(file);

        let result = build_signature(&module);
        assert!(!result.has_errors(), "Errors: {:?}", result.diagnostics);

        let sig = &result.signature;

        // Check EditorBackend export
        let editor_export = sig.get_export("EditorBackend");
        assert!(editor_export.is_some(), "EditorBackend should be exported");
        let editor_export = editor_export.unwrap();
        assert_eq!(editor_export.kind, SymbolKind::Backend);
        assert!(editor_export.body_scope.is_some(), "EditorBackend should have body_scope");

        let body_scope = editor_export.body_scope.unwrap();

        // Collect members in body scope
        let members: Vec<_> = sig.symbols.symbols_in_scope(body_scope).collect();

        // Should have both content (field) and save (command)
        assert!(members.len() >= 2, "Expected at least 2 members, got {}: {:?}",
            members.len(),
            members.iter().map(|m| &m.name).collect::<Vec<_>>());

        let content = members.iter().find(|m| m.name == "content");
        assert!(content.is_some(), "Should have content field");
        assert_eq!(content.unwrap().kind, SymbolKind::Field);

        let save = members.iter().find(|m| m.name == "save");
        assert!(save.is_some(), "Should have save command");
        assert_eq!(save.unwrap().kind, SymbolKind::Command);
    }
}
