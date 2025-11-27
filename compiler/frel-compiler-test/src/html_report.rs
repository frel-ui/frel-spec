//! HTML report generator for Frel compiler tests
//!
//! Generates a static HTML file showing all test cases with their
//! source code, AST dump, and JSON output (or errors for failing tests).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use frel_compiler_core::ast::DumpVisitor;

/// Check if a test file is in an `errors` directory (direct parent only)
fn expects_error(path: &Path) -> bool {
    path.parent()
        .and_then(|p| p.file_name())
        .map_or(false, |name| name == "errors")
}

/// Information about a single test case for the report
#[derive(Debug)]
pub struct TestReportEntry {
    pub name: String,
    pub source: String,
    pub result: TestReportResult,
}

/// The result of a test case
#[derive(Debug)]
pub enum TestReportResult {
    /// Locked success test (has .ast.json)
    Success {
        dump: String,
        json: String,
    },
    /// Locked error test (has .error.txt)
    Error {
        error: String,
    },
    /// WIP success test - expects parse to succeed
    WipSuccess {
        /// Whether the test passed (parse succeeded)
        passed: bool,
        dump: Option<String>,
        json: Option<String>,
        error: Option<String>,
    },
    /// WIP error test - expects parse to fail
    WipError {
        /// Whether the test passed (parse failed)
        passed: bool,
        dump: Option<String>,
        json: Option<String>,
        error: Option<String>,
    },
}

/// Collects test data and generates HTML report
pub struct HtmlReportGenerator {
    entries: Vec<TestReportEntry>,
}

impl HtmlReportGenerator {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Collect test data from the test directory
    pub fn collect_tests(&mut self, tests_dir: &Path) -> Result<(), String> {
        use walkdir::WalkDir;

        for entry in WalkDir::new(tests_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "frel"))
        {
            let source_path = entry.path().to_path_buf();

            // Get relative path for the test name
            let name = source_path
                .strip_prefix(tests_dir)
                .unwrap_or(&source_path)
                .with_extension("")
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/");

            // Read source file
            let source = fs::read_to_string(&source_path)
                .map_err(|e| format!("Failed to read {}: {}", source_path.display(), e))?;

            // Determine test kind and collect results
            let ast_path = source_path.with_extension("ast.json");
            let dump_path = source_path.with_extension("ast.dump");
            let error_path = source_path.with_extension("error.txt");

            let result = if ast_path.exists() {
                // Locked success test
                let json = fs::read_to_string(&ast_path).unwrap_or_default();
                let dump = if dump_path.exists() {
                    fs::read_to_string(&dump_path).unwrap_or_default()
                } else {
                    // Generate dump from parsing
                    self.generate_dump(&source)
                };
                TestReportResult::Success { dump, json }
            } else if error_path.exists() {
                // Locked error test
                let error = fs::read_to_string(&error_path).unwrap_or_default();
                TestReportResult::Error { error }
            } else if expects_error(&source_path) {
                // WIP error test - expects parse to fail
                let (dump, json, error) = self.parse_wip(&source);
                let passed = error.is_some(); // Test passes if parse failed
                TestReportResult::WipError { passed, dump, json, error }
            } else {
                // WIP success test - expects parse to succeed
                let (dump, json, error) = self.parse_wip(&source);
                let passed = error.is_none(); // Test passes if parse succeeded
                TestReportResult::WipSuccess { passed, dump, json, error }
            };

            self.entries.push(TestReportEntry {
                name,
                source,
                result,
            });
        }

