// Source location and span tracking for Frel compiler
//
// This module provides types for tracking source locations and spans,
// enabling precise error reporting with source context.

use serde::{Deserialize, Serialize};

/// A span representing a range of bytes in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// Start byte offset (inclusive)
    pub start: u32,
    /// End byte offset (exclusive)
    pub end: u32,
}

impl Span {
    /// Create a new span
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Create a span of length zero at a position (for insertion points)
    pub fn point(pos: u32) -> Self {
        Self { start: pos, end: pos }
    }

    /// Create a span covering two spans (from start of first to end of second)
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Check if this span is empty (zero length)
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Get the length of the span in bytes
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Extract the text this span covers from source
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start as usize..self.end as usize]
    }

    /// Check if this span is the default (0..0)
    /// Used for serde skip_serializing_if
    pub fn is_default(&self) -> bool {
        self.start == 0 && self.end == 0
    }
}

impl Default for Span {
    fn default() -> Self {
        Self { start: 0, end: 0 }
    }
}

/// A value with an associated source span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }
}

/// Line and column information for human-readable error messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineCol {
    /// 1-indexed line number
    pub line: u32,
    /// 1-indexed column number (in characters, not bytes)
    pub col: u32,
}

/// Index for converting byte offsets to line/column positions
pub struct LineIndex {
    /// Byte offset of the start of each line
    line_starts: Vec<u32>,
}

impl LineIndex {
    /// Build a line index from source text
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, c) in source.char_indices() {
            if c == '\n' {
                line_starts.push((i + 1) as u32);
            }
        }
        Self { line_starts }
    }

    /// Convert a byte offset to line/column
    pub fn line_col(&self, offset: u32) -> LineCol {
        let line = self
            .line_starts
            .partition_point(|&start| start <= offset)
            .saturating_sub(1);
        let line_start = self.line_starts[line];
        LineCol {
            line: (line + 1) as u32,
            col: (offset - line_start + 1),
        }
    }

    /// Get the byte offset of a line start (0-indexed line number)
    pub fn line_start(&self, line: usize) -> Option<u32> {
        self.line_starts.get(line).copied()
    }

    /// Get the byte offset of a line end (0-indexed line number)
    pub fn line_end(&self, line: usize, source: &str) -> Option<u32> {
        if line + 1 < self.line_starts.len() {
            // Not the last line - end is start of next line minus newline
            Some(self.line_starts[line + 1] - 1)
        } else if line < self.line_starts.len() {
            // Last line - end is end of source
            Some(source.len() as u32)
        } else {
            None
        }
    }

    /// Get the text of a specific line (0-indexed)
    pub fn line_text<'a>(&self, line: usize, source: &'a str) -> Option<&'a str> {
        let start = self.line_start(line)? as usize;
        let end = self.line_end(line, source)? as usize;
        Some(&source[start..end])
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_merge() {
        let a = Span::new(10, 20);
        let b = Span::new(15, 30);
        let merged = a.merge(b);
        assert_eq!(merged.start, 10);
        assert_eq!(merged.end, 30);
    }

    #[test]
    fn test_line_index() {
        let source = "line 1\nline 2\nline 3";
        let index = LineIndex::new(source);

        assert_eq!(index.line_col(0), LineCol { line: 1, col: 1 });
        assert_eq!(index.line_col(5), LineCol { line: 1, col: 6 });
        assert_eq!(index.line_col(7), LineCol { line: 2, col: 1 });
        assert_eq!(index.line_col(14), LineCol { line: 3, col: 1 });
    }

    #[test]
    fn test_line_text() {
        let source = "line 1\nline 2\nline 3";
        let index = LineIndex::new(source);

        assert_eq!(index.line_text(0, source), Some("line 1"));
        assert_eq!(index.line_text(1, source), Some("line 2"));
        assert_eq!(index.line_text(2, source), Some("line 3"));
    }
}
