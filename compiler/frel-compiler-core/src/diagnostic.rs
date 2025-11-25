// Diagnostic system for Frel compiler
//
// This module provides structured, machine-readable diagnostics with:
// - Multiple severity levels
// - Precise source spans
// - Multi-span labels for context
// - Actionable suggestions for fixes
// - JSON serialization for AI/tooling consumption

use crate::source::{LineCol, LineIndex, Span};
use serde::{Deserialize, Serialize};

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Hint => "hint",
        }
    }
}

/// A label pointing to a span with a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub span: Span,
    pub message: String,
}

impl Label {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }

    pub fn primary(span: Span) -> Self {
        Self {
            span,
            message: String::new(),
        }
    }
}

/// A suggested fix with replacement text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub span: Span,
    pub replacement: String,
    pub message: String,
}

impl Suggestion {
    pub fn new(span: Span, replacement: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            span,
            replacement: replacement.into(),
            message: message.into(),
        }
    }

    /// Create an insertion suggestion (insert text at a point)
    pub fn insert(pos: u32, text: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            span: Span::point(pos),
            replacement: text.into(),
            message: message.into(),
        }
    }

    /// Create a deletion suggestion
    pub fn delete(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            replacement: String::new(),
            message: message.into(),
        }
    }
}

/// A single diagnostic message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<String>,
    pub message: String,
    pub span: Span,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels: Vec<Label>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub suggestions: Vec<Suggestion>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub help: Option<String>,
}

impl Diagnostic {
    /// Create a new error diagnostic
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Error,
            code: None,
            message: message.into(),
            span,
            labels: Vec::new(),
            suggestions: Vec::new(),
            help: None,
        }
    }

    /// Create a new warning diagnostic
    pub fn warning(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Warning,
            code: None,
            message: message.into(),
            span,
            labels: Vec::new(),
            suggestions: Vec::new(),
            help: None,
        }
    }

    /// Set the error code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add a label
    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    /// Add a suggestion
    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Add help text
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

/// Collection of diagnostics accumulated during compilation
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// Add a diagnostic
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Add an error
    pub fn error(&mut self, message: impl Into<String>, span: Span) {
        self.add(Diagnostic::error(message, span));
    }

    /// Add a warning
    pub fn warning(&mut self, message: impl Into<String>, span: Span) {
        self.add(Diagnostic::warning(message, span));
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Get all diagnostics
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format diagnostics for terminal output
    pub fn format_terminal(&self, source: &str, filename: &str) -> String {
        let line_index = LineIndex::new(source);
        let mut output = String::new();

        for diag in &self.diagnostics {
            output.push_str(&format_diagnostic(diag, source, filename, &line_index));
            output.push('\n');
        }

        output
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

/// Format a single diagnostic for terminal output
fn format_diagnostic(diag: &Diagnostic, source: &str, filename: &str, index: &LineIndex) -> String {
    let mut output = String::new();
    let LineCol { line, col } = index.line_col(diag.span.start);

    // Header: error[E0001]: message
    let code_str = diag
        .code
        .as_ref()
        .map(|c| format!("[{}]", c))
        .unwrap_or_default();
    output.push_str(&format!(
        "{}{}: {}\n",
        diag.severity.as_str(),
        code_str,
        diag.message
    ));

    // Location: --> file:line:col
    output.push_str(&format!("  --> {}:{}:{}\n", filename, line, col));

    // Source context
    if let Some(line_text) = index.line_text((line - 1) as usize, source) {
        let line_num_width = line.to_string().len();

        // Empty line with bar
        output.push_str(&format!("{:width$} |\n", "", width = line_num_width));

        // Line with source
        output.push_str(&format!("{} | {}\n", line, line_text));

        // Underline
        let line_start = index.line_start((line - 1) as usize).unwrap_or(0);
        let underline_start = (diag.span.start - line_start) as usize;
        let underline_len = ((diag.span.end - diag.span.start) as usize).max(1);

        output.push_str(&format!(
            "{:width$} | {:>start$}{}\n",
            "",
            "",
            "^".repeat(underline_len),
            width = line_num_width,
            start = underline_start
        ));
    }

    // Additional labels
    for label in &diag.labels {
        let LineCol {
            line: label_line,
            col: label_col,
        } = index.line_col(label.span.start);
        if !label.message.is_empty() {
            output.push_str(&format!(
                "  = note: {} (at {}:{})\n",
                label.message, label_line, label_col
            ));
        }
    }

    // Help text
    if let Some(help) = &diag.help {
        output.push_str(&format!("  = help: {}\n", help));
    }

    // Suggestions
    for suggestion in &diag.suggestions {
        if !suggestion.message.is_empty() {
            output.push_str(&format!("  = suggestion: {}\n", suggestion.message));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_builder() {
        let diag = Diagnostic::error("unexpected token", Span::new(10, 15))
            .with_code("E0001")
            .with_label(Label::new(Span::new(5, 10), "previous token here"))
            .with_help("did you forget a semicolon?");

        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.code, Some("E0001".to_string()));
        assert_eq!(diag.message, "unexpected token");
        assert_eq!(diag.labels.len(), 1);
        assert!(diag.help.is_some());
    }

    #[test]
    fn test_diagnostics_collection() {
        let mut diags = Diagnostics::new();
        diags.error("error 1", Span::new(0, 5));
        diags.warning("warning 1", Span::new(10, 15));
        diags.error("error 2", Span::new(20, 25));

        assert!(diags.has_errors());
        assert_eq!(diags.error_count(), 2);
    }

    #[test]
    fn test_json_output() {
        let mut diags = Diagnostics::new();
        diags.add(
            Diagnostic::error("test error", Span::new(0, 5))
                .with_code("E0001")
                .with_help("fix it"),
        );

        let json = diags.to_json();
        assert!(json.contains("test error"));
        assert!(json.contains("E0001"));
    }
}
