// Theme parser for Frel

use crate::ast::{InstructionSet, Theme, ThemeField, ThemeMember, ThemeVariant};
use crate::lexer::token::contextual;
use crate::lexer::TokenKind;

use super::Parser;

impl<'a> Parser<'a> {
    /// Parse theme declaration
    pub(super) fn parse_theme(&mut self) -> Option<Theme> {
        self.expect_contextual(contextual::THEME)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::LBrace)?;

        let mut members = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.at_end() {
            if let Some(member) = self.parse_theme_member() {
                members.push(member);
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;

        Some(Theme { name, members })
    }

    /// Parse a theme member
    fn parse_theme_member(&mut self) -> Option<ThemeMember> {
        match self.current_kind() {
            TokenKind::Include => {
                self.advance();
                let name = self.expect_identifier()?;
                Some(ThemeMember::Include(name))
            }
            TokenKind::Set => {
                self.advance();
                let name = self.expect_identifier()?;
                self.expect(TokenKind::LBrace)?;

                let mut instructions = Vec::new();
                while !self.check(TokenKind::RBrace) && !self.at_end() {
                    if let Some(instr) = self.parse_instruction() {
                        instructions.push(instr);
                    } else {
                        self.advance();
                    }
                }

                self.expect(TokenKind::RBrace)?;

                Some(ThemeMember::InstructionSet(InstructionSet {
                    name,
                    instructions,
                }))
            }
            TokenKind::Variant => {
                self.advance();
                let name = self.expect_identifier()?;
                self.expect(TokenKind::LBrace)?;

                let mut overrides = Vec::new();
                while !self.check(TokenKind::RBrace) && !self.at_end() {
                    let field_name = self.expect_identifier()?;
                    self.expect(TokenKind::Eq)?;
                    let value = self.parse_expr()?;
                    overrides.push((field_name, value));
                }

                self.expect(TokenKind::RBrace)?;

                Some(ThemeMember::Variant(ThemeVariant { name, overrides }))
            }
            TokenKind::Identifier => {
                // Field: name : [asset] type [= init]
                let name = self.expect_identifier()?;
                self.expect(TokenKind::Colon)?;

                let is_asset = self.consume(TokenKind::Asset).is_some();
                let type_expr = self.parse_type_expr()?;

                let init = if self.consume(TokenKind::Eq).is_some() {
                    Some(self.parse_expr()?)
                } else {
                    None
                };

                Some(ThemeMember::Field(ThemeField {
                    name,
                    is_asset,
                    type_expr,
                    init,
                }))
            }
            _ => {
                self.error_expected("theme member");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    #[test]
    fn test_parse_theme() {
        let result = parse(
            r#"
module test

theme MyTheme {
    primary_color: asset Color
    padding: i32 = 16

    set button_style {
        padding { 8 }
    }

    variant Dark {
        primary_color = 0x000000
    }
}
"#,
        );
        assert!(!result.diagnostics.has_errors());
    }
}
