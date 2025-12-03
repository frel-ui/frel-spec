// Blueprint parser for Frel
//
// Handles parsing of:
// - Blueprint declarations
// - Blueprint statements (with, local decl, fragment creation, control, instructions, events)
// - Fragment creation with slots
// - Control statements (when, repeat, select)
// - Event handlers

use crate::ast::{
    Arg, Blueprint, BlueprintStmt, BlueprintValue, ControlStmt, EventHandler,
    EventParam, FragmentBody, FragmentCreation, HandlerStmt, LocalDecl, PostfixItem,
    SelectBranch, SlotBinding,
};
use crate::lexer::token::contextual;
use crate::lexer::TokenKind;

use super::layout::LayoutParser;
use super::Parser;

// Note: Instructions are now distinguished syntactically by the `..` prefix,
// not by a known-names whitelist. This makes the grammar context-free.
// Example: `.. width { 300 }` is an instruction, `text { "hello" }` is fragment creation.

impl<'a> Parser<'a> {
    /// Parse blueprint declaration
    pub(super) fn parse_blueprint(&mut self) -> Option<Blueprint> {
        let start = self.current_span().start;
        self.expect_contextual(contextual::BLUEPRINT)?;
        let name = self.expect_identifier()?;
        let params = self.parse_param_list_opt()?;
        self.expect(TokenKind::LBrace)?;

        let body = self.parse_blueprint_body()?;

        let end_span = self.current_span();
        self.expect(TokenKind::RBrace)?;

        let span = crate::source::Span::new(start, end_span.end);
        Some(Blueprint { name, params, body, span })
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
                let start = self.current_span().start;
                let name = self.expect_identifier()?;
                self.expect(TokenKind::Colon)?;
                let type_expr = self.parse_type_expr()?;
                self.expect(TokenKind::Eq)?;
                let init = self.parse_expr()?;
                let end = self.previous_span().end;
                let span = crate::source::Span::new(start, end);
                Some(BlueprintStmt::LocalDecl(LocalDecl {
                    name,
                    type_expr,
                    init,
                    span,
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
                    Some(BlueprintStmt::ContentExpr(expr))
                }
            }

            // Postfix item: .. instruction or .. on_click { ... }
            TokenKind::DotDot => {
                self.advance();
                // Check if this is an event handler (on_*)
                if self.is_event_handler_start() {
                    let handler = self.parse_postfix_event_handler()?;
                    Some(BlueprintStmt::EventHandler(handler))
                } else {
                    let instr = self.parse_instruction_expr()?;
                    Some(BlueprintStmt::Instruction(instr))
                }
            }

            // Layout block: """layout ... """
            TokenKind::LayoutBlock => self.parse_layout_stmt(),

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
                    postfix: vec![],
                }))
            }

            // Parenthesized expression
            TokenKind::LParen => {
                let expr = self.parse_expr()?;
                Some(BlueprintStmt::ContentExpr(expr))
            }

            // Slot binding: at slot: { ... }
            // These are used with layout statements to bind content to named slots
            TokenKind::At => self.parse_slot_binding_stmt(),

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
    fn parse_fragment_creation(&mut self) -> Option<BlueprintStmt> {
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

                Some(BlueprintStmt::FragmentCreation(FragmentCreation {
                    name,
                    args,
                    body,
                    postfix,
                }))
            }

            // Fragment with just postfix items: foo .. width { 100 } .. on_click { ... }
            TokenKind::DotDot => {
                let postfix = self.parse_postfix_items()?;
                Some(BlueprintStmt::FragmentCreation(FragmentCreation {
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
                Some(BlueprintStmt::ContentExpr(crate::ast::Expr::Identifier(name)))
            }
        }
    }

    /// Parse fragment body (default, slot-based, or inline blueprint with params)
    fn parse_fragment_body(&mut self) -> Option<FragmentBody> {
        self.expect(TokenKind::LBrace)?;

        // Check if it's a slot-based body: { at slotName: ... }
        if self.check(TokenKind::At) {
            let slots = self.parse_slot_bindings()?;
            self.expect(TokenKind::RBrace)?;
            Some(FragmentBody::Slots(slots))
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
            Some(FragmentBody::InlineBlueprint { params, body })
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
                Some(BlueprintValue::Inline { params, body })
            } else {
                // No params, just body
                let body = self.parse_blueprint_body()?;
                self.expect(TokenKind::RBrace)?;
                Some(BlueprintValue::Inline {
                    params: vec![],
                    body,
                })
            }
        } else if self.check(TokenKind::Identifier) {
            // Blueprint reference
            let name = self.expect_identifier()?;
            Some(BlueprintValue::Reference(name))
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

    /// Check if there's an arrow after an identifier with optional type annotation
    /// Used for event handlers: { param -> } or { param: Type -> }
    fn has_arrow_after_typed_param(&self) -> bool {
        let mut offset = 1; // Start after current identifier

        // Check for optional `: Type` after identifier
        if let Some(t) = self.peek_n(offset) {
            if t.kind == TokenKind::Colon {
                // Skip the colon and type - we just need to find the arrow eventually
                offset += 1;
                // Skip tokens until we find arrow or something that indicates it's not a param
                loop {
                    match self.peek_n(offset) {
                        Some(t) if t.kind == TokenKind::Arrow => return true,
                        Some(t)
                            if t.kind == TokenKind::RBrace
                                || t.kind == TokenKind::LBrace
                                || t.kind == TokenKind::Eq =>
                        {
                            return false
                        }
                        None => return false,
                        _ => offset += 1,
                    }
                }
            }
        }

        // No colon, check for direct arrow
        matches!(self.peek_n(offset), Some(t) if t.kind == TokenKind::Arrow)
    }

    /// Parse postfix items (instructions or event handlers): .. instr1 .. on_click { ... }
    fn parse_postfix_items(&mut self) -> Option<Vec<PostfixItem>> {
        let mut items = Vec::new();

        while self.consume(TokenKind::DotDot).is_some() {
            // Check if this is an event handler (on_*)
            if self.is_event_handler_start() {
                let handler = self.parse_postfix_event_handler()?;
                items.push(PostfixItem::EventHandler(handler));
            } else {
                let instr = self.parse_instruction_expr()?;
                items.push(PostfixItem::Instruction(instr));
            }
        }

        Some(items)
    }

    /// Parse event handler in postfix position: on_click { [param ->] body }
    fn parse_postfix_event_handler(&mut self) -> Option<EventHandler> {
        let event_name = self.expect_identifier()?;

        self.expect(TokenKind::LBrace)?;

        // Optional parameter inside braces: { param -> body } or { param: Type -> body }
        let param = if self.check(TokenKind::Identifier) && self.has_arrow_after_typed_param() {
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

        let mut body = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.at_end() {
            if let Some(stmt) = self.parse_handler_stmt() {
                body.push(stmt);
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;

        Some(EventHandler {
            event_name,
            param,
            body,
        })
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

    /// Parse repeat statement: repeat on expr [by keyExpr] { item -> body }
    fn parse_repeat_stmt(&mut self) -> Option<BlueprintStmt> {
        self.expect(TokenKind::Repeat)?;
        self.expect(TokenKind::On)?;
        let iterable = self.parse_expr()?;

        let key_expr = if self.consume(TokenKind::By).is_some() {
            Some(self.parse_expr()?)
        } else {
            None
        };

        // Expect { item -> body }
        self.expect(TokenKind::LBrace)?;
        let item_name = self.expect_identifier()?;
        self.expect(TokenKind::Arrow)?;
        let body = self.parse_blueprint_body()?;
        self.expect(TokenKind::RBrace)?;

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

    /// Parse a slot binding statement: at slot: { ... }
    fn parse_slot_binding_stmt(&mut self) -> Option<BlueprintStmt> {
        self.expect(TokenKind::At)?;
        let slot_name = self.expect_identifier()?;
        self.expect(TokenKind::Colon)?;
        let blueprint = self.parse_blueprint_value()?;
        Some(BlueprintStmt::SlotBinding(SlotBinding {
            slot_name,
            blueprint,
        }))
    }

    /// Parse a layout statement: """layout ... """
    fn parse_layout_stmt(&mut self) -> Option<BlueprintStmt> {
        let token = self.advance();
        let full_text = token.text(self.source);

        // Extract content between """layout and """
        // The token text is: """layout\n...\n"""
        let content = extract_layout_content(full_text);

        // Calculate base offset for the content (after """layout\n)
        let prefix_len = "\"\"\"layout".len();
        // Find the first newline after """layout
        let content_start_offset = full_text[prefix_len..]
            .find('\n')
            .map(|i| prefix_len + i + 1)
            .unwrap_or(prefix_len);
        let base_offset = token.span.start + content_start_offset as u32;

        // Parse the layout content
        let mut layout_parser = LayoutParser::new(content, base_offset, &mut self.diagnostics);
        let grid = layout_parser.parse()?;

        Some(BlueprintStmt::Layout(grid))
    }
}

/// Extract layout content from the token text (strip """layout and """)
fn extract_layout_content(text: &str) -> &str {
    // Skip """layout prefix
    let prefix = "\"\"\"layout";
    let after_prefix = text.strip_prefix(prefix).unwrap_or(text);

    // Skip leading newline if present
    let after_newline = after_prefix.strip_prefix('\n').unwrap_or(after_prefix);

    // Strip closing """
    let before_suffix = after_newline.strip_suffix("\"\"\"").unwrap_or(after_newline);

    // Strip trailing newline before """ if present
    before_suffix.strip_suffix('\n').unwrap_or(before_suffix)
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
    repeat on items { item ->
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

    #[test]
    fn test_blueprint_with_layout() {
        let result = parse(
            r#"
module test

blueprint Grid {
    """layout
    | slot1 | slot2 |
    | slot3 | slot4 |
    """
    at slot1: { text { "One" } }
    at slot2: { text { "Two" } }
}
"#,
        );
        for diag in result.diagnostics.iter() {
            eprintln!("Error: {:?}", diag);
        }
        assert!(!result.diagnostics.has_errors());
    }

    #[test]
    fn test_blueprint_with_layout_sizes() {
        let result = parse(
            r#"
module test

blueprint Sized {
    """layout
         ~0.5    ~0.8
      24 | slot1 | slot2 |
     ~1  | slot3 | slot4 |
    """
}
"#,
        );
        for diag in result.diagnostics.iter() {
            eprintln!("Error: {:?}", diag);
        }
        assert!(!result.diagnostics.has_errors());
    }

    #[test]
    fn test_blueprint_with_layout_instructions() {
        let result = parse(
            r#"
module test

blueprint WithInstructions {
    """layout
    .. gap { 8 }
    | a | b |
    """
}
"#,
        );
        for diag in result.diagnostics.iter() {
            eprintln!("Error: {:?}", diag);
        }
        assert!(!result.diagnostics.has_errors());
    }
}
