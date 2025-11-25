// Contract parser for Frel

use crate::ast::{Contract, ContractMethod};
use crate::lexer::TokenKind;

use super::Parser;

impl<'a> Parser<'a> {
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
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

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
}
