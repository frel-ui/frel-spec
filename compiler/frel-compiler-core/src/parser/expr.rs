// Expression parser for Frel using Pratt parsing
//
// Handles all expression types with proper precedence:
// - Ternary (? :)
// - Elvis (?:)
// - Logical OR (||)
// - Logical AND (&&)
// - Equality (== !=)
// - Comparison (< <= > >=)
// - Additive (+ -)
// - Multiplicative (* / %)
// - Exponential (**)
// - Unary (! - +)
// - Postfix (. ?. ())

use crate::ast::{FaBinaryOp, FaExpr, FaTemplateElement, FaUnaryOp};
use crate::lexer::TokenKind;

use super::Parser;

/// Precedence levels for Pratt parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Ternary,      // ? :
    Elvis,        // ?:
    Or,           // ||
    And,          // &&
    Equality,     // == !=
    Comparison,   // < <= > >=
    Additive,     // + -
    Multiplicative, // * / %
    Exponential,  // **
    Unary,        // ! - +
    Postfix,      // . ?. ()
}

impl Precedence {
    fn next(self) -> Self {
        match self {
            Precedence::None => Precedence::Ternary,
            Precedence::Ternary => Precedence::Elvis,
            Precedence::Elvis => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Additive,
            Precedence::Additive => Precedence::Multiplicative,
            Precedence::Multiplicative => Precedence::Exponential,
            Precedence::Exponential => Precedence::Unary,
            Precedence::Unary => Precedence::Postfix,
            Precedence::Postfix => Precedence::Postfix,
        }
    }
}

fn infix_precedence(kind: TokenKind) -> Option<Precedence> {
    Some(match kind {
        TokenKind::Question => Precedence::Ternary,
        TokenKind::QuestionColon => Precedence::Elvis,
        TokenKind::PipePipe => Precedence::Or,
        TokenKind::AmpAmp => Precedence::And,
        TokenKind::EqEq | TokenKind::BangEq => Precedence::Equality,
        TokenKind::Lt | TokenKind::LtEq | TokenKind::Gt | TokenKind::GtEq => Precedence::Comparison,
        TokenKind::Plus | TokenKind::Minus => Precedence::Additive,
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Multiplicative,
        TokenKind::StarStar => Precedence::Exponential,
        TokenKind::Dot | TokenKind::QuestionDot | TokenKind::LParen => Precedence::Postfix,
        _ => return None,
    })
}

fn binary_op(kind: TokenKind) -> Option<FaBinaryOp> {
    Some(match kind {
        TokenKind::Plus => FaBinaryOp::Add,
        TokenKind::Minus => FaBinaryOp::Sub,
        TokenKind::Star => FaBinaryOp::Mul,
        TokenKind::Slash => FaBinaryOp::Div,
        TokenKind::Percent => FaBinaryOp::Mod,
        TokenKind::StarStar => FaBinaryOp::Pow,
        TokenKind::EqEq => FaBinaryOp::Eq,
        TokenKind::BangEq => FaBinaryOp::Ne,
        TokenKind::Lt => FaBinaryOp::Lt,
        TokenKind::LtEq => FaBinaryOp::Le,
        TokenKind::Gt => FaBinaryOp::Gt,
        TokenKind::GtEq => FaBinaryOp::Ge,
        TokenKind::AmpAmp => FaBinaryOp::And,
        TokenKind::PipePipe => FaBinaryOp::Or,
        TokenKind::QuestionColon => FaBinaryOp::Elvis,
        _ => return None,
    })
}

impl<'a> Parser<'a> {
    /// Parse an expression
    pub(super) fn parse_expr(&mut self) -> Option<FaExpr> {
        self.parse_expr_precedence(Precedence::None)
    }

    /// Parse expression with minimum precedence (Pratt parsing)
    fn parse_expr_precedence(&mut self, min_prec: Precedence) -> Option<FaExpr> {
        // Parse prefix/primary expression
        let mut left = self.parse_prefix()?;

        // Parse infix operators while precedence is high enough
        while let Some(prec) = infix_precedence(self.current_kind()) {
            if prec <= min_prec {
                break;
            }

            left = self.parse_infix(left, prec)?;
        }

        Some(left)
    }

