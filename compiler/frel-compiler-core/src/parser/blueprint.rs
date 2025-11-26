// Blueprint parser for Frel
//
// Handles parsing of:
// - Blueprint declarations
// - Blueprint statements (with, local decl, fragment creation, control, instructions, events)
// - Fragment creation with slots
// - Control statements (when, repeat, select)
// - Event handlers

use crate::ast::{
    FaArg, FaBlueprint, FaBlueprintStmt, FaBlueprintValue, FaControlStmt, FaEventHandler,
    FaEventParam, FaFragmentBody, FaFragmentCreation, FaHandlerStmt, FaLocalDecl, FaPostfixItem,
    FaSelectBranch, FaSlotBinding,
};
use crate::lexer::TokenKind;

use super::Parser;

// Note: Instructions are now distinguished syntactically by the `..` prefix,
// not by a known-names whitelist. This makes the grammar context-free.
// Example: `.. width { 300 }` is an instruction, `text { "hello" }` is fragment creation.

impl<'a> Parser<'a> {
    /// Parse blueprint declaration
    pub(super) fn parse_blueprint(&mut self) -> Option<FaBlueprint> {
        self.expect(TokenKind::Blueprint)?;
        let name = self.expect_identifier()?;
        let params = self.parse_param_list_opt()?;
        self.expect(TokenKind::LBrace)?;

        let body = self.parse_blueprint_body()?;

        self.expect(TokenKind::RBrace)?;

        Some(FaBlueprint { name, params, body })
    }

    /// Parse blueprint body (list of statements)
    fn parse_blueprint_body(&mut self) -> Option<Vec<FaBlueprintStmt>> {
        let mut stmts = Vec::new();

        while !self.check(TokenKind::RBrace) && !self.at_end() {
            if let Some(stmt) = self.parse_blueprint_stmt() {
                stmts.push(stmt);
            } else {
                // Error recovery: skip to next statement
                self.advance();
            }
        }

        Some(stmts)
    }

    /// Parse a single blueprint statement
    fn parse_blueprint_stmt(&mut self) -> Option<FaBlueprintStmt> {
        match self.current_kind() {
            // With statement: with BackendName
            TokenKind::With => {
                self.advance();
                let name = self.expect_identifier()?;
                // Optional constructor args
                if self.check(TokenKind::LParen) {
                    // TODO: Parse backend args if needed
                    self.parse_arg_list()?;
                }
                Some(FaBlueprintStmt::With(name))
            }

            // Control statements
            TokenKind::When => self.parse_when_stmt(),
            TokenKind::Repeat => self.parse_repeat_stmt(),
            TokenKind::Select => self.parse_select_stmt(),

            // Event handlers: on_click, on_input, etc.
            TokenKind::Identifier if self.is_event_handler_start() => self.parse_event_handler(),

            // Local declaration: name : type = expr
            TokenKind::Identifier if self.is_local_decl_start() => {
                let name = self.expect_identifier()?;
                self.expect(TokenKind::Colon)?;
                let type_expr = self.parse_type_expr()?;
                self.expect(TokenKind::Eq)?;
                let init = self.parse_expr()?;
                Some(FaBlueprintStmt::LocalDecl(FaLocalDecl {
                    name,
                    type_expr,
                    init,
                }))
            }

            // Fragment creation or expression
            // Note: Instructions require `..` prefix (handled above)
            TokenKind::Identifier => {
                // Check if this is a fragment creation (followed by (, {, or ..)
                // vs an expression (everything else, including bare identifiers)
                if self.is_fragment_creation_start() {
                    self.parse_fragment_creation()
                } else {
                    let expr = self.parse_expr()?;
                    Some(FaBlueprintStmt::ContentExpr(expr))
                }
            }

            // Postfix item: .. instruction or .. on_click { ... }
            TokenKind::DotDot => {
                self.advance();
                // Check if this is an event handler (on_*)
                if self.is_event_handler_start() {
                    let handler = self.parse_postfix_event_handler()?;
                    Some(FaBlueprintStmt::EventHandler(handler))
                } else {
                    let instr = self.parse_instruction()?;
                    Some(FaBlueprintStmt::Instruction(instr))
                }
            }

            // Content expressions: string literals, numbers, colors, etc.
            TokenKind::StringLiteral
            | TokenKind::StringTemplateStart
            | TokenKind::IntLiteral
            | TokenKind::FloatLiteral
            | TokenKind::ColorLiteral
            | TokenKind::True
            | TokenKind::False
            | TokenKind::Null
            | TokenKind::LBracket => {
                let expr = self.parse_expr()?;
                Some(FaBlueprintStmt::ContentExpr(expr))
            }

            // Block statement: { ... } - wraps multiple statements
            // This appears in control structures like `when condition { ... }`
            TokenKind::LBrace => {
                // Create an anonymous/inline fragment to hold the statements
                self.advance();
                let body = self.parse_blueprint_body()?;
                self.expect(TokenKind::RBrace)?;
                // Wrap in FragmentCreation with empty name to represent a block
                Some(FaBlueprintStmt::FragmentCreation(FaFragmentCreation {
                    name: String::new(), // anonymous block
                    args: vec![],
                    body: Some(FaFragmentBody::Default(body)),
                    postfix: vec![],
                }))
            }

            // Parenthesized expression
            TokenKind::LParen => {
                let expr = self.parse_expr()?;
                Some(FaBlueprintStmt::ContentExpr(expr))
            }

            _ => {
                self.error_expected("blueprint statement");
                None
            }
        }
    }

