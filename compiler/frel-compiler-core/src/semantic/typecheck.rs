// Type checking pass for Frel semantic analysis
//
// This module implements type resolution and checking:
// 1. Resolve TypeExpr (AST) to Type (semantic)
// 2. Infer types of expressions
// 3. Check type compatibility for assignments and calls
// 4. Validate command vs method context usage

use std::collections::HashMap;

use crate::ast::{self, TypeExpr};
use crate::diagnostic::{codes, Diagnostic, Diagnostics};
use crate::source::Span;

use super::scope::{ScopeGraph, ScopeId};
use super::symbol::{Symbol, SymbolId, SymbolKind, SymbolTable};
use super::types::Type;

/// Result of type checking
#[derive(Debug)]
pub struct TypeCheckResult {
    /// Types assigned to expressions (by span)
    pub expr_types: HashMap<Span, Type>,
    /// Resolved types for type expressions (by span)
    pub type_resolutions: HashMap<Span, Type>,
    /// Diagnostics generated during type checking
    pub diagnostics: Diagnostics,
}

impl TypeCheckResult {
    pub fn new() -> Self {
        Self {
            expr_types: HashMap::new(),
            type_resolutions: HashMap::new(),
            diagnostics: Diagnostics::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }
}

impl Default for TypeCheckResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Type checker that resolves and validates types
pub struct TypeChecker<'a> {
    scopes: &'a ScopeGraph,
    symbols: &'a SymbolTable,
    /// Types assigned to symbols (filled during checking)
    symbol_types: HashMap<SymbolId, Type>,
    /// Types of expressions
    expr_types: HashMap<Span, Type>,
    /// Resolved type expressions
    type_resolutions: HashMap<Span, Type>,
    /// Diagnostics
    diagnostics: Diagnostics,
    /// Current scope
    current_scope: ScopeId,
    /// Context span for error reporting (set to containing declaration's span)
    context_span: Span,
    /// Imported names (name -> module path)
    imports: &'a HashMap<String, String>,
}

impl<'a> TypeChecker<'a> {
    pub fn new(
        scopes: &'a ScopeGraph,
        symbols: &'a SymbolTable,
        imports: &'a HashMap<String, String>,
    ) -> Self {
        Self {
            scopes,
            symbols,
            symbol_types: HashMap::new(),
            expr_types: HashMap::new(),
            type_resolutions: HashMap::new(),
            diagnostics: Diagnostics::new(),
            current_scope: ScopeId::ROOT,
            context_span: Span::default(),
            imports,
        }
    }

    /// Run type checking on a file AST
    pub fn check(mut self, file: &ast::File) -> TypeCheckResult {
        // First pass: resolve all type annotations
        self.resolve_declarations(file);

        // Second pass: type check expressions
        self.check_declarations(file);

        TypeCheckResult {
            expr_types: self.expr_types,
            type_resolutions: self.type_resolutions,
            diagnostics: self.diagnostics,
        }
    }

    /// Resolve type annotations in all declarations
    fn resolve_declarations(&mut self, file: &ast::File) {
        for decl in &file.declarations {
            match decl {
                ast::TopLevelDecl::Backend(be) => self.resolve_backend_types(be),
                ast::TopLevelDecl::Blueprint(bp) => self.resolve_blueprint_types(bp),
                ast::TopLevelDecl::Scheme(sc) => self.resolve_scheme_types(sc),
                ast::TopLevelDecl::Contract(ct) => self.resolve_contract_types(ct),
                ast::TopLevelDecl::Theme(th) => self.resolve_theme_types(th),
                ast::TopLevelDecl::Enum(_) => {} // Enums don't have type annotations
                ast::TopLevelDecl::Arena(ar) => self.resolve_arena_types(ar),
            }
        }
    }

    fn resolve_backend_types(&mut self, be: &ast::Backend) {
        // Resolve parameter types
        for param in &be.params {
            self.resolve_type_expr(&param.type_expr, Span::default());
        }

        // Resolve member types
        for member in &be.members {
            match member {
                ast::BackendMember::Field(field) => {
                    self.resolve_type_expr(&field.type_expr, Span::default());
                }
                ast::BackendMember::Method(method) => {
                    for param in &method.params {
                        self.resolve_type_expr(&param.type_expr, Span::default());
                    }
                    self.resolve_type_expr(&method.return_type, Span::default());
                }
                ast::BackendMember::Command(cmd) => {
                    for param in &cmd.params {
                        self.resolve_type_expr(&param.type_expr, Span::default());
                    }
                }
                ast::BackendMember::Include(_) => {}
            }
        }
    }

