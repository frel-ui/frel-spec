// DUMP Format Output for Frel Semantic Analysis Result
//
// This module implements a human-readable DUMP format for the SemanticResult.
// The format is indentation-based and suitable for debugging and review.

use super::scope::{ScopeGraph, ScopeId};
use super::symbol::SymbolId;
use super::types::Type;
use super::SemanticResult;
use std::collections::HashMap;

/// Dump a SemanticResult to a human-readable string
pub fn dump(result: &SemanticResult) -> String {
    let dumper = SemanticDumper::new(result);
    dumper.dump()
}

struct SemanticDumper<'a> {
    result: &'a SemanticResult,
    output: String,
    indent: usize,
}

impl<'a> SemanticDumper<'a> {
    fn new(result: &'a SemanticResult) -> Self {
        Self {
            result,
            output: String::new(),
            indent: 0,
        }
    }

    fn dump(mut self) -> String {
        // Header with statistics
        self.write(&format!(
            "SEMANTIC_RESULT scopes={} symbols={} errors={}",
            self.result.scopes.len(),
            self.result.symbols.len(),
            self.result.diagnostics.error_count()
        ));

        // Dump scopes section
        self.write("");
        self.write("SCOPES");
        self.indent();
        self.dump_scopes();
        self.dedent();

        // Dump symbols section
        self.write("");
        self.write("SYMBOLS");
        self.indent();
        self.dump_symbols();
        self.dedent();

        // Dump type resolutions section (only if there are any)
        if !self.result.type_resolutions.is_empty() {
            self.write("");
            self.write("TYPE_RESOLUTIONS");
            self.indent();
            self.dump_type_resolutions();
            self.dedent();
        }

        // Dump expression types section (only if there are any)
        if !self.result.expr_types.is_empty() {
            self.write("");
            self.write("EXPR_TYPES");
            self.indent();
            self.dump_expr_types();
            self.dedent();
        }

        // Dump diagnostics section (only if there are any)
        if !self.result.diagnostics.is_empty() {
            self.write("");
            self.write("DIAGNOSTICS");
            self.indent();
            self.dump_diagnostics();
            self.dedent();
        }

        self.output
    }

    fn write(&mut self, text: &str) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
        self.output.push_str(text);
        self.output.push('\n');
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    fn dump_scopes(&mut self) {
        // Dump scopes hierarchically starting from root
        if self.result.scopes.len() > 0 {
            self.dump_scope_tree(ScopeId::ROOT, &self.result.scopes);
        }
    }

    fn dump_scope_tree(&mut self, scope_id: ScopeId, scopes: &ScopeGraph) {
        let scope = match scopes.get(scope_id) {
            Some(s) => s,
            None => return,
        };

        let name_part = scope
            .name
            .as_ref()
            .map(|n| format!(" name={}", n))
            .unwrap_or_default();

        self.write(&format!(
            "SCOPE #{} kind={}{} span={}..{}",
            scope_id.0,
            scope.kind.as_str(),
            name_part,
            scope.span.start,
            scope.span.end
        ));

        // Dump children
        if !scope.children.is_empty() {
            self.indent();
            for &child_id in &scope.children {
                self.dump_scope_tree(child_id, scopes);
            }
            self.dedent();
        }
    }

    fn dump_symbols(&mut self) {
        // Group symbols by scope for readability
        let mut by_scope: HashMap<ScopeId, Vec<SymbolId>> = HashMap::new();
        for symbol in self.result.symbols.iter() {
            by_scope.entry(symbol.scope).or_default().push(symbol.id);
        }

        // Sort scope IDs for consistent output
        let mut scope_ids: Vec<_> = by_scope.keys().copied().collect();
        scope_ids.sort_by_key(|id| id.0);

        for scope_id in scope_ids {
            let scope_name = self
                .result
                .scopes
                .get(scope_id)
                .and_then(|s| s.name.as_ref())
                .map(|n| format!(" ({})", n))
                .unwrap_or_default();

            self.write(&format!("IN SCOPE #{}{}", scope_id.0, scope_name));
            self.indent();

            let symbol_ids = by_scope.get(&scope_id).unwrap();
            for &sym_id in symbol_ids {
                if let Some(symbol) = self.result.symbols.get(sym_id) {
                    let body_scope = symbol
                        .body_scope
                        .map(|id| format!(" body_scope=#{}", id.0))
                        .unwrap_or_default();

                    self.write(&format!(
                        "#{} {} \"{}\" span={}..{}{}",
                        sym_id.0,
                        symbol.kind.as_str().to_uppercase(),
                        symbol.name,
                        symbol.def_span.start,
                        symbol.def_span.end,
                        body_scope
                    ));
                }
            }

            self.dedent();
        }
    }

