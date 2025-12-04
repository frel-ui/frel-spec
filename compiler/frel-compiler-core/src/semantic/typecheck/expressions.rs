// Expression type inference for Frel type checking
//
// This module handles inferring and checking types of expressions.

use std::collections::HashMap;

use crate::ast;
use crate::diagnostic::{codes, Diagnostic, Diagnostics};
use crate::source::Span;

use super::super::scope::{ScopeGraph, ScopeId};
use super::super::symbol::{SymbolId, SymbolTable};
use super::super::types::Type;
use super::operators::{
    expect_bool, infer_binary_op_type, infer_unary_op_type, types_compatible,
};
use super::resolution::lookup_identifier_type;

/// Expression type checker
pub struct ExprChecker<'a> {
    pub scopes: &'a ScopeGraph,
    pub symbols: &'a SymbolTable,
    pub symbol_types: &'a HashMap<SymbolId, Type>,
    pub current_scope: ScopeId,
    pub context_span: Span,
    pub expr_types: HashMap<Span, Type>,
    pub diagnostics: Diagnostics,
}

impl<'a> ExprChecker<'a> {
    pub fn new(
        scopes: &'a ScopeGraph,
        symbols: &'a SymbolTable,
        symbol_types: &'a HashMap<SymbolId, Type>,
        current_scope: ScopeId,
        context_span: Span,
    ) -> Self {
        Self {
            scopes,
            symbols,
            symbol_types,
            current_scope,
            context_span,
            expr_types: HashMap::new(),
            diagnostics: Diagnostics::new(),
        }
    }

    /// Check an expression against an expected type (bidirectional type checking)
    ///
    /// This is used when we have a declared type and want to check the expression
    /// against it, allowing better type inference for literals like empty lists.
    pub fn check_expr_type(&mut self, expr: &ast::Expr, expected: &Type) -> Type {
        match expr {
            // For empty lists, use the expected element type
            ast::Expr::List(items) if items.is_empty() => {
                if let Type::List(elem_ty) = expected {
                    let ty = Type::List(elem_ty.clone());
                    self.expr_types.insert(self.context_span, ty.clone());
                    ty
                } else {
                    // Expected type is not a list, infer as unknown
                    let ty = Type::List(Box::new(Type::Unknown));
                    self.expr_types.insert(self.context_span, ty.clone());
                    ty
                }
            }
            // For null, use the expected nullable inner type
            ast::Expr::Null => {
                let ty = if let Type::Nullable(inner) = expected {
                    Type::Nullable(inner.clone())
                } else {
                    Type::Nullable(Box::new(Type::Unknown))
                };
                self.expr_types.insert(self.context_span, ty.clone());
                ty
            }
            // For other expressions, infer normally
            _ => self.infer_expr_type(expr),
        }
    }

