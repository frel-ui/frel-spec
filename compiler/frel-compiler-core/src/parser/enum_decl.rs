// Enum parser for Frel

use crate::ast::Enum;
use crate::lexer::token::contextual;
use crate::lexer::TokenKind;

use super::Parser;

impl<'a> Parser<'a> {
    /// Parse enum declaration
    pub(super) fn parse_enum(&mut self) -> Option<Enum> {
        self.expect_contextual(contextual::ENUM)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::LBrace)?;

        let mut variants = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.at_end() {
            if self.check(TokenKind::Identifier) {
                variants.push(self.expect_identifier()?);
            } else {
                self.error_expected("enum variant");
                break;
            }
        }

        self.expect(TokenKind::RBrace)?;

        Some(Enum { name, variants })
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    #[test]
    fn test_parse_enum() {
        let result = parse(
            r#"
module test

enum Status {
    Pending
    Active
    Completed
}
"#,
        );
        assert!(!result.diagnostics.has_errors());
        let file = result.file.unwrap();
        if let crate::ast::TopLevelDecl::Enum(e) = &file.declarations[0] {
            assert_eq!(e.variants.len(), 3);
        }
    }
}
