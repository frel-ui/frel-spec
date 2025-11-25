// Frel Core Compiler Library
//
// This crate contains the core compiler components:
// - Lexer and Parser (using PEST)
// - Abstract Syntax Tree (AST)
// - Type system and type checker
// - Semantic analysis
//
// The compiler is language-agnostic and produces an IR that can be
// consumed by host-language specific code generation plugins.

pub mod parser;
pub mod ast;
pub mod error;

pub use error::{Error, Result};

/// Compiler version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parse a Frel source file and return the AST
pub fn parse_file(source: &str) -> Result<ast::File> {
    parser::parse_file(source)
}

/// Compile a Frel source file to IR
pub fn compile(source: &str) -> Result<ast::File> {
    // TODO: Add semantic analysis and type checking
    parse_file(source)
}