    /// Infer the type of an expression
    pub fn infer_expr_type(&mut self, expr: &ast::Expr) -> Type {
        let ty = match expr {
            // Literals
            ast::Expr::Null => Type::Nullable(Box::new(Type::Unknown)),
            ast::Expr::Bool(_) => Type::Bool,
            ast::Expr::Int(n) => {
                // Infer integer size based on value
                if *n >= i32::MIN as i64 && *n <= i32::MAX as i64 {
                    Type::I32
                } else {
                    Type::I64
                }
            }
            ast::Expr::Float(_) => Type::F64,
            ast::Expr::Color(_) => Type::Color,
            ast::Expr::String(_) => Type::String,
            ast::Expr::StringTemplate(elements) => {
                // Check interpolated expressions
                for elem in elements {
                    if let ast::TemplateElement::Interpolation(inner) = elem {
                        self.infer_expr_type(inner);
                    }
                }
                Type::String
            }
            ast::Expr::List(items) => {
                if items.is_empty() {
                    Type::List(Box::new(Type::Unknown))
                } else {
                    let first_type = self.infer_expr_type(&items[0]);
                    // Check all items have compatible types
                    for item in items.iter().skip(1) {
                        let item_type = self.infer_expr_type(item);
                        if !types_compatible(&first_type, &item_type) {
                            // Report type mismatch
                            self.diagnostics.add(Diagnostic::from_code(
                                &codes::E0401,
                                self.context_span,
                                format!(
                                    "list element type mismatch: expected `{}`, found `{}`",
                                    first_type, item_type
                                ),
                            ));
                        }
                    }
                    Type::List(Box::new(first_type))
                }
            }
            ast::Expr::Object(fields) => {
                // Object literals create anonymous scheme-like types
                for (_, value) in fields {
                    self.infer_expr_type(value);
                }
                // For now, return Unknown as we don't have structural types yet
                Type::Unknown
            }
            ast::Expr::Identifier(name) => lookup_identifier_type(
                name,
                self.current_scope,
                self.symbols,
                self.scopes,
                self.symbol_types,
            ),
            ast::Expr::QualifiedName(parts) => {
                if let Some(first) = parts.first() {
                    let base_type = lookup_identifier_type(
                        first,
                        self.current_scope,
                        self.symbols,
                        self.scopes,
                        self.symbol_types,
                    );
                    // Resolve field accesses
                    let mut current = base_type;
                    for field in parts.iter().skip(1) {
                        current = self.resolve_field_access(&current, field);
                    }
                    current
                } else {
                    Type::Error
                }
            }
            ast::Expr::Binary { op, left, right } => {
                let left_type = self.infer_expr_type(left);
                let right_type = self.infer_expr_type(right);
                infer_binary_op_type(
                    *op,
                    &left_type,
                    &right_type,
                    self.context_span,
                    &mut self.diagnostics,
                )
            }
            ast::Expr::Unary { op, expr } => {
                let operand_type = self.infer_expr_type(expr);
                infer_unary_op_type(*op, &operand_type, self.context_span, &mut self.diagnostics)
            }
            ast::Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let cond_type = self.infer_expr_type(condition);
                expect_bool(&cond_type, self.context_span, &mut self.diagnostics);
                let then_type = self.infer_expr_type(then_expr);
                let else_type = self.infer_expr_type(else_expr);
                // Result type is the common type of both branches
                if types_compatible(&then_type, &else_type) {
                    then_type
                } else {
                    self.diagnostics.add(Diagnostic::from_code(
                        &codes::E0401,
                        self.context_span,
                        format!(
                            "ternary branches have incompatible types: `{}` vs `{}`",
                            then_type, else_type
                        ),
                    ));
                    Type::Error
                }
            }
            ast::Expr::FieldAccess { base, field } => {
                let base_type = self.infer_expr_type(base);
                self.resolve_field_access(&base_type, field)
            }
            ast::Expr::OptionalChain { base, field } => {
                let base_type = self.infer_expr_type(base);
                // Optional chaining requires nullable base
                let inner_type = match &base_type {
                    Type::Nullable(inner) => inner.as_ref(),
                    _ => {
                        // Not nullable - optional chain is unnecessary but allowed
                        &base_type
                    }
                };
                let field_type = self.resolve_field_access(inner_type, field);
                // Result is nullable
                field_type.make_nullable()
            }
            ast::Expr::Call { callee, args } => {
                let callee_type = self.infer_expr_type(callee);
                // Type check arguments
                for arg in args {
                    self.infer_expr_type(arg);
                }
                self.infer_call_result_type(&callee_type)
            }
        };

