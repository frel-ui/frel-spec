// DUMP Format Output for Frel AST
//
// This module implements a human-readable DUMP format for the AST.
// The format is indentation-based and suitable for debugging and review.

use super::visitor::Visitor;
use super::*;

/// A visitor that produces DUMP format output
pub struct DumpVisitor {
    output: String,
    indent: usize,
    indent_str: String,
}

impl DumpVisitor {
    /// Create a new DumpVisitor
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            indent_str: "    ".to_string(),
        }
    }

    /// Dump a file to DUMP format
    pub fn dump(file: &File) -> String {
        let mut visitor = Self::new();
        visitor.visit_file(file);
        visitor.output
    }

    fn write(&mut self, text: &str) {
        for _ in 0..self.indent {
            self.output.push_str(&self.indent_str);
        }
        self.output.push_str(text);
        self.output.push('\n');
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    // Helper to format a type expression inline
    fn type_inline(&self, type_expr: &TypeExpr) -> String {
        match type_expr {
            TypeExpr::Named(name) => name.clone(),
            TypeExpr::Nullable(inner) => format!("{}?", self.type_inline(inner)),
            TypeExpr::Ref(inner) => format!("ref {}", self.type_inline(inner)),
            TypeExpr::Draft(inner) => format!("draft {}", self.type_inline(inner)),
            TypeExpr::Asset(inner) => format!("asset {}", self.type_inline(inner)),
            TypeExpr::Blueprint(params) => {
                if params.is_empty() {
                    "Blueprint".to_string()
                } else {
                    let p: Vec<_> = params.iter().map(|p| self.type_inline(p)).collect();
                    format!("Blueprint<{}>", p.join(", "))
                }
            }
            TypeExpr::Accessor(inner) => format!("Accessor<{}>", self.type_inline(inner)),
            TypeExpr::List(elem) => format!("List<{}>", self.type_inline(elem)),
            TypeExpr::Set(elem) => format!("Set<{}>", self.type_inline(elem)),
            TypeExpr::Map(key, value) => {
                format!("Map<{}, {}>", self.type_inline(key), self.type_inline(value))
            }
            TypeExpr::Tree(elem) => format!("Tree<{}>", self.type_inline(elem)),
        }
    }

    // Helper to format an expression inline (for simple expressions)
    fn expr_inline(&self, expr: &Expr) -> String {
        match expr {
            Expr::Null => "null".to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Int(n) => n.to_string(),
            Expr::Float(f) => f.to_string(),
            Expr::Color(c) => format!("#{:08X}", c),
            Expr::String(s) => format!("{:?}", s),
            Expr::Identifier(name) => name.clone(),
            Expr::QualifiedName(parts) => parts.join("."),
            Expr::List(items) => {
                let items: Vec<_> = items.iter().map(|i| self.expr_inline(i)).collect();
                format!("[{}]", items.join(", "))
            }
            Expr::Object(fields) => {
                let fields: Vec<_> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.expr_inline(v)))
                    .collect();
                format!("{{ {} }}", fields.join(", "))
            }
            Expr::Binary { op, left, right } => {
                format!(
                    "{} {} {}",
                    self.expr_inline(left),
                    self.op_str(op),
                    self.expr_inline(right)
                )
            }
            Expr::Unary { op, expr } => {
                format!("{}{}", self.unary_op_str(op), self.expr_inline(expr))
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                format!(
                    "{} ? {} : {}",
                    self.expr_inline(condition),
                    self.expr_inline(then_expr),
                    self.expr_inline(else_expr)
                )
            }
            Expr::FieldAccess { base, field } => {
                format!("{}.{}", self.expr_inline(base), field)
            }
            Expr::OptionalChain { base, field } => {
                format!("{}?.{}", self.expr_inline(base), field)
            }
            Expr::Call { callee, args } => {
                let args: Vec<_> = args.iter().map(|a| self.expr_inline(a)).collect();
                format!("{}({})", self.expr_inline(callee), args.join(", "))
            }
            Expr::StringTemplate(elems) => {
                let parts: Vec<_> = elems
                    .iter()
                    .map(|e| match e {
                        TemplateElement::Text(t) => t.clone(),
                        TemplateElement::Interpolation(expr) => {
                            format!("${{{}}}", self.expr_inline(expr))
                        }
                    })
                    .collect();
                format!("\"{}\"", parts.join(""))
            }
        }
    }

    fn op_str(&self, op: &BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Pow => "**",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::Elvis => "?:",
        }
    }

    fn unary_op_str(&self, op: &UnaryOp) -> &'static str {
        match op {
            UnaryOp::Not => "!",
            UnaryOp::Neg => "-",
            UnaryOp::Pos => "+",
        }
    }
}

