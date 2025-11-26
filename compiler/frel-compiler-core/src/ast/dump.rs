// DUMP Format Output for Frel AST
//
// This module implements a human-readable DUMP format for the AST.
// The format is indentation-based and suitable for debugging and review.

use super::visitor::FaVisitor;
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
    pub fn dump(file: &FaFile) -> String {
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
    fn type_inline(&self, type_expr: &FaTypeExpr) -> String {
        match type_expr {
            FaTypeExpr::Named(name) => name.clone(),
            FaTypeExpr::Nullable(inner) => format!("{}?", self.type_inline(inner)),
            FaTypeExpr::Ref(inner) => format!("ref {}", self.type_inline(inner)),
            FaTypeExpr::Draft(inner) => format!("draft {}", self.type_inline(inner)),
            FaTypeExpr::Asset(inner) => format!("asset {}", self.type_inline(inner)),
            FaTypeExpr::Blueprint(params) => {
                if params.is_empty() {
                    "blueprint".to_string()
                } else {
                    let p: Vec<_> = params.iter().map(|p| self.type_inline(p)).collect();
                    format!("blueprint<{}>", p.join(", "))
                }
            }
            FaTypeExpr::Accessor(inner) => format!("accessor<{}>", self.type_inline(inner)),
            FaTypeExpr::List(elem) => format!("[{}]", self.type_inline(elem)),
            FaTypeExpr::Set(elem) => format!("set<{}>", self.type_inline(elem)),
            FaTypeExpr::Map(key, value) => {
                format!("map<{}, {}>", self.type_inline(key), self.type_inline(value))
            }
            FaTypeExpr::Tree(elem) => format!("tree<{}>", self.type_inline(elem)),
        }
    }

    // Helper to format an expression inline (for simple expressions)
    fn expr_inline(&self, expr: &FaExpr) -> String {
        match expr {
            FaExpr::Null => "null".to_string(),
            FaExpr::Bool(b) => b.to_string(),
            FaExpr::Int(n) => n.to_string(),
            FaExpr::Float(f) => f.to_string(),
            FaExpr::String(s) => format!("{:?}", s),
            FaExpr::Identifier(name) => name.clone(),
            FaExpr::QualifiedName(parts) => parts.join("."),
            FaExpr::List(items) => {
                let items: Vec<_> = items.iter().map(|i| self.expr_inline(i)).collect();
                format!("[{}]", items.join(", "))
            }
            FaExpr::Object(fields) => {
                let fields: Vec<_> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.expr_inline(v)))
                    .collect();
                format!("{{ {} }}", fields.join(", "))
            }
            FaExpr::Binary { op, left, right } => {
                format!(
                    "{} {} {}",
                    self.expr_inline(left),
                    self.op_str(op),
                    self.expr_inline(right)
                )
            }
            FaExpr::Unary { op, expr } => {
                format!("{}{}", self.unary_op_str(op), self.expr_inline(expr))
            }
            FaExpr::Ternary {
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
            FaExpr::FieldAccess { base, field } => {
                format!("{}.{}", self.expr_inline(base), field)
            }
            FaExpr::OptionalChain { base, field } => {
                format!("{}?.{}", self.expr_inline(base), field)
            }
            FaExpr::Call { callee, args } => {
                let args: Vec<_> = args.iter().map(|a| self.expr_inline(a)).collect();
                format!("{}({})", self.expr_inline(callee), args.join(", "))
            }
            FaExpr::StringTemplate(elems) => {
                let parts: Vec<_> = elems
                    .iter()
                    .map(|e| match e {
                        FaTemplateElement::Text(t) => t.clone(),
                        FaTemplateElement::Interpolation(expr) => {
                            format!("${{{}}}", self.expr_inline(expr))
                        }
                    })
                    .collect();
                format!("\"{}\"", parts.join(""))
            }
        }
    }

    fn op_str(&self, op: &FaBinaryOp) -> &'static str {
        match op {
            FaBinaryOp::Add => "+",
            FaBinaryOp::Sub => "-",
            FaBinaryOp::Mul => "*",
            FaBinaryOp::Div => "/",
            FaBinaryOp::Mod => "%",
            FaBinaryOp::Pow => "**",
            FaBinaryOp::Eq => "==",
            FaBinaryOp::Ne => "!=",
            FaBinaryOp::Lt => "<",
            FaBinaryOp::Le => "<=",
            FaBinaryOp::Gt => ">",
            FaBinaryOp::Ge => ">=",
            FaBinaryOp::And => "&&",
            FaBinaryOp::Or => "||",
            FaBinaryOp::Elvis => "?:",
        }
    }

    fn unary_op_str(&self, op: &FaUnaryOp) -> &'static str {
        match op {
            FaUnaryOp::Not => "!",
            FaUnaryOp::Neg => "-",
            FaUnaryOp::Pos => "+",
        }
    }
}

