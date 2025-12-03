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
    fn collect_imports(&mut self, file: &ast::File) {
        for import in &file.imports {
            // Store the imported name mapping to its module
            self.imports
                .insert(import.name.clone(), import.module.clone());
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

        // Check for shadowing (only for non-module scopes)
        if let Some(scope_data) = self.scopes.get(scope) {
            if scope_data.kind != ScopeKind::Module {
                if let Some(shadowed) = self.symbols.name_exists_in_ancestors(scope, name, &self.scopes) {
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
            self.resolve_blueprint_stmt(stmt);
        }
        self.current_scope = module_scope;
    }

    fn resolve_blueprint_stmt(&mut self, stmt: &ast::BlueprintStmt) {
        match stmt {
            ast::BlueprintStmt::With(name) => {
                // Resolve backend reference
                self.resolve_name(name, Span::default());
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
                // Resolve the fragment blueprint name
                self.resolve_name(&frag.name, Span::default());
                // Resolve arguments
                for arg in &frag.args {
                    self.resolve_expr(&arg.value);
                }
                // Resolve body if present
                if let Some(body) = &frag.body {
                    self.resolve_fragment_body(body);
                }
                // Resolve postfix items
                for postfix in &frag.postfix {
                    match postfix {
                        ast::PostfixItem::Instruction(instr) => self.resolve_instruction_expr(instr),
                        ast::PostfixItem::EventHandler(handler) => self.resolve_event_handler(handler),
                    }
                }
            }
            ast::BlueprintStmt::Control(ctrl) => self.resolve_control_stmt(ctrl),
            ast::BlueprintStmt::Instruction(instr) => self.resolve_instruction_expr(instr),
            ast::BlueprintStmt::EventHandler(handler) => self.resolve_event_handler(handler),
            ast::BlueprintStmt::Layout(layout) => self.resolve_layout_stmt(layout),
            ast::BlueprintStmt::SlotBinding(binding) => self.resolve_slot_binding(binding),
            ast::BlueprintStmt::ContentExpr(expr) => self.resolve_expr(expr),
        }
    }

    fn resolve_fragment_body(&mut self, body: &ast::FragmentBody) {
        match body {
            ast::FragmentBody::Default(stmts) => {
                for stmt in stmts {
                    self.resolve_blueprint_stmt(stmt);
                }
            }
            ast::FragmentBody::Slots(slots) => {
                for slot in slots {
                    self.resolve_slot_binding(slot);
                }
            }
            ast::FragmentBody::InlineBlueprint { params, body } => {
                // Create a new scope for the inline blueprint
                let inline_scope = self.scopes.create_scope(
                    ScopeKind::Blueprint,
                    self.current_scope,
                    Span::default(),
                );
                let old_scope = self.current_scope;
                self.current_scope = inline_scope;

                // Define parameters
                for param in params {
                    self.define_simple(param, SymbolKind::Parameter, inline_scope, Span::default());
                }

                // Resolve body
                for stmt in body {
                    self.resolve_blueprint_stmt(stmt);
                }

                self.current_scope = old_scope;
            }
        }
    }

    fn resolve_slot_binding(&mut self, binding: &ast::SlotBinding) {
        match &binding.blueprint {
            ast::BlueprintValue::Inline { params, body } => {
                let inline_scope = self.scopes.create_scope(
                    ScopeKind::Blueprint,
                    self.current_scope,
                    Span::default(),
                );
                let old_scope = self.current_scope;
                self.current_scope = inline_scope;

                for param in params {
                    self.define_simple(param, SymbolKind::Parameter, inline_scope, Span::default());
                }

                for stmt in body {
                    self.resolve_blueprint_stmt(stmt);
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

    fn resolve_control_stmt(&mut self, ctrl: &ast::ControlStmt) {
        match ctrl {
            ast::ControlStmt::When {
                condition,
                then_stmt,
                else_stmt,
            } => {
                self.resolve_expr(condition);
                self.resolve_blueprint_stmt(then_stmt);
                if let Some(else_stmt) = else_stmt {
                    self.resolve_blueprint_stmt(else_stmt);
                }
            }
            ast::ControlStmt::Repeat {
                iterable,
                item_name,
                key_expr,
                body,
            } => {
                self.resolve_expr(iterable);
                if let Some(key) = key_expr {
                    self.resolve_expr(key);
                }

                // Create scope for loop body with loop variable
                let loop_scope = self.scopes.create_scope(
                    ScopeKind::Block,
                    self.current_scope,
                    Span::default(),
                );
                let old_scope = self.current_scope;
                self.current_scope = loop_scope;

                self.define_simple(item_name, SymbolKind::LocalVar, loop_scope, Span::default());

                for stmt in body {
                    self.resolve_blueprint_stmt(stmt);
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
                    self.resolve_expr(&branch.condition);
                    self.resolve_blueprint_stmt(&branch.body);
                }
                if let Some(else_stmt) = else_branch {
                    self.resolve_blueprint_stmt(else_stmt);
                }
            }
        }
    }

    fn resolve_instruction_expr(&mut self, instr: &ast::InstructionExpr) {
        match instr {
            ast::InstructionExpr::Simple(inst) => {
                for (_, expr) in &inst.params {
                    self.resolve_expr(expr);
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
                self.resolve_expr(expr);
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
                    // Resolve included backend
                    self.resolve_name(name, Span::default());
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
            // Validate that the import exists in the registry
            if let Some(export) = self.registry.resolve_import(&import.module, &import.name) {
                // Import the external symbol into the local symbol table
                // This allows type checking to use normal symbol lookup
                self.inner.symbols.define_external(
                    &export.name,
                    export.kind,
                    ScopeId::ROOT,
                    import.span,
                    import.module.clone(),
                );

                // Store the imported name mapping to its module (for diagnostics)
                self.inner.imports.insert(import.name.clone(), import.module.clone());
            } else {
                // Check if the module exists at all
                if self.registry.get(&import.module).is_none() {
                    self.inner.diagnostics.error(
                        format!("module '{}' not found", import.module),
                        import.span,
                    );
                } else {
                    self.inner.diagnostics.error(
                        format!(
                            "'{}' is not exported from module '{}'",
                            import.name, import.module
                        ),
                        import.span,
                    );
                }
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
}
