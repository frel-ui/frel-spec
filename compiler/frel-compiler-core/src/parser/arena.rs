// Arena parser for Frel

use crate::ast::Arena;
use crate::lexer::token::contextual;
use crate::lexer::TokenKind;

use super::Parser;

impl<'a> Parser<'a> {
    /// Parse arena declaration
    pub(super) fn parse_arena(&mut self) -> Option<Arena> {
        self.expect_contextual(contextual::ARENA)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::LBrace)?;

        self.expect(TokenKind::For)?;
        let scheme_name = self.expect_identifier()?;

        let contract = if self.consume(TokenKind::With).is_some() {
            Some(self.expect_identifier()?)
        } else {
            None
        };

        self.expect(TokenKind::RBrace)?;

        Some(Arena {
            name,
            scheme_name,
            contract,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    #[test]
    fn test_parse_arena() {
        let result = parse(
            r#"
module test

arena UserArena {
    for User with UserAPI
}
"#,
        );
        assert!(!result.diagnostics.has_errors());
    }
}
