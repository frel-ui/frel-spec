// Blueprint parser for Frel
//
// Handles parsing of:
// - Blueprint declarations
// - Blueprint statements (with, local decl, fragment creation, control, instructions, events)
// - Fragment creation with slots
// - Control statements (when, repeat, select)
// - Event handlers

use crate::ast::{
    Arg, Blueprint, BlueprintStmt, BlueprintValue, ControlStmt, EventHandler, EventParam,
    FragmentBody, FragmentCreation, HandlerStmt, Instruction, LocalDecl, SelectBranch,
    SlotBinding,
};
use crate::token::TokenKind;

use super::Parser;

impl<'a> Parser<'a> {
    /// Parse blueprint declaration
    pub(super) fn parse_blueprint(&mut self) -> Option<Blueprint> {
        self.expect(TokenKind::Blueprint)?;
        let name = self.expect_identifier()?;
        let params = self.parse_param_list_opt()?;
        self.expect(TokenKind::LBrace)?;

        let body = self.parse_blueprint_body()?;

        self.expect(TokenKind::RBrace)?;

        Some(Blueprint { name, params, body })
    }

    /// Parse blueprint body (list of statements)
    fn parse_blueprint_body(&mut self) -> Option<Vec<BlueprintStmt>> {
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
    fn parse_blueprint_stmt(&mut self) -> Option<BlueprintStmt> {
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
                Some(BlueprintStmt::With(name))
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
                Some(BlueprintStmt::LocalDecl(LocalDecl {
                    name,
                    type_expr,
                    init,
                }))
            }

            // Fragment creation, instruction, or expression
            TokenKind::Identifier => {
                // Check if this is an expression (has continuation tokens like . or ?)
                // vs a fragment creation (followed by (, {, .., or nothing)
                if self.is_expression_continuation() {
                    let expr = self.parse_expr()?;
                    Some(BlueprintStmt::ContentExpr(expr))
                } else {
                    self.parse_fragment_or_instruction()
                }
            }

            // Postfix instruction: .. instruction
            TokenKind::DotDot => {
                self.advance();
                let instr = self.parse_instruction()?;
                Some(BlueprintStmt::Instruction(instr))
            }

            // Content expressions: string literals, numbers, etc.
            TokenKind::StringLiteral
            | TokenKind::StringTemplateStart
            | TokenKind::IntLiteral
            | TokenKind::FloatLiteral
            | TokenKind::True
            | TokenKind::False
            | TokenKind::Null
            | TokenKind::LBracket => {
                let expr = self.parse_expr()?;
                Some(BlueprintStmt::ContentExpr(expr))
            }

            // Block statement: { ... } - wraps multiple statements
            // This appears in control structures like `when condition { ... }`
            TokenKind::LBrace => {
                // Create an anonymous/inline fragment to hold the statements
                self.advance();
                let body = self.parse_blueprint_body()?;
                self.expect(TokenKind::RBrace)?;
                // Wrap in FragmentCreation with empty name to represent a block
                Some(BlueprintStmt::FragmentCreation(FragmentCreation {
                    name: String::new(), // anonymous block
                    args: vec![],
                    body: Some(FragmentBody::Default(body)),
                    instructions: vec![],
                }))
            }

            // Parenthesized expression
            TokenKind::LParen => {
                let expr = self.parse_expr()?;
                Some(BlueprintStmt::ContentExpr(expr))
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

    /// Parse fragment creation or standalone instruction
    fn parse_fragment_or_instruction(&mut self) -> Option<BlueprintStmt> {
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

                // Parse postfix instructions
                let instructions = self.parse_postfix_instructions()?;

                Some(BlueprintStmt::FragmentCreation(FragmentCreation {
                    name,
                    args,
                    body,
                    instructions,
                }))
            }

            // Fragment with just postfix instructions: foo .. width { 100 }
            TokenKind::DotDot => {
                let instructions = self.parse_postfix_instructions()?;
                Some(BlueprintStmt::FragmentCreation(FragmentCreation {
                    name,
                    args: vec![],
                    body: None,
                    instructions,
                }))
            }

            // Bare fragment name (no args, no body, no instructions)
            _ => Some(BlueprintStmt::FragmentCreation(FragmentCreation {
                name,
                args: vec![],
                body: None,
                instructions: vec![],
            })),
        }
    }

    /// Parse fragment body (default or slot-based)
    fn parse_fragment_body(&mut self) -> Option<FragmentBody> {
        self.expect(TokenKind::LBrace)?;

        // Check if it's a slot-based body: { at slotName: ... }
        if self.check(TokenKind::At) {
            let slots = self.parse_slot_bindings()?;
            self.expect(TokenKind::RBrace)?;
            Some(FragmentBody::Slots(slots))
        } else {
            // Default body: regular blueprint statements
            let body = self.parse_blueprint_body()?;
            self.expect(TokenKind::RBrace)?;
            Some(FragmentBody::Default(body))
        }
    }

