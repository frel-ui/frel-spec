// Name resolution pass for Frel semantic analysis
//
// This module implements the name resolution pass that:
// 1. Builds the scope graph from the AST
// 2. Collects declarations into the symbol table
// 3. Resolves name references to their declarations
// 4. Reports resolution errors (undefined, duplicate, shadowing)

use crate::ast::{self, TopLevelDecl};
use crate::diagnostic::{codes, Diagnostic, Diagnostics, RelatedInfo};
use crate::source::Span;

use super::scope::{ScopeGraph, ScopeId, ScopeKind};
use super::symbol::{SymbolId, SymbolKind, SymbolTable};

/// Result of name resolution
#[derive(Debug)]
pub struct ResolveResult {
    /// The scope graph
    pub scopes: ScopeGraph,
    /// The symbol table
    pub symbols: SymbolTable,
    /// Diagnostics generated during resolution
    pub diagnostics: Diagnostics,
    /// Map from name references to resolved symbols
    pub resolutions: std::collections::HashMap<Span, SymbolId>,
    /// Imported names (name -> module path)
    pub imports: std::collections::HashMap<String, String>,
}

impl ResolveResult {
    pub fn new(scopes: ScopeGraph, symbols: SymbolTable, diagnostics: Diagnostics) -> Self {
        Self {
            scopes,
            symbols,
            diagnostics,
            resolutions: std::collections::HashMap::new(),
            imports: std::collections::HashMap::new(),
        }
    }
}