    /// Check if current position is start of a local declaration
    fn is_local_decl_start(&self) -> bool {
        // identifier : type = ...
        if !self.check(TokenKind::Identifier) {
            return false;
        }
        if let Some(next) = self.peek() {
            next.kind == TokenKind::Colon
        } else {
            false
        }
    }

    /// Check if current position is start of an event handler
    fn is_event_handler_start(&self) -> bool {
        let text = self.current_text();
        text.starts_with("on_")
    }

    /// Check if current identifier starts a fragment creation
    /// Fragment creations are followed by (, {, or ..
    /// Bare identifiers without these are treated as expressions (variable references)
    fn is_fragment_creation_start(&self) -> bool {
        if let Some(next) = self.peek() {
            matches!(
                next.kind,
                TokenKind::LParen | TokenKind::LBrace | TokenKind::DotDot
            )
        } else {
            false
        }
    }

    /// Parse fragment creation
    /// Fragment creations require at least one of: args (), body {}, or postfix ..
    fn parse_fragment_creation(&mut self) -> Option<FaBlueprintStmt> {
        let name = self.expect_identifier()?;

        // Check what follows
        match self.current_kind() {
            // Fragment with args and/or body: foo(args) { body }
            TokenKind::LParen | TokenKind::LBrace => {
                let args = if self.check(TokenKind::LParen) {
                    self.parse_arg_list()?
                } else {
                    vec![]
                };

                let body = if self.check(TokenKind::LBrace) {
                    Some(self.parse_fragment_body()?)
                } else {
                    None
                };

                // Parse postfix items (instructions or event handlers)
                let postfix = self.parse_postfix_items()?;

                Some(FaBlueprintStmt::FragmentCreation(FaFragmentCreation {
                    name,
                    args,
                    body,
                    postfix,
                }))
            }

            // Fragment with just postfix items: foo .. width { 100 } .. on_click { ... }
            TokenKind::DotDot => {
                let postfix = self.parse_postfix_items()?;
                Some(FaBlueprintStmt::FragmentCreation(FaFragmentCreation {
                    name,
                    args: vec![],
                    body: None,
                    postfix,
                }))
            }

            // This shouldn't happen since is_fragment_creation_start() checks for (, {, or ..
            // but keep as fallback for safety
            _ => {
                // Treat as expression instead
                Some(FaBlueprintStmt::ContentExpr(crate::ast::FaExpr::Identifier(name)))
            }
        }
    }