        // Sort by name for consistent ordering
        self.entries.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(())
    }

    fn generate_dump(&self, source: &str) -> String {
        let result = frel_compiler_core::parse_file(source);
        if let Some(ast) = result.file {
            DumpVisitor::dump(&ast)
        } else {
            String::new()
        }
    }

    fn parse_wip(&self, source: &str) -> (Option<String>, Option<String>, Option<String>) {
        let result = frel_compiler_core::parse_file(source);

        if result.diagnostics.has_errors() {
            let error = self.format_diagnostics(&result.diagnostics, source);
            (None, None, Some(error))
        } else if let Some(ast) = result.file {
            let dump = DumpVisitor::dump(&ast);
            let json = serde_json::to_string_pretty(&ast).ok();
            (Some(dump), json, None)
        } else {
            (None, None, Some("Parse returned no AST and no errors".to_string()))
        }
    }

    fn format_diagnostics(
        &self,
        diagnostics: &frel_compiler_core::Diagnostics,
        source: &str,
    ) -> String {
        let line_index = frel_compiler_core::LineIndex::new(source);
        let mut output = String::new();

        for diag in diagnostics.iter() {
            let loc = line_index.line_col(diag.span.start);
            output.push_str(&format!(
                "error[{}]: {}\n",
                diag.code.as_deref().unwrap_or("E????"),
                diag.message
            ));
            output.push_str(&format!(" --> {}:{}\n", loc.line, loc.col));
        }

        output
    }

    /// Generate HTML report and write to file
    pub fn generate(&self, output_path: &Path) -> Result<(), String> {
        // Build navigation structure (grouped by directory)
        let nav_structure = self.build_nav_structure();

        let mut html = String::new();

        // HTML header with embedded styles and scripts
        html.push_str(&self.generate_html_header());

        // Body
        html.push_str("<body>\n");

        // Navigation sidebar
        html.push_str(&self.generate_nav_sidebar(&nav_structure));

        // Main content
        html.push_str("<main>\n");
        html.push_str("<h1>Frel Compiler Test Results</h1>\n");
        html.push_str(&format!(
            "<p class=\"summary\">Total: {} tests</p>\n",
            self.entries.len()
        ));

        // Test entries
        for entry in &self.entries {
            html.push_str(&self.generate_test_entry(entry));
        }

        html.push_str("</main>\n");

        // Scripts for interactivity
        html.push_str(&self.generate_scripts());

        html.push_str("</body>\n</html>\n");

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }

        fs::write(output_path, &html)
            .map_err(|e| format!("Failed to write report: {}", e))?;

        Ok(())
    }

    fn build_nav_structure(&self) -> BTreeMap<String, Vec<String>> {
        let mut nav: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for entry in &self.entries {
            let parts: Vec<&str> = entry.name.split('/').collect();
            let dir = if parts.len() > 1 {
                parts[..parts.len() - 1].join("/")
            } else {
                "root".to_string()
            };

            nav.entry(dir).or_default().push(entry.name.clone());
        }

        nav
    }

    fn generate_html_header(&self) -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Frel Compiler Test Results</title>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/json.min.js"></script>
    <style>
        :root {
            --bg-color: #f5f5f5;
            --card-bg: #ffffff;
            --border-color: #e0e0e0;
            --text-color: #333;
            --nav-width: 280px;
            --success-color: #22c55e;
            --error-color: #ef4444;
            --wip-color: #f59e0b;
            --wip-fail-color: #dc2626;
        }

        * {
            box-sizing: border-box;
        }

        body {
            margin: 0;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: var(--bg-color);
            color: var(--text-color);
            display: flex;
        }

        nav {
            position: fixed;
            left: 0;
            top: 0;
            width: var(--nav-width);
            height: 100vh;
            background: var(--card-bg);
            border-right: 1px solid var(--border-color);
            overflow-y: auto;
            padding: 1rem;
        }

        nav h2 {
            margin: 0 0 1rem 0;
            font-size: 1rem;
            color: #666;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }

        nav .nav-group {
            margin-bottom: 1rem;
        }

        nav .nav-group-title {
            font-weight: 600;
            font-size: 0.875rem;
            color: #444;
            margin-bottom: 0.5rem;
            cursor: pointer;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }

        nav .nav-group-title::before {
            content: '▶';
            font-size: 0.6rem;
            transition: transform 0.2s;
        }

        nav .nav-group.open .nav-group-title::before {
            transform: rotate(90deg);
        }

        nav .nav-items {
            display: none;
            padding-left: 1rem;
        }

        nav .nav-group.open .nav-items {
            display: block;
        }

        nav .nav-item {
            display: block;
            padding: 0.25rem 0;
            font-size: 0.8rem;
            color: #666;
            text-decoration: none;
            font-family: 'SF Mono', Monaco, 'Courier New', monospace;
        }

        nav .nav-item:hover {
            color: #0066cc;
        }

        nav .nav-item.success::before {
            content: '●';
            color: var(--success-color);
            margin-right: 0.5rem;
        }

        nav .nav-item.error::before {
            content: '●';
            color: var(--error-color);
            margin-right: 0.5rem;
        }

        nav .nav-item.wip::before {
            content: '●';
            color: var(--wip-color);
            margin-right: 0.5rem;
        }

        nav .nav-item.wip-fail::before {
            content: '●';
            color: var(--wip-fail-color);
            margin-right: 0.5rem;
        }

        main {
            margin-left: var(--nav-width);
            padding: 2rem;
            width: calc(100% - var(--nav-width));
        }

        h1 {
            margin: 0 0 0.5rem 0;
            font-size: 1.5rem;
        }

        .summary {
            color: #666;
            margin: 0 0 2rem 0;
        }

        .test-entry {
            background: var(--card-bg);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            margin-bottom: 1.5rem;
            overflow: hidden;
        }

        .test-header {
            display: flex;
            align-items: center;
            gap: 0.75rem;
            padding: 0.75rem 1rem;
            background: #fafafa;
            border-bottom: 1px solid var(--border-color);
        }

        .test-header .status {
            width: 10px;
            height: 10px;
            border-radius: 50%;
        }

        .test-header .status.success { background: var(--success-color); }
        .test-header .status.error { background: var(--error-color); }
        .test-header .status.wip { background: var(--wip-color); }
        .test-header .status.wip-fail { background: var(--wip-fail-color); }

        .test-header .name {
            font-family: 'SF Mono', Monaco, 'Courier New', monospace;
            font-size: 0.9rem;
            font-weight: 600;
        }

        .test-header .badge {
            font-size: 0.7rem;
            padding: 0.15rem 0.5rem;
            border-radius: 4px;
            text-transform: uppercase;
            font-weight: 600;
        }

        .test-header .badge.success { background: #dcfce7; color: #166534; }
        .test-header .badge.error { background: #fee2e2; color: #991b1b; }
        .test-header .badge.wip { background: #fef3c7; color: #92400e; }
        .test-header .badge.wip-fail { background: #fee2e2; color: #991b1b; }

        .test-content {
            display: grid;
            grid-template-columns: 1fr 1fr 1fr;
            gap: 1px;
            background: var(--border-color);
        }

        .test-content.two-cols {
            grid-template-columns: 1fr 1fr;
        }

        .test-panel {
            background: var(--card-bg);
            position: relative;
        }

        .panel-header {
            padding: 0.5rem 0.75rem;
            font-size: 0.75rem;
            font-weight: 600;
            color: #666;
            text-transform: uppercase;
            border-bottom: 1px solid var(--border-color);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .panel-header button {
            background: none;
            border: 1px solid var(--border-color);
            border-radius: 4px;
            padding: 0.2rem 0.5rem;
            font-size: 0.65rem;
            cursor: pointer;
            color: #666;
        }

        .panel-header button:hover {
            background: #f0f0f0;
        }

        .panel-content {
            padding: 0.75rem;
            overflow: hidden;
        }

        .panel-content pre {
            margin: 0;
            font-family: 'SF Mono', Monaco, 'Courier New', monospace;
            font-size: 0.8rem;
            line-height: 1.5;
            white-space: pre;
        }

        /* Expandable panels (JSON/Error) - collapsed by default */
        .panel-expandable .panel-content {
            overflow: hidden;
            cursor: pointer;
        }

        .panel-expandable .panel-content:hover {
            background: #f8f8f8;
        }

        /* Expanded state - overlay */
        .panel-expanded {
            position: fixed;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            z-index: 1000;
            background: var(--card-bg);
            border-radius: 8px;
            box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
            max-width: 90vw;
            max-height: 90vh;
            overflow: auto;
        }

        .panel-expanded .panel-content {
            overflow: auto;
            max-height: calc(90vh - 50px);
            cursor: default;
        }

        .panel-expanded .panel-content:hover {
            background: transparent;
        }

        /* Overlay backdrop */
        .overlay-backdrop {
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.5);
            z-index: 999;
        }

        .overlay-backdrop.active {
            display: block;
        }

        /* Syntax highlighting for Frel */
        .frel-keyword { color: #d73a49; font-weight: 600; }
        .frel-type { color: #6f42c1; }
        .frel-string { color: #22863a; }
        .frel-number { color: #005cc5; }
        .frel-comment { color: #6a737d; font-style: italic; }
        .frel-identifier { color: #24292e; }
        .frel-operator { color: #d73a49; }
        .frel-modifier { color: #e36209; }

        /* Line numbers */
        .line-number {
            color: #999;
            user-select: none;
            display: inline-block;
            text-align: right;
            min-width: 2ch;
        }

        /* Dump syntax highlighting */
        .dump-node { color: #d73a49; font-weight: 600; }
        .dump-attr { color: #6f42c1; }
        .dump-value { color: #22863a; }
        .dump-type { color: #005cc5; }
    </style>
</head>
"#.to_string()
    }

    fn generate_nav_sidebar(&self, nav_structure: &BTreeMap<String, Vec<String>>) -> String {
        let mut html = String::new();
        html.push_str("<nav>\n");
        html.push_str("<h2>Test Cases</h2>\n");

        for (dir, tests) in nav_structure {
            html.push_str("<div class=\"nav-group open\">\n");
            html.push_str(&format!(
                "<div class=\"nav-group-title\">{}</div>\n",
                html_escape(dir)
            ));
            html.push_str("<div class=\"nav-items\">\n");

            for test_name in tests {
                let status_class = self
                    .entries
                    .iter()
                    .find(|e| &e.name == test_name)
                    .map(|e| match &e.result {
                        TestReportResult::Success { .. } => "success",
                        TestReportResult::Error { .. } => "error",
                        TestReportResult::WipSuccess { passed: true, .. } => "wip",
                        TestReportResult::WipSuccess { passed: false, .. } => "wip-fail",
                        TestReportResult::WipError { passed: true, .. } => "wip",
                        TestReportResult::WipError { passed: false, .. } => "wip-fail",
                    })
                    .unwrap_or("wip");

                let short_name = test_name.split('/').last().unwrap_or(&test_name);
                let anchor = test_name.replace('/', "-");
                html.push_str(&format!(
                    "<a href=\"#{}\" class=\"nav-item {}\">{}</a>\n",
                    anchor,
                    status_class,
                    html_escape(short_name)
                ));
            }

            html.push_str("</div>\n</div>\n");
        }

        html.push_str("</nav>\n");
        html
    }

    fn generate_test_entry(&self, entry: &TestReportEntry) -> String {
        let mut html = String::new();
        let anchor = entry.name.replace('/', "-");

        let (status_class, badge_text) = match &entry.result {
            TestReportResult::Success { .. } => ("success", "Success"),
            TestReportResult::Error { .. } => ("error", "Error"),
            TestReportResult::WipSuccess { passed: true, .. } => ("wip", "WIP"),
            TestReportResult::WipSuccess { passed: false, .. } => ("wip-fail", "WIP FAIL"),
            TestReportResult::WipError { passed: true, .. } => ("wip", "WIP"),
            TestReportResult::WipError { passed: false, .. } => ("wip-fail", "WIP FAIL"),
        };

        html.push_str(&format!(
            "<div class=\"test-entry\" id=\"{}\">\n",
            anchor
        ));

        // Header
        html.push_str("<div class=\"test-header\">\n");
        html.push_str(&format!("<span class=\"status {}\"></span>\n", status_class));
        html.push_str(&format!(
            "<span class=\"name\">{}</span>\n",
            html_escape(&entry.name)
        ));
        html.push_str(&format!(
            "<span class=\"badge {}\">{}</span>\n",
            status_class, badge_text
        ));
        html.push_str("</div>\n");

        // Calculate line counts for sizing
        let source_lines = entry.source.lines().count();

        // Content based on result type
        match &entry.result {
            TestReportResult::Success { dump, json } => {
                let dump_lines = dump.lines().count();
                let row_lines = source_lines.max(dump_lines);

                html.push_str("<div class=\"test-content\">\n");

                // Source panel - full height
                html.push_str(&self.generate_panel("Source", &entry.source, "frel", false, row_lines));

                // Dump panel - full height
                html.push_str(&self.generate_panel("Dump", dump, "dump", false, row_lines));

                // JSON panel (expandable) - same height as others
                html.push_str(&self.generate_panel("JSON", json, "json", true, row_lines));

                html.push_str("</div>\n");
            }
            TestReportResult::Error { error } => {
                // For error tests, size by source only
                let row_lines = source_lines;

                html.push_str("<div class=\"test-content two-cols\">\n");

                // Source panel
                html.push_str(&self.generate_panel("Source", &entry.source, "frel", false, row_lines));

                // Error panel (expandable)
                html.push_str(&self.generate_panel("Error", error, "error", true, row_lines));

                html.push_str("</div>\n");
            }
            TestReportResult::WipSuccess { dump, json, error, .. }
            | TestReportResult::WipError { dump, json, error, .. } => {
                if let Some(error_msg) = error {
                    let row_lines = source_lines;

                    html.push_str("<div class=\"test-content two-cols\">\n");
                    html.push_str(&self.generate_panel("Source", &entry.source, "frel", false, row_lines));
                    html.push_str(&self.generate_panel("Error", error_msg, "error", true, row_lines));
                    html.push_str("</div>\n");
                } else {
                    let dump_str = dump.as_deref().unwrap_or("");
                    let dump_lines = dump_str.lines().count();
                    let row_lines = source_lines.max(dump_lines);

                    html.push_str("<div class=\"test-content\">\n");
                    html.push_str(&self.generate_panel("Source", &entry.source, "frel", false, row_lines));
                    html.push_str(&self.generate_panel("Dump", dump_str, "dump", false, row_lines));
                    html.push_str(&self.generate_panel(
                        "JSON",
                        json.as_deref().unwrap_or(""),
                        "json",
                        true,
                        row_lines,
                    ));
                    html.push_str("</div>\n");
                }
            }
        }

        html.push_str("</div>\n");
        html
    }

    fn generate_panel(
        &self,
        title: &str,
        content: &str,
        lang: &str,
        expandable: bool,
        row_lines: usize,
    ) -> String {
        let mut html = String::new();
        let panel_id = format!("panel-{}-{}", title.to_lowercase(), rand_id());

        // Calculate height based on line count (1.5em line-height + padding)
        // line-height is 1.5, font-size is 0.8rem = 12.8px, so each line is ~19.2px
        // Add padding (0.75rem * 2 = 24px) and header (~32px)
        let min_lines = 3;
        let lines = row_lines.max(min_lines);
        let height_px = (lines as f64 * 19.2) + 24.0;

        let panel_class = if expandable {
            "test-panel panel-expandable"
        } else {
            "test-panel"
        };

        html.push_str(&format!("<div class=\"{}\">\n", panel_class));
        html.push_str("<div class=\"panel-header\">\n");
        html.push_str(&format!("<span>{}</span>\n", title));
        if expandable {
            html.push_str("<span style=\"font-size: 0.6rem; color: #999;\">click to expand</span>\n");
        }
        html.push_str("</div>\n");

        // For expandable panels, add onclick handler to the content area
        if expandable {
            html.push_str(&format!(
                "<div class=\"panel-content\" style=\"height: {:.0}px;\" onclick=\"togglePanel('{}')\">\n",
                height_px, panel_id
            ));
        } else {
            html.push_str(&format!(
                "<div class=\"panel-content\" style=\"height: {:.0}px;\">\n",
                height_px
            ));
        }

        let highlighted = self.highlight_code(content, lang);
        html.push_str(&format!(
            "<pre id=\"{}\">{}</pre>\n",
            panel_id, highlighted
        ));
        html.push_str("</div>\n");

        html.push_str("</div>\n");
        html
    }

    fn highlight_code(&self, code: &str, lang: &str) -> String {
        match lang {
            "frel" => highlight_frel(code),
            "dump" => highlight_dump(code),
            "json" => {
                // Use class for highlight.js to process
                format!(
                    "<code class=\"language-json\">{}</code>",
                    html_escape(code)
                )
            }
            "error" => format!(
                "<code style=\"color: var(--error-color);\">{}</code>",
                html_escape(code)
            ),
            _ => html_escape(code),
        }
    }

    fn generate_scripts(&self) -> String {
        r#"
<div id="backdrop" class="overlay-backdrop" onclick="closeExpandedPanel()"></div>

<script>
    // Initialize highlight.js for JSON
    document.querySelectorAll('code.language-json').forEach((el) => {
        hljs.highlightElement(el);
    });

    let currentExpandedPanel = null;
    let originalParent = null;
    let originalNextSibling = null;

    function togglePanel(panelId) {
        const pre = document.getElementById(panelId);
        const panel = pre.closest('.test-panel');

        if (panel.classList.contains('panel-expanded')) {
            closeExpandedPanel();
        } else {
            expandPanel(panel);
        }
    }

    function expandPanel(panel) {
        // Close any currently expanded panel first
        if (currentExpandedPanel) {
            closeExpandedPanel();
        }

        // Store original position
        originalParent = panel.parentElement;
        originalNextSibling = panel.nextSibling;
        currentExpandedPanel = panel;

        // Move panel to body and expand
        document.body.appendChild(panel);
        panel.classList.add('panel-expanded');

        // Show backdrop
        document.getElementById('backdrop').classList.add('active');
        document.body.style.overflow = 'hidden';

        // Re-apply highlight.js if needed
        panel.querySelectorAll('code.language-json').forEach((el) => {
            hljs.highlightElement(el);
        });
    }

    function closeExpandedPanel() {
        if (!currentExpandedPanel) return;

        // Remove expanded class
        currentExpandedPanel.classList.remove('panel-expanded');

        // Move back to original position
        if (originalNextSibling) {
            originalParent.insertBefore(currentExpandedPanel, originalNextSibling);
        } else {
            originalParent.appendChild(currentExpandedPanel);
        }

        // Hide backdrop
        document.getElementById('backdrop').classList.remove('active');
        document.body.style.overflow = '';

        currentExpandedPanel = null;
        originalParent = null;
        originalNextSibling = null;
    }

    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') closeExpandedPanel();
    });

    // Navigation group toggle
    document.querySelectorAll('.nav-group-title').forEach(title => {
        title.addEventListener('click', () => {
            title.parentElement.classList.toggle('open');
        });
    });
</script>
"#.to_string()
    }
}

/// Generate a simple random ID for panel elements
fn rand_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    (duration.as_nanos() % 1_000_000) as u32
}

/// HTML-escape a string
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Syntax highlighting for Frel source code with line numbers
fn highlight_frel(code: &str) -> String {
    let mut result = String::new();
    let line_count = code.lines().count().max(1);
    let width = line_count.to_string().len();

    // Process each line individually
    for (i, line) in code.lines().enumerate() {
        let line_num = i + 1;
        let highlighted_line = highlight_frel_line(line);
        result.push_str(&format!(
            "<span class=\"line-number\">{:>width$}</span>  {}\n",
            line_num,
            highlighted_line,
            width = width
        ));
    }

    result
}

/// Highlight a single line of Frel source code
fn highlight_frel_line(line: &str) -> String {
    highlight_frel_content(line)
}

/// Syntax highlighting for Frel source code (content only, no line numbers)
fn highlight_frel_content(code: &str) -> String {
    let keywords = [
        "module", "scheme", "arena", "backend", "blueprint", "theme", "contract",
        "enum", "field", "virtual", "action", "event", "import", "from", "as",
        "if", "else", "match", "for", "in", "return", "let", "mut", "true", "false",
        "null", "self",
    ];

    let type_keywords = [
        "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64",
        "bool", "String", "Uuid", "Url", "Decimal", "Color", "Graphics",
        "Secret", "Blob", "Instant", "LocalDate", "LocalTime", "LocalDateTime",
        "Timezone", "Duration", "List", "Set", "Map", "Tree", "Blueprint", "Accessor",
    ];

    let modifiers = ["ref", "draft", "asset"];

    let mut result = String::new();
    let mut chars = code.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'/') {
            // Line comment
            let mut comment = String::from("//");
            chars.next();
            while let Some(&nc) = chars.peek() {
                if nc == '\n' {
                    break;
                }
                comment.push(chars.next().unwrap());
            }
            result.push_str(&format!(
                "<span class=\"frel-comment\">{}</span>",
                html_escape(&comment)
            ));
        } else if c == '"' {
            // String literal
            let mut s = String::from("\"");
            while let Some(nc) = chars.next() {
                s.push(nc);
                if nc == '"' {
                    break;
                }
                if nc == '\\' {
                    if let Some(esc) = chars.next() {
                        s.push(esc);
                    }
                }
            }
            result.push_str(&format!(
                "<span class=\"frel-string\">{}</span>",
                html_escape(&s)
            ));
        } else if c.is_ascii_digit() || (c == '-' && chars.peek().map_or(false, |nc| nc.is_ascii_digit())) {
            // Number
            let mut num = String::from(c);
            while let Some(&nc) = chars.peek() {
                if nc.is_ascii_digit() || nc == '.' || nc == '_' {
                    num.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            result.push_str(&format!(
                "<span class=\"frel-number\">{}</span>",
                html_escape(&num)
            ));
        } else if c.is_ascii_alphabetic() || c == '_' {
            // Identifier or keyword
            let mut ident = String::from(c);
            while let Some(&nc) = chars.peek() {
                if nc.is_ascii_alphanumeric() || nc == '_' {
                    ident.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            if keywords.contains(&ident.as_str()) {
                result.push_str(&format!(
                    "<span class=\"frel-keyword\">{}</span>",
                    html_escape(&ident)
                ));
            } else if type_keywords.contains(&ident.as_str()) {
                result.push_str(&format!(
                    "<span class=\"frel-type\">{}</span>",
                    html_escape(&ident)
                ));
            } else if modifiers.contains(&ident.as_str()) {
                result.push_str(&format!(
                    "<span class=\"frel-modifier\">{}</span>",
                    html_escape(&ident)
                ));
            } else {
                result.push_str(&format!(
                    "<span class=\"frel-identifier\">{}</span>",
                    html_escape(&ident)
                ));
            }
        } else if "{}[](),:;=<>+-.!&|?@".contains(c) {
            result.push_str(&format!(
                "<span class=\"frel-operator\">{}</span>",
                html_escape(&c.to_string())
            ));
        } else {
            result.push_str(&html_escape(&c.to_string()));
        }
    }

    result
}

/// Syntax highlighting for AST dump output
fn highlight_dump(code: &str) -> String {
    let node_keywords = [
        "FILE", "SCHEME", "ARENA", "BACKEND", "BLUEPRINT", "THEME", "CONTRACT",
        "ENUM", "FIELD", "VIRTUAL", "ACTION", "EVENT", "IMPORT", "VARIANT",
        "PARAM", "FRAGMENT", "IF", "ELSE", "MATCH", "FOR", "LET", "RETURN",
        "EXPR", "CALL", "BINARY", "UNARY", "LITERAL", "IDENT", "MEMBER",
    ];

    let attr_keywords = ["TYPE", "INIT", "BODY", "CONDITION", "THEN", "ELSE"];

    let mut result = String::new();

    for line in code.lines() {
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        result.push_str(indent);

        let mut words = trimmed.split_whitespace().peekable();
        let mut first = true;

        while let Some(word) = words.next() {
            if !first {
                result.push(' ');
            }
            first = false;

            if node_keywords.contains(&word) {
                result.push_str(&format!(
                    "<span class=\"dump-node\">{}</span>",
                    html_escape(word)
                ));
            } else if attr_keywords.contains(&word) {
                result.push_str(&format!(
                    "<span class=\"dump-attr\">{}</span>",
                    html_escape(word)
                ));
            } else if word.starts_with('"') || word.ends_with('"') {
                result.push_str(&format!(
                    "<span class=\"dump-value\">{}</span>",
                    html_escape(word)
                ));
            } else if word.contains('=') {
                // Attribute like module=test
                let parts: Vec<&str> = word.splitn(2, '=').collect();
                if parts.len() == 2 {
                    result.push_str(&format!(
                        "<span class=\"dump-attr\">{}</span>=<span class=\"dump-value\">{}</span>",
                        html_escape(parts[0]),
                        html_escape(parts[1])
                    ));
                } else {
                    result.push_str(&html_escape(word));
                }
            } else {
                result.push_str(&html_escape(word));
            }
        }

        result.push('\n');
    }

    result
}
