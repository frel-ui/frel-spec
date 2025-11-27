// Lexer (tokenizer) for Frel
//
// This module implements a hand-written lexer that:
// - Tokenizes Frel source code into a stream of tokens
// - Handles all literal types (numbers, strings, string templates)
// - Recovers from errors by emitting Error tokens and continuing
// - Tracks source positions for error reporting

use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::source::Span;

use super::{Token, TokenKind};

/// Lexer state
pub struct Lexer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    current_pos: usize,
    diagnostics: Diagnostics,
    /// Stack for tracking string template nesting
    template_depth: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            current_pos: 0,
            diagnostics: Diagnostics::new(),
            template_depth: 0,
        }
    }

    /// Tokenize the entire source and return tokens + diagnostics
    pub fn tokenize(mut self) -> (Vec<Token>, Diagnostics) {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        (tokens, self.diagnostics)
    }

    /// Get the next token
    fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let start = self.current_pos;

        let Some((_, ch)) = self.peek_char() else {
            return Token::new(TokenKind::Eof, Span::new(start as u32, start as u32));
        };

        let kind = match ch {
            // Single-character punctuation
            '(' => {
                self.advance();
                TokenKind::LParen
            }
            ')' => {
                self.advance();
                TokenKind::RParen
            }
            '{' => {
                self.advance();
                TokenKind::LBrace
            }
            '}' => {
                // Check if we're closing a string interpolation
                if self.template_depth > 0 {
                    return self.lex_string_template_continue(start);
                }
                self.advance();
                TokenKind::RBrace
            }
            '[' => {
                self.advance();
                TokenKind::LBracket
            }
            ']' => {
                self.advance();
                TokenKind::RBracket
            }
            ',' => {
                self.advance();
                TokenKind::Comma
            }
            ':' => {
                self.advance();
                TokenKind::Colon
            }
            '+' => {
                self.advance();
                TokenKind::Plus
            }
            '%' => {
                self.advance();
                TokenKind::Percent
            }

            // Multi-character operators starting with specific chars
            '-' => self.lex_minus(),
            '*' => self.lex_star(),
            '/' => self.lex_slash(),
            '=' => self.lex_equals(),
            '!' => self.lex_bang(),
            '<' => self.lex_less(),
            '>' => self.lex_greater(),
            '&' => self.lex_ampersand(),
            '|' => self.lex_pipe(),
            '?' => self.lex_question(),
            '.' => self.lex_dot(),

            // String literals and layout blocks
            '"' => {
                // Check for triple-quote layout block: """layout
                if self.peek_char_nth(1) == Some('"') && self.peek_char_nth(2) == Some('"') {
                    return self.lex_layout_block(start);
                }
                return self.lex_string(start);
            }

            // Color literals (#RRGGBB or #RRGGBBAA)
            '#' => return self.lex_color(start),

            // Numbers
            '0'..='9' => return self.lex_number(start),

            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => return self.lex_identifier(start),

            // Newline (significant in some contexts)
            '\n' => {
                self.advance();
                TokenKind::Newline
            }

            // Unknown character
            _ => {
                self.advance();
                self.diagnostics.add(
                    Diagnostic::error(
                        format!("unexpected character '{}'", ch),
                        Span::new(start as u32, self.current_pos as u32),
                    )
                    .with_code("E0100"),
                );
                TokenKind::Error
            }
        };

        Token::new(kind, Span::new(start as u32, self.current_pos as u32))
    }

    // --- Character operations ---

    fn peek_char(&mut self) -> Option<(usize, char)> {
        self.chars.peek().copied()
    }

    fn advance(&mut self) -> Option<(usize, char)> {
        let result = self.chars.next();
        if let Some((pos, ch)) = result {
            self.current_pos = pos + ch.len_utf8();
        }
        result
    }

    fn peek_char_nth(&self, n: usize) -> Option<char> {
        self.source[self.current_pos..].chars().nth(n)
    }

    // --- Whitespace and comments ---

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek_char() {
                Some((_, ' ')) | Some((_, '\t')) | Some((_, '\r')) => {
                    self.advance();
                }
                Some((_, '/')) => {
                    if self.peek_char_nth(1) == Some('/') {
                        self.skip_line_comment();
                    } else if self.peek_char_nth(1) == Some('*') {
                        self.skip_block_comment();
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn skip_line_comment(&mut self) {
        // Skip //
        self.advance();
        self.advance();
        // Skip until newline (but don't consume it - it might be significant)
        while let Some((_, ch)) = self.peek_char() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        let start = self.current_pos;
        // Skip /*
        self.advance();
        self.advance();

        let mut depth = 1;
        while depth > 0 {
            match self.peek_char() {
                Some((_, '*')) if self.peek_char_nth(1) == Some('/') => {
                    self.advance();
                    self.advance();
                    depth -= 1;
                }
                Some((_, '/')) if self.peek_char_nth(1) == Some('*') => {
                    self.advance();
                    self.advance();
                    depth += 1;
                }
                Some(_) => {
                    self.advance();
                }
                None => {
                    self.diagnostics.add(
                        Diagnostic::error(
                            "unterminated block comment",
                            Span::new(start as u32, self.current_pos as u32),
                        )
                        .with_code("E0101"),
                    );
                    break;
                }
            }
        }
    }

    // --- Operators ---

    fn lex_minus(&mut self) -> TokenKind {
        self.advance();
        if self.peek_char().map(|(_, c)| c) == Some('>') {
            self.advance();
            TokenKind::Arrow
        } else {
            TokenKind::Minus
        }
    }

    fn lex_star(&mut self) -> TokenKind {
        self.advance();
        if self.peek_char().map(|(_, c)| c) == Some('*') {
            self.advance();
            TokenKind::StarStar
        } else {
            TokenKind::Star
        }
    }

    fn lex_slash(&mut self) -> TokenKind {
        self.advance();
        TokenKind::Slash
    }

    fn lex_equals(&mut self) -> TokenKind {
        self.advance();
        if self.peek_char().map(|(_, c)| c) == Some('=') {
            self.advance();
            TokenKind::EqEq
        } else if self.peek_char().map(|(_, c)| c) == Some('>') {
            self.advance();
            TokenKind::FatArrow
        } else {
            TokenKind::Eq
        }
    }

    fn lex_bang(&mut self) -> TokenKind {
        self.advance();
        if self.peek_char().map(|(_, c)| c) == Some('=') {
            self.advance();
            TokenKind::BangEq
        } else {
            TokenKind::Bang
        }
    }

    fn lex_less(&mut self) -> TokenKind {
        self.advance();
        if self.peek_char().map(|(_, c)| c) == Some('=') {
            self.advance();
            TokenKind::LtEq
        } else {
            TokenKind::Lt
        }
    }

    fn lex_greater(&mut self) -> TokenKind {
        self.advance();
        if self.peek_char().map(|(_, c)| c) == Some('=') {
            self.advance();
            TokenKind::GtEq
        } else {
            TokenKind::Gt
        }
    }

    fn lex_ampersand(&mut self) -> TokenKind {
        let start = self.current_pos;
        self.advance();
        if self.peek_char().map(|(_, c)| c) == Some('&') {
            self.advance();
            TokenKind::AmpAmp
        } else {
            self.diagnostics.add(
                Diagnostic::error(
                    "expected '&&', found single '&'",
                    Span::new(start as u32, self.current_pos as u32),
                )
                .with_code("E0102")
                .with_help("Frel uses '&&' for logical AND"),
            );
            TokenKind::Error
        }
    }

    fn lex_pipe(&mut self) -> TokenKind {
        let start = self.current_pos;
        self.advance();
        if self.peek_char().map(|(_, c)| c) == Some('|') {
            self.advance();
            TokenKind::PipePipe
        } else {
            self.diagnostics.add(
                Diagnostic::error(
                    "expected '||', found single '|'",
                    Span::new(start as u32, self.current_pos as u32),
                )
                .with_code("E0103")
                .with_help("Frel uses '||' for logical OR"),
            );
            TokenKind::Error
        }
    }

    fn lex_question(&mut self) -> TokenKind {
        self.advance();
        match self.peek_char().map(|(_, c)| c) {
            Some(':') => {
                self.advance();
                TokenKind::QuestionColon
            }
            Some('.') => {
                self.advance();
                TokenKind::QuestionDot
            }
            _ => TokenKind::Question,
        }
    }

    fn lex_dot(&mut self) -> TokenKind {
        self.advance();
        if self.peek_char().map(|(_, c)| c) == Some('.') {
            self.advance();
            TokenKind::DotDot
        } else {
            TokenKind::Dot
        }
    }

    // --- Numbers ---

    fn lex_number(&mut self, start: usize) -> Token {
        // Check for hex, binary, octal
        if self.peek_char().map(|(_, c)| c) == Some('0') {
            if let Some(next) = self.peek_char_nth(1) {
                match next {
                    'x' | 'X' => return self.lex_hex_number(start),
                    'b' | 'B' => return self.lex_binary_number(start),
                    'o' | 'O' => return self.lex_octal_number(start),
                    _ => {}
                }
            }
        }

        // Decimal number
        self.lex_decimal_number(start)
    }

    fn lex_decimal_number(&mut self, start: usize) -> Token {
        // Integer part
        while let Some((_, ch)) = self.peek_char() {
            if ch.is_ascii_digit() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        // Check for float
        let is_float = self.peek_char().map(|(_, c)| c) == Some('.')
            && self.peek_char_nth(1).map_or(false, |c| c.is_ascii_digit());

        if is_float {
            self.advance(); // consume '.'

            // Fractional part
            while let Some((_, ch)) = self.peek_char() {
                if ch.is_ascii_digit() || ch == '_' {
                    self.advance();
                } else {
                    break;
                }
            }

            // Exponent
            if let Some((_, 'e')) | Some((_, 'E')) = self.peek_char() {
                self.advance();
                // Optional sign
                if let Some((_, '+')) | Some((_, '-')) = self.peek_char() {
                    self.advance();
                }
                // Exponent digits
                while let Some((_, ch)) = self.peek_char() {
                    if ch.is_ascii_digit() {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }

            Token::new(
                TokenKind::FloatLiteral,
                Span::new(start as u32, self.current_pos as u32),
            )
        } else {
            Token::new(
                TokenKind::IntLiteral,
                Span::new(start as u32, self.current_pos as u32),
            )
        }
    }

    fn lex_hex_number(&mut self, start: usize) -> Token {
        self.advance(); // '0'
        self.advance(); // 'x'

        while let Some((_, ch)) = self.peek_char() {
            if ch.is_ascii_hexdigit() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        Token::new(
            TokenKind::IntLiteral,
            Span::new(start as u32, self.current_pos as u32),
        )
    }

    fn lex_binary_number(&mut self, start: usize) -> Token {
        self.advance(); // '0'
        self.advance(); // 'b'

        while let Some((_, ch)) = self.peek_char() {
            if ch == '0' || ch == '1' || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        Token::new(
            TokenKind::IntLiteral,
            Span::new(start as u32, self.current_pos as u32),
        )
    }

    fn lex_octal_number(&mut self, start: usize) -> Token {
        self.advance(); // '0'
        self.advance(); // 'o'

        while let Some((_, ch)) = self.peek_char() {
            if ('0'..='7').contains(&ch) || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        Token::new(
            TokenKind::IntLiteral,
            Span::new(start as u32, self.current_pos as u32),
        )
    }

    // --- Colors ---

    fn lex_color(&mut self, start: usize) -> Token {
        self.advance(); // '#'

        // Count hex digits
        let hex_start = self.current_pos;
        while let Some((_, ch)) = self.peek_char() {
            if ch.is_ascii_hexdigit() {
                self.advance();
            } else {
                break;
            }
        }

        let hex_len = self.current_pos - hex_start;

        // Validate: must be exactly 6 (RGB) or 8 (RGBA) hex digits
        if hex_len != 6 && hex_len != 8 {
            self.diagnostics.add(
                Diagnostic::error(
                    format!(
                        "color literal must have 6 or 8 hex digits, found {}",
                        hex_len
                    ),
                    Span::new(start as u32, self.current_pos as u32),
                )
                .with_code("E0106")
                .with_help("use #RRGGBB or #RRGGBBAA format"),
            );
            return Token::new(
                TokenKind::Error,
                Span::new(start as u32, self.current_pos as u32),
            );
        }

        Token::new(
            TokenKind::ColorLiteral,
            Span::new(start as u32, self.current_pos as u32),
        )
    }

    // --- Strings ---

    fn lex_string(&mut self, start: usize) -> Token {
        self.advance(); // opening '"'

        let _string_start = self.current_pos;

        loop {
            match self.peek_char() {
                Some((_, '"')) => {
                    self.advance();
                    break;
                }
                Some((_, '\\')) => {
                    self.advance();
                    // Skip escaped character
                    if self.peek_char().is_some() {
                        self.advance();
                    }
                }
                Some((_, '$')) if self.peek_char_nth(1) == Some('{') => {
                    // Consume the ${ so parser can read the interpolated expression
                    self.advance(); // consume '$'
                    self.advance(); // consume '{'
                    // Track that we're inside a string template
                    self.template_depth += 1;
                    // Return the start portion as StringTemplateStart
                    return Token::new(
                        TokenKind::StringTemplateStart,
                        Span::new(start as u32, self.current_pos as u32),
                    );
                }
                Some((_, '\n')) | None => {
                    self.diagnostics.add(
                        Diagnostic::error(
                            "unterminated string literal",
                            Span::new(start as u32, self.current_pos as u32),
                        )
                        .with_code("E0104"),
                    );
                    return Token::new(
                        TokenKind::Error,
                        Span::new(start as u32, self.current_pos as u32),
                    );
                }
                Some(_) => {
                    self.advance();
                }
            }
        }

        Token::new(
            TokenKind::StringLiteral,
            Span::new(start as u32, self.current_pos as u32),
        )
    }

    /// Continue lexing a string template after an interpolation
    fn lex_string_template_continue(&mut self, start: usize) -> Token {
        // We're at the '}' that ends an interpolation
        self.advance(); // consume '}'

        loop {
            match self.peek_char() {
                Some((_, '"')) => {
                    self.advance();
                    // We're exiting the string template
                    self.template_depth -= 1;
                    return Token::new(
                        TokenKind::StringTemplateEnd,
                        Span::new(start as u32, self.current_pos as u32),
                    );
                }
                Some((_, '\\')) => {
                    self.advance();
                    if self.peek_char().is_some() {
                        self.advance();
                    }
                }
                Some((_, '$')) if self.peek_char_nth(1) == Some('{') => {
                    // Consume the ${ so parser can read the next interpolated expression
                    self.advance(); // consume '$'
                    self.advance(); // consume '{'
                    // Note: template_depth stays the same - we're still in the same string template
                    return Token::new(
                        TokenKind::StringTemplateMiddle,
                        Span::new(start as u32, self.current_pos as u32),
                    );
                }
                Some((_, '\n')) | None => {
                    self.diagnostics.add(
                        Diagnostic::error(
                            "unterminated string template",
                            Span::new(start as u32, self.current_pos as u32),
                        )
                        .with_code("E0105"),
                    );
                    // Clean up template depth on error
                    self.template_depth = self.template_depth.saturating_sub(1);
                    return Token::new(
                        TokenKind::Error,
                        Span::new(start as u32, self.current_pos as u32),
                    );
                }
                Some(_) => {
                    self.advance();
                }
            }
        }
    }

    // --- Layout blocks ---

    /// Lex a triple-quoted layout block: """layout ... """
    fn lex_layout_block(&mut self, start: usize) -> Token {
        // Consume opening """
        self.advance(); // first "
        self.advance(); // second "
        self.advance(); // third "

        // Check for block type identifier (must be "layout")
        let block_type_start = self.current_pos;
        while let Some((_, ch)) = self.peek_char() {
            if ch.is_ascii_alphabetic() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let block_type = &self.source[block_type_start..self.current_pos];

        if block_type != "layout" {
            self.diagnostics.add(
                Diagnostic::error(
                    format!(
                        "unknown triple-quoted block type '{}', expected 'layout'",
                        block_type
                    ),
                    Span::new(block_type_start as u32, self.current_pos as u32),
                )
                .with_code("E0107")
                .with_help("triple-quoted blocks must start with 'layout'"),
            );
            // Continue to consume until closing """ for recovery
        }

        // Scan content until closing """
        loop {
            match self.peek_char() {
                Some((_, '"')) => {
                    // Check for closing """
                    if self.peek_char_nth(1) == Some('"') && self.peek_char_nth(2) == Some('"') {
                        self.advance(); // first "
                        self.advance(); // second "
                        self.advance(); // third "

                        return Token::new(
                            TokenKind::LayoutBlock,
                            Span::new(start as u32, self.current_pos as u32),
                        );
                    }
                    // Not a closing """, just a regular " in content
                    self.advance();
                }
                Some(_) => {
                    self.advance();
                }
                None => {
                    // Unclosed block
                    self.diagnostics.add(
                        Diagnostic::error(
                            "unterminated layout block",
                            Span::new(start as u32, self.current_pos as u32),
                        )
                        .with_code("E0108")
                        .with_help("layout blocks must end with \"\"\""),
                    );
                    return Token::new(
                        TokenKind::Error,
                        Span::new(start as u32, self.current_pos as u32),
                    );
                }
            }
        }
    }

    // --- Identifiers and keywords ---

    fn lex_identifier(&mut self, start: usize) -> Token {
        while let Some((_, ch)) = self.peek_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let text = &self.source[start..self.current_pos];
        let kind = TokenKind::keyword_from_str(text).unwrap_or(TokenKind::Identifier);

        Token::new(kind, Span::new(start as u32, self.current_pos as u32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(source: &str) -> Vec<TokenKind> {
        let lexer = Lexer::new(source);
        let (tokens, _) = lexer.tokenize();
        tokens.into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_contextual_keywords() {
        // Contextual keywords (top-level declarations) are lexed as Identifier
        assert_eq!(
            lex("blueprint backend scheme"),
            vec![
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_always_reserved_keywords() {
        // Always-reserved keywords are lexed as their specific token types
        assert_eq!(
            lex("when else repeat with include"),
            vec![
                TokenKind::When,
                TokenKind::Else,
                TokenKind::Repeat,
                TokenKind::With,
                TokenKind::Include,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_operators() {
        assert_eq!(
            lex("+ - * / ** == != <= >= && || ?. ?:"),
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::StarStar,
                TokenKind::EqEq,
                TokenKind::BangEq,
                TokenKind::LtEq,
                TokenKind::GtEq,
                TokenKind::AmpAmp,
                TokenKind::PipePipe,
                TokenKind::QuestionDot,
                TokenKind::QuestionColon,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_numbers() {
        let tokens = lex("42 3.14 0xFF 0b1010 0o777 1.0e10");
        assert_eq!(
            tokens,
            vec![
                TokenKind::IntLiteral,
                TokenKind::FloatLiteral,
                TokenKind::IntLiteral,
                TokenKind::IntLiteral,
                TokenKind::IntLiteral,
                TokenKind::FloatLiteral,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_strings() {
        assert_eq!(
            lex(r#""hello""#),
            vec![TokenKind::StringLiteral, TokenKind::Eof]
        );
    }

    #[test]
    fn test_string_template() {
        // String with interpolation starts as StringTemplateStart
        let tokens = lex(r#""hello ${name}""#);
        assert_eq!(tokens[0], TokenKind::StringTemplateStart);
    }

    #[test]
    fn test_identifiers() {
        assert_eq!(
            lex("foo _bar Baz123"),
            vec![
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(
            lex("(){}[]:,."),
            vec![
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::LBrace,
                TokenKind::RBrace,
                TokenKind::LBracket,
                TokenKind::RBracket,
                TokenKind::Colon,
                TokenKind::Comma,
                TokenKind::Dot,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_comments() {
        assert_eq!(
            lex("foo // comment\nbar /* block */ baz"),
            vec![
                TokenKind::Identifier,
                TokenKind::Newline,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_dotdot() {
        assert_eq!(
            lex(".. width { 100 }"),
            vec![
                TokenKind::DotDot,
                TokenKind::Identifier,
                TokenKind::LBrace,
                TokenKind::IntLiteral,
                TokenKind::RBrace,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_arrow() {
        assert_eq!(
            lex("-> =>"),
            vec![TokenKind::Arrow, TokenKind::FatArrow, TokenKind::Eof]
        );
    }

    #[test]
    fn test_error_recovery() {
        let lexer = Lexer::new("foo @ bar");
        let (tokens, diags) = lexer.tokenize();
        // Should have tokens even with error
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Error));
        assert!(diags.has_errors());
        // Should continue after error
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Identifier));
    }

    #[test]
    fn test_color_literals() {
        // Valid 6-digit RGB
        assert_eq!(
            lex("#FF0000"),
            vec![TokenKind::ColorLiteral, TokenKind::Eof]
        );
        // Valid 8-digit RGBA
        assert_eq!(
            lex("#FF0000FF"),
            vec![TokenKind::ColorLiteral, TokenKind::Eof]
        );
        // Multiple colors
        assert_eq!(
            lex("#FFFFFF #000000"),
            vec![TokenKind::ColorLiteral, TokenKind::ColorLiteral, TokenKind::Eof]
        );
        // Color in expression context
        assert_eq!(
            lex("color: #FF0000"),
            vec![
                TokenKind::Identifier,
                TokenKind::Colon,
                TokenKind::ColorLiteral,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_color_literal_errors() {
        // Too few digits
        let lexer = Lexer::new("#FFF");
        let (tokens, diags) = lexer.tokenize();
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Error));
        assert!(diags.has_errors());

        // Too many digits
        let lexer = Lexer::new("#FFFFFFFFF");
        let (tokens, diags) = lexer.tokenize();
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Error));
        assert!(diags.has_errors());
    }

    #[test]
    fn test_layout_block_basic() {
        let tokens = lex(
            r#""""layout
| a | b |
""""#,
        );
        assert_eq!(tokens, vec![TokenKind::LayoutBlock, TokenKind::Eof]);
    }

    #[test]
    fn test_layout_block_with_sizes() {
        let tokens = lex(
            r#""""layout
     ~0.5    ~0.8
  24 | slot1 | slot2 |
 ~1  | slot3 | slot4 |
""""#,
        );
        assert_eq!(tokens, vec![TokenKind::LayoutBlock, TokenKind::Eof]);
    }

    #[test]
    fn test_layout_block_with_instructions() {
        let tokens = lex(
            r#""""layout
.. gap { 8 }
| a | b |
""""#,
        );
        assert_eq!(tokens, vec![TokenKind::LayoutBlock, TokenKind::Eof]);
    }

    #[test]
    fn test_layout_block_unclosed() {
        let lexer = Lexer::new(
            r#""""layout
| a | b |"#,
        );
        let (tokens, diags) = lexer.tokenize();
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Error));
        assert!(diags.has_errors());
    }

    #[test]
    fn test_layout_block_unknown_type() {
        let lexer = Lexer::new(
            r#""""unknown
content
""""#,
        );
        let (tokens, diags) = lexer.tokenize();
        // Should still produce a LayoutBlock token (for error recovery)
        assert!(tokens.iter().any(|t| t.kind == TokenKind::LayoutBlock));
        assert!(diags.has_errors());
    }

    #[test]
    fn test_layout_block_with_quote_inside() {
        // A single " inside layout block shouldn't close it
        let tokens = lex(
            r#""""layout
| "quoted" |
""""#,
        );
        assert_eq!(tokens, vec![TokenKind::LayoutBlock, TokenKind::Eof]);
    }

    #[test]
    fn test_string_not_affected_by_layout() {
        // Regular strings should still work
        assert_eq!(
            lex(r#""hello""#),
            vec![TokenKind::StringLiteral, TokenKind::Eof]
        );
    }

    #[test]
    fn test_layout_block_followed_by_tokens() {
        // Layout block followed by slot bindings
        let tokens = lex(
            r#""""layout
| a | b |
"""
at slot1"#,
        );
        assert_eq!(
            tokens,
            vec![
                TokenKind::LayoutBlock,
                TokenKind::Newline,
                TokenKind::At,
                TokenKind::Identifier,
                TokenKind::Eof
            ]
        );
    }
}
