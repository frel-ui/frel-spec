// Operator type rules for Frel type checking
//
// This module handles type inference and validation for binary and unary operators.

use crate::ast;
use crate::diagnostic::{codes, Diagnostic, Diagnostics};
use crate::source::Span;

use super::super::types::Type;

/// Infer the result type of a binary operation
pub fn infer_binary_op_type(
    op: ast::BinaryOp,
    left: &Type,
    right: &Type,
    span: Span,
    diagnostics: &mut Diagnostics,
) -> Type {
    use ast::BinaryOp::*;
    match op {
        // Arithmetic
        Add | Sub | Mul | Div | Mod | Pow => {
            if left.is_numeric() && right.is_numeric() {
                // Return the "larger" numeric type
                common_numeric_type(left, right)
            } else if matches!(op, Add) && (left.is_text() || right.is_text()) {
                // String concatenation
                Type::String
            } else {
                report_binary_type_error(op, left, right, span, diagnostics);
                Type::Error
            }
        }
        // Comparison
        Eq | Ne => {
            // Any types can be compared for equality
            Type::Bool
        }
        Lt | Le | Gt | Ge => {
            if left.is_numeric() && right.is_numeric() {
                Type::Bool
            } else {
                report_binary_type_error(op, left, right, span, diagnostics);
                Type::Error
            }
        }
        // Logical
        And | Or => {
            if *left == Type::Bool && *right == Type::Bool {
                Type::Bool
            } else {
                report_binary_type_error(op, left, right, span, diagnostics);
                Type::Error
            }
        }
        // Null coalescing
        Elvis => {
            // T? ?: T -> T
            if let Type::Nullable(inner) = left {
                if types_compatible(inner, right) {
                    return (**inner).clone();
                }
            }
            report_binary_type_error(op, left, right, span, diagnostics);
            Type::Error
        }
    }
}

/// Infer the result type of a unary operation
pub fn infer_unary_op_type(
    op: ast::UnaryOp,
    operand: &Type,
    span: Span,
    diagnostics: &mut Diagnostics,
) -> Type {
    use ast::UnaryOp::*;
    match op {
        Not => {
            if *operand == Type::Bool {
                Type::Bool
            } else {
                diagnostics.add(Diagnostic::from_code(
                    &codes::E0401,
                    span,
                    format!("cannot apply `!` to type `{}`", operand),
                ));
                Type::Error
            }
        }
        Neg | Pos => {
            if operand.is_numeric() {
                operand.clone()
            } else {
                diagnostics.add(Diagnostic::from_code(
                    &codes::E0401,
                    span,
                    format!("cannot apply `-`/`+` to type `{}`", operand),
                ));
                Type::Error
            }
        }
    }
}

/// Get the common numeric type for two numeric types
pub fn common_numeric_type(left: &Type, right: &Type) -> Type {
    // Decimal wins over everything
    if *left == Type::Decimal || *right == Type::Decimal {
        return Type::Decimal;
    }
    // Float wins over integer
    if left.is_float() || right.is_float() {
        if *left == Type::F64 || *right == Type::F64 {
            return Type::F64;
        }
        return Type::F32;
    }
    // Larger integer wins
    if *left == Type::I64 || *right == Type::I64 || *left == Type::U64 || *right == Type::U64 {
        return Type::I64;
    }
    Type::I32
}

/// Check if two types are compatible
pub fn types_compatible(expected: &Type, actual: &Type) -> bool {
    if expected == actual {
        return true;
    }
    // Error types are compatible with anything (to suppress cascading errors)
    if expected.is_error() || actual.is_error() {
        return true;
    }
    // Unknown is compatible with anything
    if *expected == Type::Unknown || *actual == Type::Unknown {
        return true;
    }
    // Nullable compatibility
    if let Type::Nullable(inner) = expected {
        return types_compatible(inner, actual);
    }
    // Numeric widening
    if expected.is_numeric() && actual.is_numeric() {
        // Allow implicit widening (smaller -> larger)
        return true; // Simplified for now
    }
    false
}

/// Expect a boolean type, reporting an error if not
pub fn expect_bool(ty: &Type, span: Span, diagnostics: &mut Diagnostics) {
    if *ty != Type::Bool && *ty != Type::Unknown && !ty.is_error() {
        diagnostics.add(Diagnostic::from_code(
            &codes::E0401,
            span,
            format!("expected `bool`, found `{}`", ty),
        ));
    }
}

/// Expect an iterable type, reporting an error if not
pub fn expect_iterable(ty: &Type, span: Span, diagnostics: &mut Diagnostics) {
    let is_iterable = ty.is_collection() || *ty == Type::Unknown || ty.is_error();
    if !is_iterable {
        diagnostics.add(Diagnostic::from_code(
            &codes::E0401,
            span,
            format!("expected an iterable type, found `{}`", ty),
        ));
    }
}

fn report_binary_type_error(
    op: ast::BinaryOp,
    left: &Type,
    right: &Type,
    span: Span,
    diagnostics: &mut Diagnostics,
) {
    diagnostics.add(Diagnostic::from_code(
        &codes::E0405,
        span,
        format!(
            "cannot apply `{:?}` to types `{}` and `{}`",
            op, left, right
        ),
    ));
}