    fn dump_type_resolutions(&mut self) {
        // Sort by span start for consistent output
        let mut resolutions: Vec<_> = self.result.type_resolutions.iter().collect();
        resolutions.sort_by_key(|(span, _)| span.start);

        for (span, ty) in resolutions {
            self.write(&format!(
                "{}..{} -> {}",
                span.start,
                span.end,
                self.format_type(ty)
            ));
        }
    }

    fn dump_expr_types(&mut self) {
        // Sort by span start for consistent output
        let mut expr_types: Vec<_> = self.result.expr_types.iter().collect();
        expr_types.sort_by_key(|(span, _)| span.start);

        for (span, ty) in expr_types {
            self.write(&format!(
                "{}..{} -> {}",
                span.start,
                span.end,
                self.format_type(ty)
            ));
        }
    }

    fn dump_diagnostics(&mut self) {
        for diag in self.result.diagnostics.iter() {
            let code = diag.code.as_deref().unwrap_or("E????");
            self.write(&format!(
                "[{}] {} at {}..{}",
                code, diag.message, diag.span.start, diag.span.end
            ));
        }
    }

    fn format_type(&self, ty: &Type) -> String {
        match ty {
            Type::Scheme(id) => self.format_composite_type("scheme", *id),
            Type::Backend(id) => self.format_composite_type("backend", *id),
            Type::Blueprint(id) => self.format_composite_type("blueprint", *id),
            Type::Contract(id) => self.format_composite_type("contract", *id),
            Type::Theme(id) => self.format_composite_type("theme", *id),
            Type::Enum(id) => self.format_composite_type("enum", *id),
            Type::Nullable(inner) => format!("{}?", self.format_type(inner)),
            Type::Ref(inner) => format!("ref {}", self.format_type(inner)),
            Type::Draft(inner) => format!("draft {}", self.format_type(inner)),
            Type::Asset(inner) => format!("asset {}", self.format_type(inner)),
            Type::List(elem) => format!("[{}]", self.format_type(elem)),
            Type::Set(elem) => format!("set<{}>", self.format_type(elem)),
            Type::Map(k, v) => format!("map<{}, {}>", self.format_type(k), self.format_type(v)),
            Type::Tree(elem) => format!("tree<{}>", self.format_type(elem)),
            Type::Function { params, ret } => {
                let param_strs: Vec<_> = params.iter().map(|p| self.format_type(p)).collect();
                format!("fn({}) -> {}", param_strs.join(", "), self.format_type(ret))
            }
            Type::BlueprintInstance { blueprint, params } => {
                let param_strs: Vec<_> = params.iter().map(|p| self.format_type(p)).collect();
                format!(
                    "{}({})",
                    self.format_composite_type("blueprint", *blueprint),
                    param_strs.join(", ")
                )
            }
            Type::Accessor(inner) => format!("accessor<{}>", self.format_type(inner)),
            // Simple types use Display
            _ => ty.to_string(),
        }
    }

    fn format_composite_type(&self, kind: &str, id: SymbolId) -> String {
        if let Some(symbol) = self.result.symbols.get(id) {
            format!("{} \"{}\"", kind, symbol.name)
        } else {
            format!("{}#{}", kind, id.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::semantic::analyze;

    fn analyze_and_dump(source: &str) -> String {
        let parse_result = parser::parse(source);
        assert!(
            !parse_result.diagnostics.has_errors(),
            "Parse errors: {:?}",
            parse_result.diagnostics
        );
        let result = analyze(&parse_result.file.unwrap());
        dump(&result)
    }

    #[test]
    fn test_dump_simple() {
        let source = r#"
module test

backend Counter {
    count: i32 = 0
}
"#;
        let output = analyze_and_dump(source);

        assert!(output.contains("SEMANTIC_RESULT"));
        assert!(output.contains("SCOPES"));
        assert!(output.contains("SYMBOLS"));
        assert!(output.contains("BACKEND"));
        assert!(output.contains("Counter"));
        assert!(output.contains("count"));
    }

    #[test]
    fn test_dump_with_types() {
        let source = r#"
module test

scheme Item {
    id: i64
    name: String
}
"#;
        let output = analyze_and_dump(source);

        assert!(output.contains("SCHEME"));
        assert!(output.contains("Item"));
        assert!(output.contains("FIELD"));
        assert!(output.contains("id"));
        assert!(output.contains("name"));
    }

    #[test]
    fn test_dump_with_errors() {
        let source = r#"
module test

backend A { }
backend A { }
"#;
        let parse_result = parser::parse(source);
        let result = analyze(&parse_result.file.unwrap());
        let output = dump(&result);

        assert!(output.contains("errors=1"));
        assert!(output.contains("DIAGNOSTICS"));
    }
}
