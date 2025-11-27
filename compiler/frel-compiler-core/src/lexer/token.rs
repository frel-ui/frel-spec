// Token definitions for Frel lexer
//
// This module defines all token types produced by the lexer.

use crate::source::Span;
use serde::{Deserialize, Serialize};

/// Contextual keywords - only reserved at top-level positions.
/// These can be used as identifiers (field names, parameters, etc.) inside declarations.
pub mod contextual {
    pub const MODULE: &str = "module";
    pub const IMPORT: &str = "import";
    pub const BLUEPRINT: &str = "blueprint";
    pub const BACKEND: &str = "backend";
    pub const CONTRACT: &str = "contract";
    pub const SCHEME: &str = "scheme";
    pub const ENUM: &str = "enum";
    pub const THEME: &str = "theme";
    pub const ARENA: &str = "arena";
}

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
    // Note: Top-level declaration keywords (module, import, blueprint, backend,
    // contract, scheme, enum, theme, arena) are CONTEXTUAL - they are lexed as
    // Identifier and only treated as keywords at top-level positions.
    // See is_contextual_keyword() for the list.

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

    // Layout block
    LayoutBlock, // """layout ... """

    // Identifiers
    Identifier,

    // Special
    Newline,     // Significant in some contexts (slot separators)
    Eof,         // End of file
    Error,       // Lexer error token
}

impl TokenKind {
    /// Check if this token is a keyword (always reserved)
    pub fn is_keyword(&self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            With
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

    /// Check if this is an identifier that could be a contextual keyword at top-level.
    /// Contextual keywords are only reserved at positions where a top-level declaration
    /// can appear, and can be used as identifiers elsewhere (field names, parameters, etc.)
    pub fn is_contextual_keyword(s: &str) -> bool {
        use contextual::*;
        matches!(
            s,
            MODULE | IMPORT | BLUEPRINT | BACKEND | CONTRACT | SCHEME | ENUM | THEME | ARENA
        )
    }

    /// Check if this identifier can start a top-level declaration
    pub fn is_top_level_start_str(s: &str) -> bool {
        use contextual::*;
        matches!(
            s,
            BLUEPRINT | BACKEND | CONTRACT | SCHEME | ENUM | THEME | ARENA
        )
    }

    /// Get the keyword for an identifier, if any.
    /// Note: Contextual keywords (module, import, blueprint, etc.) are NOT included here.
    /// They are lexed as Identifier and handled contextually in the parser.
    pub fn keyword_from_str(s: &str) -> Option<TokenKind> {
        use TokenKind::*;
        Some(match s {
            // Contextual keywords are NOT here - they stay as Identifier
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
            // Note: Contextual keywords (module, import, blueprint, etc.) are
            // lexed as Identifier, so they're covered by the Identifier case.
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
            LayoutBlock => "layout block",
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
        // Contextual keywords are NOT returned by keyword_from_str
        assert_eq!(TokenKind::keyword_from_str("blueprint"), None);
        assert_eq!(TokenKind::keyword_from_str("module"), None);
        // Always-reserved keywords are returned
        assert_eq!(TokenKind::keyword_from_str("true"), Some(TokenKind::True));
        assert_eq!(TokenKind::keyword_from_str("when"), Some(TokenKind::When));
        assert_eq!(TokenKind::keyword_from_str("foo"), None);
    }

    #[test]
    fn test_contextual_keywords() {
        assert!(TokenKind::is_contextual_keyword("module"));
        assert!(TokenKind::is_contextual_keyword("blueprint"));
        assert!(TokenKind::is_contextual_keyword("backend"));
        assert!(!TokenKind::is_contextual_keyword("when"));
        assert!(!TokenKind::is_contextual_keyword("foo"));
    }

    #[test]
    fn test_top_level_start() {
        assert!(TokenKind::is_top_level_start_str("blueprint"));
        assert!(TokenKind::is_top_level_start_str("backend"));
        assert!(!TokenKind::is_top_level_start_str("module")); // module is not a declaration start
        assert!(!TokenKind::is_top_level_start_str("import")); // import is not a declaration start
        assert!(!TokenKind::is_top_level_start_str("when"));
    }
}
