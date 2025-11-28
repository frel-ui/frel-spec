// Terminal formatting for diagnostics
//
// This module handles the presentation of diagnostics for terminal output.
// It is separate from the core diagnostic structures to allow for different
// output formats (terminal, JSON, LSP protocol, etc.)

use super::{Diagnostic, Severity};
use crate::source::{LineCol, LineIndex};

/// Format a single diagnostic for terminal output
pub fn format_diagnostic(
    diag: &Diagnostic,
    source: &str,
    filename: &str,
    index: &LineIndex,
) -> String {
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

    // Related information (cross-file references)
    for related in &diag.related {
        let related_location = if let Some(ref file) = related.file {
            let LineCol {
                line: rel_line,
                col: rel_col,
            } = index.line_col(related.span.start);
            format!("{}:{}:{}", file, rel_line, rel_col)
        } else {
            let LineCol {
                line: rel_line,
                col: rel_col,
            } = index.line_col(related.span.start);
            format!("{}:{}", rel_line, rel_col)
        };
        output.push_str(&format!(
            "  = note: {} (at {})\n",
            related.message, related_location
        ));
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

/// Format multiple diagnostics for terminal output
pub fn format_diagnostics(
    diagnostics: &[Diagnostic],
    source: &str,
    filename: &str,
) -> String {
    let line_index = LineIndex::new(source);
    let mut output = String::new();

    for diag in diagnostics {
        output.push_str(&format_diagnostic(diag, source, filename, &line_index));
        output.push('\n');
    }

    output
}

/// Format a summary line for diagnostics
pub fn format_summary(errors: usize, warnings: usize) -> String {
    match (errors, warnings) {
        (0, 0) => String::new(),
        (0, w) => format!("warning: {} warning{} emitted\n", w, if w == 1 { "" } else { "s" }),
        (e, 0) => format!("error: could not compile due to {} error{}\n", e, if e == 1 { "" } else { "s" }),
        (e, w) => format!(
            "error: could not compile due to {} error{} and {} warning{}\n",
            e, if e == 1 { "" } else { "s" },
            w, if w == 1 { "" } else { "s" }
        ),
    }
}

/// ANSI color codes for terminal output
pub mod colors {
    use super::Severity;

    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const RED: &str = "\x1b[31m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const CYAN: &str = "\x1b[36m";
    pub const GREEN: &str = "\x1b[32m";

    /// Get the color for a severity level
    pub fn for_severity(severity: Severity) -> &'static str {
        match severity {
            Severity::Error => RED,
            Severity::Warning => YELLOW,
            Severity::Info => BLUE,
            Severity::Hint => CYAN,
        }
    }
}

/// Format a diagnostic with ANSI colors
pub fn format_diagnostic_colored(
    diag: &Diagnostic,
    source: &str,
    filename: &str,
    index: &LineIndex,
) -> String {
    let mut output = String::new();
    let LineCol { line, col } = index.line_col(diag.span.start);
    let severity_color = colors::for_severity(diag.severity);

    // Header: error[E0001]: message
    let code_str = diag
        .code
        .as_ref()
        .map(|c| format!("[{}]", c))
        .unwrap_or_default();
    output.push_str(&format!(
        "{}{}{}{}{}: {}{}{}\n",
        colors::BOLD,
        severity_color,
        diag.severity.as_str(),
        code_str,
        colors::RESET,
        colors::BOLD,
        diag.message,
        colors::RESET,
    ));

    // Location: --> file:line:col
    output.push_str(&format!(
        "  {}-->{} {}:{}:{}\n",
        colors::BLUE, colors::RESET, filename, line, col
    ));

    // Source context
    if let Some(line_text) = index.line_text((line - 1) as usize, source) {
        let line_num_width = line.to_string().len();

        // Empty line with bar
        output.push_str(&format!(
            "{}{:width$} |{}\n",
            colors::BLUE,
            "",
            colors::RESET,
            width = line_num_width
        ));

        // Line with source
        output.push_str(&format!(
            "{}{} |{} {}\n",
            colors::BLUE, line, colors::RESET, line_text
        ));

        // Underline
        let line_start = index.line_start((line - 1) as usize).unwrap_or(0);
        let underline_start = (diag.span.start - line_start) as usize;
        let underline_len = ((diag.span.end - diag.span.start) as usize).max(1);

        output.push_str(&format!(
            "{}{:width$} |{} {:>start$}{}{}{}\n",
            colors::BLUE,
            "",
            colors::RESET,
            "",
            severity_color,
            "^".repeat(underline_len),
            colors::RESET,
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
                "  {}= note:{} {} (at {}:{})\n",
                colors::CYAN, colors::RESET, label.message, label_line, label_col
            ));
        }
    }

    // Related information
    for related in &diag.related {
        let related_location = if let Some(ref file) = related.file {
            let LineCol {
                line: rel_line,
                col: rel_col,
            } = index.line_col(related.span.start);
            format!("{}:{}:{}", file, rel_line, rel_col)
        } else {
            let LineCol {
                line: rel_line,
                col: rel_col,
            } = index.line_col(related.span.start);
            format!("{}:{}", rel_line, rel_col)
        };
        output.push_str(&format!(
            "  {}= note:{} {} (at {})\n",
            colors::CYAN, colors::RESET, related.message, related_location
        ));
    }

    // Help text
    if let Some(help) = &diag.help {
        output.push_str(&format!(
            "  {}= help:{} {}\n",
            colors::GREEN, colors::RESET, help
        ));
    }

    // Suggestions
    for suggestion in &diag.suggestions {
        if !suggestion.message.is_empty() {
            output.push_str(&format!(
                "  {}= suggestion:{} {}\n",
                colors::GREEN, colors::RESET, suggestion.message
            ));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::Span;

    #[test]
    fn test_format_summary() {
        assert_eq!(format_summary(0, 0), "");
        assert_eq!(format_summary(1, 0), "error: could not compile due to 1 error\n");
        assert_eq!(format_summary(2, 0), "error: could not compile due to 2 errors\n");
        assert_eq!(format_summary(0, 1), "warning: 1 warning emitted\n");
        assert_eq!(format_summary(0, 2), "warning: 2 warnings emitted\n");
        assert_eq!(
            format_summary(1, 1),
            "error: could not compile due to 1 error and 1 warning\n"
        );
    }

    #[test]
    fn test_format_diagnostic() {
        let source = "blueprint Test { }";
        let diag = Diagnostic::error("test error", Span::new(10, 14))
            .with_code("E0201");
        let index = LineIndex::new(source);

        let output = format_diagnostic(&diag, source, "test.frel", &index);
        assert!(output.contains("error[E0201]: test error"));
        assert!(output.contains("--> test.frel:1:11"));
    }
}
