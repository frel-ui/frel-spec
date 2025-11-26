// Backend parser for Frel

use crate::ast::{FaBackend, FaBackendMember, FaCommand, FaField, FaMethod};
use crate::lexer::TokenKind;

use super::Parser;

impl<'a> Parser<'a> {
    /// Parse backend declaration
    pub(super) fn parse_backend(&mut self) -> Option<FaBackend> {
        self.expect(TokenKind::Backend)?;
        let name = self.expect_identifier()?;
        let params = self.parse_param_list_opt()?;
        self.expect(TokenKind::LBrace)?;

        let mut members = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.at_end() {
            if let Some(member) = self.parse_backend_member() {
                members.push(member);
            } else {
                // Error recovery: skip to next member or closing brace
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;

        Some(FaBackend {
            name,
            params,
            members,
        })
    }

    /// Parse a backend member
    fn parse_backend_member(&mut self) -> Option<FaBackendMember> {
        match self.current_kind() {
            TokenKind::Include => {
                self.advance();
                let name = self.expect_identifier()?;
                Some(FaBackendMember::Include(name))
            }
            TokenKind::Method => {
                self.advance();
                let name = self.expect_identifier()?;
                let params = self.parse_param_list()?;
                self.expect(TokenKind::Colon)?;
                let return_type = self.parse_type_expr()?;
                Some(FaBackendMember::Method(FaMethod {
                    name,
                    params,
                    return_type,
                }))
            }
            TokenKind::Command => {
                self.advance();
                let name = self.expect_identifier()?;
                let params = self.parse_param_list()?;
                Some(FaBackendMember::Command(FaCommand { name, params }))
            }
            TokenKind::Identifier => {
                // Field: name : type [= init]
                let name = self.expect_identifier()?;
                self.expect(TokenKind::Colon)?;
                let type_expr = self.parse_type_expr()?;
                let init = if self.consume(TokenKind::Eq).is_some() {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                Some(FaBackendMember::Field(FaField {
                    name,
                    type_expr,
                    init,
                }))
            }
            _ => {
                self.error_expected("backend member (field, method, command, or include)");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    #[test]
    fn test_parse_backend() {
        let result = parse(
            r#"
module test

backend Counter {
    count: i32 = 0
    method increment(): i32
    command reset()
}
"#,
        );
        assert!(!result.diagnostics.has_errors());
        let file = result.file.unwrap();
        assert_eq!(file.declarations.len(), 1);
    }
}
