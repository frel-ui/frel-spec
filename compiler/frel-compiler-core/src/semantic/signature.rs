// Module signature for Frel semantic analysis
//
// This module provides:
// - ModuleSignature: The cacheable/serializable interface of a compiled module
// - SignatureRegistry: Collection of module signatures for cross-module resolution
//
// The signature contains everything needed to compile code that imports from
// this module, without needing the original source.

use super::scope::{ScopeGraph, ScopeId, ScopeKind};
use super::symbol::{SymbolId, SymbolKind, SymbolTable};
use crate::source::Span;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Current signature format version
pub const SIGNATURE_VERSION: u32 = 1;

/// A module's public interface, cacheable and serializable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSignature {
    /// Signature format version (for compatibility checking)
    pub version: u32,

    /// Module path (e.g., "test.data")
    pub path: String,

    /// Exported declarations (top-level types visible to importers)
    pub exports: Vec<ExportedDecl>,

    /// Scope graph for this module
    pub scopes: SerializableScopeGraph,

    /// Symbol table for this module
    pub symbols: SerializableSymbolTable,
}

impl ModuleSignature {
    /// Create a new module signature
    pub fn new(
        path: String,
        exports: Vec<ExportedDecl>,
        scopes: &ScopeGraph,
        symbols: &SymbolTable,
    ) -> Self {
        Self {
            version: SIGNATURE_VERSION,
            path,
            exports,
            scopes: SerializableScopeGraph::from(scopes),
            symbols: SerializableSymbolTable::from(symbols),
        }
    }

    /// Check if this signature is compatible with the current version
    pub fn is_compatible(&self) -> bool {
        self.version == SIGNATURE_VERSION
    }

    /// Get an exported declaration by name
    pub fn get_export(&self, name: &str) -> Option<&ExportedDecl> {
        self.exports.iter().find(|e| e.name == name)
    }

    /// Look up a symbol by ID
    pub fn get_symbol(&self, id: SymbolId) -> Option<&SerializableSymbol> {
        self.symbols.get(id)
    }

    /// Look up a scope by ID
    pub fn get_scope(&self, id: ScopeId) -> Option<&SerializableScope> {
        self.scopes.get(id)
    }
}

/// An exported declaration from a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedDecl {
    /// Name of the exported type
    pub name: String,

    /// Kind of declaration
    pub kind: SymbolKind,

    /// Symbol ID in the module's symbol table
    pub symbol_id: SymbolId,

    /// Body scope ID (for looking up members)
    pub body_scope: Option<ScopeId>,
}

impl ExportedDecl {
    pub fn new(name: String, kind: SymbolKind, symbol_id: SymbolId, body_scope: Option<ScopeId>) -> Self {
        Self {
            name,
            kind,
            symbol_id,
            body_scope,
        }
    }
}

/// Serializable version of ScopeGraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableScopeGraph {
    scopes: Vec<SerializableScope>,
}

impl SerializableScopeGraph {
    pub fn get(&self, id: ScopeId) -> Option<&SerializableScope> {
        self.scopes.get(id.0 as usize)
    }

    pub fn iter(&self) -> impl Iterator<Item = &SerializableScope> {
        self.scopes.iter()
    }

    pub fn len(&self) -> usize {
        self.scopes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }
}

impl From<&ScopeGraph> for SerializableScopeGraph {
    fn from(graph: &ScopeGraph) -> Self {
        Self {
            scopes: graph.iter().map(SerializableScope::from).collect(),
        }
    }
}

/// Serializable version of Scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableScope {
    pub id: ScopeId,
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    pub name: Option<String>,
    pub span: Span,
    pub children: Vec<ScopeId>,
}

impl From<&super::scope::Scope> for SerializableScope {
    fn from(scope: &super::scope::Scope) -> Self {
        Self {
            id: scope.id,
            kind: scope.kind,
            parent: scope.parent,
            name: scope.name.clone(),
            span: scope.span,
            children: scope.children.clone(),
        }
    }
}

/// Serializable version of SymbolTable
///
/// The name_lookup map is rebuilt on access since JSON doesn't support
/// tuple keys directly.
#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableSymbolTable {
    symbols: Vec<SerializableSymbol>,
    /// Cached lookup map (not serialized, rebuilt on demand)
    #[serde(skip)]
    name_lookup: std::sync::OnceLock<HashMap<(ScopeId, String), SymbolId>>,
}

impl SerializableSymbolTable {
    pub fn get(&self, id: SymbolId) -> Option<&SerializableSymbol> {
        self.symbols.get(id.0 as usize)
    }

    fn ensure_lookup(&self) -> &HashMap<(ScopeId, String), SymbolId> {
        self.name_lookup.get_or_init(|| {
            let mut lookup = HashMap::new();
            for symbol in &self.symbols {
                lookup.insert((symbol.scope, symbol.name.clone()), symbol.id);
            }
            lookup
        })
    }

