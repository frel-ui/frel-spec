use pest::Parser;
use pest_derive::Parser;

use crate::ast;
use crate::error::{Error, Result};

#[derive(Parser)]
#[grammar = "frel.pest"]
pub struct FrelParser;

/// Parse a Frel source file
pub fn parse_file(source: &str) -> Result<ast::File> {
    let _pairs = FrelParser::parse(Rule::file, source)
        .map_err(|e| Error::ParseError(e.to_string()))?;

    // TODO: Build AST from parse tree
    // For now, return a placeholder
    Ok(ast::File {
        module: ast::ModulePath {
            segments: vec!["placeholder".to_string()],
        },
        imports: vec![],
        declarations: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_module() {
        let source = r#"
            module test
        "#;

        let result = parse_file(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_module_with_blueprint() {
        let source = r#"
            module test

            blueprint Counter {
                count = 0

                text { "Count: ${count}" }
            }
        "#;

        let result = parse_file(source);
        assert!(result.is_ok());
    }
}