impl Default for DumpVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl FaVisitor for DumpVisitor {
    type Result = ();

    // =========================================================================
    // File-level
    // =========================================================================

    fn visit_file(&mut self, file: &FaFile) {
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

    fn visit_import(&mut self, import: &FaImport) {
        self.write(&format!("IMPORT {} FROM {}", import.name, import.module));
    }

    // =========================================================================
    // Top-level declarations
    // =========================================================================

    fn visit_top_level_decl(&mut self, decl: &FaTopLevelDecl) {
        match decl {
            FaTopLevelDecl::Blueprint(bp) => self.visit_blueprint(bp),
            FaTopLevelDecl::Backend(be) => self.visit_backend(be),
            FaTopLevelDecl::Contract(ct) => self.visit_contract(ct),
            FaTopLevelDecl::Scheme(sc) => self.visit_scheme(sc),
            FaTopLevelDecl::Enum(en) => self.visit_enum(en),
            FaTopLevelDecl::Theme(th) => self.visit_theme(th),
            FaTopLevelDecl::Arena(ar) => self.visit_arena(ar),
        }
    }

    fn visit_blueprint(&mut self, blueprint: &FaBlueprint) {
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

    fn visit_backend(&mut self, backend: &FaBackend) {
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

    fn visit_contract(&mut self, contract: &FaContract) {
        self.write(&format!("CONTRACT {}", contract.name));
        self.indent();

        for method in &contract.methods {
            self.visit_contract_method(method);
        }

        self.dedent();
    }

    fn visit_scheme(&mut self, scheme: &FaScheme) {
        self.write(&format!("SCHEME {}", scheme.name));
        self.indent();

        for member in &scheme.members {
            self.visit_scheme_member(member);
        }

        self.dedent();
    }

    fn visit_enum(&mut self, enum_decl: &FaEnum) {
        let variants = enum_decl.variants.join(", ");
        self.write(&format!("ENUM {} {{ {} }}", enum_decl.name, variants));
    }

    fn visit_theme(&mut self, theme: &FaTheme) {
        self.write(&format!("THEME {}", theme.name));
        self.indent();

        for member in &theme.members {
            self.visit_theme_member(member);
        }

        self.dedent();
    }

    fn visit_arena(&mut self, arena: &FaArena) {
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

    fn visit_blueprint_stmt(&mut self, stmt: &FaBlueprintStmt) {
        match stmt {
            FaBlueprintStmt::With(name) => {
                self.write(&format!("WITH {}", name));
            }
            FaBlueprintStmt::LocalDecl(decl) => {
                self.visit_local_decl(decl);
            }
            FaBlueprintStmt::FragmentCreation(frag) => {
                self.visit_fragment_creation(frag);
            }
            FaBlueprintStmt::Control(ctrl) => {
                self.visit_control_stmt(ctrl);
            }
            FaBlueprintStmt::Instruction(instr) => {
                self.visit_instruction(instr);
            }
            FaBlueprintStmt::EventHandler(handler) => {
                self.visit_event_handler(handler);
            }
            FaBlueprintStmt::ContentExpr(expr) => {
                self.write(&format!("CONTENT {}", self.expr_inline(expr)));
            }
        }
    }

    fn visit_local_decl(&mut self, decl: &FaLocalDecl) {
        self.write(&format!(
            "LOCAL {} TYPE {} INIT {}",
            decl.name,
            self.type_inline(&decl.type_expr),
            self.expr_inline(&decl.init)
        ));
    }

    fn visit_fragment_creation(&mut self, frag: &FaFragmentCreation) {
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

        for instr in &frag.instructions {
            self.visit_instruction(instr);
        }

        self.dedent();
    }

    fn visit_fragment_body(&mut self, body: &FaFragmentBody) {
        match body {
            FaFragmentBody::Default(stmts) => {
                for stmt in stmts {
                    self.visit_blueprint_stmt(stmt);
                }
            }
            FaFragmentBody::Slots(slots) => {
                for slot in slots {
                    self.visit_slot_binding(slot);
                }
            }
            FaFragmentBody::InlineBlueprint { params, body } => {
                self.write(&format!("INLINE({})", params.join(", ")));
                self.indent();
                for stmt in body {
                    self.visit_blueprint_stmt(stmt);
                }
                self.dedent();
            }
        }
    }

    fn visit_slot_binding(&mut self, binding: &FaSlotBinding) {
        self.write(&format!("SLOT {}", binding.slot_name));
        self.indent();
        self.visit_blueprint_value(&binding.blueprint);
        self.dedent();
    }

    fn visit_blueprint_value(&mut self, value: &FaBlueprintValue) {
        match value {
            FaBlueprintValue::Reference(name) => {
                self.write(&format!("REF {}", name));
            }
            FaBlueprintValue::Inline { params, body } => {
                self.write(&format!("INLINE({})", params.join(", ")));
                self.indent();
                for stmt in body {
                    self.visit_blueprint_stmt(stmt);
                }
                self.dedent();
            }
        }
    }

    fn visit_control_stmt(&mut self, ctrl: &FaControlStmt) {
        match ctrl {
            FaControlStmt::When {
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
            FaControlStmt::Repeat {
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
            FaControlStmt::Select {
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

    fn visit_select_branch(&mut self, branch: &FaSelectBranch) {
        self.write(&format!("CASE {}", self.expr_inline(&branch.condition)));
        self.indent();
        self.visit_blueprint_stmt(&branch.body);
        self.dedent();
    }

    fn visit_instruction(&mut self, instr: &FaInstruction) {
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

    fn visit_event_handler(&mut self, handler: &FaEventHandler) {
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

    fn visit_event_param(&mut self, _param: &FaEventParam) {
        // Handled inline in visit_event_handler
    }

    fn visit_handler_stmt(&mut self, stmt: &FaHandlerStmt) {
        match stmt {
            FaHandlerStmt::Assignment { name, value } => {
                self.write(&format!("{} = {}", name, self.expr_inline(value)));
            }
            FaHandlerStmt::CommandCall { name, args } => {
                let args: Vec<_> = args.iter().map(|a| self.expr_inline(a)).collect();
                self.write(&format!("{}({})", name, args.join(", ")));
            }
        }
    }

    // =========================================================================
    // Backend members
    // =========================================================================

    fn visit_backend_member(&mut self, member: &FaBackendMember) {
        match member {
            FaBackendMember::Include(name) => {
                self.write(&format!("INCLUDE {}", name));
            }
            FaBackendMember::Field(field) => {
                self.visit_field(field);
            }
            FaBackendMember::Method(method) => {
                self.visit_method(method);
            }
            FaBackendMember::Command(command) => {
                self.visit_command(command);
            }
        }
    }

    fn visit_field(&mut self, field: &FaField) {
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

    fn visit_method(&mut self, method: &FaMethod) {
        let params: Vec<_> = method.params.iter().map(|p| self.format_param(p)).collect();
        self.write(&format!(
            "METHOD {}({}) -> {}",
            method.name,
            params.join(", "),
            self.type_inline(&method.return_type)
        ));
    }

    fn visit_command(&mut self, command: &FaCommand) {
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

    fn visit_contract_method(&mut self, method: &FaContractMethod) {
        let params: Vec<_> = method.params.iter().map(|p| self.format_param(p)).collect();
        let ret = method
            .return_type
            .as_ref()
            .map(|t| format!(" -> {}", self.type_inline(t)))
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

    fn visit_scheme_member(&mut self, member: &FaSchemeMember) {
        match member {
            FaSchemeMember::Field(field) => self.visit_scheme_field(field),
            FaSchemeMember::Virtual(vf) => self.visit_virtual_field(vf),
        }
    }

    fn visit_scheme_field(&mut self, field: &FaSchemeField) {
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

    fn visit_virtual_field(&mut self, field: &FaVirtualField) {
        self.write(&format!(
            "VIRTUAL {} TYPE {} = {}",
            field.name,
            self.type_inline(&field.type_expr),
            self.expr_inline(&field.expr)
        ));
    }

    fn visit_field_instruction(&mut self, _instr: &FaFieldInstruction) {
        // Handled inline in visit_scheme_field
    }

    // =========================================================================
    // Theme members
    // =========================================================================

    fn visit_theme_member(&mut self, member: &FaThemeMember) {
        match member {
            FaThemeMember::Include(name) => {
                self.write(&format!("INCLUDE {}", name));
            }
            FaThemeMember::Field(field) => {
                self.visit_theme_field(field);
            }
            FaThemeMember::InstructionSet(set) => {
                self.visit_instruction_set(set);
            }
            FaThemeMember::Variant(variant) => {
                self.visit_theme_variant(variant);
            }
        }
    }

    fn visit_theme_field(&mut self, field: &FaThemeField) {
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

    fn visit_instruction_set(&mut self, set: &FaInstructionSet) {
        self.write(&format!("SET {}", set.name));
        self.indent();
        for instr in &set.instructions {
            self.visit_instruction(instr);
        }
        self.dedent();
    }

    fn visit_theme_variant(&mut self, variant: &FaThemeVariant) {
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

    fn visit_parameter(&mut self, _param: &FaParameter) {
        // Handled inline via format_param
    }

    fn visit_arg(&mut self, _arg: &FaArg) {
        // Handled inline via format_arg
    }

    // =========================================================================
    // Types
    // =========================================================================

    fn visit_type_expr(&mut self, _type_expr: &FaTypeExpr) {
        // Handled inline via type_inline
    }

    // =========================================================================
    // Expressions
    // =========================================================================

    fn visit_expr(&mut self, _expr: &FaExpr) {
        // Handled inline via expr_inline
    }

    fn visit_template_element(&mut self, _elem: &FaTemplateElement) {
        // Handled inline in expr_inline
    }

    fn visit_binary_op(&mut self, _op: &FaBinaryOp) {
        // Handled inline via op_str
    }

    fn visit_unary_op(&mut self, _op: &FaUnaryOp) {
        // Handled inline via unary_op_str
    }
}

impl DumpVisitor {
    fn format_param(&self, param: &FaParameter) -> String {
        let default = param
            .default
            .as_ref()
            .map(|d| format!(" = {}", self.expr_inline(d)))
            .unwrap_or_default();
        format!("{}: {}{}", param.name, self.type_inline(&param.type_expr), default)
    }

    fn format_arg(&self, arg: &FaArg) -> String {
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
        let file = FaFile {
            module: "test".to_string(),
            imports: vec![],
            declarations: vec![],
        };

        let output = DumpVisitor::dump(&file);
        assert!(output.contains("FILE module=test"));
    }

    #[test]
    fn test_dump_enum() {
        let file = FaFile {
            module: "test".to_string(),
            imports: vec![],
            declarations: vec![FaTopLevelDecl::Enum(FaEnum {
                name: "Status".to_string(),
                variants: vec!["Active".to_string(), "Inactive".to_string()],
            })],
        };

        let output = DumpVisitor::dump(&file);
        assert!(output.contains("ENUM Status { Active, Inactive }"));
    }

    #[test]
    fn test_dump_backend_compact() {
        let file = FaFile {
            module: "test".to_string(),
            imports: vec![],
            declarations: vec![FaTopLevelDecl::Backend(FaBackend {
                name: "Counter".to_string(),
                params: vec![],
                members: vec![
                    FaBackendMember::Field(FaField {
                        name: "count".to_string(),
                        type_expr: FaTypeExpr::Named("i32".to_string()),
                        init: Some(FaExpr::Int(0)),
                    }),
                ],
            })],
        };

        let output = DumpVisitor::dump(&file);
        assert!(output.contains("FIELD count TYPE i32 INIT 0"));
    }
}
