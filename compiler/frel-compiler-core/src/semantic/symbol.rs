// Symbol table for Frel semantic analysis
//
// This module provides symbol representation and the symbol table
// that tracks all named entities in a Frel program.

use super::scope::{ScopeGraph, ScopeId};
use crate::source::Span;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SymbolId(pub u32);

/// The kind of symbol (what the name refers to)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    /// A backend declaration
    Backend,
    /// A blueprint declaration
    Blueprint,
    /// A scheme (data type) declaration
    Scheme,
    /// A contract declaration
    Contract,
    /// A theme declaration
    Theme,
    /// An enum declaration
    Enum,
    /// An enum variant
    EnumVariant,
    /// An arena declaration
    Arena,
    /// A field in a backend, scheme, or theme
    Field,
    /// A virtual/computed field in a scheme
    VirtualField,
    /// A method in a backend or contract
    Method,
    /// A command in a backend
    Command,
    /// A query in a backend
    Query,
    /// A parameter (function/method/blueprint parameter)
    Parameter,
    /// A local variable
    LocalVar,
    /// An instruction set in a theme
    InstructionSet,
    /// A theme variant
    ThemeVariant,
    /// An import alias
    Import,
}

impl SymbolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::Backend => "backend",
            SymbolKind::Blueprint => "blueprint",
            SymbolKind::Scheme => "scheme",
            SymbolKind::Contract => "contract",
            SymbolKind::Theme => "theme",
            SymbolKind::Enum => "enum",
            SymbolKind::EnumVariant => "enum variant",
            SymbolKind::Arena => "arena",
            SymbolKind::Field => "field",
            SymbolKind::VirtualField => "virtual field",
            SymbolKind::Method => "method",
            SymbolKind::Command => "command",
            SymbolKind::Query => "query",
            SymbolKind::Parameter => "parameter",
            SymbolKind::LocalVar => "local variable",
            SymbolKind::InstructionSet => "instruction set",
            SymbolKind::ThemeVariant => "theme variant",
            SymbolKind::Import => "import",
        }
    }

    /// Check if this symbol kind defines a type
    pub fn is_type_definition(&self) -> bool {
        matches!(
            self,
            SymbolKind::Backend
                | SymbolKind::Blueprint
                | SymbolKind::Scheme
                | SymbolKind::Contract
                | SymbolKind::Theme
                | SymbolKind::Enum
        )
    }

    /// Check if this symbol kind is callable
    pub fn is_callable(&self) -> bool {
        matches!(
            self,
            SymbolKind::Method
                | SymbolKind::Command
                | SymbolKind::Query
                | SymbolKind::Blueprint
        )
    }
}

/// A symbol representing a named entity
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Unique identifier
    pub id: SymbolId,
    /// Name of the symbol
    pub name: String,
    /// Kind of symbol
    pub kind: SymbolKind,
    /// Scope this symbol is defined in
    pub scope: ScopeId,
    /// Span where this symbol is defined
    pub def_span: Span,
    /// For type definitions, the scope they create
    pub body_scope: Option<ScopeId>,
    /// For imports, the resolved symbol (if resolved)
    pub resolved_import: Option<SymbolId>,
    /// Source module for external/imported symbols (None = local)
    pub source_module: Option<String>,
}

impl Symbol {
    pub fn new(
        id: SymbolId,
        name: impl Into<String>,
        kind: SymbolKind,
        scope: ScopeId,
        def_span: Span,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            scope,
            def_span,
            body_scope: None,
            resolved_import: None,
            source_module: None,
        }
    }

    /// Create a symbol that defines a scope (backend, blueprint, etc.)
    pub fn with_body_scope(mut self, body_scope: ScopeId) -> Self {
        self.body_scope = body_scope.into();
        self
    }

    /// Check if this symbol is from an external module (imported)
    pub fn is_external(&self) -> bool {
        self.source_module.is_some()
    }
}