    /// Parse prefix expression (unary or primary)
    fn parse_prefix(&mut self) -> Option<FaExpr> {
        match self.current_kind() {
            TokenKind::Bang => {
                self.advance();
                let expr = self.parse_expr_precedence(Precedence::Unary)?;
                Some(FaExpr::Unary {
                    op: FaUnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_expr_precedence(Precedence::Unary)?;
                Some(FaExpr::Unary {
                    op: FaUnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Plus => {
                self.advance();
                let expr = self.parse_expr_precedence(Precedence::Unary)?;
                Some(FaExpr::Unary {
                    op: FaUnaryOp::Pos,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }

    /// Parse infix expression
    fn parse_infix(&mut self, left: FaExpr, prec: Precedence) -> Option<FaExpr> {
        match self.current_kind() {
            // Ternary: a ? b : c
            TokenKind::Question => {
                self.advance();
                let then_expr = self.parse_expr()?;
                self.expect(TokenKind::Colon)?;
                let else_expr = self.parse_expr_precedence(prec)?;
                Some(FaExpr::Ternary {
                    condition: Box::new(left),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                })
            }

            // Field access: a.b
            TokenKind::Dot => {
                self.advance();
                let field = self.expect_identifier()?;
                Some(FaExpr::FieldAccess {
                    base: Box::new(left),
                    field,
                })
            }

            // Optional chain: a?.b
            TokenKind::QuestionDot => {
                self.advance();
                let field = self.expect_identifier()?;
                Some(FaExpr::OptionalChain {
                    base: Box::new(left),
                    field,
                })
            }

            // Function call: a(b, c)
            TokenKind::LParen => {
                self.advance();
                let args = self.parse_call_args()?;
                self.expect(TokenKind::RParen)?;
                Some(FaExpr::Call {
                    callee: Box::new(left),
                    args,
                })
            }

            // Binary operators
            kind => {
                if let Some(op) = binary_op(kind) {
                    self.advance();
                    // For left-associative operators, use same precedence so higher precedence binds tighter
                    // For right-associative operators (like **), use one lower so same precedence continues
                    let right_prec = if kind == TokenKind::StarStar {
                        // Right associative: allow same precedence to continue
                        Precedence::Multiplicative // one below Exponential
                    } else {
                        // Left associative: same precedence, so higher precedence ops bind
                        prec
                    };
                    let right = self.parse_expr_precedence(right_prec)?;
                    Some(FaExpr::Binary {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    })
                } else {
                    self.error_unexpected();
                    None
                }
            }
        }
    }

    /// Parse primary expression
    fn parse_primary(&mut self) -> Option<FaExpr> {
        match self.current_kind() {
            // Literals
            TokenKind::Null => {
                self.advance();
                Some(FaExpr::Null)
            }
            TokenKind::True => {
                self.advance();
                Some(FaExpr::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                Some(FaExpr::Bool(false))
            }
            TokenKind::IntLiteral => {
                let text = self.current_text();
                let value = self.parse_int_literal(text);
                self.advance();
                Some(FaExpr::Int(value))
            }
            TokenKind::FloatLiteral => {
                let text = self.current_text();
                let value = self.parse_float_literal(text);
                self.advance();
                Some(FaExpr::Float(value))
            }
            TokenKind::StringLiteral => {
                let text = self.current_text();
                let value = self.parse_string_content(text);
                self.advance();
                Some(FaExpr::String(value))
            }
            TokenKind::StringTemplateStart => {
                self.parse_string_template()
            }

            // List literal: [a, b, c]
            TokenKind::LBracket => {
                self.advance();
                let elements = self.parse_list_elements()?;
                self.expect(TokenKind::RBracket)?;
                Some(FaExpr::List(elements))
            }

            // Object literal or grouping: { a: 1 } or (expr)
            TokenKind::LBrace => {
                self.advance();
                if self.check(TokenKind::RBrace) {
                    self.advance();
                    Some(FaExpr::Object(vec![]))
                } else if self.is_object_field_start() {
                    let fields = self.parse_object_fields()?;
                    self.expect(TokenKind::RBrace)?;
                    Some(FaExpr::Object(fields))
                } else {
                    self.error_expected("object field");
                    None
                }
            }

            // Parenthesized expression
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Some(expr)
            }

            // Identifier or qualified name
            TokenKind::Identifier => {
                let first = self.current_text().to_string();
                self.advance();

                // Check if this is a qualified name (Enum.Variant or module.name)
                // But NOT if followed by more complex postfix ops
                if self.check(TokenKind::Dot) {
                    if let Some(next) = self.peek() {
                        if next.kind == TokenKind::Identifier {
                            // Could be qualified name or field access
                            // For now, parse first identifier and let postfix handle the rest
                            // This handles both Enum.Variant and obj.field
                        }
                    }
                }

                Some(FaExpr::Identifier(first))
            }

            _ => {
                self.error_expected("expression");
                None
            }
        }
    }

    /// Parse call arguments
    fn parse_call_args(&mut self) -> Option<Vec<FaExpr>> {
        if self.check(TokenKind::RParen) {
            return Some(vec![]);
        }

        let mut args = vec![self.parse_expr()?];

        while self.consume(TokenKind::Comma).is_some() {
            if self.check(TokenKind::RParen) {
                break; // Trailing comma
            }
            args.push(self.parse_expr()?);
        }

        Some(args)
    }

    /// Parse list elements
    fn parse_list_elements(&mut self) -> Option<Vec<FaExpr>> {
        if self.check(TokenKind::RBracket) {
            return Some(vec![]);
        }

        let mut elements = vec![self.parse_expr()?];

        while self.consume(TokenKind::Comma).is_some() {
            if self.check(TokenKind::RBracket) {
                break; // Trailing comma
            }
            elements.push(self.parse_expr()?);
        }

        Some(elements)
    }

    /// Check if we're at the start of an object field
    fn is_object_field_start(&self) -> bool {
        if !self.check(TokenKind::Identifier) {
            return false;
        }
        // Check for identifier followed by colon
        if let Some(next) = self.peek() {
            next.kind == TokenKind::Colon
        } else {
            false
        }
    }

    /// Parse object fields
    fn parse_object_fields(&mut self) -> Option<Vec<(String, FaExpr)>> {
        let mut fields = vec![self.parse_object_field()?];

        while self.consume(TokenKind::Comma).is_some() {
            if self.check(TokenKind::RBrace) {
                break; // Trailing comma
            }
            fields.push(self.parse_object_field()?);
        }

        Some(fields)
    }

    /// Parse a single object field
    fn parse_object_field(&mut self) -> Option<(String, FaExpr)> {
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Colon)?;
        let value = self.parse_expr()?;
        Some((name, value))
    }

    /// Parse string template: "text ${expr} more"
    fn parse_string_template(&mut self) -> Option<FaExpr> {
        let mut elements = Vec::new();

        // Get the initial text (before first ${)
        let start_text = self.current_text();
        if let Some(text) = self.extract_template_text(start_text) {
            if !text.is_empty() {
                elements.push(FaTemplateElement::Text(text));
            }
        }
        self.advance();

        // We're now after the ${ (lexer consumed it)
        // Parse alternating expressions and text parts
        loop {
            // Parse the interpolated expression
            let expr = self.parse_expr()?;
            elements.push(FaTemplateElement::Interpolation(Box::new(expr)));

            // The lexer handles the closing } and returns the next string part
            // as StringTemplateMiddle or StringTemplateEnd
            match self.current_kind() {
                TokenKind::StringTemplateMiddle => {
                    let text = self.current_text();
                    if let Some(t) = self.extract_template_middle_text(text) {
                        if !t.is_empty() {
                            elements.push(FaTemplateElement::Text(t));
                        }
                    }
                    self.advance();
                    // Continue loop for next interpolation
                }
                TokenKind::StringTemplateEnd => {
                    let text = self.current_text();
                    if let Some(t) = self.extract_template_end_text(text) {
                        if !t.is_empty() {
                            elements.push(FaTemplateElement::Text(t));
                        }
                    }
                    self.advance();
                    break;
                }
                _ => {
                    self.error_expected("string template continuation");
                    break;
                }
            }
        }

        Some(FaExpr::StringTemplate(elements))
    }

    /// Extract text from template start: "text ${
    fn extract_template_text(&self, s: &str) -> Option<String> {
        // Remove leading " and trailing ${
        let s = s.strip_prefix('"')?;
        let s = s.strip_suffix("${")?;
        Some(self.unescape_string(s))
    }

    /// Extract text from template middle: } text ${
    fn extract_template_middle_text(&self, s: &str) -> Option<String> {
        let s = s.strip_prefix('}')?;
        let s = s.strip_suffix("${")?;
        Some(self.unescape_string(s))
    }

    /// Extract text from template end: } text"
    fn extract_template_end_text(&self, s: &str) -> Option<String> {
        let s = s.strip_prefix('}')?;
        let s = s.strip_suffix('"')?;
        Some(self.unescape_string(s))
    }

    /// Parse integer literal (handles hex, binary, octal)
    fn parse_int_literal(&self, s: &str) -> i64 {
        let s = s.replace('_', "");
        if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            i64::from_str_radix(hex, 16).unwrap_or(0)
        } else if let Some(bin) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
            i64::from_str_radix(bin, 2).unwrap_or(0)
        } else if let Some(oct) = s.strip_prefix("0o").or_else(|| s.strip_prefix("0O")) {
            i64::from_str_radix(oct, 8).unwrap_or(0)
        } else {
            s.parse().unwrap_or(0)
        }
    }

    /// Parse float literal
    fn parse_float_literal(&self, s: &str) -> f64 {
        let s = s.replace('_', "");
        s.parse().unwrap_or(0.0)
    }

    /// Parse string content (remove quotes, handle escapes)
    fn parse_string_content(&self, s: &str) -> String {
        let inner = &s[1..s.len() - 1]; // Remove quotes
        self.unescape_string(inner)
    }

    /// Unescape string escape sequences
    fn unescape_string(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('\'') => result.push('\''),
                    Some('$') => result.push('$'),
                    Some(c) => {
                        result.push('\\');
                        result.push(c);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(c);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;
    use crate::ast::FaExpr;

    fn parse_expr(source: &str) -> Option<FaExpr> {
        // Wrap in a backend to test expression parsing
        let full_source = format!("module test\nbackend Test {{ x: i32 = {} }}", source);
        let result = parse(&full_source);
        if result.diagnostics.has_errors() {
            for d in result.diagnostics.iter() {
                eprintln!("{:?}", d);
            }
            return None;
        }
        let file = result.file?;
        if let crate::ast::FaTopLevelDecl::Backend(backend) = &file.declarations[0] {
            if let crate::ast::FaBackendMember::Field(field) = &backend.members[0] {
                return field.init.clone();
            }
        }
        None
    }

    #[test]
    fn test_literals() {
        assert!(matches!(parse_expr("42"), Some(FaExpr::Int(42))));
        assert!(matches!(parse_expr("3.14"), Some(FaExpr::Float(f)) if (f - 3.14).abs() < 0.001));
        assert!(matches!(parse_expr("true"), Some(FaExpr::Bool(true))));
        assert!(matches!(parse_expr("false"), Some(FaExpr::Bool(false))));
        assert!(matches!(parse_expr("null"), Some(FaExpr::Null)));
    }

    #[test]
    fn test_hex_binary_octal() {
        assert!(matches!(parse_expr("0xFF"), Some(FaExpr::Int(255))));
        assert!(matches!(parse_expr("0b1010"), Some(FaExpr::Int(10))));
        assert!(matches!(parse_expr("0o77"), Some(FaExpr::Int(63))));
    }

    #[test]
    fn test_string() {
        if let Some(FaExpr::String(s)) = parse_expr(r#""hello""#) {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_binary_ops() {
        assert!(matches!(parse_expr("1 + 2"), Some(FaExpr::Binary { .. })));
        assert!(matches!(parse_expr("a && b"), Some(FaExpr::Binary { .. })));
        assert!(matches!(parse_expr("x == y"), Some(FaExpr::Binary { .. })));
    }

    #[test]
    fn test_unary_ops() {
        assert!(matches!(parse_expr("!x"), Some(FaExpr::Unary { .. })));
        assert!(matches!(parse_expr("-5"), Some(FaExpr::Unary { .. })));
    }

    #[test]
    fn test_ternary() {
        assert!(matches!(parse_expr("a ? b : c"), Some(FaExpr::Ternary { .. })));
    }

    #[test]
    fn test_field_access() {
        assert!(matches!(parse_expr("a.b"), Some(FaExpr::FieldAccess { .. })));
        assert!(matches!(parse_expr("a?.b"), Some(FaExpr::OptionalChain { .. })));
    }

    #[test]
    fn test_call() {
        assert!(matches!(parse_expr("foo()"), Some(FaExpr::Call { .. })));
        assert!(matches!(parse_expr("foo(1, 2)"), Some(FaExpr::Call { .. })));
    }

    #[test]
    fn test_list() {
        if let Some(FaExpr::List(elements)) = parse_expr("[1, 2, 3]") {
            assert_eq!(elements.len(), 3);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_object() {
        if let Some(FaExpr::Object(fields)) = parse_expr("{ a: 1, b: 2 }") {
            assert_eq!(fields.len(), 2);
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_precedence() {
        // 1 + 2 * 3 should be 1 + (2 * 3)
        if let Some(FaExpr::Binary { op, left, right }) = parse_expr("1 + 2 * 3") {
            assert!(matches!(op, crate::ast::FaBinaryOp::Add));
            assert!(matches!(*left, FaExpr::Int(1)));
            assert!(matches!(*right, FaExpr::Binary { .. }));
        } else {
            panic!("Expected binary");
        }
    }
}