impl Default for DumpVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Visitor for DumpVisitor {
    type Result = ();

    // =========================================================================
    // File-level
    // =========================================================================

    fn visit_file(&mut self, file: &File) {
        self.write(&format!("FILE module={}", file.module));
        self.indent();

        for import in &file.imports {
            self.visit_import(import);
        }

        for decl in &file.declarations {
            self.visit_top_level_decl(decl);
        }

        self.dedent();
    }

    fn visit_import(&mut self, import: &Import) {
        self.write(&format!("IMPORT {} FROM {}", import.name, import.module));
    }

    // =========================================================================
    // Top-level declarations
    // =========================================================================

    fn visit_top_level_decl(&mut self, decl: &TopLevelDecl) {
        match decl {
            TopLevelDecl::Blueprint(bp) => self.visit_blueprint(bp),
            TopLevelDecl::Backend(be) => self.visit_backend(be),
            TopLevelDecl::Contract(ct) => self.visit_contract(ct),
            TopLevelDecl::Scheme(sc) => self.visit_scheme(sc),
            TopLevelDecl::Enum(en) => self.visit_enum(en),
            TopLevelDecl::Theme(th) => self.visit_theme(th),
            TopLevelDecl::Arena(ar) => self.visit_arena(ar),
        }
    }

    fn visit_blueprint(&mut self, blueprint: &Blueprint) {
        let params = if blueprint.params.is_empty() {
            String::new()
        } else {
            let p: Vec<_> = blueprint
                .params
                .iter()
                .map(|p| self.format_param(p))
                .collect();
            format!("({})", p.join(", "))
        };
        self.write(&format!("BLUEPRINT {}{}", blueprint.name, params));
        self.indent();

        for stmt in &blueprint.body {
            self.visit_blueprint_stmt(stmt);
        }

        self.dedent();
    }

    fn visit_backend(&mut self, backend: &Backend) {
        let params = if backend.params.is_empty() {
            String::new()
        } else {
            let p: Vec<_> = backend
                .params
                .iter()
                .map(|p| self.format_param(p))
                .collect();
            format!("({})", p.join(", "))
        };
        self.write(&format!("BACKEND {}{}", backend.name, params));
        self.indent();

        for member in &backend.members {
            self.visit_backend_member(member);
        }

        self.dedent();
    }

    fn visit_contract(&mut self, contract: &Contract) {
        self.write(&format!("CONTRACT {}", contract.name));
        self.indent();

        for method in &contract.methods {
            self.visit_contract_method(method);
        }

        self.dedent();
    }

    fn visit_scheme(&mut self, scheme: &Scheme) {
        self.write(&format!("SCHEME {}", scheme.name));
        self.indent();

        for member in &scheme.members {
            self.visit_scheme_member(member);
        }

        self.dedent();
    }

    fn visit_enum(&mut self, enum_decl: &Enum) {
        let variants = enum_decl.variants.join(", ");
        self.write(&format!("ENUM {} {{ {} }}", enum_decl.name, variants));
    }

    fn visit_theme(&mut self, theme: &Theme) {
        self.write(&format!("THEME {}", theme.name));
        self.indent();

        for member in &theme.members {
            self.visit_theme_member(member);
        }

        self.dedent();
    }