/// Symbol table: arena-based storage with scope-based lookup
#[derive(Debug, Default)]
pub struct SymbolTable {
    /// All symbols stored in an arena
    symbols: Vec<Symbol>,
    /// Map from (scope, name) to symbol ID for fast lookup
    name_lookup: HashMap<(ScopeId, String), SymbolId>,
    /// Map from scope to symbols defined in that scope
    scope_symbols: HashMap<ScopeId, Vec<SymbolId>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            name_lookup: HashMap::new(),
            scope_symbols: HashMap::new(),
        }
    }

    /// Define a new symbol in a scope
    ///
    /// Returns the symbol ID, or None if a symbol with that name already exists
    /// in the scope (duplicate definition).
    pub fn define(
        &mut self,
        name: impl Into<String>,
        kind: SymbolKind,
        scope: ScopeId,
        def_span: Span,
    ) -> Option<SymbolId> {
        let name = name.into();
        let key = (scope, name.clone());

        // Check for duplicate in same scope
        if self.name_lookup.contains_key(&key) {
            return None;
        }

        let id = SymbolId(self.symbols.len() as u32);
        let symbol = Symbol::new(id, name, kind, scope, def_span);
        self.symbols.push(symbol);

        self.name_lookup.insert(key, id);
        self.scope_symbols.entry(scope).or_default().push(id);

        Some(id)
    }

    /// Define a symbol that creates a scope (backend, blueprint, etc.)
    pub fn define_with_scope(
        &mut self,
        name: impl Into<String>,
        kind: SymbolKind,
        scope: ScopeId,
        body_scope: ScopeId,
        def_span: Span,
    ) -> Option<SymbolId> {
        let id = self.define(name, kind, scope, def_span)?;
        if let Some(symbol) = self.symbols.get_mut(id.0 as usize) {
            symbol.body_scope = Some(body_scope);
        }
        Some(id)
    }

    /// Define an external symbol imported from another module
    ///
    /// Returns the symbol ID, or None if a symbol with that name already exists
    /// in the scope (duplicate definition).
    pub fn define_external(
        &mut self,
        name: impl Into<String>,
        kind: SymbolKind,
        scope: ScopeId,
        def_span: Span,
        source_module: String,
    ) -> Option<SymbolId> {
        let name = name.into();
        let key = (scope, name.clone());

        // Check for duplicate in same scope
        if self.name_lookup.contains_key(&key) {
            return None;
        }

        let id = SymbolId(self.symbols.len() as u32);
        let mut symbol = Symbol::new(id, name, kind, scope, def_span);
        symbol.source_module = Some(source_module);
        self.symbols.push(symbol);

        self.name_lookup.insert(key, id);
        self.scope_symbols.entry(scope).or_default().push(id);

        Some(id)
    }

    /// Get a symbol by ID
    pub fn get(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id.0 as usize)
    }

    /// Get a mutable symbol by ID
    pub fn get_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
        self.symbols.get_mut(id.0 as usize)
    }

    /// Look up a name in a specific scope only (no parent lookup)
    pub fn lookup_local(&self, scope: ScopeId, name: &str) -> Option<SymbolId> {
        self.name_lookup.get(&(scope, name.to_string())).copied()
    }

    /// Look up a name following the scope chain (local -> parent -> ... -> module)
    /// Does NOT check imports (that's done separately in resolve.rs)
    pub fn lookup_in_scope_chain(&self, scope: ScopeId, name: &str, scopes: &ScopeGraph) -> Option<SymbolId> {
        // First check the current scope
        if let Some(id) = self.lookup_local(scope, name) {
            return Some(id);
        }

        // Then check ancestors
        for ancestor in scopes.ancestors(scope) {
            if let Some(id) = self.lookup_local(ancestor, name) {
                return Some(id);
            }
        }

        None
    }

    /// Look up a name in immediate children of a scope (for finding loop scopes, etc.)
    /// Returns the symbol ID and the child scope ID where it was found
    pub fn lookup_in_children(&self, scope: ScopeId, name: &str, scopes: &ScopeGraph) -> Option<(SymbolId, ScopeId)> {
        if let Some(parent_scope) = scopes.get(scope) {
            for &child_scope in &parent_scope.children {
                if let Some(id) = self.lookup_local(child_scope, name) {
                    return Some((id, child_scope));
                }
            }
        }
        None
    }

    /// Get all symbols defined in a scope
    pub fn symbols_in_scope(&self, scope: ScopeId) -> impl Iterator<Item = &Symbol> {
        self.scope_symbols
            .get(&scope)
            .into_iter()
            .flat_map(|ids| ids.iter().filter_map(|&id| self.get(id)))
    }

    /// Get all symbols in the table
    pub fn iter(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.iter()
    }

    /// Get the number of symbols
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Check if a name exists anywhere in the ancestor chain (for shadowing check)
    pub fn name_exists_in_ancestors(&self, scope: ScopeId, name: &str, scopes: &ScopeGraph) -> Option<SymbolId> {
        for ancestor in scopes.ancestors(scope) {
            if let Some(id) = self.lookup_local(ancestor, name) {
                return Some(id);
            }
        }
        None
    }
}

