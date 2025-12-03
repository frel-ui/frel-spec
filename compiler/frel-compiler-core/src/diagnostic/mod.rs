// Diagnostic system for Frel compiler
//
// This module provides structured, machine-readable diagnostics with:
// - Stable error codes (see codes.rs)
// - Multiple severity levels
// - Precise source spans
// - Multi-span labels for context
// - Actionable suggestions for fixes
// - Related information for cross-file references
// - LSP-compatible tags for IDE integration
// - JSON serialization for tooling consumption
// - Output-agnostic design via DiagnosticSink trait

pub mod codes;
pub mod format;
pub mod sink;

use crate::source::{LineIndex, Span};
use serde::{Deserialize, Serialize};

pub use codes::{Category, ErrorCode};
pub use format::{format_diagnostic, format_diagnostic_colored, format_diagnostics, format_summary};
pub use sink::{CollectingSink, CountingSink, DiagnosticSink, NullSink, StreamingSink};

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

/// Related information pointing to another location
///
/// Used for "defined here", "previous occurrence", etc.
/// Supports cross-file references via the optional file field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedInfo {
    pub span: Span,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub file: Option<String>,
    pub message: String,
}

impl RelatedInfo {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            file: None,
            message: message.into(),
        }
    }

    pub fn in_file(span: Span, file: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            span,
            file: Some(file.into()),
            message: message.into(),
        }
    }
}

/// Diagnostic tags for IDE presentation hints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticTag {
    /// The code is unnecessary (rendered grayed out)
    Unnecessary,
    /// The code is deprecated (rendered with strikethrough)
    Deprecated,
}

