
// Parser for Frel
//
// This module implements a hand-written recursive descent parser with:
// - Pratt parsing for expressions
// - Error recovery via synchronization
// - Multiple error reporting
// - Span tracking for all AST nodes

mod arena;
mod backend;
mod blueprint;
mod common;
mod contract;
mod enum_decl;
mod expr;
mod scheme;
mod theme;
mod types;

use crate::ast;
use crate::diagnostic::{Diagnostic, Diagnostics, Label};
use crate::lexer::{Lexer, Token, TokenKind};
use crate::source::Span;

/// Parser state
pub struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    cursor: usize,
    diagnostics: Diagnostics,
}

/// Result of parsing - either success or failure with partial AST
pub struct ParseResult {
    pub file: Option<ast::FaFile>,
    pub diagnostics: Diagnostics,
}

impl<'a> Parser<'a> {
    /// Create a new parser from source code
    pub fn new(source: &'a str) -> Self {
        let lexer = Lexer::new(source);
        let (tokens, lex_diags) = lexer.tokenize();

        Self {
            source,
            tokens,
            cursor: 0,
            diagnostics: lex_diags,
        }
    }

    /// Parse the source and return the AST with diagnostics
    pub fn parse(mut self) -> ParseResult {
        let file = self.parse_file();
        ParseResult {
            file,
            diagnostics: self.diagnostics,
        }
    }

    // =========================================================================
    // Token operations
    // =========================================================================

    /// Get the current token
    fn current(&self) -> &Token {
        self.tokens.get(self.cursor).unwrap_or_else(|| {
            self.tokens
                .last()
                .expect("token stream should always have EOF")
        })
    }

    /// Get the current token kind
    fn current_kind(&self) -> TokenKind {
        self.current().kind
    }

    /// Get the current span
    fn current_span(&self) -> Span {
        self.current().span
    }