    fn visit_arena(&mut self, arena: &Arena) {
        let contract = arena
            .contract
            .as_ref()
            .map(|c| format!(" WITH {}", c))
            .unwrap_or_default();
        self.write(&format!(
            "ARENA {} FOR {}{}",
            arena.name, arena.scheme_name, contract
        ));
    }

    // =========================================================================
    // Blueprint members
    // =========================================================================

    fn visit_blueprint_stmt(&mut self, stmt: &BlueprintStmt) {
        match stmt {
            BlueprintStmt::With(name) => {
                self.write(&format!("WITH {}", name));
            }
            BlueprintStmt::LocalDecl(decl) => {
                self.visit_local_decl(decl);
            }
            BlueprintStmt::FragmentCreation(frag) => {
                self.visit_fragment_creation(frag);
            }
            BlueprintStmt::Control(ctrl) => {
                self.visit_control_stmt(ctrl);
            }
            BlueprintStmt::Instruction(instr) => {
                self.visit_instruction_expr(instr);
            }
            BlueprintStmt::EventHandler(handler) => {
                self.visit_event_handler(handler);
            }
            BlueprintStmt::ContentExpr(expr) => {
                self.write(&format!("CONTENT {}", self.expr_inline(expr)));
            }
        }
    }

    fn visit_local_decl(&mut self, decl: &LocalDecl) {
        self.write(&format!(
            "LOCAL {} TYPE {} INIT {}",
            decl.name,
            self.type_inline(&decl.type_expr),
            self.expr_inline(&decl.init)
        ));
    }

    fn visit_fragment_creation(&mut self, frag: &FragmentCreation) {
        let name = if frag.name.is_empty() {
            "BLOCK".to_string()
        } else {
            format!("FRAGMENT {}", frag.name)
        };

        let args = if frag.args.is_empty() {
            String::new()
        } else {
            let a: Vec<_> = frag.args.iter().map(|a| self.format_arg(a)).collect();
            format!(" ARGS({})", a.join(", "))
        };

        self.write(&format!("{}{}", name, args));
        self.indent();

        if let Some(body) = &frag.body {
            self.visit_fragment_body(body);
        }

        for item in &frag.postfix {
            self.visit_postfix_item(item);
        }

        self.dedent();
    }

    fn visit_postfix_item(&mut self, item: &PostfixItem) {
        match item {
            PostfixItem::Instruction(instr) => self.visit_instruction_expr(instr),
            PostfixItem::EventHandler(handler) => self.visit_event_handler(handler),
        }
    }

    fn visit_fragment_body(&mut self, body: &FragmentBody) {
        match body {
            FragmentBody::Default(stmts) => {
                for stmt in stmts {
                    self.visit_blueprint_stmt(stmt);
                }
            }
            FragmentBody::Slots(slots) => {
                for slot in slots {
                    self.visit_slot_binding(slot);
                }
            }
            FragmentBody::InlineBlueprint { params, body } => {
                self.write(&format!("INLINE({})", params.join(", ")));
                self.indent();
                for stmt in body {
                    self.visit_blueprint_stmt(stmt);
                }
                self.dedent();
            }
        }
    }

    fn visit_slot_binding(&mut self, binding: &SlotBinding) {
        self.write(&format!("SLOT {}", binding.slot_name));
        self.indent();
        self.visit_blueprint_value(&binding.blueprint);
        self.dedent();
    }

    fn visit_blueprint_value(&mut self, value: &BlueprintValue) {
        match value {
            BlueprintValue::Reference(name) => {
                self.write(&format!("REF {}", name));
            }
            BlueprintValue::Inline { params, body } => {
                self.write(&format!("INLINE({})", params.join(", ")));
                self.indent();
                for stmt in body {
                    self.visit_blueprint_stmt(stmt);
                }
                self.dedent();
            }
        }
    }

