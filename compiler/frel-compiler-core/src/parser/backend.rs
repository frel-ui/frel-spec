// Backend parser for Frel
//
// Handles parsing of:
// - Backend declarations
// - Contract declarations
// - Scheme declarations
// - Enum declarations
// - Theme declarations
// - Arena declarations

use crate::ast::{
    Arena, Backend, BackendMember, Command, Contract, ContractMethod, Enum, Field,
    FieldInstruction, InstructionSet, Method, Parameter, Scheme, SchemeField, SchemeMember,
    Theme, ThemeField, ThemeMember, ThemeVariant, VirtualField, Instruction,
};
use crate::token::TokenKind;

use super::Parser;

impl<'a> Parser<'a> {
    // =========================================================================
    // Backend
    // =========================================================================

    /// Parse backend declaration
    pub(super) fn parse_backend(&mut self) -> Option<Backend> {
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

        Some(Backend {
            name,
            params,
            members,
        })
    }

    /// Parse a backend member
    fn parse_backend_member(&mut self) -> Option<BackendMember> {
        match self.current_kind() {
            TokenKind::Include => {
                self.advance();
                let name = self.expect_identifier()?;
                Some(BackendMember::Include(name))
            }
            TokenKind::Method => {
                self.advance();
                let name = self.expect_identifier()?;
                let params = self.parse_param_list()?;
                self.expect(TokenKind::Colon)?;
                let return_type = self.parse_type_expr()?;
                Some(BackendMember::Method(Method {
                    name,
                    params,
                    return_type,
                }))
            }
            TokenKind::Command => {
                self.advance();
                let name = self.expect_identifier()?;
                let params = self.parse_param_list()?;
                Some(BackendMember::Command(Command { name, params }))
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
                Some(BackendMember::Field(Field {
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

    // =========================================================================
    // Contract
    // =========================================================================

    /// Parse contract declaration
    pub(super) fn parse_contract(&mut self) -> Option<Contract> {
        self.expect(TokenKind::Contract)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::LBrace)?;

        let mut methods = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.at_end() {
            if let Some(method) = self.parse_contract_method() {
                methods.push(method);
            } else {
                self.advance();
            }
        }

        self.expect(TokenKind::RBrace)?;

        Some(Contract { name, methods })
    }

    /// Parse a contract method
    fn parse_contract_method(&mut self) -> Option<ContractMethod> {
        let name = self.expect_identifier()?;
        let params = self.parse_param_list()?;

        let return_type = if self.consume(TokenKind::Colon).is_some() {
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        Some(ContractMethod {
            name,
            params,
            return_type,
        })
    }

    // =========================================================================
    // Scheme
    // =========================================================================

    /// Parse scheme declaration
    pub(super) fn parse_scheme(&mut self) -> Option<Scheme> {
        self.expect(TokenKind::Scheme)?;
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

    // =========================================================================
    // Enum
    // =========================================================================

    /// Parse enum declaration
    pub(super) fn parse_enum(&mut self) -> Option<Enum> {
        self.expect(TokenKind::Enum)?;
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

    // =========================================================================
    // Theme
    // =========================================================================

    /// Parse theme declaration
    pub(super) fn parse_theme(&mut self) -> Option<Theme> {
        self.expect(TokenKind::Theme)?;
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

    // =========================================================================
    // Arena
    // =========================================================================

    /// Parse arena declaration
    pub(super) fn parse_arena(&mut self) -> Option<Arena> {
        self.expect(TokenKind::Arena)?;
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

    // =========================================================================
    // Common utilities
    // =========================================================================

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

    #[test]
    fn test_parse_contract() {
        let result = parse(
            r#"
module test

contract API {
    fetch(id: i32): String
    save(data: String)
}
"#,
        );
        assert!(!result.diagnostics.has_errors());
    }

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