/// Name resolver that builds the scope graph and symbol table
pub struct Resolver {
    scopes: ScopeGraph,
    symbols: SymbolTable,
    diagnostics: Diagnostics,
    resolutions: std::collections::HashMap<Span, SymbolId>,
    /// Current scope being processed
    current_scope: ScopeId,
    /// Context span for error reporting (set to containing declaration's span)
    context_span: Span,
    /// Imported names (name -> module path)
    imports: std::collections::HashMap<String, String>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: ScopeGraph::new(),
            symbols: SymbolTable::new(),
            diagnostics: Diagnostics::new(),
            resolutions: std::collections::HashMap::new(),
            current_scope: ScopeId::ROOT,
            context_span: Span::default(),
            imports: std::collections::HashMap::new(),
        }
    }

    /// Resolve names in a file AST
    pub fn resolve(mut self, file: &ast::File) -> ResolveResult {
        // Create root/module scope
        self.current_scope = self.scopes.create_root(Span::default());

        // Collect imports
        self.collect_imports(file);

        // First pass: collect all top-level declarations
        self.collect_top_level_declarations(file);

        // Second pass: resolve within each declaration body
        self.resolve_declarations(file);

        ResolveResult {
            scopes: self.scopes,
            symbols: self.symbols,
            diagnostics: self.diagnostics,
            resolutions: self.resolutions,
            imports: self.imports,
        }
    }

    /// Collect import statements
    ///
    /// In Phase 1 (without registry), we can only handle single-declaration imports.
    /// Glob imports (`import foo.*`) require registry validation in Phase 2.
    fn collect_imports(&mut self, file: &ast::File) {
        for import in &file.imports {
            if import.import_all {
                // Glob imports need registry - skip in Phase 1
                continue;
            }
            // Single-declaration import: split path as module.name
            if let Some((module, name)) = import.path.rsplit_once('.') {
                self.imports.insert(name.to_string(), module.to_string());
            }
        }
    }

    /// Collect all top-level declarations into the symbol table
    fn collect_top_level_declarations(&mut self, file: &ast::File) {
        let module_scope = self.current_scope;

        for decl in &file.declarations {
            match decl {
                TopLevelDecl::Blueprint(bp) => {
                    self.define_type_declaration(
                        &bp.name,
                        SymbolKind::Blueprint,
                        ScopeKind::Blueprint,
                        module_scope,
                        bp.span,
                    );
                }
                TopLevelDecl::Backend(be) => {
                    self.define_type_declaration(
                        &be.name,
                        SymbolKind::Backend,
                        ScopeKind::Backend,
                        module_scope,
                        be.span,
                    );
                }
                TopLevelDecl::Scheme(sc) => {
                    self.define_type_declaration(
                        &sc.name,
                        SymbolKind::Scheme,
                        ScopeKind::Scheme,
                        module_scope,
                        sc.span,
                    );
                }
                TopLevelDecl::Contract(ct) => {
                    self.define_type_declaration(
                        &ct.name,
                        SymbolKind::Contract,
                        ScopeKind::Contract,
                        module_scope,
                        ct.span,
                    );
                }
                TopLevelDecl::Theme(th) => {
                    self.define_type_declaration(
                        &th.name,
                        SymbolKind::Theme,
                        ScopeKind::Theme,
                        module_scope,
                        th.span,
                    );
                }
                TopLevelDecl::Enum(en) => {
                    self.define_type_declaration(
                        &en.name,
                        SymbolKind::Enum,
                        ScopeKind::Enum,
                        module_scope,
                        en.span,
                    );
                }
                TopLevelDecl::Arena(ar) => {
                    self.define_simple(&ar.name, SymbolKind::Arena, module_scope, ar.span);
                }
            }
        }
    }

    /// Define a type declaration that creates its own scope
    fn define_type_declaration(
        &mut self,
        name: &str,
        kind: SymbolKind,
        scope_kind: ScopeKind,
        parent_scope: ScopeId,
        span: Span,
    ) -> Option<(SymbolId, ScopeId)> {
        // Check for duplicate
        if let Some(existing) = self.symbols.lookup_local(parent_scope, name) {
            self.report_duplicate(name, span, existing);
            return None;
        }

        // Create the body scope for this declaration
        let body_scope = self.scopes.create_named_scope(scope_kind, parent_scope, name, span);

        // Define the symbol
        let symbol_id = self
            .symbols
            .define_with_scope(name, kind, parent_scope, body_scope, span)?;

        Some((symbol_id, body_scope))
    }

    /// Define a simple symbol (no body scope)
    fn define_simple(
        &mut self,
        name: &str,
        kind: SymbolKind,
        scope: ScopeId,
        span: Span,
    ) -> Option<SymbolId> {
        // Check for duplicate
        if let Some(existing) = self.symbols.lookup_local(scope, name) {
            self.report_duplicate(name, span, existing);
            return None;
        }

        // Check for shadowing - only for scopes where names are accessed directly
        // Skip shadowing check for:
        // - Module scope (nothing above it)
        // - Type body scopes (Scheme, Backend, Contract, Theme, Enum) - members are accessed
        //   via qualifiers (e.g., `obj.field`), not bare identifiers
        if let Some(scope_data) = self.scopes.get(scope) {
            let skip_shadowing_check = matches!(
                scope_data.kind,
                ScopeKind::Module
                    | ScopeKind::Scheme
                    | ScopeKind::Backend
                    | ScopeKind::Contract
                    | ScopeKind::Theme
                    | ScopeKind::Enum
            );
            if !skip_shadowing_check {
                if let Some(shadowed) =
                    self.symbols.name_exists_in_ancestors(scope, name, &self.scopes)
                {
                    self.report_shadowing(name, span, shadowed);
                    return None;
                }
            }
        }

        self.symbols.define(name, kind, scope, span)
    }

    /// Resolve references within declarations
    fn resolve_declarations(&mut self, file: &ast::File) {
        for decl in &file.declarations {
            match decl {
                TopLevelDecl::Blueprint(bp) => self.resolve_blueprint(bp),
                TopLevelDecl::Backend(be) => self.resolve_backend(be),
                TopLevelDecl::Scheme(sc) => self.resolve_scheme(sc),
                TopLevelDecl::Contract(ct) => self.resolve_contract(ct),
                TopLevelDecl::Theme(th) => self.resolve_theme(th),
                TopLevelDecl::Enum(en) => self.resolve_enum(en),
                TopLevelDecl::Arena(ar) => self.resolve_arena(ar),
            }
        }
    }

    fn resolve_blueprint(&mut self, bp: &ast::Blueprint) {
        let module_scope = ScopeId::ROOT;

        // Find the symbol and its body scope
        let Some(symbol_id) = self.symbols.lookup_local(module_scope, &bp.name) else {
            return; // Was not defined (duplicate error already reported)
        };
        let Some(symbol) = self.symbols.get(symbol_id) else {
            return;
        };
        let Some(body_scope) = symbol.body_scope else {
            return;
        };

        // Define parameters in body scope
        for param in &bp.params {
            self.define_simple(&param.name, SymbolKind::Parameter, body_scope, Span::default());
        }

        // Resolve body statements
        self.current_scope = body_scope;
        for stmt in &bp.body {
            self.resolve_blueprint_stmt(stmt, &bp.params);
        }
        self.current_scope = module_scope;
    }

    fn resolve_blueprint_stmt(&mut self, stmt: &ast::BlueprintStmt, params: &[ast::Parameter]) {
        match stmt {
            ast::BlueprintStmt::With(name) => {
                // Resolve backend reference and import its members into the blueprint scope
                // Look up from current scope to find both module-level backends and parameters
                if let Some(symbol_id) = self.symbols.lookup_in_scope_chain(self.current_scope, name, &self.scopes) {
                    if let Some(symbol) = self.symbols.get(symbol_id) {
                        // Get the body scope - either directly from the symbol (for backends)
                        // or by looking up the parameter's type (for parameters)
                        let body_scope = if let Some(scope) = symbol.body_scope {
                            Some(scope)
                        } else if symbol.kind == SymbolKind::Parameter {
                            // For parameters, look up the type from the AST and find its body scope
                            self.get_parameter_type_body_scope(name, params)
                        } else {
                            None
                        };

                        if let Some(backend_body_scope) = body_scope {
                            // Collect members to import (avoid borrowing issues)
                            let members_to_import: Vec<_> = self.symbols
                                .symbols_in_scope(backend_body_scope)
                                .map(|s| (s.name.clone(), s.kind, s.def_span))
                                .collect();

                            // Import each member into the current blueprint scope
                            for (member_name, member_kind, member_span) in members_to_import {
                                self.define_simple(&member_name, member_kind, self.current_scope, member_span);
                            }
                        }
                    }
                } else {
                    // Backend not found - report error
                    self.diagnostics.add(Diagnostic::from_code(
                        &codes::E0301,
                        Span::default(),
                        format!("cannot find backend `{}` in this scope", name),
                    ));
                }
            }
            ast::BlueprintStmt::LocalDecl(decl) => {
                // Resolve the initializer first (before adding to scope)
                self.resolve_expr(&decl.init);
                // Then define the local
                self.define_simple(
                    &decl.name,
                    SymbolKind::LocalVar,
                    self.current_scope,
                    decl.span,
                );
            }
            ast::BlueprintStmt::FragmentCreation(frag) => {
                // Resolve the fragment blueprint name (skip for anonymous blocks which have empty names)
                if !frag.name.is_empty() {
                    self.resolve_name(&frag.name, Span::default());
                }
                // Resolve arguments
                for arg in &frag.args {
                    self.resolve_expr(&arg.value);
                }
                // Resolve body if present
                if let Some(body) = &frag.body {
                    self.resolve_fragment_body(body, params);
                }
                // Resolve postfix items
                for postfix in &frag.postfix {
                    match postfix {
                        ast::PostfixItem::Instruction(instr) => self.resolve_instruction_expr(instr),
                        ast::PostfixItem::EventHandler(handler) => self.resolve_event_handler(handler),
                    }
                }
            }
            ast::BlueprintStmt::Control(ctrl) => self.resolve_control_stmt(ctrl, params),
            ast::BlueprintStmt::Instruction(instr) => self.resolve_instruction_expr(instr),
            ast::BlueprintStmt::EventHandler(handler) => self.resolve_event_handler(handler),
            ast::BlueprintStmt::Layout(layout) => self.resolve_layout_stmt(layout),
            ast::BlueprintStmt::SlotBinding(binding) => self.resolve_slot_binding(binding, params),
            ast::BlueprintStmt::ContentExpr(expr) => self.resolve_expr(expr),
        }
    }

    /// Helper to get the body scope of a parameter's type
    fn get_parameter_type_body_scope(&self, param_name: &str, params: &[ast::Parameter]) -> Option<ScopeId> {
        // Find the parameter in the AST
        let param = params.iter().find(|p| p.name == param_name)?;

        // Get the type name from the type expression
        let type_name = match &param.type_expr {
            ast::TypeExpr::Named(name) => name,
            _ => return None, // Can't use 'with' on non-named types
        };

        // Look up the type in the module scope
        let type_symbol_id = self.symbols.lookup_in_scope_chain(ScopeId::ROOT, type_name, &self.scopes)?;
        let type_symbol = self.symbols.get(type_symbol_id)?;

        // Return its body scope
        type_symbol.body_scope
    }

    fn resolve_fragment_body(&mut self, body: &ast::FragmentBody, params: &[ast::Parameter]) {
        match body {
            ast::FragmentBody::Default(stmts) => {
                for stmt in stmts {
                    self.resolve_blueprint_stmt(stmt, params);
                }
            }
            ast::FragmentBody::Slots(slots) => {
                for slot in slots {
                    self.resolve_slot_binding(slot, params);
                }
            }
            ast::FragmentBody::InlineBlueprint { params: inline_params, body } => {
                // Create a new scope for the inline blueprint
                let inline_scope = self.scopes.create_scope(
                    ScopeKind::Blueprint,
                    self.current_scope,
                    Span::default(),
                );
                let old_scope = self.current_scope;
                self.current_scope = inline_scope;

                // Define parameters
                for param in inline_params {
                    self.define_simple(param, SymbolKind::Parameter, inline_scope, Span::default());
                }

                // Resolve body - inline blueprints don't inherit outer params for 'with'
                for stmt in body {
                    self.resolve_blueprint_stmt(stmt, &[]);
                }

                self.current_scope = old_scope;
            }
        }
    }

    fn resolve_slot_binding(&mut self, binding: &ast::SlotBinding, _params: &[ast::Parameter]) {
        match &binding.blueprint {
            ast::BlueprintValue::Inline { params: inline_params, body } => {
                let inline_scope = self.scopes.create_scope(
                    ScopeKind::Blueprint,
                    self.current_scope,
                    Span::default(),
                );
                let old_scope = self.current_scope;
                self.current_scope = inline_scope;

                for param in inline_params {
                    self.define_simple(param, SymbolKind::Parameter, inline_scope, Span::default());
                }

                // Inline blueprints don't inherit outer params for 'with'
                for stmt in body {
                    self.resolve_blueprint_stmt(stmt, &[]);
                }

                self.current_scope = old_scope;
            }
            ast::BlueprintValue::Reference(name) => {
                self.resolve_name(name, Span::default());
            }
        }
    }

    fn resolve_layout_stmt(&mut self, layout: &ast::LayoutStmt) {
        for instr in &layout.instructions {
            self.resolve_instruction_expr(instr);
        }
    }

    fn resolve_control_stmt(&mut self, ctrl: &ast::ControlStmt, params: &[ast::Parameter]) {
        match ctrl {
            ast::ControlStmt::When {
                condition,
                then_stmt,
                else_stmt,
            } => {
                self.resolve_expr(condition);
                self.resolve_blueprint_stmt(then_stmt, params);
                if let Some(else_stmt) = else_stmt {
                    self.resolve_blueprint_stmt(else_stmt, params);
                }
            }
            ast::ControlStmt::Repeat {
                iterable,
                item_name,
                key_expr,
                body,
            } => {
                // Resolve iterable in current scope (e.g., `todos` from backend)
                self.resolve_expr(iterable);

                // Create scope for loop body with loop variable
                // The loop variable must be defined BEFORE resolving key_expr
                // because `by todo.id` needs access to `todo`
                let loop_scope = self.scopes.create_scope(
                    ScopeKind::Block,
                    self.current_scope,
                    Span::default(),
                );
                let old_scope = self.current_scope;
                self.current_scope = loop_scope;

                // Define the explicit loop variable (e.g., `item` in `repeat on items { item -> ... }`)
                self.define_simple(item_name, SymbolKind::LocalVar, loop_scope, Span::default());

                // Now resolve key_expr with loop variable in scope
                if let Some(key) = key_expr {
                    self.resolve_expr(key);
                }

                for stmt in body {
                    self.resolve_blueprint_stmt(stmt, params);
                }

                self.current_scope = old_scope;
            }
            ast::ControlStmt::Select {
                discriminant,
                branches,
                else_branch,
            } => {
                if let Some(disc) = discriminant {
                    self.resolve_expr(disc);
                }
                for branch in branches {
                    // When there's a discriminant, skip resolution for simple identifiers.
                    // They may be enum variant names that can only be resolved once we know
                    // the discriminant type in the typecheck phase.
                    let should_skip = discriminant.is_some()
                        && matches!(&branch.condition, ast::Expr::Identifier(_));
                    if !should_skip {
                        self.resolve_expr(&branch.condition);
                    }
                    self.resolve_blueprint_stmt(&branch.body, params);
                }
                if let Some(else_stmt) = else_branch {
                    self.resolve_blueprint_stmt(else_stmt, params);
                }
            }
        }
    }

    fn resolve_instruction_expr(&mut self, instr: &ast::InstructionExpr) {
        use super::instructions::instruction_registry;
        let registry = instruction_registry();

        match instr {
            ast::InstructionExpr::Simple(inst) => {
                // Set context span for error reporting
                self.context_span = inst.span;

                for (param_name, expr) in &inst.params {
                    // Check if this is a simple identifier
                    if let ast::Expr::Identifier(value) = expr {
                        // Check if this is a valid keyword for this instruction parameter
                        let is_valid_keyword = registry.is_valid_keyword(&inst.name, param_name, value);

                        // Check if the instruction accepts expressions for this parameter
                        let accepts_expr = registry.accepts_expression(&inst.name, param_name);

                        if is_valid_keyword {
                            // Valid keyword - skip resolution (it's a contextual keyword)
                            continue;
                        } else if accepts_expr {
                            // Instruction accepts expressions - resolve the identifier
                            self.resolve_expr(expr);
                        } else {
                            // Instruction only accepts keywords but this isn't a valid one.
                            // Skip resolution (error will be reported in type checker)
                            continue;
                        }
                    } else {
                        // Not a simple identifier - always resolve
                        self.resolve_expr(expr);
                    }
                }
            }
            ast::InstructionExpr::When {
                condition,
                then_instr,
                else_instr,
            } => {
                self.resolve_expr(condition);
                self.resolve_instruction_expr(then_instr);
                if let Some(else_instr) = else_instr {
                    self.resolve_instruction_expr(else_instr);
                }
            }
            ast::InstructionExpr::Ternary {
                condition,
                then_instr,
                else_instr,
            } => {
                self.resolve_expr(condition);
                self.resolve_instruction_expr(then_instr);
                self.resolve_instruction_expr(else_instr);
            }
            ast::InstructionExpr::Reference(expr) => {
                // Single identifiers (like `.. focusable`) are instruction names, not variable references.
                // Only resolve field access expressions (like `.. theme.primary_button`).
                if !matches!(expr, ast::Expr::Identifier(_)) {
                    self.resolve_expr(expr);
                }
            }
        }
    }

    fn resolve_event_handler(&mut self, handler: &ast::EventHandler) {
        // Create scope for handler body
        let handler_scope = self.scopes.create_scope(
            ScopeKind::Block,
            self.current_scope,
            Span::default(),
        );
        let old_scope = self.current_scope;
        self.current_scope = handler_scope;

        // Define event parameter if present
        if let Some(param) = &handler.param {
            self.define_simple(&param.name, SymbolKind::Parameter, handler_scope, Span::default());
        }

        // Resolve handler statements
        for stmt in &handler.body {
            match stmt {
                ast::HandlerStmt::Assignment { name, value } => {
                    // Resolve the value first
                    self.resolve_expr(value);
                    // Then resolve the target (should exist)
                    self.resolve_name(name, Span::default());
                }
                ast::HandlerStmt::CommandCall { name, args } => {
                    // Resolve command name
                    self.resolve_name(name, Span::default());
                    // Resolve arguments
                    for arg in args {
                        self.resolve_expr(arg);
                    }
                }
            }
        }

        self.current_scope = old_scope;
    }

    fn resolve_backend(&mut self, be: &ast::Backend) {
        let module_scope = ScopeId::ROOT;

        let Some(symbol_id) = self.symbols.lookup_local(module_scope, &be.name) else {
            return;
        };
        let Some(symbol) = self.symbols.get(symbol_id) else {
            return;
        };
        let Some(body_scope) = symbol.body_scope else {
            return;
        };

        // Define parameters
        for param in &be.params {
            self.define_simple(&param.name, SymbolKind::Parameter, body_scope, Span::default());
        }

        // Process members
        for member in &be.members {
            match member {
                ast::BackendMember::Include(name) => {
                    // Resolve included backend and import its members
                    if let Some(included_id) = self.symbols.lookup_in_scope_chain(ScopeId::ROOT, name, &self.scopes) {
                        if let Some(included_symbol) = self.symbols.get(included_id) {
                            if let Some(included_body_scope) = included_symbol.body_scope {
                                // Collect members to import (avoid borrowing issues)
                                let members_to_import: Vec<_> = self.symbols
                                    .symbols_in_scope(included_body_scope)
                                    .map(|s| (s.name.clone(), s.kind, s.def_span))
                                    .collect();

                                // Import each member into the current backend scope
                                for (member_name, member_kind, member_span) in members_to_import {
                                    self.define_simple(&member_name, member_kind, body_scope, member_span);
                                }
                            }
                        }
                    } else {
                        // Backend not found - report error
                        self.diagnostics.add(Diagnostic::from_code(
                            &codes::E0301,
                            Span::default(),
                            format!("cannot find backend `{}` in this scope", name),
                        ));
                    }
                }
                ast::BackendMember::Field(field) => {
                    self.define_simple(&field.name, SymbolKind::Field, body_scope, field.span);
                    if let Some(init) = &field.init {
                        self.current_scope = body_scope;
                        self.context_span = field.span;
                        self.resolve_expr(init);
                        self.current_scope = module_scope;
                    }
                }
                ast::BackendMember::Method(method) => {
                    self.define_simple(&method.name, SymbolKind::Method, body_scope, method.span);
                }
                ast::BackendMember::Command(cmd) => {
                    self.define_simple(&cmd.name, SymbolKind::Command, body_scope, cmd.span);
                }
            }
        }
    }

    fn resolve_scheme(&mut self, sc: &ast::Scheme) {
        let module_scope = ScopeId::ROOT;

        let Some(symbol_id) = self.symbols.lookup_local(module_scope, &sc.name) else {
            return;
        };
        let Some(symbol) = self.symbols.get(symbol_id) else {
            return;
        };
        let Some(body_scope) = symbol.body_scope else {
            return;
        };

        for member in &sc.members {
            match member {
                ast::SchemeMember::Field(field) => {
                    self.define_simple(&field.name, SymbolKind::Field, body_scope, field.span);
                }
                ast::SchemeMember::Virtual(virt) => {
                    self.define_simple(&virt.name, SymbolKind::VirtualField, body_scope, virt.span);
                    // Resolve the virtual expression
                    self.current_scope = body_scope;
                    self.context_span = virt.span;
                    self.resolve_expr(&virt.expr);
                    self.current_scope = module_scope;
                }
            }
        }
    }

    fn resolve_contract(&mut self, ct: &ast::Contract) {
        let module_scope = ScopeId::ROOT;

        let Some(symbol_id) = self.symbols.lookup_local(module_scope, &ct.name) else {
            return;
        };
        let Some(symbol) = self.symbols.get(symbol_id) else {
            return;
        };
        let Some(body_scope) = symbol.body_scope else {
            return;
        };

        for method in &ct.methods {
            self.define_simple(&method.name, SymbolKind::Method, body_scope, method.span);
        }
    }

    fn resolve_theme(&mut self, th: &ast::Theme) {
        let module_scope = ScopeId::ROOT;

        let Some(symbol_id) = self.symbols.lookup_local(module_scope, &th.name) else {
            return;
        };
        let Some(symbol) = self.symbols.get(symbol_id) else {
            return;
        };
        let Some(body_scope) = symbol.body_scope else {
            return;
        };

        for member in &th.members {
            match member {
                ast::ThemeMember::Include(name) => {
                    self.resolve_name(name, Span::default());
                }
                ast::ThemeMember::Field(field) => {
                    self.define_simple(&field.name, SymbolKind::Field, body_scope, field.span);
                    if let Some(init) = &field.init {
                        self.current_scope = body_scope;
                        self.context_span = field.span;
                        self.resolve_expr(init);
                        self.current_scope = module_scope;
                    }
                }
                ast::ThemeMember::InstructionSet(iset) => {
                    self.define_simple(&iset.name, SymbolKind::InstructionSet, body_scope, Span::default());
                }
                ast::ThemeMember::Variant(variant) => {
                    self.define_simple(&variant.name, SymbolKind::ThemeVariant, body_scope, Span::default());
                }
            }
        }
    }

    fn resolve_enum(&mut self, en: &ast::Enum) {
        let module_scope = ScopeId::ROOT;

        let Some(symbol_id) = self.symbols.lookup_local(module_scope, &en.name) else {
            return;
        };
        let Some(symbol) = self.symbols.get(symbol_id) else {
            return;
        };
        let Some(body_scope) = symbol.body_scope else {
            return;
        };

        // Define enum variants
        for variant in &en.variants {
            self.define_simple(variant, SymbolKind::EnumVariant, body_scope, Span::default());
        }
    }

    fn resolve_arena(&mut self, ar: &ast::Arena) {
        // Resolve scheme reference
        self.resolve_name(&ar.scheme_name, Span::default());
        // Resolve contract reference if present
        if let Some(contract) = &ar.contract {
            self.resolve_name(contract, Span::default());
        }
    }

    /// Resolve a name reference
    fn resolve_name(&mut self, name: &str, span: Span) -> Option<SymbolId> {
        // 4-layer lookup: local -> parent -> imports -> module
        if let Some(id) = self.symbols.lookup_in_scope_chain(self.current_scope, name, &self.scopes) {
            self.resolutions.insert(span, id);
            return Some(id);
        }

        // Check imports - if name is imported, accept it without error
        // (the imported symbol's actual definition is in another module)
        if self.imports.contains_key(name) {
            // Imported name - no error, but no symbol ID either since
            // the actual symbol is in another module
            return None;
        }

        // Not found
        self.report_undefined(name, span);
        None
    }

    /// Resolve an expression
    fn resolve_expr(&mut self, expr: &ast::Expr) {
        match expr {
            ast::Expr::Null
            | ast::Expr::Bool(_)
            | ast::Expr::Int(_)
            | ast::Expr::Float(_)
            | ast::Expr::Color(_)
            | ast::Expr::String(_) => {
                // Literals don't need resolution
            }
            ast::Expr::StringTemplate(elements) => {
                for elem in elements {
                    if let ast::TemplateElement::Interpolation(inner) = elem {
                        self.resolve_expr(inner);
                    }
                }
            }
            ast::Expr::List(items) => {
                for item in items {
                    self.resolve_expr(item);
                }
            }
            ast::Expr::Object(fields) => {
                for (_, value) in fields {
                    self.resolve_expr(value);
                }
            }
            ast::Expr::Identifier(name) => {
                self.resolve_name(name, self.context_span);
            }
            ast::Expr::QualifiedName(parts) => {
                // Resolve the first part, then field accesses
                if let Some(first) = parts.first() {
                    self.resolve_name(first, self.context_span);
                }
                // Additional parts are field accesses, resolved during type checking
            }
            ast::Expr::Binary { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            ast::Expr::Unary { expr, .. } => {
                self.resolve_expr(expr);
            }
            ast::Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.resolve_expr(condition);
                self.resolve_expr(then_expr);
                self.resolve_expr(else_expr);
            }
            ast::Expr::FieldAccess { base, .. } => {
                self.resolve_expr(base);
                // Field name resolved during type checking
            }
            ast::Expr::OptionalChain { base, .. } => {
                self.resolve_expr(base);
            }
            ast::Expr::Call { callee, args } => {
                self.resolve_expr(callee);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
        }
    }

    // ========================================================================
    // Error reporting
    // ========================================================================

    fn report_duplicate(&mut self, name: &str, span: Span, existing: SymbolId) {
        let existing_symbol = self.symbols.get(existing);
        let existing_span = existing_symbol.map(|s| s.def_span).unwrap_or_default();

        let diag = Diagnostic::from_code(
            &codes::E0302,
            span,
            format!("`{}` is already defined in this scope", name),
        )
        .with_related(RelatedInfo::new(
            existing_span,
            format!("`{}` previously defined here", name),
        ));

        self.diagnostics.add(diag);
    }

    fn report_shadowing(&mut self, name: &str, span: Span, shadowed: SymbolId) {
        let shadowed_symbol = self.symbols.get(shadowed);
        let shadowed_span = shadowed_symbol.map(|s| s.def_span).unwrap_or_default();

        let diag = Diagnostic::from_code(
            &codes::E0303,
            span,
            format!("`{}` would shadow a name in an outer scope", name),
        )
        .with_related(RelatedInfo::new(
            shadowed_span,
            format!("`{}` is defined in an outer scope here", name),
        ))
        .with_help("Frel does not allow shadowing. Consider using a different name.");

        self.diagnostics.add(diag);
    }

    fn report_undefined(&mut self, name: &str, span: Span) {
        let diag = Diagnostic::from_code(
            &codes::E0301,
            span,
            format!("cannot find `{}` in this scope", name),
        );

        self.diagnostics.add(diag);
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolve names in a file AST
pub fn resolve(file: &ast::File) -> ResolveResult {
    Resolver::new().resolve(file)
}

/// Resolve names in a file AST with access to external module signatures
///
/// This extends basic resolution by resolving imported names against
/// the provided SignatureRegistry, enabling cross-module type checking.
pub fn resolve_with_registry(
    file: &ast::File,
    registry: &super::signature::SignatureRegistry,
) -> ResolveResult {
    ResolverWithRegistry::new(registry).resolve(file)
}

/// Name resolver with access to external module signatures
struct ResolverWithRegistry<'a> {
    inner: Resolver,
    registry: &'a super::signature::SignatureRegistry,
}

impl<'a> ResolverWithRegistry<'a> {
    fn new(registry: &'a super::signature::SignatureRegistry) -> Self {
        Self {
            inner: Resolver::new(),
            registry,
        }
    }

    fn resolve(mut self, file: &ast::File) -> ResolveResult {
        // Create root/module scope
        self.inner.current_scope = self.inner.scopes.create_root(Span::default());

        // Collect imports and validate them against registry
        self.collect_and_validate_imports(file);

        // First pass: collect all top-level declarations
        self.inner.collect_top_level_declarations(file);

        // Second pass: resolve within each declaration body
        self.inner.resolve_declarations(file);

        ResolveResult {
            scopes: self.inner.scopes,
            symbols: self.inner.symbols,
            diagnostics: self.inner.diagnostics,
            resolutions: self.inner.resolutions,
            imports: self.inner.imports,
        }
    }

    fn collect_and_validate_imports(&mut self, file: &ast::File) {
        for import in &file.imports {
            if import.import_all {
                // Glob import: `import foo.bar.*`
                // The path is the module path
                if let Some(module_sig) = self.registry.get(&import.path) {
                    for export in module_sig.all_exports() {
                        self.import_external_with_body(
                            &export.name,
                            export.kind,
                            import.span,
                            &import.path,
                            export.body_scope,
                            module_sig,
                        );
                        self.inner
                            .imports
                            .insert(export.name.clone(), import.path.clone());
                    }
                } else {
                    self.inner.diagnostics.error(
                        format!("module '{}' not found", import.path),
                        import.span,
                    );
                }
            } else {
                // Single-declaration import: `import foo.bar.Baz`
                // The path includes module + declaration name
                if let Some((module, name)) = import.path.rsplit_once('.') {
                    if let Some(module_sig) = self.registry.get(module) {
                        if let Some(export) = module_sig.get_export(name) {
                            self.import_external_with_body(
                                &export.name,
                                export.kind,
                                import.span,
                                module,
                                export.body_scope,
                                module_sig,
                            );
                            self.inner
                                .imports
                                .insert(name.to_string(), module.to_string());
                        } else {
                            self.inner.diagnostics.error(
                                format!("'{}' is not exported from module '{}'", name, module),
                                import.span,
                            );
                        }
                    } else {
                        self.inner.diagnostics.error(
                            format!("module '{}' not found", module),
                            import.span,
                        );
                    }
                } else {
                    // Single-component path - not valid for single-declaration import
                    self.inner.diagnostics.error(
                        format!(
                            "invalid import '{}': use 'import {}.*' to import all exports",
                            import.path, import.path
                        ),
                        import.span,
                    );
                }
            }
        }
    }

    /// Import an external declaration, including its body scope and member symbols
    fn import_external_with_body(
        &mut self,
        name: &str,
        kind: SymbolKind,
        span: Span,
        source_module: &str,
        body_scope: Option<ScopeId>,
        module_sig: &super::signature::ModuleSignature,
    ) {
        // Define the external symbol
        let symbol_id = self.inner.symbols.define_external(
            name,
            kind,
            ScopeId::ROOT,
            span,
            source_module.to_string(),
        );

        // If the symbol has a body scope, create a local copy with its members
        if let (Some(symbol_id), Some(orig_body_scope)) = (symbol_id, body_scope) {
            // Get the scope kind from the original scope
            let scope_kind = module_sig
                .get_scope(orig_body_scope)
                .map(|s| s.kind)
                .unwrap_or(ScopeKind::Block);

            // Create a new body scope in our local scope graph
            let local_body_scope = self.inner.scopes.create_named_scope(
                scope_kind,
                ScopeId::ROOT,
                name,
                span,
            );

            // Set the body scope on the imported symbol
            if let Some(symbol) = self.inner.symbols.get_mut(symbol_id) {
                symbol.body_scope = Some(local_body_scope);
            }

            // Copy all member symbols from the original body scope
            let members: Vec<_> = module_sig
                .symbols
                .symbols_in_scope(orig_body_scope)
                .map(|s| (s.name.clone(), s.kind))
                .collect();

            for (member_name, member_kind) in members {
                self.inner.symbols.define_external(
                    &member_name,
                    member_kind,
                    local_body_scope,
                    span,
                    source_module.to_string(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn parse_and_resolve(source: &str) -> ResolveResult {
        let parse_result = parser::parse(source);
        assert!(
            !parse_result.diagnostics.has_errors(),
            "Parse errors: {:?}",
            parse_result.diagnostics
        );
        let file = parse_result.file.unwrap();
        resolve(&file)
    }

    #[test]
    fn test_resolve_backend() {
        let source = r#"
module test

backend Counter {
    count: i32 = 0
    command increment()
    command decrement()
}
"#;
        let result = parse_and_resolve(source);
        assert!(!result.diagnostics.has_errors());

        // Should have module scope + backend scope
        assert_eq!(result.scopes.len(), 2);

        // Should have Counter + its members
        assert!(result.symbols.len() >= 4);
    }

    #[test]
    fn test_resolve_blueprint_with_backend() {
        let source = r#"
module test

backend Counter {
    count: i32 = 0
}

blueprint CounterView {
    with Counter
}
"#;
        let result = parse_and_resolve(source);
        assert!(!result.diagnostics.has_errors());
    }

    #[test]
    fn test_resolve_repeat_with_key() {
        // Test that the loop variable is available in the key expression
        let source = r#"
module test

scheme Todo {
    id: Uuid
    text: String
    done: bool
}

backend TodoBackend {
    todos: List<Todo> = []
}

blueprint TodoItem { }

blueprint TodoList {
    with TodoBackend

    repeat on todos by todo.id { todo ->
        TodoItem()
    }
}
"#;
        let result = parse_and_resolve(source);
        // The loop variable `todo` should be available in `by todo.id`
        // (No errors about `todo` not being found)
        assert!(
            !result.diagnostics.has_errors(),
            "Expected no errors, got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_resolve_duplicate_error() {
        let source = r#"
module test

backend Foo { }
backend Foo { }
"#;
        let result = parse_and_resolve(source);
        assert!(result.diagnostics.has_errors());
        assert_eq!(result.diagnostics.error_count(), 1);

        // Check it's the right error code
        let errors: Vec<_> = result.diagnostics.iter().collect();
        assert_eq!(errors[0].code, Some("E0302".to_string()));
    }

    #[test]
    fn test_resolve_scheme() {
        let source = r#"
module test

scheme Person {
    name: string
    age: i32
    virtual is_adult: bool = age >= 18
}
"#;
        let result = parse_and_resolve(source);
        assert!(!result.diagnostics.has_errors());

        // Should have fields and virtual defined
        let scheme_symbols: Vec<_> = result.symbols.iter()
            .filter(|s| s.kind == SymbolKind::Field || s.kind == SymbolKind::VirtualField)
            .collect();
        assert_eq!(scheme_symbols.len(), 3);
    }

    #[test]
    fn test_resolve_enum() {
        let source = r#"
module test

enum Status {
    Active
    Inactive
    Pending
}
"#;
        let result = parse_and_resolve(source);
        assert!(!result.diagnostics.has_errors());

        // Should have enum + 3 variants
        let variants: Vec<_> = result.symbols.iter()
            .filter(|s| s.kind == SymbolKind::EnumVariant)
            .collect();
        assert_eq!(variants.len(), 3);
    }

    #[test]
    fn test_import_backend_with_commands() {
        use super::super::signature::SignatureRegistry;
        use super::super::signature_builder::build_signature;
        use crate::Module;

        // Build signature for backend module
        let backend_source = r#"
module test.backend

backend EditorBackend {
    content: String
    command save()
}
"#;
        let parse_result = parser::parse(backend_source);
        assert!(!parse_result.diagnostics.has_errors());
        let backend_file = parse_result.file.unwrap();
        let backend_module = Module::from_file(backend_file);
        let sig_result = build_signature(&backend_module);
        assert!(!sig_result.has_errors(), "Signature errors: {:?}", sig_result.diagnostics);

        // Verify signature has both members
        let export = sig_result.signature.get_export("EditorBackend").unwrap();
        let body_scope = export.body_scope.unwrap();
        let members: Vec<_> = sig_result.signature.symbols.symbols_in_scope(body_scope).collect();
        assert_eq!(members.len(), 2, "Signature should have 2 members: {:?}",
            members.iter().map(|m| &m.name).collect::<Vec<_>>());

        // Register signature
        let mut registry = SignatureRegistry::new();
        registry.register(sig_result.signature);

        // Now resolve the importing module
        let editor_source = r#"
module test.editor

import test.backend.EditorBackend

blueprint Editor {
    with EditorBackend
}
"#;
        let parse_result = parser::parse(editor_source);
        assert!(!parse_result.diagnostics.has_errors());
        let editor_file = parse_result.file.unwrap();
        let result = resolve_with_registry(&editor_file, &registry);

        // Check that EditorBackend was imported with its body scope
        let editor_backend_id = result.symbols.lookup_local(ScopeId::ROOT, "EditorBackend");
        assert!(editor_backend_id.is_some(), "EditorBackend should be imported");

        let editor_backend = result.symbols.get(editor_backend_id.unwrap()).unwrap();
        assert!(editor_backend.body_scope.is_some(), "Imported EditorBackend should have body_scope");

        let imported_body_scope = editor_backend.body_scope.unwrap();
        let imported_members: Vec<_> = result.symbols.symbols_in_scope(imported_body_scope).collect();

        // Both content and save should be imported
        assert_eq!(imported_members.len(), 2, "Imported backend should have 2 members: {:?}",
            imported_members.iter().map(|m| &m.name).collect::<Vec<_>>());

        let has_content = imported_members.iter().any(|m| m.name == "content");
        let has_save = imported_members.iter().any(|m| m.name == "save");
        assert!(has_content, "Should have content field");
        assert!(has_save, "Should have save command");
    }

    #[test]
    fn test_resolve_text_fragment_in_repeat() {
        // Test that using `text` fragment doesn't conflict with `text` field in scheme
        let source = r#"
module test

scheme Todo {
    id: Uuid
    text: String
    done: bool
}

backend TodoBackend {
    todos: List<Todo> = []
}

blueprint text { }

blueprint TodoList {
    with TodoBackend

    repeat on todos by todo.id { todo ->
        text { todo.text }
    }
}
"#;
        let result = parse_and_resolve(source);
        // Should not have shadowing errors - `text` in `text { todo.text }`
        // is a fragment reference, not a new definition
        assert!(
            !result.diagnostics.has_errors(),
            "Expected no errors, got: {:?}",
            result.diagnostics
        );
    }
}
