// Contract parser for Frel

use crate::ast::{Contract, ContractMethod};
use crate::lexer::token::contextual;
use crate::lexer::TokenKind;

use super::Parser;

impl<'a> Parser<'a> {
    /// Parse contract declaration
    pub(super) fn parse_contract(&mut self) -> Option<Contract> {
        let start = self.current_span().start;
        self.expect_contextual(contextual::CONTRACT)?;
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

        let end_span = self.current_span();
        self.expect(TokenKind::RBrace)?;

        let span = crate::source::Span::new(start, end_span.end);
        Some(Contract { name, methods, span })
    }

    /// Parse a contract method
    fn parse_contract_method(&mut self) -> Option<ContractMethod> {
        let start = self.current_span().start;
        let name = self.expect_identifier()?;
        let params = self.parse_param_list()?;

        let return_type = if self.consume(TokenKind::Colon).is_some() {
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        let span = crate::source::Span::new(start, self.previous_span().end);
        Some(ContractMethod {
            name,
            params,
            return_type,
            span,
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