/// A single diagnostic message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub code: Option<String>,
    pub message: String,
    pub span: Span,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub labels: Vec<Label>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub suggestions: Vec<Suggestion>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub help: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub related: Vec<RelatedInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<DiagnosticTag>,
    /// Custom data for code actions (LSP)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub data: Option<serde_json::Value>,
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
            related: Vec::new(),
            tags: Vec::new(),
            data: None,
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
            related: Vec::new(),
            tags: Vec::new(),
            data: None,
        }
    }

    /// Create a new info diagnostic
    pub fn info(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Info,
            code: None,
            message: message.into(),
            span,
            labels: Vec::new(),
            suggestions: Vec::new(),
            help: None,
            related: Vec::new(),
            tags: Vec::new(),
            data: None,
        }
    }

    /// Create a new hint diagnostic
    pub fn hint(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Hint,
            code: None,
            message: message.into(),
            span,
            labels: Vec::new(),
            suggestions: Vec::new(),
            help: None,
            related: Vec::new(),
            tags: Vec::new(),
            data: None,
        }
    }

    /// Create a diagnostic from an ErrorCode
    pub fn from_code(code: &ErrorCode, span: Span, message: impl Into<String>) -> Self {
        Self {
            severity: code.default_severity,
            code: Some(code.code.to_string()),
            message: message.into(),
            span,
            labels: Vec::new(),
            suggestions: Vec::new(),
            help: None,
            related: Vec::new(),
            tags: Vec::new(),
            data: None,
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

    /// Add multiple labels
    pub fn with_labels(mut self, labels: impl IntoIterator<Item = Label>) -> Self {
        self.labels.extend(labels);
        self
    }

    /// Add a suggestion
    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Add multiple suggestions
    pub fn with_suggestions(mut self, suggestions: impl IntoIterator<Item = Suggestion>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }

    /// Add help text
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Add related information
    pub fn with_related(mut self, related: RelatedInfo) -> Self {
        self.related.push(related);
        self
    }

    /// Add multiple related info
    pub fn with_related_all(mut self, related: impl IntoIterator<Item = RelatedInfo>) -> Self {
        self.related.extend(related);
        self
    }

    /// Add a diagnostic tag
    pub fn with_tag(mut self, tag: DiagnosticTag) -> Self {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self
    }

    /// Set custom data for code actions
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Mark as unnecessary (grayed out in IDE)
    pub fn unnecessary(self) -> Self {
        self.with_tag(DiagnosticTag::Unnecessary)
    }

    /// Mark as deprecated (strikethrough in IDE)
    pub fn deprecated(self) -> Self {
        self.with_tag(DiagnosticTag::Deprecated)
    }
}

/// Collection of diagnostics accumulated during compilation
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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

    /// Get the number of warnings
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Get the total number of diagnostics
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Get all diagnostics
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }

    /// Get diagnostics as a slice
    pub fn as_slice(&self) -> &[Diagnostic] {
        &self.diagnostics
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

    /// Format diagnostics with colors for terminal output
    pub fn format_terminal_colored(&self, source: &str, filename: &str) -> String {
        let line_index = LineIndex::new(source);
        let mut output = String::new();

        for diag in &self.diagnostics {
            output.push_str(&format_diagnostic_colored(diag, source, filename, &line_index));
            output.push('\n');
        }

        // Add summary
        output.push_str(&format_summary(self.error_count(), self.warning_count()));

        output
    }

    /// Merge another diagnostics collection into this one
    pub fn merge(&mut self, other: Diagnostics) {
        self.diagnostics.extend(other.diagnostics);
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

impl<'a> IntoIterator for &'a Diagnostics {
    type Item = &'a Diagnostic;
    type IntoIter = std::slice::Iter<'a, Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.iter()
    }
}

impl FromIterator<Diagnostic> for Diagnostics {
    fn from_iter<I: IntoIterator<Item = Diagnostic>>(iter: I) -> Self {
        Self {
            diagnostics: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_builder() {
        let diag = Diagnostic::error("unexpected token", Span::new(10, 15))
            .with_code("E0201")
            .with_label(Label::new(Span::new(5, 10), "previous token here"))
            .with_help("did you forget a semicolon?");

        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.code, Some("E0201".to_string()));
        assert_eq!(diag.message, "unexpected token");
        assert_eq!(diag.labels.len(), 1);
        assert!(diag.help.is_some());
    }

    #[test]
    fn test_diagnostic_from_code() {
        let diag = Diagnostic::from_code(&codes::E0301, Span::new(0, 5), "cannot find `foo`");

        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.code, Some("E0301".to_string()));
        assert_eq!(diag.message, "cannot find `foo`");
    }

    #[test]
    fn test_diagnostic_with_related() {
        let diag = Diagnostic::error("duplicate definition", Span::new(100, 110))
            .with_code("E0302")
            .with_related(RelatedInfo::new(Span::new(50, 60), "previously defined here"));

        assert_eq!(diag.related.len(), 1);
        assert_eq!(diag.related[0].message, "previously defined here");
    }

    #[test]
    fn test_diagnostic_tags() {
        let diag = Diagnostic::warning("unused variable", Span::new(0, 5))
            .unnecessary()
            .deprecated();

        assert!(diag.tags.contains(&DiagnosticTag::Unnecessary));
        assert!(diag.tags.contains(&DiagnosticTag::Deprecated));
    }

    #[test]
    fn test_diagnostics_collection() {
        let mut diags = Diagnostics::new();
        diags.error("error 1", Span::new(0, 5));
        diags.warning("warning 1", Span::new(10, 15));
        diags.error("error 2", Span::new(20, 25));

        assert!(diags.has_errors());
        assert_eq!(diags.error_count(), 2);
        assert_eq!(diags.warning_count(), 1);
        assert_eq!(diags.len(), 3);
    }

    #[test]
    fn test_json_output() {
        let mut diags = Diagnostics::new();
        diags.add(
            Diagnostic::error("test error", Span::new(0, 5))
                .with_code("E0201")
                .with_help("fix it"),
        );

        let json = diags.to_json();
        assert!(json.contains("test error"));
        assert!(json.contains("E0201"));
    }

    #[test]
    fn test_diagnostics_merge() {
        let mut diags1 = Diagnostics::new();
        diags1.error("error 1", Span::new(0, 5));

        let mut diags2 = Diagnostics::new();
        diags2.warning("warning 1", Span::new(10, 15));

        diags1.merge(diags2);
        assert_eq!(diags1.len(), 2);
    }
}
