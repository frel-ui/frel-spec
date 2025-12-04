// Common parser utilities for Frel declarations
//
// Shared utilities used across multiple declaration parsers.

use crate::ast::{Expr, Instruction, InstructionExpr, Parameter};
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

    /// Parse a simple instruction (used in themes)
    pub(super) fn parse_instruction(&mut self) -> Option<Instruction> {
        let start = self.current_span().start;
        let name = self.expect_identifier()?;

        let params = if self.consume(TokenKind::LBrace).is_some() {
            let params = self.parse_instruction_params()?;
            self.expect(TokenKind::RBrace)?;
            params
        } else {
            vec![]
        };

        let end = self.previous_span().end;
        let span = crate::source::Span::new(start, end);
        Some(Instruction { name, params, span })
    }

    /// Parse an instruction expression (used in blueprints)
    ///
    /// Instruction expressions can be:
    /// - Simple: `width { 30 }`
    /// - Conditional when: `when <condition> { inst } [else { inst }]`
    /// - Ternary: `<condition> ? <inst> else <inst>`
    /// - Reference: `theme.status_badge` (field access or identifier)
    pub(super) fn parse_instruction_expr(&mut self) -> Option<InstructionExpr> {
        // Check for `when` conditional
        if self.consume(TokenKind::When).is_some() {
            return self.parse_when_instruction_expr();
        }

        // Capture start position for span tracking
        let start = self.current_span().start;

        // Parse an expression, but stop before `?` so we can handle instruction ternary
        // (which uses `? ... else` instead of expression ternary's `? ... :`)
        let expr = self.parse_expr_before_question()?;

        // Check what follows
        if self.consume(TokenKind::Question).is_some() {
            // Ternary: <condition> ? <inst> else <inst>
            let then_instr = Box::new(self.parse_instruction_expr()?);
            self.expect(TokenKind::Else)?;
            let else_instr = Box::new(self.parse_instruction_expr()?);
            return Some(InstructionExpr::Ternary {
                condition: expr,
                then_instr,
                else_instr,
            });
        }

        if self.check(TokenKind::LBrace) {
            // Simple instruction: `name { params }`
            // The expression should be an identifier
            let name = match expr {
                Expr::Identifier(name) => name,
                _ => {
                    self.error_expected("identifier for instruction name");
                    return None;
                }
            };
            self.advance(); // consume '{'
            let params = self.parse_instruction_params()?;
            self.expect(TokenKind::RBrace)?;
            let end = self.previous_span().end;
            let span = crate::source::Span::new(start, end);
            return Some(InstructionExpr::Simple(Instruction { name, params, span }));
        }

        // Reference: field access or identifier
        Some(InstructionExpr::Reference(expr))
    }

    /// Parse when instruction expression: when <condition> { inst } [else { inst }]
    fn parse_when_instruction_expr(&mut self) -> Option<InstructionExpr> {
        let condition = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;
        let then_instr = Box::new(self.parse_instruction_expr()?);
        self.expect(TokenKind::RBrace)?;

        let else_instr = if self.consume(TokenKind::Else).is_some() {
            self.expect(TokenKind::LBrace)?;
            let instr = Box::new(self.parse_instruction_expr()?);
            self.expect(TokenKind::RBrace)?;
            Some(instr)
        } else {
            None
        };

        Some(InstructionExpr::When {
            condition,
            then_instr,
            else_instr,
        })
    }

    /// Parse instruction parameters (name: value pairs or just values)
    pub(super) fn parse_instruction_params(&mut self) -> Option<Vec<(String, crate::ast::Expr)>> {
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