    /// Parse fragment body (default, slot-based, or inline blueprint with params)
    fn parse_fragment_body(&mut self) -> Option<FaFragmentBody> {
        self.expect(TokenKind::LBrace)?;

        // Check if it's a slot-based body: { at slotName: ... }
        if self.check(TokenKind::At) {
            let slots = self.parse_slot_bindings()?;
            self.expect(TokenKind::RBrace)?;
            Some(FaFragmentBody::Slots(slots))
        } else if self.check(TokenKind::Identifier) && self.has_arrow_after_params() {
            // Inline blueprint with parameters: { param -> body } or { p1, p2 -> body }
            // This is used when passing an inline blueprint as fragment content
            let mut params = vec![self.expect_identifier()?];
            while self.consume(TokenKind::Comma).is_some() {
                params.push(self.expect_identifier()?);
            }
            self.expect(TokenKind::Arrow)?;
            let body = self.parse_blueprint_body()?;
            self.expect(TokenKind::RBrace)?;
            Some(FaFragmentBody::InlineBlueprint { params, body })
        } else {
            // Default body: regular blueprint statements
            let body = self.parse_blueprint_body()?;
            self.expect(TokenKind::RBrace)?;
            Some(FaFragmentBody::Default(body))
        }
    }

    /// Parse slot bindings: at slot1: value, at slot2: value
    fn parse_slot_bindings(&mut self) -> Option<Vec<FaSlotBinding>> {
        let mut bindings = Vec::new();

        while self.check(TokenKind::At) {
            self.advance();
            let slot_name = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;
            let blueprint = self.parse_blueprint_value()?;
            bindings.push(FaSlotBinding {
                slot_name,
                blueprint,
            });

            // Slots can be separated by comma or newline
            if !self.consume(TokenKind::Comma).is_some() {
                // Allow newline as separator (already skipped by advance)
            }
        }

        Some(bindings)
    }

    /// Parse blueprint value (inline blueprint or reference)
    fn parse_blueprint_value(&mut self) -> Option<FaBlueprintValue> {
        // Check for inline blueprint: { [params ->] body }
        if self.check(TokenKind::LBrace) {
            self.advance();

            // Check if this is { param -> body } or { param1, param2 -> body }
            // We need to look ahead to see if there's an arrow after identifier(s)
            if self.check(TokenKind::Identifier) && self.has_arrow_after_params() {
                // Parse parameters: param or param1, param2, ...
                let mut params = vec![self.expect_identifier()?];
                while self.consume(TokenKind::Comma).is_some() {
                    params.push(self.expect_identifier()?);
                }
                self.expect(TokenKind::Arrow)?;
                let body = self.parse_blueprint_body()?;
                self.expect(TokenKind::RBrace)?;
                Some(FaBlueprintValue::Inline { params, body })
            } else {
                // No params, just body
                let body = self.parse_blueprint_body()?;
                self.expect(TokenKind::RBrace)?;
                Some(FaBlueprintValue::Inline {
                    params: vec![],
                    body,
                })
            }
        } else if self.check(TokenKind::Identifier) {
            // Blueprint reference
            let name = self.expect_identifier()?;
            Some(FaBlueprintValue::Reference(name))
        } else {
            self.error_expected("blueprint value");
            None
        }
    }

    /// Check if there's an arrow after a sequence of comma-separated identifiers
    /// Used to distinguish { param -> body } from { identifier ... }
    fn has_arrow_after_params(&self) -> bool {
        let mut offset = 1; // Start after current identifier

        loop {
            match self.peek_n(offset) {
                Some(t) if t.kind == TokenKind::Arrow => return true,
                Some(t) if t.kind == TokenKind::Comma => {
                    // Expect identifier after comma
                    offset += 1;
                    match self.peek_n(offset) {
                        Some(t) if t.kind == TokenKind::Identifier => {
                            offset += 1;
                            // Continue looking for more commas or arrow
                        }
                        _ => return false,
                    }
                }
                _ => return false,
            }
        }
    }

    /// Parse postfix items (instructions or event handlers): .. instr1 .. on_click { ... }
    fn parse_postfix_items(&mut self) -> Option<Vec<FaPostfixItem>> {
        let mut items = Vec::new();

        while self.consume(TokenKind::DotDot).is_some() {
            // Check if this is an event handler (on_*)
            if self.is_event_handler_start() {
                let handler = self.parse_postfix_event_handler()?;
                items.push(FaPostfixItem::EventHandler(handler));
            } else {
                let instr = self.parse_instruction()?;
                items.push(FaPostfixItem::Instruction(instr));
            }
        }

        Some(items)
    }