/// Result of looking up a name
#[derive(Debug, Clone)]
pub enum LookupResult {
    /// Found exactly one symbol
    Found(SymbolId),
    /// Name not found
    NotFound,
    /// Name would shadow a symbol in an outer scope
    WouldShadow(SymbolId),
    /// Duplicate definition in the same scope
    Duplicate(SymbolId),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::scope::ScopeKind;

    #[test]
    fn test_symbol_table_define() {
        let mut scopes = ScopeGraph::new();
        let root = scopes.create_root(Span::default());

        let mut table = SymbolTable::new();
        let backend_id = table
            .define("MyBackend", SymbolKind::Backend, root, Span::new(0, 20))
            .unwrap();

        assert!(table.get(backend_id).is_some());
        assert_eq!(table.get(backend_id).unwrap().name, "MyBackend");
        assert_eq!(table.get(backend_id).unwrap().kind, SymbolKind::Backend);

        // Duplicate should fail
        assert!(table
            .define("MyBackend", SymbolKind::Backend, root, Span::new(30, 50))
            .is_none());
    }

    #[test]
    fn test_symbol_table_lookup() {
        let mut scopes = ScopeGraph::new();
        let root = scopes.create_root(Span::default());
        let backend_scope = scopes.create_named_scope(
            ScopeKind::Backend,
            root,
            "MyBackend",
            Span::new(0, 100),
        );

        let mut table = SymbolTable::new();
        table.define("Backend1", SymbolKind::Backend, root, Span::new(0, 20));
        table.define("count", SymbolKind::Field, backend_scope, Span::new(30, 40));

        // Local lookup in root
        assert!(table.lookup_local(root, "Backend1").is_some());
        assert!(table.lookup_local(root, "count").is_none());

        // Local lookup in backend scope
        assert!(table.lookup_local(backend_scope, "count").is_some());
        assert!(table.lookup_local(backend_scope, "Backend1").is_none());

        // Chain lookup from backend scope
        assert!(table.lookup_in_scope_chain(backend_scope, "Backend1", &scopes).is_some());
        assert!(table.lookup_in_scope_chain(backend_scope, "count", &scopes).is_some());
    }

    #[test]
    fn test_symbol_kind() {
        assert!(SymbolKind::Backend.is_type_definition());
        assert!(SymbolKind::Scheme.is_type_definition());
        assert!(!SymbolKind::Field.is_type_definition());

        assert!(SymbolKind::Method.is_callable());
        assert!(SymbolKind::Blueprint.is_callable());
        assert!(!SymbolKind::Field.is_callable());
    }

    #[test]
    fn test_symbols_in_scope() {
        let mut scopes = ScopeGraph::new();
        let root = scopes.create_root(Span::default());

        let mut table = SymbolTable::new();
        table.define("A", SymbolKind::Backend, root, Span::new(0, 10));
        table.define("B", SymbolKind::Blueprint, root, Span::new(20, 30));
        table.define("C", SymbolKind::Scheme, root, Span::new(40, 50));

        let names: Vec<_> = table.symbols_in_scope(root).map(|s| &s.name).collect();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&&"A".to_string()));
        assert!(names.contains(&&"B".to_string()));
        assert!(names.contains(&&"C".to_string()));
    }
}