    fn visit_control_stmt(&mut self, ctrl: &ControlStmt) {
        match ctrl {
            ControlStmt::When {
                condition,
                then_stmt,
                else_stmt,
            } => {
                self.write(&format!("WHEN {}", self.expr_inline(condition)));
                self.indent();
                self.visit_blueprint_stmt(then_stmt);
                self.dedent();
                if let Some(else_s) = else_stmt {
                    self.write("ELSE");
                    self.indent();
                    self.visit_blueprint_stmt(else_s);
                    self.dedent();
                }
            }
            ControlStmt::Repeat {
                iterable,
                item_name,
                key_expr,
                body,
            } => {
                let item = item_name.as_deref().unwrap_or("it");
                let key = key_expr
                    .as_ref()
                    .map(|k| format!(" BY {}", self.expr_inline(k)))
                    .unwrap_or_default();
                self.write(&format!(
                    "REPEAT {} ON {}{}",
                    item,
                    self.expr_inline(iterable),
                    key
                ));
                self.indent();
                self.visit_blueprint_stmt(body);
                self.dedent();
            }
            ControlStmt::Select {
                discriminant,
                branches,
                else_branch,
            } => {
                let on = discriminant
                    .as_ref()
                    .map(|d| format!(" ON {}", self.expr_inline(d)))
                    .unwrap_or_default();
                self.write(&format!("SELECT{}", on));
                self.indent();
                for branch in branches {
                    self.visit_select_branch(branch);
                }
                if let Some(else_b) = else_branch {
                    self.write("ELSE");
                    self.indent();
                    self.visit_blueprint_stmt(else_b);
                    self.dedent();
                }
                self.dedent();
            }
        }
    }

    fn visit_select_branch(&mut self, branch: &SelectBranch) {
        self.write(&format!("CASE {}", self.expr_inline(&branch.condition)));
        self.indent();
        self.visit_blueprint_stmt(&branch.body);
        self.dedent();
    }

    fn visit_instruction_expr(&mut self, instr: &InstructionExpr) {
        match instr {
            InstructionExpr::Simple(simple) => self.visit_instruction(simple),
            InstructionExpr::When {
                condition,
                then_instr,
                else_instr,
            } => {
                self.write(&format!("WHEN {}", self.expr_inline(condition)));
                self.indent();
                self.visit_instruction_expr(then_instr);
                self.dedent();
                if let Some(else_instr) = else_instr {
                    self.write("ELSE");
                    self.indent();
                    self.visit_instruction_expr(else_instr);
                    self.dedent();
                }
            }
            InstructionExpr::Ternary {
                condition,
                then_instr,
                else_instr,
            } => {
                self.write(&format!("TERNARY {}", self.expr_inline(condition)));
                self.indent();
                self.write("THEN");
                self.indent();
                self.visit_instruction_expr(then_instr);
                self.dedent();
                self.write("ELSE");
                self.indent();
                self.visit_instruction_expr(else_instr);
                self.dedent();
                self.dedent();
            }
            InstructionExpr::Reference(expr) => {
                self.write(&format!("INSTR_REF {}", self.expr_inline(expr)));
            }
        }
    }

    fn visit_instruction(&mut self, instr: &Instruction) {
        if instr.params.is_empty() {
            self.write(&format!("INSTR {}", instr.name));
        } else {
            let params: Vec<_> = instr
                .params
                .iter()
                .map(|(k, v)| format!("{}: {}", k, self.expr_inline(v)))
                .collect();
            self.write(&format!("INSTR {} {{ {} }}", instr.name, params.join(", ")));
        }
    }

    fn visit_event_handler(&mut self, handler: &EventHandler) {
        let param = handler
            .param
            .as_ref()
            .map(|p| {
                let t = p
                    .type_expr
                    .as_ref()
                    .map(|t| format!(": {}", self.type_inline(t)))
                    .unwrap_or_default();
                format!("({}{})", p.name, t)
            })
            .unwrap_or_default();
        self.write(&format!("ON {}{}", handler.event_name, param));
        self.indent();

        for stmt in &handler.body {
            self.visit_handler_stmt(stmt);
        }

        self.dedent();
    }

    fn visit_event_param(&mut self, _param: &EventParam) {
        // Handled inline in visit_event_handler
    }