    /// Peek at the next token (after current)
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor + 1)
    }

    /// Peek at token n positions ahead
    fn peek_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.cursor + n)
    }

    /// Check if we're at the end of file
    fn at_end(&self) -> bool {
        self.current_kind() == TokenKind::Eof
    }

    /// Check if the current token matches the expected kind
    fn check(&self, kind: TokenKind) -> bool {
        self.current_kind() == kind
    }

    /// Check if any of the kinds match
    #[allow(dead_code)]
    fn check_any(&self, kinds: &[TokenKind]) -> bool {
        kinds.contains(&self.current_kind())
    }

    /// Advance to the next token and return the previous one
    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if !self.at_end() {
            self.cursor += 1;
        }
        // Skip newlines in most contexts
        self.skip_newlines();
        token
    }

    /// Advance without skipping newlines
    #[allow(dead_code)]
    fn advance_raw(&mut self) -> Token {
        let token = self.current().clone();
        if !self.at_end() {
            self.cursor += 1;
        }
        token
    }

    /// Skip newline tokens
    fn skip_newlines(&mut self) {
        while self.current_kind() == TokenKind::Newline {
            self.cursor += 1;
        }
    }

    /// Consume a token if it matches, otherwise return None
    fn consume(&mut self, kind: TokenKind) -> Option<Token> {
        if self.check(kind) {
            Some(self.advance())
        } else {
            None
        }
    }

    /// Expect a specific token, emit error if not found
    fn expect(&mut self, kind: TokenKind) -> Option<Token> {
        if self.check(kind) {
            Some(self.advance())
        } else {
            self.error_expected(kind.display_name());
            None
        }
    }

    /// Expect an identifier, return its text
    fn expect_identifier(&mut self) -> Option<String> {
        if self.check(TokenKind::Identifier) {
            let token = self.advance();
            Some(token.text(self.source).to_string())
        } else {
            self.error_expected("identifier");
            None
        }
    }

    /// Get the text of the current token
    fn current_text(&self) -> &str {
        self.current().text(self.source)
    }

    /// Check if the next token (after current identifier) continues an expression
    /// This distinguishes `item.name` (expression) from `item { }` (fragment creation)
    #[allow(dead_code)]
    fn is_expression_continuation(&self) -> bool {
        // Current token is an identifier, peek at what follows
        if let Some(next) = self.peek() {
            matches!(
                next.kind,
                // Field access or optional chaining
                TokenKind::Dot
                    | TokenKind::QuestionDot
                    // Binary operators
                    | TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Star
                    | TokenKind::Slash
                    | TokenKind::Percent
                    | TokenKind::StarStar
                    | TokenKind::EqEq
                    | TokenKind::BangEq
                    | TokenKind::Lt
                    | TokenKind::LtEq
                    | TokenKind::Gt
                    | TokenKind::GtEq
                    | TokenKind::AmpAmp
                    | TokenKind::PipePipe
                    | TokenKind::Question
                    | TokenKind::QuestionColon
            )
        } else {
            false
        }
    }

    // =========================================================================
    // Error handling
    // =========================================================================

    /// Report an "expected X" error
    fn error_expected(&mut self, what: &str) {
        let span = self.current_span();
        let got = self.current_kind().display_name();
        self.diagnostics.add(
            Diagnostic::error(format!("expected {}, found {}", what, got), span)
                .with_code("E0200"),
        );
    }

    /// Report an "expected X" error with suggestion
    #[allow(dead_code)]
    fn error_expected_with_suggestion(&mut self, what: &str, suggestion: &str) {
        let span = self.current_span();
        let got = self.current_kind().display_name();
        self.diagnostics.add(
            Diagnostic::error(format!("expected {}, found {}", what, got), span)
                .with_code("E0200")
                .with_help(suggestion),
        );
    }

    /// Report an unexpected token error
    fn error_unexpected(&mut self) {
        let span = self.current_span();
        let kind = self.current_kind().display_name();
        self.diagnostics.add(
            Diagnostic::error(format!("unexpected {}", kind), span).with_code("E0201"),
        );
    }

    /// Report an unclosed delimiter error
    #[allow(dead_code)]
    fn error_unclosed(&mut self, what: &str, open_span: Span) {
        let span = self.current_span();
        self.diagnostics.add(
            Diagnostic::error(format!("unclosed {}", what), span)
                .with_code("E0202")
                .with_label(Label::new(open_span, format!("{} opened here", what))),
        );
    }

    /// Synchronize to the next recovery point after an error
    #[allow(dead_code)]
    fn synchronize(&mut self) {
        while !self.at_end() {
            // Stop at synchronization points
            if self.current_kind().is_sync_point() {
                // If it's a closing brace, consume it
                if self.check(TokenKind::RBrace) {
                    self.advance();
                }
                return;
            }
            self.advance();
        }
    }

    /// Synchronize to the next top-level declaration
    fn synchronize_to_top_level(&mut self) {
        while !self.at_end() {
            if self.current_kind().is_top_level_start() {
                return;
            }
            self.advance();
        }
    }

    // =========================================================================
    // Top-level parsing
    // =========================================================================

    /// Parse a complete file
    fn parse_file(&mut self) -> Option<ast::FaFile> {
        self.skip_newlines();

        // Parse module declaration
        let module = self.parse_module_decl()?;

        // Parse imports
        let mut imports = Vec::new();
        while self.check(TokenKind::Import) {
            if let Some(import) = self.parse_import() {
                imports.push(import);
            } else {
                self.synchronize_to_top_level();
            }
        }

        // Parse declarations
        let mut declarations = Vec::new();
        while !self.at_end() {
            if let Some(decl) = self.parse_top_level_decl() {
                declarations.push(decl);
            } else {
                self.synchronize_to_top_level();
            }
        }

        Some(ast::FaFile {
            module,
            imports,
            declarations,
        })
    }

    /// Parse module declaration: module foo.bar.baz
    fn parse_module_decl(&mut self) -> Option<String> {
        self.expect(TokenKind::Module)?;
        self.parse_module_path()
    }

    /// Parse a module path: foo.bar.baz
    fn parse_module_path(&mut self) -> Option<String> {
        let mut path = self.expect_identifier()?;

        while self.consume(TokenKind::Dot).is_some() {
            let part = self.expect_identifier()?;
            path.push('.');
            path.push_str(&part);
        }

        Some(path)
    }

    /// Parse import statement: import foo.bar.Baz
    fn parse_import(&mut self) -> Option<ast::FaImport> {
        self.expect(TokenKind::Import)?;

        // Parse module path up to the last component
        let mut parts = vec![self.expect_identifier()?];

        while self.consume(TokenKind::Dot).is_some() {
            parts.push(self.expect_identifier()?);
        }

        // Last part is the name, rest is module path
        let name = parts.pop()?;
        let module = parts.join(".");

        Some(ast::FaImport { module, name })
    }

    /// Parse a top-level declaration
    fn parse_top_level_decl(&mut self) -> Option<ast::FaTopLevelDecl> {
        match self.current_kind() {
            TokenKind::Blueprint => self.parse_blueprint().map(ast::FaTopLevelDecl::Blueprint),
            TokenKind::Backend => self.parse_backend().map(ast::FaTopLevelDecl::Backend),
            TokenKind::Contract => self.parse_contract().map(ast::FaTopLevelDecl::Contract),
            TokenKind::Scheme => self.parse_scheme().map(ast::FaTopLevelDecl::Scheme),
            TokenKind::Enum => self.parse_enum().map(ast::FaTopLevelDecl::Enum),
            TokenKind::Theme => self.parse_theme().map(ast::FaTopLevelDecl::Theme),
            TokenKind::Arena => self.parse_arena().map(ast::FaTopLevelDecl::Arena),
            _ => {
                self.error_expected("declaration (blueprint, backend, scheme, enum, contract, theme, or arena)");
                None
            }
        }
    }
}

/// Parse Frel source code
pub fn parse(source: &str) -> ParseResult {
    Parser::new(source).parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_module() {
        let result = parse("module test.example");
        assert!(!result.diagnostics.has_errors());
        let file = result.file.unwrap();
        assert_eq!(file.module, "test.example");
    }

    #[test]
    fn test_parse_import() {
        let result = parse("module test\nimport foo.bar.Baz");
        assert!(!result.diagnostics.has_errors());
        let file = result.file.unwrap();
        assert_eq!(file.imports.len(), 1);
        assert_eq!(file.imports[0].module, "foo.bar");
        assert_eq!(file.imports[0].name, "Baz");
    }

    #[test]
    fn test_error_recovery() {
        // Missing module keyword - should error but continue
        let result = parse("foo.bar\nblueprint Test {}");
        assert!(result.diagnostics.has_errors());
    }
}
