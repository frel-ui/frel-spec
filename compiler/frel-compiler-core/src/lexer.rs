// Lexer (tokenizer) for Frel
//
// This module implements a hand-written lexer that:
// - Tokenizes Frel source code into a stream of tokens
// - Handles all literal types (numbers, strings, string templates)
// - Recovers from errors by emitting Error tokens and continuing
// - Tracks source positions for error reporting

use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::source::Span;
use crate::token::{Token, TokenKind};

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

        let Some((pos, ch)) = self.peek_char() else {
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

            // String literals
            '"' => return self.lex_string(start),

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

    // --- Strings ---

    fn lex_string(&mut self, start: usize) -> Token {
        self.advance(); // opening '"'

        let mut has_interpolation = false;
        let string_start = self.current_pos;

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
                    has_interpolation = true;
                    // Return the start portion as StringTemplateStart
                    // The parser will handle the interpolation
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
    pub fn lex_string_template_continue(&mut self, start: usize) -> Token {
        // We're at the '}' that ends an interpolation
        self.advance(); // consume '}'

        loop {
            match self.peek_char() {
                Some((_, '"')) => {
                    self.advance();
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
    fn test_keywords() {
        assert_eq!(
            lex("blueprint backend scheme"),
            vec![
                TokenKind::Blueprint,
                TokenKind::Backend,
                TokenKind::Scheme,
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
}
