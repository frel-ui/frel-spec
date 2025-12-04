// Type checking pass for Frel semantic analysis
//
// This module implements type resolution and checking:
// 1. Resolve TypeExpr (AST) to Type (semantic)
// 2. Infer types of expressions
// 3. Check type compatibility for assignments and calls
// 4. Validate command vs method context usage

mod expressions;
mod operators;
mod resolution;

use std::collections::HashMap;

use crate::ast::{self, TypeExpr};
use crate::diagnostic::{codes, Diagnostic, Diagnostics};
use crate::source::Span;

use super::instructions::instruction_registry;
use super::scope::{ScopeGraph, ScopeId};
use super::symbol::{SymbolId, SymbolTable};
use super::types::Type;

pub use operators::types_compatible;
use resolution::TypeResolver;

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

    // =========================================================================
    // Type Resolution (First Pass)
    // =========================================================================

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
                ast::TopLevelDecl::Arena(_) => {} // Arena references resolved during name resolution
            }
        }
    }

    fn resolve_backend_types(&mut self, be: &ast::Backend) {
        // Resolve parameter types (use backend span as fallback since Parameter has no span)
        for param in &be.params {
            self.resolve_type_expr(&param.type_expr, be.span);
        }

        // Resolve member types
        for member in &be.members {
            match member {
                ast::BackendMember::Field(field) => {
                    self.resolve_type_expr(&field.type_expr, field.span);
                }
                ast::BackendMember::Method(method) => {
                    for param in &method.params {
                        self.resolve_type_expr(&param.type_expr, method.span);
                    }
                    self.resolve_type_expr(&method.return_type, method.span);
                }
                ast::BackendMember::Command(cmd) => {
                    for param in &cmd.params {
                        self.resolve_type_expr(&param.type_expr, cmd.span);
                    }
                }
                ast::BackendMember::Include(_) => {}
            }
        }
    }

    fn resolve_blueprint_types(&mut self, bp: &ast::Blueprint) {
        // Use blueprint span for parameters since Parameter has no span
        for param in &bp.params {
            self.resolve_type_expr(&param.type_expr, bp.span);
        }

        for stmt in &bp.body {
            self.resolve_blueprint_stmt_types(stmt, bp.span);
        }
    }

    fn resolve_blueprint_stmt_types(&mut self, stmt: &ast::BlueprintStmt, context_span: Span) {
        match stmt {
            ast::BlueprintStmt::LocalDecl(decl) => {
                // LocalDecl now has its own span
                self.resolve_type_expr(&decl.type_expr, decl.span);
            }
            ast::BlueprintStmt::FragmentCreation(frag) => {
                if let Some(body) = &frag.body {
                    self.resolve_fragment_body_types(body, context_span);
                }
            }
            ast::BlueprintStmt::Control(ctrl) => {
                self.resolve_control_stmt_types(ctrl, context_span);
            }
            ast::BlueprintStmt::Layout(layout) => {
                // Layout doesn't have explicit types
                let _ = layout;
            }
            ast::BlueprintStmt::SlotBinding(binding) => {
                self.resolve_slot_binding_types(binding, context_span);
            }
            _ => {}
        }
    }

    fn resolve_fragment_body_types(&mut self, body: &ast::FragmentBody, context_span: Span) {
        match body {
            ast::FragmentBody::Default(stmts) => {
                for stmt in stmts {
                    self.resolve_blueprint_stmt_types(stmt, context_span);
                }
            }
            ast::FragmentBody::Slots(slots) => {
                for slot in slots {
                    self.resolve_slot_binding_types(slot, context_span);
                }
            }
            ast::FragmentBody::InlineBlueprint { body, .. } => {
                for stmt in body {
                    self.resolve_blueprint_stmt_types(stmt, context_span);
                }
            }
        }
    }

    fn resolve_slot_binding_types(&mut self, binding: &ast::SlotBinding, context_span: Span) {
        if let ast::BlueprintValue::Inline { body, .. } = &binding.blueprint {
            for stmt in body {
                self.resolve_blueprint_stmt_types(stmt, context_span);
            }
        }
    }

    fn resolve_control_stmt_types(&mut self, ctrl: &ast::ControlStmt, context_span: Span) {
        match ctrl {
            ast::ControlStmt::When {
                then_stmt,
                else_stmt,
                ..
            } => {
                self.resolve_blueprint_stmt_types(then_stmt, context_span);
                if let Some(else_stmt) = else_stmt {
                    self.resolve_blueprint_stmt_types(else_stmt, context_span);
                }
            }
            ast::ControlStmt::Repeat { body, .. } => {
                // Type assignment for loop variables happens in the check phase
                // where we have access to infer_expr_type
                for stmt in body {
                    self.resolve_blueprint_stmt_types(stmt, context_span);
                }
            }
            ast::ControlStmt::Select {
                branches,
                else_branch,
                ..
            } => {
                for branch in branches {
                    self.resolve_blueprint_stmt_types(&branch.body, context_span);
                }
                if let Some(else_stmt) = else_branch {
                    self.resolve_blueprint_stmt_types(else_stmt, context_span);
                }
            }
        }
    }

    fn resolve_scheme_types(&mut self, sc: &ast::Scheme) {
        for member in &sc.members {
            match member {
                ast::SchemeMember::Field(field) => {
                    self.resolve_type_expr(&field.type_expr, field.span);
                }
                ast::SchemeMember::Virtual(virt) => {
                    self.resolve_type_expr(&virt.type_expr, virt.span);
                }
            }
        }
    }

    fn resolve_contract_types(&mut self, ct: &ast::Contract) {
        // Enter the contract's body scope for method lookups
        let saved_scope = self.current_scope;
        if let Some(symbol_id) = self.symbols.lookup_local(ScopeId::ROOT, &ct.name) {
            if let Some(symbol) = self.symbols.get(symbol_id) {
                if let Some(body_scope) = symbol.body_scope {
                    self.current_scope = body_scope;
                }
            }
        }

        for method in &ct.methods {
            // Resolve parameter types and return type
            let param_types: Vec<Type> = method
                .params
                .iter()
                .map(|p| self.resolve_type_expr(&p.type_expr, method.span))
                .collect();
            let ret_type = method
                .return_type
                .as_ref()
                .map(|rt| self.resolve_type_expr(rt, method.span))
                .unwrap_or(Type::Unit);
            let method_type = Type::Function {
                params: param_types,
                ret: Box::new(ret_type),
            };
            // Store the method's function type
            if let Some(method_symbol_id) =
                self.symbols.lookup_local(self.current_scope, &method.name)
            {
                self.symbol_types.insert(method_symbol_id, method_type);
            }
        }

        self.current_scope = saved_scope;
    }

    fn resolve_theme_types(&mut self, th: &ast::Theme) {
        for member in &th.members {
            if let ast::ThemeMember::Field(field) = member {
                self.resolve_type_expr(&field.type_expr, field.span);
            }
        }
    }

    /// Resolve a TypeExpr to a Type
    pub fn resolve_type_expr(&mut self, type_expr: &TypeExpr, span: Span) -> Type {
        let mut resolver = TypeResolver::new(self.scopes, self.symbols, self.imports);
        resolver.current_scope = self.current_scope;
        let ty = resolver.resolve_type_expr(type_expr, span);

        // Merge results back
        self.type_resolutions.extend(resolver.type_resolutions);
        self.diagnostics.merge(resolver.diagnostics);
        ty
    }

    // =========================================================================
    // Declaration Checking (Second Pass)
    // =========================================================================

    /// Type check all declarations
    fn check_declarations(&mut self, file: &ast::File) {
        for decl in &file.declarations {
            match decl {
                ast::TopLevelDecl::Backend(be) => self.check_backend(be),
                ast::TopLevelDecl::Blueprint(bp) => self.check_blueprint(bp),
                ast::TopLevelDecl::Scheme(sc) => self.check_scheme(sc),
                ast::TopLevelDecl::Theme(th) => self.check_theme(th),
                _ => {} // Other declarations don't need expression checking
            }
        }
    }

    fn check_theme(&mut self, th: &ast::Theme) {
        // Enter the theme's body scope for field lookups
        let saved_scope = self.current_scope;
        if let Some(symbol_id) = self.symbols.lookup_local(ScopeId::ROOT, &th.name) {
            if let Some(symbol) = self.symbols.get(symbol_id) {
                if let Some(body_scope) = symbol.body_scope {
                    self.current_scope = body_scope;
                }
            }
        }

        // Resolve all field types and store in symbol_types
        for member in &th.members {
            if let ast::ThemeMember::Field(field) = member {
                let field_type = self.resolve_type_expr(&field.type_expr, field.span);
                if let Some(field_symbol_id) =
                    self.symbols.lookup_local(self.current_scope, &field.name)
                {
                    self.symbol_types.insert(field_symbol_id, field_type);
                }
            }
        }

        self.current_scope = saved_scope;
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

        // First pass: resolve all field and method types and store in symbol_types
        // This is needed so that field references in initializers can be resolved
        for member in &be.members {
            match member {
                ast::BackendMember::Include(included_name) => {
                    // Import types from the included backend
                    if let Some(included_id) = self
                        .symbols
                        .lookup_in_scope_chain(ScopeId::ROOT, included_name, self.scopes)
                    {
                        if let Some(included_symbol) = self.symbols.get(included_id) {
                            if let Some(included_body_scope) = included_symbol.body_scope {
                                // For each symbol in the included backend, copy its type
                                let included_members: Vec<_> = self
                                    .symbols
                                    .symbols_in_scope(included_body_scope)
                                    .map(|s| (s.name.clone(), s.id))
                                    .collect();

                                for (member_name, included_member_id) in included_members {
                                    // Get the type from the included backend's symbol
                                    if let Some(member_type) =
                                        self.symbol_types.get(&included_member_id).cloned()
                                    {
                                        // Find the imported symbol in current backend scope and set its type
                                        if let Some(local_member_id) =
                                            self.symbols.lookup_local(self.current_scope, &member_name)
                                        {
                                            self.symbol_types.insert(local_member_id, member_type);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                ast::BackendMember::Field(field) => {
                    let field_type = self.resolve_type_expr(&field.type_expr, field.span);
                    // Look up the field's symbol and store its type
                    if let Some(field_symbol_id) =
                        self.symbols.lookup_local(self.current_scope, &field.name)
                    {
                        self.symbol_types.insert(field_symbol_id, field_type);
                    }
                }
                ast::BackendMember::Method(method) => {
                    // Resolve parameter types and return type
                    let param_types: Vec<Type> = method
                        .params
                        .iter()
                        .map(|p| self.resolve_type_expr(&p.type_expr, method.span))
                        .collect();
                    let ret_type = self.resolve_type_expr(&method.return_type, method.span);
                    let method_type = Type::Function {
                        params: param_types,
                        ret: Box::new(ret_type),
                    };
                    if let Some(method_symbol_id) =
                        self.symbols.lookup_local(self.current_scope, &method.name)
                    {
                        self.symbol_types.insert(method_symbol_id, method_type);
                    }
                }
                ast::BackendMember::Command(cmd) => {
                    // Commands return Unit (no return type)
                    let param_types: Vec<Type> = cmd
                        .params
                        .iter()
                        .map(|p| self.resolve_type_expr(&p.type_expr, cmd.span))
                        .collect();
                    let cmd_type = Type::Function {
                        params: param_types,
                        ret: Box::new(Type::Unit),
                    };
                    if let Some(cmd_symbol_id) =
                        self.symbols.lookup_local(self.current_scope, &cmd.name)
                    {
                        self.symbol_types.insert(cmd_symbol_id, cmd_type);
                    }
                }
            }
        }

        // Second pass: check all field initializers
        for member in &be.members {
            if let ast::BackendMember::Field(field) = member {
                if let Some(init) = &field.init {
                    self.context_span = field.span;
                    // Get the expected type (already resolved in first pass)
                    if let Some(field_symbol_id) =
                        self.symbols.lookup_local(self.current_scope, &field.name)
                    {
                        let expected_type = self
                            .symbol_types
                            .get(&field_symbol_id)
                            .cloned()
                            .unwrap_or(Type::Unknown);
                        // Check the initializer against the expected type
                        let _init_type = self.check_expr_type(init, &expected_type);
                        // TODO: Check that init_type is compatible with expected_type
                    }
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

        // First pass: resolve types for `with` imported symbols and LocalDecl
        for stmt in &bp.body {
            match stmt {
                ast::BlueprintStmt::With(backend_name) => {
                    // Import types from the backend
                    // Look up from current scope to find both module-level backends and parameters
                    if let Some(backend_id) = self
                        .symbols
                        .lookup_in_scope_chain(self.current_scope, backend_name, self.scopes)
                    {
                        if let Some(backend_symbol) = self.symbols.get(backend_id) {
                            if let Some(backend_body_scope) = backend_symbol.body_scope {
                                // For each symbol in the backend, copy its type to the blueprint's imported symbol
                                let backend_members: Vec<_> = self
                                    .symbols
                                    .symbols_in_scope(backend_body_scope)
                                    .map(|s| (s.name.clone(), s.id))
                                    .collect();

                                for (member_name, backend_member_id) in backend_members {
                                    // Get the type from the backend's symbol
                                    if let Some(member_type) =
                                        self.symbol_types.get(&backend_member_id).cloned()
                                    {
                                        // Find the imported symbol in blueprint scope and set its type
                                        if let Some(blueprint_member_id) =
                                            self.symbols.lookup_local(self.current_scope, &member_name)
                                        {
                                            self.symbol_types.insert(blueprint_member_id, member_type);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                ast::BlueprintStmt::LocalDecl(decl) => {
                    let decl_type = self.resolve_type_expr(&decl.type_expr, decl.span);
                    // Look up the local's symbol and store its type
                    if let Some(local_symbol_id) =
                        self.symbols.lookup_local(self.current_scope, &decl.name)
                    {
                        self.symbol_types.insert(local_symbol_id, decl_type);
                    }
                }
                _ => {}
            }
        }

        // Second pass: check all statements
        for stmt in &bp.body {
            self.check_blueprint_stmt(stmt);
        }

        self.current_scope = saved_scope;
        self.context_span = Span::default();
    }

    fn check_blueprint_stmt(&mut self, stmt: &ast::BlueprintStmt) {
        match stmt {
            ast::BlueprintStmt::LocalDecl(decl) => {
                self.context_span = decl.span;
                // Get the expected type (already resolved in first pass)
                if let Some(local_symbol_id) =
                    self.symbols.lookup_local(self.current_scope, &decl.name)
                {
                    let expected_type = self
                        .symbol_types
                        .get(&local_symbol_id)
                        .cloned()
                        .unwrap_or(Type::Unknown);
                    // Check the initializer against the expected type
                    let _init_type = self.check_expr_type(&decl.init, &expected_type);
                    // TODO: Check that init_type is compatible with expected_type
                } else {
                    let _init_type = self.infer_expr_type(&decl.init);
                }
            }
            ast::BlueprintStmt::FragmentCreation(frag) => {
                for arg in &frag.args {
                    self.infer_expr_type(&arg.value);
                }
                if let Some(body) = &frag.body {
                    self.check_fragment_body(body);
                }
                // Check postfix items (instructions, event handlers)
                for postfix in &frag.postfix {
                    match postfix {
                        ast::PostfixItem::Instruction(instr) => self.check_instruction_expr(instr),
                        ast::PostfixItem::EventHandler(handler) => self.check_event_handler(handler),
                    }
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
                operators::expect_bool(&cond_type, self.context_span, &mut self.diagnostics);
                self.check_blueprint_stmt(then_stmt);
                if let Some(else_stmt) = else_stmt {
                    self.check_blueprint_stmt(else_stmt);
                }
            }
            ast::ControlStmt::Repeat {
                iterable,
                item_name,
                key_expr,
                body,
            } => {
                let iter_type = self.infer_expr_type(iterable);
                operators::expect_iterable(&iter_type, self.context_span, &mut self.diagnostics);

                // Get element type from iterable and assign to loop variables
                let element_type = iter_type
                    .element_type()
                    .cloned()
                    .unwrap_or(Type::Unknown);

                // Find the loop scope by looking up the item variable in children
                // (the loop scope is created as a child of current_scope during resolve)
                let saved_scope = self.current_scope;
                if let Some((item_id, loop_scope)) = self.symbols.lookup_in_children(
                    self.current_scope,
                    item_name,
                    self.scopes,
                ) {
                    // Set the type of the loop variable
                    self.symbol_types.insert(item_id, element_type);

                    // Enter the loop scope for checking the body
                    self.current_scope = loop_scope;
                }

                if let Some(key) = key_expr {
                    self.infer_expr_type(key);
                }
                for stmt in body {
                    self.check_blueprint_stmt(stmt);
                }
                self.current_scope = saved_scope;
            }
            ast::ControlStmt::Select {
                discriminant,
                branches,
                else_branch,
            } => {
                // Infer discriminant type if present
                let disc_type = discriminant.as_ref().map(|d| self.infer_expr_type(d));

                for branch in branches {
                    // Special handling for enum variant matching
                    if let (Some(Type::Enum(enum_id)), ast::Expr::Identifier(variant_name)) =
                        (&disc_type, &branch.condition)
                    {
                        // Check if the identifier is a valid enum variant
                        if let Some(enum_symbol) = self.symbols.get(*enum_id) {
                            if let Some(body_scope) = enum_symbol.body_scope {
                                if self
                                    .symbols
                                    .lookup_local(body_scope, variant_name)
                                    .is_none()
                                {
                                    // Not a valid variant
                                    self.diagnostics.add(Diagnostic::from_code(
                                        &codes::E0301,
                                        self.context_span,
                                        format!(
                                            "no variant `{}` in enum `{}`",
                                            variant_name, enum_symbol.name
                                        ),
                                    ));
                                }
                                // If found, it's a valid enum variant - no error
                            }
                        }
                    } else {
                        // Regular expression condition
                        self.infer_expr_type(&branch.condition);
                    }
                    self.check_blueprint_stmt(&branch.body);
                }
                if let Some(else_stmt) = else_branch {
                    self.check_blueprint_stmt(else_stmt);
                }
            }
        }
    }

    fn check_instruction_expr(&mut self, instr: &ast::InstructionExpr) {
        let registry = instruction_registry();

        match instr {
            ast::InstructionExpr::Simple(inst) => {
                // Set context span for error reporting
                self.context_span = inst.span;

                for (param_name, expr) in &inst.params {
                    // Check if this is a simple identifier that should be validated as a keyword
                    if let ast::Expr::Identifier(value) = expr {
                        // Check if this instruction parameter only accepts keywords (not expressions)
                        let accepts_expr = registry.accepts_expression(&inst.name, param_name);

                        if !accepts_expr {
                            // This parameter only accepts keywords - validate the value
                            let is_valid = registry.is_valid_keyword(&inst.name, param_name, value);
                            if !is_valid {
                                // Report invalid keyword error
                                if let Some(valid_keywords) =
                                    registry.valid_keywords(&inst.name, param_name)
                                {
                                    let expected = valid_keywords.join(", ");
                                    self.diagnostics.add(Diagnostic::from_code(
                                        &codes::E0705,
                                        self.context_span,
                                        format!(
                                            "invalid value '{}' for '{}' instruction, expected one of: {}",
                                            value, inst.name, expected
                                        ),
                                    ));
                                }
                            }
                        } else {
                            // This parameter accepts expressions - infer the type
                            self.infer_expr_type(expr);
                        }
                    } else {
                        // Non-identifier expression - infer the type
                        self.infer_expr_type(expr);
                    }
                }
            }
            ast::InstructionExpr::When {
                condition,
                then_instr,
                else_instr,
            } => {
                let cond_type = self.infer_expr_type(condition);
                operators::expect_bool(&cond_type, self.context_span, &mut self.diagnostics);
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
                operators::expect_bool(&cond_type, self.context_span, &mut self.diagnostics);
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

        // First pass: resolve all field types and store in symbol_types
        // This is needed so that field references in virtual field expressions can be resolved
        for member in &sc.members {
            match member {
                ast::SchemeMember::Field(field) => {
                    let field_type = self.resolve_type_expr(&field.type_expr, field.span);
                    if let Some(field_symbol_id) =
                        self.symbols.lookup_local(self.current_scope, &field.name)
                    {
                        self.symbol_types.insert(field_symbol_id, field_type);
                    }
                }
                ast::SchemeMember::Virtual(virt) => {
                    let virt_type = self.resolve_type_expr(&virt.type_expr, virt.span);
                    if let Some(virt_symbol_id) =
                        self.symbols.lookup_local(self.current_scope, &virt.name)
                    {
                        self.symbol_types.insert(virt_symbol_id, virt_type);
                    }
                }
            }
        }

        // Second pass: check virtual field expressions
        for member in &sc.members {
            if let ast::SchemeMember::Virtual(virt) = member {
                self.context_span = virt.span;
                // Get the expected type (already resolved in first pass)
                if let Some(virt_symbol_id) =
                    self.symbols.lookup_local(self.current_scope, &virt.name)
                {
                    let expected_type = self
                        .symbol_types
                        .get(&virt_symbol_id)
                        .cloned()
                        .unwrap_or(Type::Unknown);
                    // Check the expression against the expected type
                    let _expr_type = self.check_expr_type(&virt.expr, &expected_type);
                    // TODO: Check that expr_type is compatible with expected_type
                }
            }
        }

        self.current_scope = saved_scope;
        self.context_span = Span::default();
    }

    // =========================================================================
    // Expression Type Checking
    // =========================================================================

    /// Check an expression against an expected type (bidirectional type checking)
    pub fn check_expr_type(&mut self, expr: &ast::Expr, expected: &Type) -> Type {
        let mut checker = expressions::ExprChecker::new(
            self.scopes,
            self.symbols,
            &self.symbol_types,
            self.current_scope,
            self.context_span,
        );
        let ty = checker.check_expr_type(expr, expected);

        // Merge results back
        self.expr_types.extend(checker.expr_types);
        self.diagnostics.merge(checker.diagnostics);
        ty
    }

    /// Infer the type of an expression
    pub fn infer_expr_type(&mut self, expr: &ast::Expr) -> Type {
        let mut checker = expressions::ExprChecker::new(
            self.scopes,
            self.symbols,
            &self.symbol_types,
            self.current_scope,
            self.context_span,
        );
        let ty = checker.infer_expr_type(expr);

        // Merge results back
        self.expr_types.extend(checker.expr_types);
        self.diagnostics.merge(checker.diagnostics);
        ty
    }
}

// =============================================================================
// Public API
// =============================================================================

/// Run type checking on a resolved AST
pub fn typecheck(
    file: &ast::File,
    scopes: &ScopeGraph,
    symbols: &SymbolTable,
    imports: &HashMap<String, String>,
) -> TypeCheckResult {
    TypeChecker::new(scopes, symbols, imports).check(file)
}

/// Run type checking with access to external module signatures
///
/// This extends basic type checking by resolving imported types against
/// the provided SignatureRegistry, enabling cross-module type checking.
///
/// Note: With the unified symbol approach, external symbols are imported into the
/// local SymbolTable during name resolution, so the registry parameter is kept
/// for API compatibility but the actual cross-module resolution happens at name
/// resolution time.
pub fn typecheck_with_registry(
    file: &ast::File,
    scopes: &ScopeGraph,
    symbols: &SymbolTable,
    imports: &HashMap<String, String>,
    _registry: &super::signature::SignatureRegistry,
) -> TypeCheckResult {
    // Registry is not used here - cross-module symbols are already in the symbol table
    TypeChecker::new(scopes, symbols, imports).check(file)
}

// =============================================================================
// Tests
// =============================================================================

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
        typecheck(
            &file,
            &resolve_result.scopes,
            &resolve_result.symbols,
            &resolve_result.imports,
        )
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
        assert!(!result.has_errors(), "Errors: {:?}", result.diagnostics);
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

    #[test]
    fn test_empty_list_uses_expected_type() {
        // Test that empty list literals use the expected type from field declaration
        let source = r#"
module test

backend TodoBackend {
    items : List<String> = []
}
"#;
        let result = typecheck_source(source);
        assert!(!result.has_errors(), "Should have no errors");

        // Check that the expr_types contains List<String>, not List<unknown>
        let has_string_list = result
            .expr_types
            .values()
            .any(|ty| matches!(ty, Type::List(inner) if **inner == Type::String));
        assert!(
            has_string_list,
            "Empty list should be typed as List<String>"
        );
    }

    #[test]
    fn test_field_references_in_expressions() {
        // Test that field references in initializers resolve to the correct type
        let source = r#"
module test

backend Calculator {
    a : i32 = 10
    b : i32 = 20
    sum : i32 = a + b
    product : i32 = a * b
    isPositive : bool = sum > 0
}
"#;
        let result = typecheck_source(source);
        // Should have no errors - field types should be resolved correctly
        assert!(
            !result.has_errors(),
            "Field references should resolve correctly, got errors: {:?}",
            result.diagnostics
        );
    }

    fn resolve_and_typecheck_source(source: &str) -> (resolve::ResolveResult, TypeCheckResult) {
        let parse_result = parser::parse(source);
        assert!(
            !parse_result.diagnostics.has_errors(),
            "Parse errors: {:?}",
            parse_result.diagnostics
        );
        let file = parse_result.file.unwrap();
        let resolve_result = resolve::resolve(&file);
        let typecheck_result = typecheck(
            &file,
            &resolve_result.scopes,
            &resolve_result.symbols,
            &resolve_result.imports,
        );
        (resolve_result, typecheck_result)
    }

    #[test]
    fn test_select_on_enum_valid_variants() {
        // Test that valid enum variants in select statements are recognized
        let source = r#"
module test

enum Status { Pending Active Completed }

blueprint StatusView {
    status : Status = Status.Pending

    select on status {
        Pending => { x1 : i32 = 1 }
        Active => { x2 : i32 = 2 }
        Completed => { x3 : i32 = 3 }
    }
}
"#;
        let (resolve_result, typecheck_result) = resolve_and_typecheck_source(source);
        // Should have no resolve errors for enum variants
        assert!(
            !resolve_result.diagnostics.has_errors(),
            "Resolve errors for valid enum variants: {:?}",
            resolve_result.diagnostics
        );
        // Should have no typecheck errors
        assert!(
            !typecheck_result.has_errors(),
            "Typecheck errors for valid enum variants: {:?}",
            typecheck_result.diagnostics
        );
    }

    #[test]
    fn test_select_on_enum_invalid_variant() {
        // Test that invalid enum variants in select statements are caught
        let source = r#"
module test

enum Status { Pending Active Completed }

blueprint StatusView {
    status : Status = Status.Pending

    select on status {
        Pending => { x1 : i32 = 1 }
        Invalid => { x2 : i32 = 2 }
    }
}
"#;
        let (resolve_result, typecheck_result) = resolve_and_typecheck_source(source);
        // Should have no resolve errors (resolution is deferred for select branches)
        assert!(
            !resolve_result.diagnostics.has_errors(),
            "Should not have resolve errors: {:?}",
            resolve_result.diagnostics
        );
        // Should have typecheck error for invalid variant
        assert!(
            typecheck_result.has_errors(),
            "Should have typecheck error for invalid variant 'Invalid'"
        );
        assert!(
            typecheck_result.diagnostics.iter().any(|d| d.message.contains("no variant `Invalid`")),
            "Should have error about invalid variant: {:?}",
            typecheck_result.diagnostics
        );
    }
}