    pub fn lookup_local(&self, scope: ScopeId, name: &str) -> Option<SymbolId> {
        self.ensure_lookup().get(&(scope, name.to_string())).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = &SerializableSymbol> {
        self.symbols.iter()
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

impl Clone for SerializableSymbolTable {
    fn clone(&self) -> Self {
        Self {
            symbols: self.symbols.clone(),
            name_lookup: std::sync::OnceLock::new(),
        }
    }
}

impl From<&SymbolTable> for SerializableSymbolTable {
    fn from(table: &SymbolTable) -> Self {
        let symbols: Vec<SerializableSymbol> = table.iter().map(SerializableSymbol::from).collect();

        Self {
            symbols,
            name_lookup: std::sync::OnceLock::new(),
        }
    }
}

/// Serializable version of Symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableSymbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub scope: ScopeId,
    pub def_span: Span,
    pub body_scope: Option<ScopeId>,
    pub source_module: Option<String>,
}

impl From<&super::symbol::Symbol> for SerializableSymbol {
    fn from(symbol: &super::symbol::Symbol) -> Self {
        Self {
            id: symbol.id,
            name: symbol.name.clone(),
            kind: symbol.kind,
            scope: symbol.scope,
            def_span: symbol.def_span,
            body_scope: symbol.body_scope,
            source_module: symbol.source_module.clone(),
        }
    }
}

/// Registry of module signatures for cross-module resolution
#[derive(Debug, Default)]
pub struct SignatureRegistry {
    /// Module path -> ModuleSignature
    signatures: HashMap<String, ModuleSignature>,
}

impl SignatureRegistry {
    pub fn new() -> Self {
        Self {
            signatures: HashMap::new(),
        }
    }

    /// Register a module signature
    pub fn register(&mut self, signature: ModuleSignature) {
        self.signatures.insert(signature.path.clone(), signature);
    }

    /// Get a module signature by path
    pub fn get(&self, module_path: &str) -> Option<&ModuleSignature> {
        self.signatures.get(module_path)
    }

    /// Check if a module is registered
    pub fn contains(&self, module_path: &str) -> bool {
        self.signatures.contains_key(module_path)
    }

    /// Resolve an import (module_path, name) -> ExportedDecl
    pub fn resolve_import(&self, module_path: &str, name: &str) -> Option<&ExportedDecl> {
        self.get(module_path)?.get_export(name)
    }

    /// Get all registered module paths
    pub fn module_paths(&self) -> impl Iterator<Item = &String> {
        self.signatures.keys()
    }

    /// Number of registered modules
    pub fn len(&self) -> usize {
        self.signatures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.signatures.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_version() {
        let sig = ModuleSignature {
            version: SIGNATURE_VERSION,
            path: "test".to_string(),
            exports: vec![],
            scopes: SerializableScopeGraph { scopes: vec![] },
            symbols: SerializableSymbolTable {
                symbols: vec![],
                name_lookup: std::sync::OnceLock::new(),
            },
        };

        assert!(sig.is_compatible());
    }

    #[test]
    fn test_signature_serialization() {
        let sig = ModuleSignature {
            version: SIGNATURE_VERSION,
            path: "test.module".to_string(),
            exports: vec![ExportedDecl::new(
                "User".to_string(),
                SymbolKind::Scheme,
                SymbolId(0),
                Some(ScopeId(1)),
            )],
            scopes: SerializableScopeGraph { scopes: vec![] },
            symbols: SerializableSymbolTable {
                symbols: vec![],
                name_lookup: std::sync::OnceLock::new(),
            },
        };

        // Test JSON serialization
        let json = serde_json::to_string(&sig).unwrap();
        let deserialized: ModuleSignature = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.path, "test.module");
        assert_eq!(deserialized.exports.len(), 1);
        assert_eq!(deserialized.exports[0].name, "User");
    }

    #[test]
    fn test_signature_registry() {
        let mut registry = SignatureRegistry::new();

        let sig = ModuleSignature {
            version: SIGNATURE_VERSION,
            path: "test.data".to_string(),
            exports: vec![
                ExportedDecl::new("User".to_string(), SymbolKind::Scheme, SymbolId(0), Some(ScopeId(1))),
                ExportedDecl::new("Order".to_string(), SymbolKind::Scheme, SymbolId(1), Some(ScopeId(2))),
            ],
            scopes: SerializableScopeGraph { scopes: vec![] },
            symbols: SerializableSymbolTable {
                symbols: vec![],
                name_lookup: std::sync::OnceLock::new(),
            },
        };

        registry.register(sig);

        assert!(registry.contains("test.data"));
        assert!(!registry.contains("test.other"));

        let user = registry.resolve_import("test.data", "User");
        assert!(user.is_some());
        assert_eq!(user.unwrap().kind, SymbolKind::Scheme);

        let missing = registry.resolve_import("test.data", "Missing");
        assert!(missing.is_none());
    }
}