    /// Parse event handler in postfix position: on_click { ... }
    fn parse_postfix_event_handler(&mut self) -> Option<FaEventHandler> {
        let event_name = self.expect_identifier()?;

        self.expect(TokenKind::LBrace)?;

        let mut body = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.at_end() {
            if let Some(stmt) = self.parse_handler_stmt() {
                body.push(stmt);
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;

        Some(FaEventHandler {
            event_name,
            param: None, // Postfix event handlers don't have params
            body,
        })
    }

    /// Parse argument list
    fn parse_arg_list(&mut self) -> Option<Vec<FaArg>> {
        self.expect(TokenKind::LParen)?;

        if self.check(TokenKind::RParen) {
            self.advance();
            return Some(vec![]);
        }

        let mut args = vec![self.parse_arg()?];

        while self.consume(TokenKind::Comma).is_some() {
            if self.check(TokenKind::RParen) {
                break;
            }
            args.push(self.parse_arg()?);
        }

        self.expect(TokenKind::RParen)?;
        Some(args)
    }

    /// Parse a single argument (named or positional)
    fn parse_arg(&mut self) -> Option<FaArg> {
        // Check for named argument: name = value
        if self.check(TokenKind::Identifier) {
            if let Some(next) = self.peek() {
                if next.kind == TokenKind::Eq {
                    let name = self.expect_identifier()?;
                    self.advance(); // consume '='
                    let value = self.parse_expr()?;
                    return Some(FaArg {
                        name: Some(name),
                        value,
                    });
                }
            }
        }

        // Positional argument
        let value = self.parse_expr()?;
        Some(FaArg { name: None, value })
    }

    // =========================================================================
    // Control statements
    // =========================================================================

    /// Parse when statement: when condition stmt [else stmt]
    fn parse_when_stmt(&mut self) -> Option<FaBlueprintStmt> {
        self.expect(TokenKind::When)?;
        let condition = self.parse_expr()?;
        let then_stmt = Box::new(self.parse_blueprint_stmt()?);

        let else_stmt = if self.consume(TokenKind::Else).is_some() {
            Some(Box::new(self.parse_blueprint_stmt()?))
        } else {
            None
        };

        Some(FaBlueprintStmt::Control(FaControlStmt::When {
            condition,
            then_stmt,
            else_stmt,
        }))
    }

    /// Parse repeat statement: repeat on expr [as name] [by keyExpr] stmt
    fn parse_repeat_stmt(&mut self) -> Option<FaBlueprintStmt> {
        self.expect(TokenKind::Repeat)?;
        self.expect(TokenKind::On)?;
        let iterable = self.parse_expr()?;

        let item_name = if self.consume(TokenKind::As).is_some() {
            Some(self.expect_identifier()?)
        } else {
            None
        };

        let key_expr = if self.consume(TokenKind::By).is_some() {
            Some(self.parse_expr()?)
        } else {
            None
        };

        let body = Box::new(self.parse_blueprint_stmt()?);

        Some(FaBlueprintStmt::Control(FaControlStmt::Repeat {
            iterable,
            item_name,
            key_expr,
            body,
        }))
    }

    /// Parse select statement: select [on expr] { branches }
    fn parse_select_stmt(&mut self) -> Option<FaBlueprintStmt> {
        self.expect(TokenKind::Select)?;

        let discriminant = if self.consume(TokenKind::On).is_some() {
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.expect(TokenKind::LBrace)?;

        let mut branches = Vec::new();
        let mut else_branch = None;

        while !self.check(TokenKind::RBrace) && !self.at_end() {
            if self.consume(TokenKind::Else).is_some() {
                self.expect(TokenKind::FatArrow)?;
                else_branch = Some(Box::new(self.parse_blueprint_stmt()?));
                break;
            }

            let condition = self.parse_expr()?;
            self.expect(TokenKind::FatArrow)?;
            let body = Box::new(self.parse_blueprint_stmt()?);

            branches.push(FaSelectBranch { condition, body });
        }

        self.expect(TokenKind::RBrace)?;

        Some(FaBlueprintStmt::Control(FaControlStmt::Select {
            discriminant,
            branches,
            else_branch,
        }))
    }

    // =========================================================================
    // Event handlers
    // =========================================================================

    /// Parse event handler: on_click [param ->] { body }
    fn parse_event_handler(&mut self) -> Option<FaBlueprintStmt> {
        let event_name = self.expect_identifier()?;

        // Optional parameter: param -> or param: Type ->
        let param = if self.check(TokenKind::Identifier) {
            let name = self.expect_identifier()?;
            let type_expr = if self.consume(TokenKind::Colon).is_some() {
                Some(self.parse_type_expr()?)
            } else {
                None
            };
            self.expect(TokenKind::Arrow)?;
            Some(FaEventParam { name, type_expr })
        } else {
            None
        };

        self.expect(TokenKind::LBrace)?;

        let mut body = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.at_end() {
            if let Some(stmt) = self.parse_handler_stmt() {
                body.push(stmt);
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;

        Some(FaBlueprintStmt::EventHandler(FaEventHandler {
            event_name,
            param,
            body,
        }))
    }

    /// Parse a handler statement (assignment or command call)
    fn parse_handler_stmt(&mut self) -> Option<FaHandlerStmt> {
        let name = self.expect_identifier()?;

        match self.current_kind() {
            TokenKind::Eq => {
                self.advance();
                let value = self.parse_expr()?;
                Some(FaHandlerStmt::Assignment { name, value })
            }
            TokenKind::LParen => {
                self.advance();
                let mut args = Vec::new();
                if !self.check(TokenKind::RParen) {
                    args.push(self.parse_expr()?);
                    while self.consume(TokenKind::Comma).is_some() {
                        args.push(self.parse_expr()?);
                    }
                }
                self.expect(TokenKind::RParen)?;
                Some(FaHandlerStmt::CommandCall { name, args })
            }
            _ => {
                // Bare identifier - treat as command call with no args
                Some(FaHandlerStmt::CommandCall {
                    name,
                    args: vec![],
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    #[test]
    fn test_simple_blueprint() {
        let result = parse(
            r#"
module test

blueprint Counter {
    count: i32 = 0
}
"#,
        );
        assert!(!result.diagnostics.has_errors());
    }

    #[test]
    fn test_blueprint_with_fragment() {
        let result = parse(
            r#"
module test

blueprint App {
    column {
        text { "Hello" }
    }
}
"#,
        );
        assert!(!result.diagnostics.has_errors());
    }

    #[test]
    fn test_blueprint_with_instructions() {
        let result = parse(
            r#"
module test

blueprint Styled {
    box .. width { 100 } .. height { 50 }
}
"#,
        );
        assert!(!result.diagnostics.has_errors());
    }

    #[test]
    fn test_blueprint_with_when() {
        let result = parse(
            r#"
module test

blueprint Conditional {
    when visible {
        text { "Shown" }
    }
}
"#,
        );
        assert!(!result.diagnostics.has_errors());
    }

    #[test]
    fn test_blueprint_with_repeat() {
        let result = parse(
            r#"
module test

blueprint List {
    repeat on items as item {
        text { item.name }
    }
}
"#,
        );
        for diag in result.diagnostics.iter() {
            eprintln!("Error: {:?}", diag);
        }
        assert!(!result.diagnostics.has_errors());
    }

    #[test]
    fn test_blueprint_with_event() {
        let result = parse(
            r#"
module test

blueprint Button {
    on_click {
        count = count + 1
    }
}
"#,
        );
        assert!(!result.diagnostics.has_errors());
    }

    #[test]
    fn test_blueprint_with_backend() {
        let result = parse(
            r#"
module test

blueprint App {
    with AppBackend

    button {
        on_click { increment() }
    }
}
"#,
        );
        assert!(!result.diagnostics.has_errors());
    }
}