    fn resolve_blueprint_types(&mut self, bp: &ast::Blueprint) {
        for param in &bp.params {
            self.resolve_type_expr(&param.type_expr, Span::default());
        }

        for stmt in &bp.body {
            self.resolve_blueprint_stmt_types(stmt);
        }
    }

    fn resolve_blueprint_stmt_types(&mut self, stmt: &ast::BlueprintStmt) {
        match stmt {
            ast::BlueprintStmt::LocalDecl(decl) => {
                self.resolve_type_expr(&decl.type_expr, Span::default());
            }
            ast::BlueprintStmt::FragmentCreation(frag) => {
                if let Some(body) = &frag.body {
                    self.resolve_fragment_body_types(body);
                }
            }
            ast::BlueprintStmt::Control(ctrl) => {
                self.resolve_control_stmt_types(ctrl);
            }
            ast::BlueprintStmt::Layout(layout) => {
                // Layout doesn't have explicit types
                let _ = layout;
            }
            ast::BlueprintStmt::SlotBinding(binding) => {
                self.resolve_slot_binding_types(binding);
            }
            _ => {}
        }
    }

    fn resolve_fragment_body_types(&mut self, body: &ast::FragmentBody) {
        match body {
            ast::FragmentBody::Default(stmts) => {
                for stmt in stmts {
                    self.resolve_blueprint_stmt_types(stmt);
                }
            }
            ast::FragmentBody::Slots(slots) => {
                for slot in slots {
                    self.resolve_slot_binding_types(slot);
                }
            }
            ast::FragmentBody::InlineBlueprint { body, .. } => {
                for stmt in body {
                    self.resolve_blueprint_stmt_types(stmt);
                }
            }
        }
    }

    fn resolve_slot_binding_types(&mut self, binding: &ast::SlotBinding) {
        if let ast::BlueprintValue::Inline { body, .. } = &binding.blueprint {
            for stmt in body {
                self.resolve_blueprint_stmt_types(stmt);
            }
        }
    }

    fn resolve_control_stmt_types(&mut self, ctrl: &ast::ControlStmt) {
        match ctrl {
            ast::ControlStmt::When {
                then_stmt,
                else_stmt,
                ..
            } => {
                self.resolve_blueprint_stmt_types(then_stmt);
                if let Some(else_stmt) = else_stmt {
                    self.resolve_blueprint_stmt_types(else_stmt);
                }
            }
            ast::ControlStmt::Repeat { body, .. } => {
                for stmt in body {
                    self.resolve_blueprint_stmt_types(stmt);
                }
            }
            ast::ControlStmt::Select {
                branches,
                else_branch,
                ..
            } => {
                for branch in branches {
                    self.resolve_blueprint_stmt_types(&branch.body);
                }
                if let Some(else_stmt) = else_branch {
                    self.resolve_blueprint_stmt_types(else_stmt);
                }
            }
        }
    }

    fn resolve_scheme_types(&mut self, sc: &ast::Scheme) {
        for member in &sc.members {
            match member {
                ast::SchemeMember::Field(field) => {
                    self.resolve_type_expr(&field.type_expr, Span::default());
                }
                ast::SchemeMember::Virtual(virt) => {
                    self.resolve_type_expr(&virt.type_expr, Span::default());
                }
            }
        }
    }

    fn resolve_contract_types(&mut self, ct: &ast::Contract) {
        for method in &ct.methods {
            for param in &method.params {
                self.resolve_type_expr(&param.type_expr, Span::default());
            }
            if let Some(ret) = &method.return_type {
                self.resolve_type_expr(ret, Span::default());
            }
        }
    }

    fn resolve_theme_types(&mut self, th: &ast::Theme) {
        for member in &th.members {
            if let ast::ThemeMember::Field(field) = member {
                self.resolve_type_expr(&field.type_expr, Span::default());
            }
        }
    }

