// Scope graph for Frel semantic analysis
//
// This module provides the scope hierarchy used for name resolution.
// Frel uses a 4-layer scoping model: local -> parent -> imports -> module
// with no shadowing allowed.

use crate::source::Span;
use serde::{Deserialize, Serialize};

/// Unique identifier for a scope in the scope graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScopeId(pub u32);

impl ScopeId {
    /// The root/module scope ID
    pub const ROOT: ScopeId = ScopeId(0);
}

/// The kind of scope, determines what names are valid and how lookup works
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeKind {
    /// Module-level scope (top-level declarations)
    Module,
    /// Backend scope (fields, methods, commands)
    Backend,
    /// Blueprint scope (local decls, children)
    Blueprint,
    /// Scheme scope (fields, virtuals)
    Scheme,
    /// Contract scope (methods)
    Contract,
    /// Theme scope (fields, instruction sets, variants)
    Theme,
    /// Enum scope (variants)
    Enum,
    /// Block scope (local variables within handlers, repeat loops, etc.)
    Block,
    /// Function/method parameter scope
    Parameters,
}

impl ScopeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScopeKind::Module => "module",
            ScopeKind::Backend => "backend",
            ScopeKind::Blueprint => "blueprint",
            ScopeKind::Scheme => "scheme",
            ScopeKind::Contract => "contract",
            ScopeKind::Theme => "theme",
            ScopeKind::Enum => "enum",
            ScopeKind::Block => "block",
            ScopeKind::Parameters => "parameters",
        }
    }
}

/// A scope in the scope graph
#[derive(Debug, Clone)]
pub struct Scope {
    /// Unique identifier
    pub id: ScopeId,
    /// Kind of scope
    pub kind: ScopeKind,
    /// Parent scope (None for root/module scope)
    pub parent: Option<ScopeId>,
    /// Name of the scope (for named scopes like backends, blueprints)
    pub name: Option<String>,
    /// Span in source where this scope is defined
    pub span: Span,
    /// Child scopes
    pub children: Vec<ScopeId>,
}

impl Scope {
    /// Create a new scope
    pub fn new(id: ScopeId, kind: ScopeKind, parent: Option<ScopeId>, span: Span) -> Self {
        Self {
            id,
            kind,
            parent,
            name: None,
            span,
            children: Vec::new(),
        }
    }

    /// Create a named scope
    pub fn named(
        id: ScopeId,
        kind: ScopeKind,
        parent: Option<ScopeId>,
        name: impl Into<String>,
        span: Span,
    ) -> Self {
        Self {
            id,
            kind,
            parent,
            name: Some(name.into()),
            span,
            children: Vec::new(),
        }
    }

    /// Check if this scope can contain a given kind of declaration
    pub fn can_contain_declaration(&self) -> bool {
        matches!(
            self.kind,
            ScopeKind::Module
                | ScopeKind::Backend
                | ScopeKind::Blueprint
                | ScopeKind::Scheme
                | ScopeKind::Contract
                | ScopeKind::Theme
                | ScopeKind::Enum
        )
    }

    /// Check if this scope represents a type definition
    pub fn is_type_definition(&self) -> bool {
        matches!(
            self.kind,
            ScopeKind::Backend
                | ScopeKind::Blueprint
                | ScopeKind::Scheme
                | ScopeKind::Contract
                | ScopeKind::Theme
                | ScopeKind::Enum
        )
    }
}

/// Arena-based storage for scopes
#[derive(Debug, Default)]
pub struct ScopeGraph {
    scopes: Vec<Scope>,
}

impl ScopeGraph {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    /// Create the root/module scope
    pub fn create_root(&mut self, span: Span) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);
        self.scopes.push(Scope::new(id, ScopeKind::Module, None, span));
        id
    }

    /// Create a new scope with a parent
    pub fn create_scope(
        &mut self,
        kind: ScopeKind,
        parent: ScopeId,
        span: Span,
    ) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);
        self.scopes.push(Scope::new(id, kind, Some(parent), span));
        // Add as child to parent
        if let Some(parent_scope) = self.scopes.get_mut(parent.0 as usize) {
            parent_scope.children.push(id);
        }
        id
    }

    /// Create a named scope with a parent
    pub fn create_named_scope(
        &mut self,
        kind: ScopeKind,
        parent: ScopeId,
        name: impl Into<String>,
        span: Span,
    ) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);
        self.scopes
            .push(Scope::named(id, kind, Some(parent), name, span));
        // Add as child to parent
        if let Some(parent_scope) = self.scopes.get_mut(parent.0 as usize) {
            parent_scope.children.push(id);
        }
        id
    }

    /// Get a scope by ID
    pub fn get(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id.0 as usize)
    }

    /// Get a mutable scope by ID
    pub fn get_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(id.0 as usize)
    }

    /// Get the parent scope of a scope
    pub fn parent(&self, id: ScopeId) -> Option<ScopeId> {
        self.get(id).and_then(|s| s.parent)
    }

    /// Iterate over all ancestors of a scope (excluding the scope itself)
    pub fn ancestors(&self, id: ScopeId) -> impl Iterator<Item = ScopeId> + '_ {
        std::iter::successors(self.parent(id), move |&id| self.parent(id))
    }

    /// Check if scope `inner` is nested inside `outer`
    pub fn is_nested_in(&self, inner: ScopeId, outer: ScopeId) -> bool {
        self.ancestors(inner).any(|id| id == outer)
    }

    /// Get all scopes
    pub fn iter(&self) -> impl Iterator<Item = &Scope> {
        self.scopes.iter()
    }

    /// Get the number of scopes
    pub fn len(&self) -> usize {
        self.scopes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_graph() {
        let mut graph = ScopeGraph::new();
        let root = graph.create_root(Span::default());
        assert_eq!(root, ScopeId::ROOT);

        let backend = graph.create_named_scope(
            ScopeKind::Backend,
            root,
            "MyBackend",
            Span::new(10, 50),
        );

        let method = graph.create_scope(ScopeKind::Parameters, backend, Span::new(20, 30));

        // Check parent relationships
        assert_eq!(graph.parent(root), None);
        assert_eq!(graph.parent(backend), Some(root));
        assert_eq!(graph.parent(method), Some(backend));

        // Check ancestors
        let ancestors: Vec<_> = graph.ancestors(method).collect();
        assert_eq!(ancestors, vec![backend, root]);

        // Check nesting
        assert!(graph.is_nested_in(method, root));
        assert!(graph.is_nested_in(backend, root));
        assert!(!graph.is_nested_in(root, backend));
    }

    #[test]
    fn test_scope_kind() {
        assert_eq!(ScopeKind::Module.as_str(), "module");
        assert_eq!(ScopeKind::Backend.as_str(), "backend");

        let scope = Scope::new(ScopeId(0), ScopeKind::Module, None, Span::default());
        assert!(scope.can_contain_declaration());

        let block = Scope::new(ScopeId(1), ScopeKind::Block, Some(ScopeId(0)), Span::default());
        assert!(!block.can_contain_declaration());
    }
}