    fn visit_handler_stmt(&mut self, stmt: &HandlerStmt) {
        match stmt {
            HandlerStmt::Assignment { name, value } => {
                self.write(&format!("{} = {}", name, self.expr_inline(value)));
            }
            HandlerStmt::CommandCall { name, args } => {
                let args: Vec<_> = args.iter().map(|a| self.expr_inline(a)).collect();
                self.write(&format!("{}({})", name, args.join(", ")));
            }
        }
    }

    // =========================================================================
    // Backend members
    // =========================================================================

    fn visit_backend_member(&mut self, member: &BackendMember) {
        match member {
            BackendMember::Include(name) => {
                self.write(&format!("INCLUDE {}", name));
            }
            BackendMember::Field(field) => {
                self.visit_field(field);
            }
            BackendMember::Method(method) => {
                self.visit_method(method);
            }
            BackendMember::Command(command) => {
                self.visit_command(command);
            }
        }
    }

    fn visit_field(&mut self, field: &Field) {
        let init = field
            .init
            .as_ref()
            .map(|e| format!(" INIT {}", self.expr_inline(e)))
            .unwrap_or_default();
        self.write(&format!(
            "FIELD {} TYPE {}{}",
            field.name,
            self.type_inline(&field.type_expr),
            init
        ));
    }

    fn visit_method(&mut self, method: &Method) {
        let params: Vec<_> = method.params.iter().map(|p| self.format_param(p)).collect();
        self.write(&format!(
            "METHOD {}({}) RETURN {}",
            method.name,
            params.join(", "),
            self.type_inline(&method.return_type)
        ));
    }

    fn visit_command(&mut self, command: &Command) {
        let params: Vec<_> = command
            .params
            .iter()
            .map(|p| self.format_param(p))
            .collect();
        self.write(&format!("COMMAND {}({})", command.name, params.join(", ")));
    }

    // =========================================================================
    // Contract members
    // =========================================================================

    fn visit_contract_method(&mut self, method: &ContractMethod) {
        let params: Vec<_> = method.params.iter().map(|p| self.format_param(p)).collect();
        let ret = method
            .return_type
            .as_ref()
            .map(|t| format!(" RETURN {}", self.type_inline(t)))
            .unwrap_or_default();
        self.write(&format!(
            "METHOD {}({}){}",
            method.name,
            params.join(", "),
            ret
        ));
    }

    // =========================================================================
    // Scheme members
    // =========================================================================

    fn visit_scheme_member(&mut self, member: &SchemeMember) {
        match member {
            SchemeMember::Field(field) => self.visit_scheme_field(field),
            SchemeMember::Virtual(vf) => self.visit_virtual_field(vf),
        }
    }

    fn visit_scheme_field(&mut self, field: &SchemeField) {
        let instrs = if field.instructions.is_empty() {
            String::new()
        } else {
            let i: Vec<_> = field
                .instructions
                .iter()
                .map(|i| {
                    if let Some(v) = &i.value {
                        format!("{}({})", i.name, self.expr_inline(v))
                    } else {
                        i.name.clone()
                    }
                })
                .collect();
            format!(" [{}]", i.join(", "))
        };
        self.write(&format!(
            "FIELD {} TYPE {}{}",
            field.name,
            self.type_inline(&field.type_expr),
            instrs
        ));
    }

    fn visit_virtual_field(&mut self, field: &VirtualField) {
        self.write(&format!(
            "VIRTUAL {} TYPE {} = {}",
            field.name,
            self.type_inline(&field.type_expr),
            self.expr_inline(&field.expr)
        ));
    }

    fn visit_field_instruction(&mut self, _instr: &FieldInstruction) {
        // Handled inline in visit_scheme_field
    }

    // =========================================================================
    // Theme members
    // =========================================================================

    fn visit_theme_member(&mut self, member: &ThemeMember) {
        match member {
            ThemeMember::Include(name) => {
                self.write(&format!("INCLUDE {}", name));
            }
            ThemeMember::Field(field) => {
                self.visit_theme_field(field);
            }
            ThemeMember::InstructionSet(set) => {
                self.visit_instruction_set(set);
            }
            ThemeMember::Variant(variant) => {
                self.visit_theme_variant(variant);
            }
        }
    }

