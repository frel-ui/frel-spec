// Diagnostic sink abstraction for output-agnostic diagnostic collection
//
// The DiagnosticSink trait decouples diagnostic emission from consumption,
// enabling different modes of operation:
// - Batch collection (CLI)
// - Streaming (LSP/IDE)
// - File-scoped (incremental compilation)

use super::{Diagnostic, Severity};

/// A sink that receives diagnostics during compilation
///
/// Implementations can collect, stream, filter, or process diagnostics
/// as they are emitted by compiler passes.
pub trait DiagnosticSink {
    /// Emit a diagnostic to the sink
    fn emit(&mut self, diagnostic: Diagnostic);

    /// Check if any errors have been emitted
    fn has_errors(&self) -> bool;

    /// Get the count of errors emitted
    fn error_count(&self) -> usize;
}

/// Collects all diagnostics into a vector
///
/// This is the default sink used for batch compilation mode.
#[derive(Debug, Default)]
pub struct CollectingSink {
    diagnostics: Vec<Diagnostic>,
    error_count: usize,
}

impl CollectingSink {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            error_count: 0,
        }
    }

    /// Get all collected diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Take ownership of collected diagnostics
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    /// Iterate over collected diagnostics
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }
}

impl DiagnosticSink for CollectingSink {
    fn emit(&mut self, diagnostic: Diagnostic) {
        if diagnostic.severity == Severity::Error {
            self.error_count += 1;
        }
        self.diagnostics.push(diagnostic);
    }

    fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    fn error_count(&self) -> usize {
        self.error_count
    }
}

/// Streams diagnostics immediately via a callback
///
/// Useful for LSP integration where diagnostics should be published
/// as soon as they are discovered.
pub struct StreamingSink<F>
where
    F: FnMut(&Diagnostic),
{
    callback: F,
    error_count: usize,
}

impl<F> StreamingSink<F>
where
    F: FnMut(&Diagnostic),
{
    pub fn new(callback: F) -> Self {
        Self {
            callback,
            error_count: 0,
        }
    }
}

impl<F> DiagnosticSink for StreamingSink<F>
where
    F: FnMut(&Diagnostic),
{
    fn emit(&mut self, diagnostic: Diagnostic) {
        if diagnostic.severity == Severity::Error {
            self.error_count += 1;
        }
        (self.callback)(&diagnostic);
    }

    fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    fn error_count(&self) -> usize {
        self.error_count
    }
}

/// Filters diagnostics to only those from a specific file
///
/// Useful for incremental compilation where we only want to update
/// diagnostics for the file being recompiled.
pub struct FileScopedSink<'a, S>
where
    S: DiagnosticSink,
{
    file_path: &'a str,
    inner: &'a mut S,
    scoped_error_count: usize,
}

impl<'a, S> FileScopedSink<'a, S>
where
    S: DiagnosticSink,
{
    pub fn new(file_path: &'a str, inner: &'a mut S) -> Self {
        Self {
            file_path,
            inner,
            scoped_error_count: 0,
        }
    }

    /// Get the file path this sink is scoped to
    pub fn file_path(&self) -> &str {
        self.file_path
    }
}

impl<'a, S> DiagnosticSink for FileScopedSink<'a, S>
where
    S: DiagnosticSink,
{
    fn emit(&mut self, diagnostic: Diagnostic) {
        // For now, pass through all diagnostics
        // In the future, we can filter by file when we have file info in diagnostics
        if diagnostic.severity == Severity::Error {
            self.scoped_error_count += 1;
        }
        self.inner.emit(diagnostic);
    }

    fn has_errors(&self) -> bool {
        self.scoped_error_count > 0
    }

    fn error_count(&self) -> usize {
        self.scoped_error_count
    }
}

/// A sink that counts diagnostics by severity without storing them
///
/// Useful for quick validation passes where full diagnostic info isn't needed.
#[derive(Debug, Default)]
pub struct CountingSink {
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    hint_count: usize,
}

impl CountingSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    pub fn info_count(&self) -> usize {
        self.info_count
    }

    pub fn hint_count(&self) -> usize {
        self.hint_count
    }

    pub fn total_count(&self) -> usize {
        self.error_count + self.warning_count + self.info_count + self.hint_count
    }
}

impl DiagnosticSink for CountingSink {
    fn emit(&mut self, diagnostic: Diagnostic) {
        match diagnostic.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
            Severity::Info => self.info_count += 1,
            Severity::Hint => self.hint_count += 1,
        }
    }

    fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    fn error_count(&self) -> usize {
        self.error_count
    }
}

/// A sink that discards all diagnostics
///
/// Useful for testing or when you only care about the presence of errors.
#[derive(Debug, Default)]
pub struct NullSink {
    error_count: usize,
}

impl NullSink {
    pub fn new() -> Self {
        Self { error_count: 0 }
    }
}

impl DiagnosticSink for NullSink {
    fn emit(&mut self, diagnostic: Diagnostic) {
        if diagnostic.severity == Severity::Error {
            self.error_count += 1;
        }
    }

    fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    fn error_count(&self) -> usize {
        self.error_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::Span;

    fn make_error() -> Diagnostic {
        Diagnostic::error("test error", Span::new(0, 5))
    }

    fn make_warning() -> Diagnostic {
        Diagnostic::warning("test warning", Span::new(0, 5))
    }

    #[test]
    fn test_collecting_sink() {
        let mut sink = CollectingSink::new();
        assert!(!sink.has_errors());
        assert_eq!(sink.error_count(), 0);

        sink.emit(make_error());
        assert!(sink.has_errors());
        assert_eq!(sink.error_count(), 1);
        assert_eq!(sink.diagnostics().len(), 1);

        sink.emit(make_warning());
        assert_eq!(sink.error_count(), 1);
        assert_eq!(sink.diagnostics().len(), 2);
    }

    #[test]
    fn test_streaming_sink() {
        let mut received = Vec::new();
        {
            let mut sink = StreamingSink::new(|d| {
                received.push(d.message.clone());
            });
            sink.emit(make_error());
            sink.emit(make_warning());
            assert!(sink.has_errors());
        }
        assert_eq!(received.len(), 2);
    }

    #[test]
    fn test_counting_sink() {
        let mut sink = CountingSink::new();
        sink.emit(make_error());
        sink.emit(make_error());
        sink.emit(make_warning());

        assert!(sink.has_errors());
        assert_eq!(sink.error_count(), 2);
        assert_eq!(sink.warning_count(), 1);
        assert_eq!(sink.total_count(), 3);
    }

    #[test]
    fn test_null_sink() {
        let mut sink = NullSink::new();
        sink.emit(make_error());
        assert!(sink.has_errors());
        assert_eq!(sink.error_count(), 1);
    }
}
