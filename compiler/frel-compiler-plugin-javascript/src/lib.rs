// Frel JavaScript Code Generation Plugin
//
// This crate implements JavaScript code generation from Frel AST.
// It produces ES6 modules that can run in modern JavaScript environments.

use frel_compiler_core::ast;

pub mod codegen;

/// Generate JavaScript code from a Frel AST
pub fn generate(file: &ast::File) -> String {
    codegen::generate_file(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_empty_module() {
        let file = ast::File {
            module: ast::ModulePath {
                segments: vec!["test".to_string()],
            },
            imports: vec![],
            declarations: vec![],
        };

        let output = generate(&file);
        assert!(output.contains("// Module: test"));
    }
}
