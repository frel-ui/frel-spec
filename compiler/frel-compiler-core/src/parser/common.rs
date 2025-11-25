// Common parser utilities for Frel declarations
//
// Shared utilities used across multiple declaration parsers.

use crate::ast::{Instruction, Parameter};
use crate::lexer::TokenKind;

use super::Parser;

impl<'a> Parser<'a> {
    /// Parse optional parameter list (may be absent)
    pub(super) fn parse_param_list_opt(&mut self) -> Option<Vec<Parameter>> {
        if self.consume(TokenKind::LParen).is_some() {
            let params = self.parse_params()?;
            self.expect(TokenKind::RParen)?;
            Some(params)
        } else {
            Some(vec![])
        }
    }

    /// Parse required parameter list
    pub(super) fn parse_param_list(&mut self) -> Option<Vec<Parameter>> {
        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;
        Some(params)
    }

    /// Parse parameter list contents
    fn parse_params(&mut self) -> Option<Vec<Parameter>> {
        if self.check(TokenKind::RParen) {
            return Some(vec![]);
        }

        let mut params = vec![self.parse_param()?];

        while self.consume(TokenKind::Comma).is_some() {
            if self.check(TokenKind::RParen) {
                break; // Trailing comma
            }
            params.push(self.parse_param()?);
        }

        Some(params)
    }

    /// Parse a single parameter
    fn parse_param(&mut self) -> Option<Parameter> {
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Colon)?;
        let type_expr = self.parse_type_expr()?;

        let default = if self.consume(TokenKind::Eq).is_some() {
            Some(self.parse_expr()?)
        } else {
            None
        };

        Some(Parameter {
            name,
            type_expr,
            default,
        })
    }

    /// Parse an instruction (used in themes and blueprints)
    pub(super) fn parse_instruction(&mut self) -> Option<Instruction> {
        let name = self.expect_identifier()?;

        let params = if self.consume(TokenKind::LBrace).is_some() {
            let params = self.parse_instruction_params()?;
            self.expect(TokenKind::RBrace)?;
            params
        } else {
            vec![]
        };

        Some(Instruction { name, params })
    }

    /// Parse instruction parameters (name: value pairs or just values)
    fn parse_instruction_params(&mut self) -> Option<Vec<(String, crate::ast::Expr)>> {
        if self.check(TokenKind::RBrace) {
            return Some(vec![]);
        }

        let mut params = Vec::new();

        // Check if it's named or positional
        if self.check(TokenKind::Identifier) {
            if let Some(next) = self.peek() {
                if next.kind == TokenKind::Colon {
                    // Named parameters
                    loop {
                        let name = self.expect_identifier()?;
                        self.expect(TokenKind::Colon)?;
                        let value = self.parse_expr()?;
                        params.push((name, value));

                        if !self.check(TokenKind::Identifier) || self.check(TokenKind::RBrace) {
                            break;
                        }
                    }
                    return Some(params);
                }
            }
        }

        // Single value (shorthand)
        let value = self.parse_expr()?;
        params.push(("value".to_string(), value));

        Some(params)
    }
}