    fn visit_theme_field(&mut self, field: &ThemeField) {
        let asset = if field.is_asset { "ASSET " } else { "" };
        let init = field
            .init
            .as_ref()
            .map(|e| format!(" = {}", self.expr_inline(e)))
            .unwrap_or_default();
        self.write(&format!(
            "{}FIELD {} TYPE {}{}",
            asset,
            field.name,
            self.type_inline(&field.type_expr),
            init
        ));
    }

    fn visit_instruction_set(&mut self, set: &InstructionSet) {
        self.write(&format!("SET {}", set.name));
        self.indent();
        for instr in &set.instructions {
            self.visit_instruction(instr);
        }
        self.dedent();
    }

    fn visit_theme_variant(&mut self, variant: &ThemeVariant) {
        self.write(&format!("VARIANT {}", variant.name));
        self.indent();
        for (name, value) in &variant.overrides {
            self.write(&format!("{} = {}", name, self.expr_inline(value)));
        }
        self.dedent();
    }

    // =========================================================================
    // Common elements
    // =========================================================================

    fn visit_parameter(&mut self, _param: &Parameter) {
        // Handled inline via format_param
    }

    fn visit_arg(&mut self, _arg: &Arg) {
        // Handled inline via format_arg
    }

    // =========================================================================
    // Types
    // =========================================================================

    fn visit_type_expr(&mut self, _type_expr: &TypeExpr) {
        // Handled inline via type_inline
    }

    // =========================================================================
    // Expressions
    // =========================================================================

    fn visit_expr(&mut self, _expr: &Expr) {
        // Handled inline via expr_inline
    }

    fn visit_template_element(&mut self, _elem: &TemplateElement) {
        // Handled inline in expr_inline
    }

    fn visit_binary_op(&mut self, _op: &BinaryOp) {
        // Handled inline via op_str
    }

    fn visit_unary_op(&mut self, _op: &UnaryOp) {
        // Handled inline via unary_op_str
    }
}

impl DumpVisitor {
    fn format_param(&self, param: &Parameter) -> String {
        let default = param
            .default
            .as_ref()
            .map(|d| format!(" = {}", self.expr_inline(d)))
            .unwrap_or_default();
        format!("{}: {}{}", param.name, self.type_inline(&param.type_expr), default)
    }

    fn format_arg(&self, arg: &Arg) -> String {
        if let Some(name) = &arg.name {
            format!("{}: {}", name, self.expr_inline(&arg.value))
        } else {
            self.expr_inline(&arg.value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dump_simple_file() {
        let file = File {
            module: "test".to_string(),
            imports: vec![],
            declarations: vec![],
        };

        let output = DumpVisitor::dump(&file);
        assert!(output.contains("FILE module=test"));
    }

    #[test]
    fn test_dump_enum() {
        let file = File {
            module: "test".to_string(),
            imports: vec![],
            declarations: vec![TopLevelDecl::Enum(Enum {
                name: "Status".to_string(),
                variants: vec!["Active".to_string(), "Inactive".to_string()],
            })],
        };

        let output = DumpVisitor::dump(&file);
        assert!(output.contains("ENUM Status { Active, Inactive }"));
    }

    #[test]
    fn test_dump_backend_compact() {
        let file = File {
            module: "test".to_string(),
            imports: vec![],
            declarations: vec![TopLevelDecl::Backend(Backend {
                name: "Counter".to_string(),
                params: vec![],
                members: vec![
                    BackendMember::Field(Field {
                        name: "count".to_string(),
                        type_expr: TypeExpr::Named("i32".to_string()),
                        init: Some(Expr::Int(0)),
                    }),
                ],
            })],
        };

        let output = DumpVisitor::dump(&file);
        assert!(output.contains("FIELD count TYPE i32 INIT 0"));
    }
}
