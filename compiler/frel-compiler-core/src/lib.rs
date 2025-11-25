// Frel Core Compiler Library
//
// This crate contains the core compiler components:
// - Hand-written lexer and recursive descent parser
// - Abstract Syntax Tree (AST)
// - Diagnostic system with structured error reporting
// - Type system and type checker (TODO)
// - Semantic analysis (TODO)
//
// The compiler is language-agnostic and produces an IR that can be
// consumed by host-language specific code generation plugins.

pub mod ast;
pub mod diagnostic;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod source;
pub mod token;

pub use diagnostic::{Diagnostic, Diagnostics, Severity};
pub use error::{Error, Result};
pub use parser::ParseResult;
pub use source::{LineIndex, Span, Spanned};

/// Compiler version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parse a Frel source file and return the AST with diagnostics
pub fn parse_file(source: &str) -> ParseResult {
    parser::parse(source)
}

/// Compile a Frel source file to IR
/// Returns the AST and any diagnostics (errors, warnings)
pub fn compile(source: &str) -> ParseResult {
    // TODO: Add semantic analysis and type checking
    parse_file(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_module() {
        let result = parse_file("module test.example");
        assert!(!result.diagnostics.has_errors());
        assert!(result.file.is_some());
    }

    #[test]
    fn test_parse_complete_example() {
        let source = r#"
module test.counter

backend CounterBackend {
    count: i32 = 0
    command increment()
    command decrement()
}

blueprint Counter {
    with CounterBackend

    column {
        text { "Count" }
        button {
            on_click { increment() }
            text { "+" }
        }
    }
}
"#;
        let result = parse_file(source);
        if result.diagnostics.has_errors() {
            for diag in result.diagnostics.iter() {
                eprintln!("{}", diag.message);
            }
        }
        assert!(!result.diagnostics.has_errors());

        let file = result.file.unwrap();
        assert_eq!(file.module, "test.counter");
        assert_eq!(file.declarations.len(), 2);
    }

    #[test]
    fn test_error_reporting() {
        let source = "module test\nblueprint { }"; // Missing name
        let result = parse_file(source);
        assert!(result.diagnostics.has_errors());
    }

    #[test]
    fn test_multiple_errors() {
        let source = r#"
module test
blueprint A { @ }
blueprint B { }
"#;
        let result = parse_file(source);
        // Should report error for the @ but also parse blueprint B
        assert!(result.diagnostics.has_errors());
    }
}