        // Use context_span since Expr doesn't carry its own span
        self.expr_types.insert(self.context_span, ty.clone());
        ty
    }

    /// Resolve a field access on a type
    pub fn resolve_field_access(&mut self, base_type: &Type, field: &str) -> Type {
        match base_type {
            // Ref types unwrap to their inner type for field access
            Type::Ref(inner) => self.resolve_field_access(inner, field),
            Type::Scheme(symbol_id) | Type::Backend(symbol_id) | Type::Theme(symbol_id) => {
                // Look up field in the type's scope
                if let Some(symbol) = self.symbols.get(*symbol_id) {
                    if let Some(body_scope) = symbol.body_scope {
                        if let Some(field_id) = self.symbols.lookup_local(body_scope, field) {
                            if self.symbols.get(field_id).is_some() {
                                // Return the field's type
                                return self
                                    .symbol_types
                                    .get(&field_id)
                                    .cloned()
                                    .unwrap_or(Type::Unknown);
                            }
                        }
                    }
                }
                self.diagnostics.add(Diagnostic::from_code(
                    &codes::E0301,
                    self.context_span,
                    format!("no field `{}` on type `{}`", field, self.type_name(base_type)),
                ));
                Type::Error
            }
            Type::Contract(symbol_id) => {
                // Look up method in the contract's scope
                if let Some(symbol) = self.symbols.get(*symbol_id) {
                    if let Some(body_scope) = symbol.body_scope {
                        if let Some(method_id) = self.symbols.lookup_local(body_scope, field) {
                            if self.symbols.get(method_id).is_some() {
                                // Return the method's type
                                return self
                                    .symbol_types
                                    .get(&method_id)
                                    .cloned()
                                    .unwrap_or(Type::Unknown);
                            }
                        }
                    }
                }
                self.diagnostics.add(Diagnostic::from_code(
                    &codes::E0301,
                    self.context_span,
                    format!("no method `{}` on contract `{}`", field, self.type_name(base_type)),
                ));
                Type::Error
            }
            Type::Enum(symbol_id) => {
                // Enum variant access
                if let Some(symbol) = self.symbols.get(*symbol_id) {
                    if let Some(body_scope) = symbol.body_scope {
                        if self.symbols.lookup_local(body_scope, field).is_some() {
                            // Return the enum type itself (variant has same type as enum)
                            return base_type.clone();
                        }
                    }
                }
                self.diagnostics.add(Diagnostic::from_code(
                    &codes::E0301,
                    self.context_span,
                    format!("no variant `{}` in enum `{}`", field, self.type_name(base_type)),
                ));
                Type::Error
            }
            Type::Nullable(_) => {
                // Cannot access field on nullable without optional chaining
                self.diagnostics.add(Diagnostic::from_code(
                    &codes::E0406,
                    self.context_span,
                    "cannot access field on nullable type without optional chaining `?.`",
                ));
                Type::Error
            }
            Type::Error | Type::Unknown => Type::Error,
            _ => {
                self.diagnostics.add(Diagnostic::from_code(
                    &codes::E0401,
                    self.context_span,
                    format!("type `{}` does not have fields", self.type_name(base_type)),
                ));
                Type::Error
            }
        }
    }

    /// Infer the result type of a function/method call
    fn infer_call_result_type(&self, callee_type: &Type) -> Type {
        match callee_type {
            Type::Function { ret, .. } => (**ret).clone(),
            Type::Blueprint(_) => {
                // Blueprint instantiation returns a fragment (represented as Unit for now)
                Type::Unit
            }
            _ => Type::Unknown,
        }
    }

    /// Format a type for display in error messages, resolving symbol names
    fn type_name(&self, ty: &Type) -> String {
        match ty {
            Type::Scheme(id) | Type::Backend(id) | Type::Blueprint(id) | Type::Contract(id) | Type::Theme(id) | Type::Enum(id) => {
                if let Some(symbol) = self.symbols.get(*id) {
                    symbol.name.clone()
                } else {
                    ty.to_string()
                }
            }
            Type::Ref(inner) => format!("ref {}", self.type_name(inner)),
            Type::Draft(inner) => format!("draft {}", self.type_name(inner)),
            Type::Nullable(inner) => format!("{}?", self.type_name(inner)),
            Type::List(inner) => format!("List<{}>", self.type_name(inner)),
            Type::Set(inner) => format!("Set<{}>", self.type_name(inner)),
            Type::Map(k, v) => format!("Map<{}, {}>", self.type_name(k), self.type_name(v)),
            Type::Tree(inner) => format!("Tree<{}>", self.type_name(inner)),
            _ => ty.to_string(),
        }
    }
}
