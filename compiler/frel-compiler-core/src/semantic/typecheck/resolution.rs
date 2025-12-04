// Type resolution for Frel type checking
//
// This module handles resolving TypeExpr (AST) to Type (semantic).

use std::collections::HashMap;

use crate::ast::TypeExpr;
use crate::diagnostic::{codes, Diagnostic, Diagnostics};
use crate::source::Span;

use super::super::scope::{ScopeGraph, ScopeId};
use super::super::symbol::{Symbol, SymbolId, SymbolKind, SymbolTable};
use super::super::types::Type;

/// Type resolver that converts TypeExpr AST nodes to semantic Types
pub struct TypeResolver<'a> {
    pub scopes: &'a ScopeGraph,
    pub symbols: &'a SymbolTable,
    pub imports: &'a HashMap<String, String>,
    pub current_scope: ScopeId,
    pub type_resolutions: HashMap<Span, Type>,
    pub diagnostics: Diagnostics,
}

impl<'a> TypeResolver<'a> {
    pub fn new(
        scopes: &'a ScopeGraph,
        symbols: &'a SymbolTable,
        imports: &'a HashMap<String, String>,
    ) -> Self {
        Self {
            scopes,
            symbols,
            imports,
            current_scope: ScopeId::ROOT,
            type_resolutions: HashMap::new(),
            diagnostics: Diagnostics::new(),
        }
    }

    /// Resolve a TypeExpr to a Type
    pub fn resolve_type_expr(&mut self, type_expr: &TypeExpr, span: Span) -> Type {
        let ty = match type_expr {
            TypeExpr::Named(name) => self.resolve_named_type(name, span),
            TypeExpr::Nullable(inner) => {
                let inner_ty = self.resolve_type_expr(inner, span);
                Type::Nullable(Box::new(inner_ty))
            }
            TypeExpr::Ref(inner) => {
                let inner_ty = self.resolve_type_expr(inner, span);
                Type::Ref(Box::new(inner_ty))
            }
            TypeExpr::Draft(inner) => {
                let inner_ty = self.resolve_type_expr(inner, span);
                Type::Draft(Box::new(inner_ty))
            }
            TypeExpr::Asset(inner) => {
                let inner_ty = self.resolve_type_expr(inner, span);
                Type::Asset(Box::new(inner_ty))
            }
            TypeExpr::List(elem) => {
                let elem_ty = self.resolve_type_expr(elem, span);
                Type::List(Box::new(elem_ty))
            }
            TypeExpr::Set(elem) => {
                let elem_ty = self.resolve_type_expr(elem, span);
                Type::Set(Box::new(elem_ty))
            }
            TypeExpr::Map(key, value) => {
                let key_ty = self.resolve_type_expr(key, span);
                let value_ty = self.resolve_type_expr(value, span);
                Type::Map(Box::new(key_ty), Box::new(value_ty))
            }
            TypeExpr::Tree(elem) => {
                let elem_ty = self.resolve_type_expr(elem, span);
                Type::Tree(Box::new(elem_ty))
            }
            TypeExpr::Blueprint(params) => {
                // Blueprint type with parameter types
                // For now, just resolve the parameters
                let _param_types: Vec<_> = params
                    .iter()
                    .map(|p| self.resolve_type_expr(p, span))
                    .collect();
                // Blueprint types without a specific symbol are represented as Unknown for now
                // TODO: This needs better handling for parametric blueprints
                Type::Unknown
            }
            TypeExpr::Accessor(inner) => {
                let inner_ty = self.resolve_type_expr(inner, span);
                Type::Accessor(Box::new(inner_ty))
            }
        };

        self.type_resolutions.insert(span, ty.clone());
        ty
    }

    /// Resolve a named type (either intrinsic or user-defined)
    pub fn resolve_named_type(&mut self, name: &str, span: Span) -> Type {
        // First try intrinsic types
        if let Some(ty) = Type::from_intrinsic_name(name) {
            return ty;
        }

        // Then look up user-defined types in the symbol table
        if let Some(symbol_id) = self
            .symbols
            .lookup_in_scope_chain(self.current_scope, name, self.scopes)
        {
            if let Some(symbol) = self.symbols.get(symbol_id) {
                return symbol_to_type(symbol);
            }
        }

        // Check imports - if name is imported, treat as external type
        if self.imports.contains_key(name) {
            // Imported type - return Unknown since we don't have the actual definition
            // This allows the code to type-check without cross-module resolution
            return Type::Unknown;
        }

        // Type not found
        self.diagnostics.add(
            Diagnostic::from_code(&codes::E0402, span, format!("unknown type `{}`", name))
                .with_help("Check the spelling or make sure the type is defined or imported."),
        );
        Type::Error
    }
}

/// Convert a symbol to its corresponding type
pub fn symbol_to_type(symbol: &Symbol) -> Type {
    match symbol.kind {
        SymbolKind::Scheme => Type::Scheme(symbol.id),
        SymbolKind::Backend => Type::Backend(symbol.id),
        SymbolKind::Blueprint => Type::Blueprint(symbol.id),
        SymbolKind::Contract => Type::Contract(symbol.id),
        SymbolKind::Theme => Type::Theme(symbol.id),
        SymbolKind::Enum => Type::Enum(symbol.id),
        _ => Type::Error, // Not a type definition
    }
}

/// Look up the type of an identifier in the symbol table
pub fn lookup_identifier_type(
    name: &str,
    current_scope: ScopeId,
    symbols: &SymbolTable,
    scopes: &ScopeGraph,
    symbol_types: &HashMap<SymbolId, Type>,
) -> Type {
    if let Some(symbol_id) = symbols.lookup_in_scope_chain(current_scope, name, scopes) {
        if let Some(symbol) = symbols.get(symbol_id) {
            // Check if we have a type for this symbol
            if let Some(ty) = symbol_types.get(&symbol_id) {
                return ty.clone();
            }
            // Otherwise, derive from symbol kind
            return match symbol.kind {
                SymbolKind::Field
                | SymbolKind::VirtualField
                | SymbolKind::Parameter
                | SymbolKind::LocalVar => {
                    // Would need to look up the declared type
                    Type::Unknown
                }
                SymbolKind::Backend => Type::Backend(symbol_id),
                SymbolKind::Blueprint => Type::Blueprint(symbol_id),
                SymbolKind::Scheme => Type::Scheme(symbol_id),
                SymbolKind::Enum => Type::Enum(symbol_id),
                SymbolKind::Contract => Type::Contract(symbol_id),
                SymbolKind::Theme => Type::Theme(symbol_id),
                _ => Type::Unknown,
            };
        }
    }

    // Not found - already reported during name resolution
    Type::Error
}
