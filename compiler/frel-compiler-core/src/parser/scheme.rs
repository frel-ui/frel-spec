// Scheme parser for Frel

use crate::ast::{FieldInstruction, Scheme, SchemeField, SchemeMember, VirtualField};
use crate::lexer::token::contextual;
use crate::lexer::TokenKind;

use super::Parser;

impl<'a> Parser<'a> {
    /// Parse scheme declaration
    pub(super) fn parse_scheme(&mut self) -> Option<Scheme> {
        self.expect_contextual(contextual::SCHEME)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::LBrace)?;

        let mut members = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.at_end() {
            if let Some(member) = self.parse_scheme_member() {
                members.push(member);
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;

        Some(Scheme { name, members })
    }

    /// Parse a scheme member
    fn parse_scheme_member(&mut self) -> Option<SchemeMember> {
        if self.check(TokenKind::Virtual) {
            self.advance();
            let name = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;
            let type_expr = self.parse_type_expr()?;
            self.expect(TokenKind::Eq)?;
            let expr = self.parse_expr()?;
            Some(SchemeMember::Virtual(VirtualField {
                name,
                type_expr,
                expr,
            }))
        } else if self.check(TokenKind::Identifier) {
            let name = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;
            let type_expr = self.parse_type_expr()?;

            // Parse field instructions: .. identity, .. readonly, etc.
            let mut instructions = Vec::new();
            while self.consume(TokenKind::DotDot).is_some() {
                if let Some(instr) = self.parse_field_instruction() {
                    instructions.push(instr);
                }
            }

            Some(SchemeMember::Field(SchemeField {
                name,
                type_expr,
                instructions,
            }))
        } else {
            self.error_expected("scheme field");
            None
        }
    }

    /// Parse a field instruction (.. identity, .. default { value })
    fn parse_field_instruction(&mut self) -> Option<FieldInstruction> {
        let name = self.expect_identifier()?;

        let value = if self.consume(TokenKind::LBrace).is_some() {
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RBrace)?;
            Some(expr)
        } else {
            None
        };

        Some(FieldInstruction { name, value })
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    #[test]
    fn test_parse_scheme() {
        let result = parse(
            r#"
module test

scheme User {
    id: i32 .. identity
    name: String .. default { "unknown" }
    virtual display_name: String = name
}
"#,
        );
        assert!(!result.diagnostics.has_errors());
    }
}
