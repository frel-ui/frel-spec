// Lexer module for Frel
//
// This module provides tokenization of Frel source code:
// - token.rs: Token and TokenKind definitions
// - scan.rs: Lexer implementation

mod scan;
mod token;

pub use scan::Lexer;
pub use token::{Token, TokenKind};
