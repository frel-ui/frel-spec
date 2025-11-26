// Token definitions for Frel lexer
//
// This module defines all token types produced by the lexer,
// derived from the PEST grammar specification.

use crate::source::Span;
use serde::{Deserialize, Serialize};

/// A token with its kind and source span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Get the text of this token from source
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        self.span.text(source)
    }
}

/// Token kinds produced by the lexer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenKind {
    // Keywords - declarations
    Module,
    Import,
    Blueprint,
    Backend,
    Contract,
    Scheme,
    Enum,
    Theme,
    Arena,

    // Keywords - blueprint/backend members
    With,
    Include,
    Method,
    Command,
    Virtual,
    Set,
    Variant,
    For,

    // Keywords - control flow
    When,
    Else,
    Repeat,
    On,
    As,
    By,
    Select,

    // Keywords - type modifiers
    Ref,
    Draft,
    Asset,
    // Note: List, Map, Tree, Set, Accessor, Blueprint are NOT keywords
    // They're identifiers handled contextually in the type parser

    // Keywords - literals
    True,
    False,
    Null,

    // Keywords - instructions (subset, commonly used)
    At,

    // Operators
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    StarStar,    // **
    Eq,          // =
    EqEq,        // ==
    BangEq,      // !=
    Lt,          // <
    LtEq,        // <=
    Gt,          // >
    GtEq,        // >=
    Bang,        // !
    AmpAmp,      // &&
    PipePipe,    // ||
    Question,    // ?
    QuestionColon, // ?:
    QuestionDot, // ?.
    Arrow,       // ->
    FatArrow,    // =>
    DotDot,      // ..

    // Punctuation
    LParen,      // (
    RParen,      // )
    LBrace,      // {
    RBrace,      // }
    LBracket,    // [
    RBracket,    // ]
    Comma,       // ,
    Colon,       // :
    Dot,         // .

    // Literals
    IntLiteral,        // 42, 0x2A, 0b101010, 0o52
    FloatLiteral,      // 3.14, 1.0e10
    ColorLiteral,      // #RRGGBB, #RRGGBBAA
    StringLiteral,     // "hello"

    // String template parts
    StringTemplateStart,   // "text ${
    StringTemplateMiddle,  // } more ${
    StringTemplateEnd,     // } end"

    // Identifiers
    Identifier,

    // Special
    Newline,     // Significant in some contexts (slot separators)
    Eof,         // End of file
    Error,       // Lexer error token
}

impl TokenKind {
    /// Check if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            Module
                | Import
                | Blueprint
                | Backend
                | Contract
                | Scheme
                | Enum
                | Theme
                | Arena
                | With
                | Include
                | Method
                | Command
                | Virtual
                | Set
                | Variant
                | For
                | When
                | Else
                | Repeat
                | On
                | As
                | By
                | Select
                | Ref
                | Draft
                | Asset
                | True
                | False
                | Null
                | At
        )
    }

    /// Check if this token can start a top-level declaration
    pub fn is_top_level_start(&self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            Blueprint | Backend | Contract | Scheme | Enum | Theme | Arena
        )
    }

    /// Check if this token is a synchronization point for error recovery
    pub fn is_sync_point(&self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            RBrace | Blueprint | Backend | Contract | Scheme | Enum | Theme | Arena | Eof
        )
    }

    /// Get the keyword for an identifier, if any
    pub fn keyword_from_str(s: &str) -> Option<TokenKind> {
        use TokenKind::*;
        Some(match s {
            "module" => Module,
            "import" => Import,
            "blueprint" => Blueprint,
            "backend" => Backend,
            "contract" => Contract,
            "scheme" => Scheme,
            "enum" => Enum,
            "theme" => Theme,
            "arena" => Arena,
            "with" => With,
            "include" => Include,
            "method" => Method,
            "command" => Command,
            "virtual" => Virtual,
            "set" => Set,
            "variant" => Variant,
            "for" => For,
            "when" => When,
            "else" => Else,
            "repeat" => Repeat,
            "on" => On,
            "as" => As,
            "by" => By,
            "select" => Select,
            "ref" => Ref,
            "draft" => Draft,
            "asset" => Asset,
            // Note: List, Set, Map, Tree, Accessor are NOT keywords
            // They're just identifiers that are treated specially in type contexts
            "true" => True,
            "false" => False,
            "null" => Null,
            "at" => At,
            _ => return None,
        })
    }

    /// Get display name for error messages
    pub fn display_name(&self) -> &'static str {
        use TokenKind::*;
        match self {
            Module => "'module'",
            Import => "'import'",
            Blueprint => "'blueprint'",
            Backend => "'backend'",
            Contract => "'contract'",
            Scheme => "'scheme'",
            Enum => "'enum'",
            Theme => "'theme'",
            Arena => "'arena'",
            With => "'with'",
            Include => "'include'",
            Method => "'method'",
            Command => "'command'",
            Virtual => "'virtual'",
            Set => "'set'",
            Variant => "'variant'",
            For => "'for'",
            When => "'when'",
            Else => "'else'",
            Repeat => "'repeat'",
            On => "'on'",
            As => "'as'",
            By => "'by'",
            Select => "'select'",
            Ref => "'ref'",
            Draft => "'draft'",
            Asset => "'asset'",
            True => "'true'",
            False => "'false'",
            Null => "'null'",
            At => "'at'",
            Plus => "'+'",
            Minus => "'-'",
            Star => "'*'",
            Slash => "'/'",
            Percent => "'%'",
            StarStar => "'**'",
            Eq => "'='",
            EqEq => "'=='",
            BangEq => "'!='",
            Lt => "'<'",
            LtEq => "'<='",
            Gt => "'>'",
            GtEq => "'>='",
            Bang => "'!'",
            AmpAmp => "'&&'",
            PipePipe => "'||'",
            Question => "'?'",
            QuestionColon => "'?:'",
            QuestionDot => "'?.'",
            Arrow => "'->'",
            FatArrow => "'=>'",
            DotDot => "'..'",
            LParen => "'('",
            RParen => "')'",
            LBrace => "'{'",
            RBrace => "'}'",
            LBracket => "'['",
            RBracket => "']'",
            Comma => "','",
            Colon => "':'",
            Dot => "'.'",
            IntLiteral => "integer",
            FloatLiteral => "float",
            ColorLiteral => "color",
            StringLiteral => "string",
            StringTemplateStart => "string template",
            StringTemplateMiddle => "string template",
            StringTemplateEnd => "string template",
            Identifier => "identifier",
            Newline => "newline",
            Eof => "end of file",
            Error => "error",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_lookup() {
        assert_eq!(
            TokenKind::keyword_from_str("blueprint"),
            Some(TokenKind::Blueprint)
        );
        assert_eq!(TokenKind::keyword_from_str("true"), Some(TokenKind::True));
        assert_eq!(TokenKind::keyword_from_str("foo"), None);
    }

    #[test]
    fn test_is_sync_point() {
        assert!(TokenKind::RBrace.is_sync_point());
        assert!(TokenKind::Blueprint.is_sync_point());
        assert!(!TokenKind::Plus.is_sync_point());
    }
}
