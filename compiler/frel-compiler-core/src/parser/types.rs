// Type expression parser for Frel
//
// Handles parsing of type expressions including:
// - Named types (String, i32, UserType)
// - Nullable types (T?)
// - Reference types (ref T, draft T)
// - Collection types (List<T>, Map<K, V>)
// - Blueprint types (Blueprint<P1, P2>)

use crate::ast::TypeExpr;
use crate::token::TokenKind;

use super::Parser;

impl<'a> Parser<'a> {
    /// Parse a type expression
    pub(super) fn parse_type_expr(&mut self) -> Option<TypeExpr> {
        let base = self.parse_type_base()?;

        // Check for nullable suffix
        if self.consume(TokenKind::Question).is_some() {
            Some(TypeExpr::Nullable(Box::new(base)))
        } else {
            Some(base)
        }
    }

    /// Parse the base type (before nullable modifier)
    fn parse_type_base(&mut self) -> Option<TypeExpr> {
        match self.current_kind() {
            TokenKind::Ref => {
                self.advance();
                let inner = self.parse_type_base()?;
                Some(TypeExpr::Ref(Box::new(inner)))
            }
            TokenKind::Draft => {
                self.advance();
                let inner = self.parse_type_base()?;
                Some(TypeExpr::Draft(Box::new(inner)))
            }
            TokenKind::Asset => {
                self.advance();
                let inner = self.parse_type_base()?;
                Some(TypeExpr::Asset(Box::new(inner)))
            }
            TokenKind::Identifier => {
                let name = self.current_text().to_string();
                self.advance();

                // Handle built-in generic types by name
                match name.as_str() {
                    "Blueprint" => {
                        if self.consume(TokenKind::Lt).is_some() {
                            let params = self.parse_type_list()?;
                            self.expect(TokenKind::Gt)?;
                            Some(TypeExpr::Blueprint(params))
                        } else {
                            Some(TypeExpr::Blueprint(vec![]))
                        }
                    }
                    "List" => {
                        self.expect(TokenKind::Lt)?;
                        let elem = self.parse_type_expr()?;
                        self.expect(TokenKind::Gt)?;
                        Some(TypeExpr::List(Box::new(elem)))
                    }
                    "Set" => {
                        if self.consume(TokenKind::Lt).is_some() {
                            let elem = self.parse_type_expr()?;
                            self.expect(TokenKind::Gt)?;
                            Some(TypeExpr::Set(Box::new(elem)))
                        } else {
                            // 'Set' without <> is just a named type
                            Some(TypeExpr::Named(name))
                        }
                    }
                    "Map" => {
                        self.expect(TokenKind::Lt)?;
                        let key = self.parse_type_expr()?;
                        self.expect(TokenKind::Comma)?;
                        let value = self.parse_type_expr()?;
                        self.expect(TokenKind::Gt)?;
                        Some(TypeExpr::Map(Box::new(key), Box::new(value)))
                    }
                    "Tree" => {
                        self.expect(TokenKind::Lt)?;
                        let elem = self.parse_type_expr()?;
                        self.expect(TokenKind::Gt)?;
                        Some(TypeExpr::Tree(Box::new(elem)))
                    }
                    "Accessor" => {
                        self.expect(TokenKind::Lt)?;
                        let elem = self.parse_type_expr()?;
                        self.expect(TokenKind::Gt)?;
                        Some(TypeExpr::Accessor(Box::new(elem)))
                    }
                    _ => Some(TypeExpr::Named(name)),
                }
            }
            _ => {
                self.error_expected("type");
                None
            }
        }
    }

    /// Parse a comma-separated list of types
    fn parse_type_list(&mut self) -> Option<Vec<TypeExpr>> {
        let mut types = vec![self.parse_type_expr()?];

        while self.consume(TokenKind::Comma).is_some() {
            types.push(self.parse_type_expr()?);
        }

        Some(types)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    fn parse_type(source: &str) -> Option<crate::ast::TypeExpr> {
        // Wrap in a minimal backend field to test type parsing
        let full_source = format!("module test\nbackend Test {{ x: {} }}", source);
        let result = parse(&full_source);
        if result.diagnostics.has_errors() {
            return None;
        }
        let file = result.file?;
        if let crate::ast::TopLevelDecl::Backend(backend) = &file.declarations[0] {
            if let crate::ast::BackendMember::Field(field) = &backend.members[0] {
                return Some(field.type_expr.clone());
            }
        }
        None
    }

    #[test]
    fn test_named_type() {
        let t = parse_type("String").unwrap();
        assert!(matches!(t, crate::ast::TypeExpr::Named(s) if s == "String"));
    }

    #[test]
    fn test_nullable_type() {
        let t = parse_type("String?").unwrap();
        assert!(matches!(t, crate::ast::TypeExpr::Nullable(_)));
    }

    #[test]
    fn test_ref_type() {
        let t = parse_type("ref User").unwrap();
        assert!(matches!(t, crate::ast::TypeExpr::Ref(_)));
    }

    #[test]
    fn test_list_type() {
        let t = parse_type("List<String>").unwrap();
        assert!(matches!(t, crate::ast::TypeExpr::List(_)));
    }

    #[test]
    fn test_map_type() {
        let t = parse_type("Map<String, i32>").unwrap();
        assert!(matches!(t, crate::ast::TypeExpr::Map(_, _)));
    }

    #[test]
    fn test_complex_type() {
        let t = parse_type("List<ref User>?").unwrap();
        if let crate::ast::TypeExpr::Nullable(inner) = t {
            assert!(matches!(*inner, crate::ast::TypeExpr::List(_)));
        } else {
            panic!("Expected Nullable");
        }
    }
}