    fn resolve_arena_types(&mut self, _ar: &ast::Arena) {
        // Arena references scheme and contract by name, resolved during name resolution
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
    fn resolve_named_type(&mut self, name: &str, span: Span) -> Type {
        // First try intrinsic types
        if let Some(ty) = Type::from_intrinsic_name(name) {
            return ty;
        }

        // Then look up user-defined types in the symbol table
        if let Some(symbol_id) = self.symbols.lookup_in_scope_chain(self.current_scope, name, self.scopes) {
            if let Some(symbol) = self.symbols.get(symbol_id) {
                return self.symbol_to_type(symbol);
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

    /// Convert a symbol to its corresponding type
    fn symbol_to_type(&self, symbol: &Symbol) -> Type {
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

    /// Type check all declarations
    fn check_declarations(&mut self, file: &ast::File) {
        for decl in &file.declarations {
            match decl {
                ast::TopLevelDecl::Backend(be) => self.check_backend(be),
                ast::TopLevelDecl::Blueprint(bp) => self.check_blueprint(bp),
                ast::TopLevelDecl::Scheme(sc) => self.check_scheme(sc),
                _ => {} // Other declarations don't need expression checking
            }
        }
    }

    fn check_backend(&mut self, be: &ast::Backend) {
        // Enter the backend's body scope for field lookups
        let saved_scope = self.current_scope;
        if let Some(symbol_id) = self.symbols.lookup_local(ScopeId::ROOT, &be.name) {
            if let Some(symbol) = self.symbols.get(symbol_id) {
                if let Some(body_scope) = symbol.body_scope {
                    self.current_scope = body_scope;
                }
            }
        }

        for member in &be.members {
            if let ast::BackendMember::Field(field) = member {
                if let Some(init) = &field.init {
                    self.context_span = field.span;
                    let _init_type = self.infer_expr_type(init);
                    // TODO: Check that init_type is compatible with field.type_expr
                }
            }
        }

        self.current_scope = saved_scope;
        self.context_span = Span::default();
    }

    fn check_blueprint(&mut self, bp: &ast::Blueprint) {
        // Enter the blueprint's body scope for local/field lookups
        let saved_scope = self.current_scope;
        if let Some(symbol_id) = self.symbols.lookup_local(ScopeId::ROOT, &bp.name) {
            if let Some(symbol) = self.symbols.get(symbol_id) {
                if let Some(body_scope) = symbol.body_scope {
                    self.current_scope = body_scope;
                }
            }
        }

        for stmt in &bp.body {
            self.check_blueprint_stmt(stmt);
        }

        self.current_scope = saved_scope;
    }

    fn check_blueprint_stmt(&mut self, stmt: &ast::BlueprintStmt) {
        match stmt {
            ast::BlueprintStmt::LocalDecl(decl) => {
                let _init_type = self.infer_expr_type(&decl.init);
                // TODO: Check compatibility with declared type
            }
            ast::BlueprintStmt::FragmentCreation(frag) => {
                for arg in &frag.args {
                    self.infer_expr_type(&arg.value);
                }
                if let Some(body) = &frag.body {
                    self.check_fragment_body(body);
                }
            }
            ast::BlueprintStmt::Control(ctrl) => self.check_control_stmt(ctrl),
            ast::BlueprintStmt::Instruction(instr) => self.check_instruction_expr(instr),
            ast::BlueprintStmt::EventHandler(handler) => self.check_event_handler(handler),
            ast::BlueprintStmt::ContentExpr(expr) => {
                self.infer_expr_type(expr);
            }
            _ => {}
        }
    }

    fn check_fragment_body(&mut self, body: &ast::FragmentBody) {
        match body {
            ast::FragmentBody::Default(stmts) => {
                for stmt in stmts {
                    self.check_blueprint_stmt(stmt);
                }
            }
            ast::FragmentBody::Slots(slots) => {
                for slot in slots {
                    if let ast::BlueprintValue::Inline { body, .. } = &slot.blueprint {
                        for stmt in body {
                            self.check_blueprint_stmt(stmt);
                        }
                    }
                }
            }
            ast::FragmentBody::InlineBlueprint { body, .. } => {
                for stmt in body {
                    self.check_blueprint_stmt(stmt);
                }
            }
        }
    }

    fn check_control_stmt(&mut self, ctrl: &ast::ControlStmt) {
        match ctrl {
            ast::ControlStmt::When {
                condition,
                then_stmt,
                else_stmt,
            } => {
                let cond_type = self.infer_expr_type(condition);
                self.expect_bool(&cond_type, self.context_span);
                self.check_blueprint_stmt(then_stmt);
                if let Some(else_stmt) = else_stmt {
                    self.check_blueprint_stmt(else_stmt);
                }
            }
            ast::ControlStmt::Repeat {
                iterable,
                key_expr,
                body,
                ..
            } => {
                let iter_type = self.infer_expr_type(iterable);
                self.expect_iterable(&iter_type, self.context_span);
                if let Some(key) = key_expr {
                    self.infer_expr_type(key);
                }
                for stmt in body {
                    self.check_blueprint_stmt(stmt);
                }
            }
            ast::ControlStmt::Select {
                discriminant,
                branches,
                else_branch,
            } => {
                if let Some(disc) = discriminant {
                    self.infer_expr_type(disc);
                }
                for branch in branches {
                    self.infer_expr_type(&branch.condition);
                    self.check_blueprint_stmt(&branch.body);
                }
                if let Some(else_stmt) = else_branch {
                    self.check_blueprint_stmt(else_stmt);
                }
            }
        }
    }

    fn check_instruction_expr(&mut self, instr: &ast::InstructionExpr) {
        match instr {
            ast::InstructionExpr::Simple(inst) => {
                for (_, expr) in &inst.params {
                    self.infer_expr_type(expr);
                }
            }
            ast::InstructionExpr::When {
                condition,
                then_instr,
                else_instr,
            } => {
                let cond_type = self.infer_expr_type(condition);
                self.expect_bool(&cond_type, self.context_span);
                self.check_instruction_expr(then_instr);
                if let Some(else_instr) = else_instr {
                    self.check_instruction_expr(else_instr);
                }
            }
            ast::InstructionExpr::Ternary {
                condition,
                then_instr,
                else_instr,
            } => {
                let cond_type = self.infer_expr_type(condition);
                self.expect_bool(&cond_type, self.context_span);
                self.check_instruction_expr(then_instr);
                self.check_instruction_expr(else_instr);
            }
            ast::InstructionExpr::Reference(expr) => {
                self.infer_expr_type(expr);
            }
        }
    }

    fn check_event_handler(&mut self, handler: &ast::EventHandler) {
        for stmt in &handler.body {
            match stmt {
                ast::HandlerStmt::Assignment { value, .. } => {
                    self.infer_expr_type(value);
                    // TODO: Check that value is compatible with target
                }
                ast::HandlerStmt::CommandCall { args, .. } => {
                    for arg in args {
                        self.infer_expr_type(arg);
                    }
                    // TODO: Validate this is a command, not a method (E0603)
                }
            }
        }
    }

    fn check_scheme(&mut self, sc: &ast::Scheme) {
        // Enter the scheme's body scope for field lookups
        let saved_scope = self.current_scope;
        if let Some(symbol_id) = self.symbols.lookup_local(ScopeId::ROOT, &sc.name) {
            if let Some(symbol) = self.symbols.get(symbol_id) {
                if let Some(body_scope) = symbol.body_scope {
                    self.current_scope = body_scope;
                }
            }
        }

        for member in &sc.members {
            if let ast::SchemeMember::Virtual(virt) = member {
                let _expr_type = self.infer_expr_type(&virt.expr);
                // TODO: Check compatibility with declared type
            }
        }

        self.current_scope = saved_scope;
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
                        if !self.types_compatible(&first_type, &item_type) {
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
            ast::Expr::Identifier(name) => self.lookup_identifier_type(name, self.context_span),
            ast::Expr::QualifiedName(parts) => {
                if let Some(first) = parts.first() {
                    let base_type = self.lookup_identifier_type(first, self.context_span);
                    // Resolve field accesses
                    let mut current = base_type;
                    for field in parts.iter().skip(1) {
                        current = self.resolve_field_access(&current, field, self.context_span);
                    }
                    current
                } else {
                    Type::Error
                }
            }
            ast::Expr::Binary { op, left, right } => {
                let left_type = self.infer_expr_type(left);
                let right_type = self.infer_expr_type(right);
                self.infer_binary_op_type(*op, &left_type, &right_type, self.context_span)
            }
            ast::Expr::Unary { op, expr } => {
                let operand_type = self.infer_expr_type(expr);
                self.infer_unary_op_type(*op, &operand_type, self.context_span)
            }
            ast::Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let cond_type = self.infer_expr_type(condition);
                self.expect_bool(&cond_type, self.context_span);
                let then_type = self.infer_expr_type(then_expr);
                let else_type = self.infer_expr_type(else_expr);
                // Result type is the common type of both branches
                if self.types_compatible(&then_type, &else_type) {
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
                self.resolve_field_access(&base_type, field, self.context_span)
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
                let field_type = self.resolve_field_access(inner_type, field, self.context_span);
                // Result is nullable
                field_type.make_nullable()
            }
            ast::Expr::Call { callee, args } => {
                let callee_type = self.infer_expr_type(callee);
                // Type check arguments
                for arg in args {
                    self.infer_expr_type(arg);
                }
                self.infer_call_result_type(&callee_type, self.context_span)
            }
        };

        self.expr_types.insert(Span::default(), ty.clone());
        ty
    }

    /// Look up the type of an identifier
    fn lookup_identifier_type(&mut self, name: &str, _span: Span) -> Type {
        // Check if it's an intrinsic type being used as a value (e.g., enum access)
        // For now, look up in the symbol table
        if let Some(symbol_id) =
            self.symbols
                .lookup_in_scope_chain(self.current_scope, name, self.scopes)
        {
            if let Some(symbol) = self.symbols.get(symbol_id) {
                // Check if we have a type for this symbol
                if let Some(ty) = self.symbol_types.get(&symbol_id) {
                    return ty.clone();
                }
                // Otherwise, derive from symbol kind
                return match symbol.kind {
                    SymbolKind::Field | SymbolKind::VirtualField | SymbolKind::Parameter | SymbolKind::LocalVar => {
                        // Would need to look up the declared type
                        Type::Unknown
                    }
                    SymbolKind::Backend => Type::Backend(symbol_id),
                    SymbolKind::Blueprint => Type::Blueprint(symbol_id),
                    SymbolKind::Scheme => Type::Scheme(symbol_id),
                    SymbolKind::Enum => Type::Enum(symbol_id),
                    _ => Type::Unknown,
                };
            }
        }

        // Not found - already reported during name resolution
        Type::Error
    }

    /// Resolve a field access on a type
    fn resolve_field_access(&mut self, base_type: &Type, field: &str, span: Span) -> Type {
        match base_type {
            Type::Scheme(symbol_id) | Type::Backend(symbol_id) => {
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
                    span,
                    format!("no field `{}` on type `{}`", field, base_type),
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
                    span,
                    format!("no variant `{}` in enum `{}`", field, base_type),
                ));
                Type::Error
            }
            Type::Nullable(_) => {
                // Cannot access field on nullable without optional chaining
                self.diagnostics.add(Diagnostic::from_code(
                    &codes::E0406,
                    span,
                    "cannot access field on nullable type without optional chaining `?.`",
                ));
                Type::Error
            }
            Type::Error | Type::Unknown => Type::Error,
            _ => {
                self.diagnostics.add(Diagnostic::from_code(
                    &codes::E0401,
                    span,
                    format!("type `{}` does not have fields", base_type),
                ));
                Type::Error
            }
        }
    }

    /// Infer the result type of a binary operation
    fn infer_binary_op_type(
        &mut self,
        op: ast::BinaryOp,
        left: &Type,
        right: &Type,
        span: Span,
    ) -> Type {
        use ast::BinaryOp::*;
        match op {
            // Arithmetic
            Add | Sub | Mul | Div | Mod | Pow => {
                if left.is_numeric() && right.is_numeric() {
                    // Return the "larger" numeric type
                    self.common_numeric_type(left, right)
                } else if matches!(op, Add) && (left.is_text() || right.is_text()) {
                    // String concatenation
                    Type::String
                } else {
                    self.report_binary_type_error(op, left, right, span);
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
                    self.report_binary_type_error(op, left, right, span);
                    Type::Error
                }
            }
            // Logical
            And | Or => {
                if *left == Type::Bool && *right == Type::Bool {
                    Type::Bool
                } else {
                    self.report_binary_type_error(op, left, right, span);
                    Type::Error
                }
            }
            // Null coalescing
            Elvis => {
                // T? ?: T -> T
                if let Type::Nullable(inner) = left {
                    if self.types_compatible(inner, right) {
                        return (**inner).clone();
                    }
                }
                self.report_binary_type_error(op, left, right, span);
                Type::Error
            }
        }
    }

    /// Infer the result type of a unary operation
    fn infer_unary_op_type(&mut self, op: ast::UnaryOp, operand: &Type, span: Span) -> Type {
        use ast::UnaryOp::*;
        match op {
            Not => {
                if *operand == Type::Bool {
                    Type::Bool
                } else {
                    self.diagnostics.add(Diagnostic::from_code(
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
                    self.diagnostics.add(Diagnostic::from_code(
                        &codes::E0401,
                        span,
                        format!("cannot apply `-`/`+` to type `{}`", operand),
                    ));
                    Type::Error
                }
            }
        }
    }

    /// Infer the result type of a function/method call
    fn infer_call_result_type(&self, callee_type: &Type, _span: Span) -> Type {
        match callee_type {
            Type::Function { ret, .. } => (**ret).clone(),
            Type::Blueprint(_) => {
                // Blueprint instantiation returns a fragment (represented as Unit for now)
                Type::Unit
            }
            _ => Type::Unknown,
        }
    }

    /// Get the common numeric type for two numeric types
    fn common_numeric_type(&self, left: &Type, right: &Type) -> Type {
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
        if *left == Type::I64
            || *right == Type::I64
            || *left == Type::U64
            || *right == Type::U64
        {
            return Type::I64;
        }
        Type::I32
    }

    /// Check if two types are compatible
    fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
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
            return self.types_compatible(inner, actual);
        }
        // Numeric widening
        if expected.is_numeric() && actual.is_numeric() {
            // Allow implicit widening (smaller -> larger)
            return true; // Simplified for now
        }
        false
    }

    /// Expect a boolean type
    fn expect_bool(&mut self, ty: &Type, span: Span) {
        if *ty != Type::Bool && *ty != Type::Unknown && !ty.is_error() {
            self.diagnostics.add(Diagnostic::from_code(
                &codes::E0401,
                span,
                format!("expected `bool`, found `{}`", ty),
            ));
        }
    }

    /// Expect an iterable type
    fn expect_iterable(&mut self, ty: &Type, span: Span) {
        let is_iterable = ty.is_collection()
            || *ty == Type::Unknown
            || ty.is_error();
        if !is_iterable {
            self.diagnostics.add(Diagnostic::from_code(
                &codes::E0401,
                span,
                format!("expected an iterable type, found `{}`", ty),
            ));
        }
    }

    fn report_binary_type_error(
        &mut self,
        op: ast::BinaryOp,
        left: &Type,
        right: &Type,
        span: Span,
    ) {
        self.diagnostics.add(Diagnostic::from_code(
            &codes::E0405,
            span,
            format!(
                "cannot apply `{:?}` to types `{}` and `{}`",
                op, left, right
            ),
        ));
    }
}

/// Run type checking on a resolved AST
pub fn typecheck(
    file: &ast::File,
    scopes: &ScopeGraph,
    symbols: &SymbolTable,
    imports: &std::collections::HashMap<String, String>,
) -> TypeCheckResult {
    TypeChecker::new(scopes, symbols, imports).check(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::semantic::resolve;

    fn typecheck_source(source: &str) -> TypeCheckResult {
        let parse_result = parser::parse(source);
        assert!(
            !parse_result.diagnostics.has_errors(),
            "Parse errors: {:?}",
            parse_result.diagnostics
        );
        let file = parse_result.file.unwrap();
        let resolve_result = resolve::resolve(&file);
        typecheck(&file, &resolve_result.scopes, &resolve_result.symbols, &resolve_result.imports)
    }

    #[test]
    fn test_resolve_intrinsic_types() {
        let source = r#"
module test

scheme Person {
    name: String
    age: i32
    score: f64
    active: bool
    created: Instant
    id: Uuid
}
"#;
        let result = typecheck_source(source);
        assert!(
            !result.has_errors(),
            "Errors: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_resolve_nullable_types() {
        let source = r#"
module test

scheme Item {
    name: String?
    count: i32?
}
"#;
        let result = typecheck_source(source);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_resolve_collection_types() {
        let source = r#"
module test

scheme Container {
    items: List<String>
    counts: List<i32>
}
"#;
        let result = typecheck_source(source);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_unknown_type_error() {
        let source = r#"
module test

scheme Item {
    data: UnknownType
}
"#;
        let result = typecheck_source(source);
        assert!(result.has_errors());
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.code == Some("E0402".to_string())));
    }

    #[test]
    fn test_type_display() {
        // Verify our new types display correctly
        assert_eq!(format!("{}", Type::Uuid), "Uuid");
        assert_eq!(format!("{}", Type::Instant), "Instant");
        assert_eq!(format!("{}", Type::Duration), "Duration");
        assert_eq!(format!("{}", Type::Secret), "Secret");
    }

    #[test]
    fn test_backend_field_type_error_span() {
        // Test that type errors in backend field initializers have proper spans
        let source = r#"
module test

backend Calculator {
    a : bool = true
    b : i32 = 20
    sum : i32 = a * b
}
"#;
        let result = typecheck_source(source);
        // Should have a type error for bool * i32
        assert!(result.has_errors(), "Expected type error for bool * i32");
        // Check that the error has a non-default span
        let error = result.diagnostics.iter().next().unwrap();
        assert!(
            error.span.start > 0 || error.span.end > 0,
            "Error span should not be default: {:?}",
            error.span
        );
    }
}