    /// Parse slot bindings: at slot1: value, at slot2: value
    fn parse_slot_bindings(&mut self) -> Option<Vec<SlotBinding>> {
        let mut bindings = Vec::new();

        while self.check(TokenKind::At) {
            self.advance();
            let slot_name = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;
            let blueprint = self.parse_blueprint_value()?;
            bindings.push(SlotBinding {
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
    fn parse_blueprint_value(&mut self) -> Option<BlueprintValue> {
        // Check for inline blueprint: { body } or params -> { body }
        if self.check(TokenKind::LBrace) {
            self.advance();
            let body = self.parse_blueprint_body()?;
            self.expect(TokenKind::RBrace)?;
            Some(BlueprintValue::Inline {
                params: vec![],
                body,
            })
        } else if self.check(TokenKind::Identifier) {
            // Could be: identifier (reference) or identifier -> { body } (inline with param)
            let first = self.expect_identifier()?;

            if self.check(TokenKind::Arrow) {
                // param -> { body }
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let body = self.parse_blueprint_body()?;
                self.expect(TokenKind::RBrace)?;
                Some(BlueprintValue::Inline {
                    params: vec![first],
                    body,
                })
            } else if self.check(TokenKind::Comma) {
                // Multiple params: a, b -> { body }
                let mut params = vec![first];
                while self.consume(TokenKind::Comma).is_some() {
                    params.push(self.expect_identifier()?);
                }
                self.expect(TokenKind::Arrow)?;
                self.expect(TokenKind::LBrace)?;
                let body = self.parse_blueprint_body()?;
                self.expect(TokenKind::RBrace)?;
                Some(BlueprintValue::Inline { params, body })
            } else {
                // Just a reference
                Some(BlueprintValue::Reference(first))
            }
        } else {
            self.error_expected("blueprint value");
            None
        }
    }

    /// Parse postfix instructions: .. instr1 .. instr2
    fn parse_postfix_instructions(&mut self) -> Option<Vec<Instruction>> {
        let mut instructions = Vec::new();

        while self.consume(TokenKind::DotDot).is_some() {
            let instr = self.parse_instruction()?;
            instructions.push(instr);
        }

        Some(instructions)
    }

    /// Parse argument list
    fn parse_arg_list(&mut self) -> Option<Vec<Arg>> {
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
    fn parse_arg(&mut self) -> Option<Arg> {
        // Check for named argument: name = value
        if self.check(TokenKind::Identifier) {
            if let Some(next) = self.peek() {
                if next.kind == TokenKind::Eq {
                    let name = self.expect_identifier()?;
                    self.advance(); // consume '='
                    let value = self.parse_expr()?;
                    return Some(Arg {
                        name: Some(name),
                        value,
                    });
                }
            }
        }

        // Positional argument
        let value = self.parse_expr()?;
        Some(Arg { name: None, value })
    }

    // =========================================================================
    // Control statements
    // =========================================================================

    /// Parse when statement: when condition stmt [else stmt]
    fn parse_when_stmt(&mut self) -> Option<BlueprintStmt> {
        self.expect(TokenKind::When)?;
        let condition = self.parse_expr()?;
        let then_stmt = Box::new(self.parse_blueprint_stmt()?);

        let else_stmt = if self.consume(TokenKind::Else).is_some() {
            Some(Box::new(self.parse_blueprint_stmt()?))
        } else {
            None
        };

        Some(BlueprintStmt::Control(ControlStmt::When {
            condition,
            then_stmt,
            else_stmt,
        }))
    }

    /// Parse repeat statement: repeat on expr [as name] [by keyExpr] stmt
    fn parse_repeat_stmt(&mut self) -> Option<BlueprintStmt> {
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

        Some(BlueprintStmt::Control(ControlStmt::Repeat {
            iterable,
            item_name,
            key_expr,
            body,
        }))
    }

    /// Parse select statement: select [on expr] { branches }
    fn parse_select_stmt(&mut self) -> Option<BlueprintStmt> {
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

            branches.push(SelectBranch { condition, body });
        }

        self.expect(TokenKind::RBrace)?;

        Some(BlueprintStmt::Control(ControlStmt::Select {
            discriminant,
            branches,
            else_branch,
        }))
    }

    // =========================================================================
    // Event handlers
    // =========================================================================

    /// Parse event handler: on_click [param ->] { body }
    fn parse_event_handler(&mut self) -> Option<BlueprintStmt> {
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
            Some(EventParam { name, type_expr })
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

        Some(BlueprintStmt::EventHandler(EventHandler {
            event_name,
            param,
            body,
        }))
    }

    /// Parse a handler statement (assignment or command call)
    fn parse_handler_stmt(&mut self) -> Option<HandlerStmt> {
        let name = self.expect_identifier()?;

        match self.current_kind() {
            TokenKind::Eq => {
                self.advance();
                let value = self.parse_expr()?;
                Some(HandlerStmt::Assignment { name, value })
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
                Some(HandlerStmt::CommandCall { name, args })
            }
            _ => {
                // Bare identifier - treat as command call with no args
                Some(HandlerStmt::CommandCall {
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
