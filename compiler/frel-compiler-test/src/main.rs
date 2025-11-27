use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use frel_compiler_core::ast::DumpVisitor;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod html_report;

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum OutputFormat {
    /// JSON output format (default)
    Json,
    /// Human-readable DUMP format
    Dump,
    /// Both JSON and DUMP formats
    Both,
}

#[derive(Parser)]
#[command(name = "frel-compiler-test")]
#[command(about = "Testing framework for the Frel compiler")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Optional filter pattern (e.g., "parser/blueprint" or "simple")
    #[arg(global = true)]
    filter: Option<String>,

    /// Update expected output files with actual results
    #[arg(long, global = true)]
    update: bool,

    /// Show detailed diff on failures
    #[arg(long, global = true)]
    verbose: bool,

    /// Output format for AST files
    #[arg(long, value_enum, default_value = "json", global = true)]
    format: OutputFormat,
}

#[derive(Subcommand)]
enum Command {
    /// Run tests (default)
    Test {
        /// Optional filter pattern
        filter: Option<String>,
    },
    /// Generate HTML report of all test cases
    Report {
        /// Output file path (default: compiler/target/test-results.html)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug)]
enum TestKind {
    /// Locked success test: has .ast.json, expects parse to succeed and output to match
    Success {
        expected_ast: PathBuf,
        expected_dump: Option<PathBuf>,
    },
    /// Locked error test: has .error.txt, expects parse to fail and error to match
    Error { expected_error: PathBuf },
    /// WIP success test: no output files, expects parse to succeed (output not verified)
    WipSuccess,
    /// WIP error test: no output files, expects parse to fail (output not verified)
    WipError,
}

#[derive(Debug)]
struct TestCase {
    name: String,
    source_path: PathBuf,
    kind: TestKind,
}

#[derive(Debug)]
enum TestResult {
    Passed,
    Failed { message: String, diff: Option<String> },
}

fn main() {
    let cli = Cli::parse();

    // Find the tests directory relative to the binary or current directory
    let tests_dir = find_tests_dir();

    if !tests_dir.exists() {
        eprintln!(
            "{} Tests directory not found: {}",
            "error:".red().bold(),
            tests_dir.display()
        );
        std::process::exit(1);
    }

    match &cli.command {
        Some(Command::Report { output }) => {
            run_report_command(&tests_dir, output.as_ref());
        }
        Some(Command::Test { filter }) => {
            run_test_command(&tests_dir, filter.as_deref(), &cli);
        }
        None => {
            // Default: run tests with the global filter
            run_test_command(&tests_dir, cli.filter.as_deref(), &cli);
        }
    }
}

fn run_report_command(tests_dir: &Path, output: Option<&PathBuf>) {
    let default_output = find_target_dir().join("test-results.html");
    let output_path = output.unwrap_or(&default_output);

    println!(
        "{} HTML report...",
        "Generating".green().bold()
    );

    let mut generator = html_report::HtmlReportGenerator::new();

    if let Err(e) = generator.collect_tests(tests_dir) {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }

    if let Err(e) = generator.generate(output_path) {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }

    println!(
        "{} Report written to: {}",
        "Done!".green().bold(),
        output_path.display()
    );
}

fn run_test_command(tests_dir: &Path, filter: Option<&str>, cli: &Cli) {
    // Discover test cases
    let test_cases = discover_tests(tests_dir, filter);

    if test_cases.is_empty() {
        println!("{}", "No test cases found.".yellow());
        std::process::exit(0);
    }

    println!(
        "{} {} test case(s)\n",
        "Running".green().bold(),
        test_cases.len()
    );

    // Run tests
    let mut passed = 0;
    let mut failed = 0;

    for test in &test_cases {
        let result = run_test(test, cli.update, cli.format);

        match &result {
            TestResult::Passed => {
                println!("  {} {}", "PASS".green(), test.name);
                passed += 1;
            }
            TestResult::Failed { message, diff } => {
                println!("  {} {}", "FAIL".red(), test.name);
                println!("       {}", message);
                if cli.verbose {
                    if let Some(d) = diff {
                        println!("{}", d);
                    }
                }
                failed += 1;
            }
        }
    }

    // Print summary
    println!();
    println!(
        "Results: {} passed, {} failed",
        passed.to_string().green(),
        failed.to_string().red()
    );

    if failed > 0 {
        std::process::exit(1);
    }
}

fn find_tests_dir() -> PathBuf {
    // Try to find compiler/test-data relative to current directory
    let candidates = [
        PathBuf::from("compiler/test-data"),
        PathBuf::from("test-data"),
        PathBuf::from("../test-data"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    // Default to compiler/test-data
    PathBuf::from("compiler/test-data")
}

fn find_target_dir() -> PathBuf {
    // Try to find compiler/target relative to current directory
    let candidates = [
        PathBuf::from("compiler/target"),
        PathBuf::from("target"),
        PathBuf::from("../target"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    // Default to compiler/target
    PathBuf::from("compiler/target")
}

/// Check if a test file is in an `errors` directory (direct parent only)
fn expects_error(path: &Path) -> bool {
    path.parent()
        .and_then(|p| p.file_name())
        .map_or(false, |name| name == "errors")
}

fn discover_tests(tests_dir: &Path, filter: Option<&str>) -> Vec<TestCase> {
    let mut tests = Vec::new();

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

        // Apply filter
        if let Some(f) = filter {
            if !name.contains(f) {
                continue;
            }
        }

        // Determine test kind based on existing output files and directory convention
        let ast_path = source_path.with_extension("ast.json");
        let dump_path = source_path.with_extension("ast.dump");
        let error_path = source_path.with_extension("error.txt");

        let kind = if ast_path.exists() {
            // Locked success test
            TestKind::Success {
                expected_ast: ast_path,
                expected_dump: if dump_path.exists() {
                    Some(dump_path)
                } else {
                    None
                },
            }
        } else if error_path.exists() {
            // Locked error test
            TestKind::Error {
                expected_error: error_path,
            }
        } else if expects_error(&source_path) {
            // WIP error test (in errors/ directory)
            TestKind::WipError
        } else {
            // WIP success test (not in errors/ directory)
            TestKind::WipSuccess
        };

        tests.push(TestCase {
            name,
            source_path,
            kind,
        });
    }

    // Sort by name for consistent ordering
    tests.sort_by(|a, b| a.name.cmp(&b.name));

    tests
}

fn run_test(test: &TestCase, update: bool, format: OutputFormat) -> TestResult {
    match &test.kind {
        TestKind::WipSuccess => {
            if update {
                run_wip_success_update(test, format)
            } else {
                run_wip_success_test(test)
            }
        }
        TestKind::WipError => {
            if update {
                run_wip_error_update(test)
            } else {
                run_wip_error_test(test)
            }
        }
        TestKind::Success {
            expected_ast,
            expected_dump,
        } => run_success_test(test, expected_ast, expected_dump.as_ref(), update, format),
        TestKind::Error { expected_error } => run_error_test(test, expected_error, update),
    }
}

/// WIP Success test (not update mode): just check that parsing succeeds
fn run_wip_success_test(test: &TestCase) -> TestResult {
    let source = match fs::read_to_string(&test.source_path) {
        Ok(s) => s,
        Err(e) => {
            return TestResult::Failed {
                message: format!("Failed to read source: {}", e),
                diff: None,
            }
        }
    };

    let result = frel_compiler_core::parse_file(&source);

    if result.diagnostics.has_errors() {
        let error_msg = format_diagnostics(&result.diagnostics, &source);
        TestResult::Failed {
            message: format!("Expected parse to succeed but got errors:\n{}", error_msg),
            diff: None,
        }
    } else if result.file.is_some() {
        TestResult::Passed
    } else {
        TestResult::Failed {
            message: "Parse returned no AST and no errors".to_string(),
            diff: None,
        }
    }
}

/// WIP Error test (not update mode): just check that parsing fails
fn run_wip_error_test(test: &TestCase) -> TestResult {
    let source = match fs::read_to_string(&test.source_path) {
        Ok(s) => s,
        Err(e) => {
            return TestResult::Failed {
                message: format!("Failed to read source: {}", e),
                diff: None,
            }
        }
    };

    let result = frel_compiler_core::parse_file(&source);

    if result.diagnostics.has_errors() {
        TestResult::Passed
    } else {
        TestResult::Failed {
            message: "Expected parse to fail but it succeeded".to_string(),
            diff: None,
        }
    }
}

/// WIP Success update mode: create .ast.json (and optionally .ast.dump)
fn run_wip_success_update(test: &TestCase, format: OutputFormat) -> TestResult {
    let source = match fs::read_to_string(&test.source_path) {
        Ok(s) => s,
        Err(e) => {
            return TestResult::Failed {
                message: format!("Failed to read source: {}", e),
                diff: None,
            }
        }
    };

    let result = frel_compiler_core::parse_file(&source);

    if result.diagnostics.has_errors() {
        let error_msg = format_diagnostics(&result.diagnostics, &source);
        return TestResult::Failed {
            message: format!("Cannot update: parse failed with errors:\n{}", error_msg),
            diff: None,
        };
    }

    let ast = match result.file {
        Some(ast) => ast,
        None => {
            return TestResult::Failed {
                message: "Parse returned no AST and no errors".to_string(),
                diff: None,
            }
        }
    };

    // Write JSON if needed
    if format == OutputFormat::Json || format == OutputFormat::Both {
        let ast_path = test.source_path.with_extension("ast.json");
        match serde_json::to_string_pretty(&ast) {
            Ok(json) => {
                if let Err(e) = fs::write(&ast_path, &json) {
                    return TestResult::Failed {
                        message: format!("Failed to write AST JSON file: {}", e),
                        diff: None,
                    };
                }
            }
            Err(e) => {
                return TestResult::Failed {
                    message: format!("Failed to serialize AST: {}", e),
                    diff: None,
                }
            }
        }
    }

    // Write DUMP if needed
    if format == OutputFormat::Dump || format == OutputFormat::Both {
        let dump_path = test.source_path.with_extension("ast.dump");
        let dump_output = DumpVisitor::dump(&ast);
        if let Err(e) = fs::write(&dump_path, &dump_output) {
            return TestResult::Failed {
                message: format!("Failed to write AST DUMP file: {}", e),
                diff: None,
            };
        }
    }

    TestResult::Passed
}

/// WIP Error update mode: create .error.txt
fn run_wip_error_update(test: &TestCase) -> TestResult {
    let source = match fs::read_to_string(&test.source_path) {
        Ok(s) => s,
        Err(e) => {
            return TestResult::Failed {
                message: format!("Failed to read source: {}", e),
                diff: None,
            }
        }
    };

    let result = frel_compiler_core::parse_file(&source);

    if !result.diagnostics.has_errors() {
        return TestResult::Failed {
            message: "Cannot update: expected parse to fail but it succeeded".to_string(),
            diff: None,
        };
    }

    let error_path = test.source_path.with_extension("error.txt");
    let error_msg = format_diagnostics(&result.diagnostics, &source);
    if let Err(e) = fs::write(&error_path, &error_msg) {
        return TestResult::Failed {
            message: format!("Failed to write error file: {}", e),
            diff: None,
        };
    }

    TestResult::Passed
}

fn run_success_test(
    test: &TestCase,
    expected_ast: &Path,
    expected_dump: Option<&PathBuf>,
    update: bool,
    format: OutputFormat,
) -> TestResult {
    // Read source file
    let source = match fs::read_to_string(&test.source_path) {
        Ok(s) => s,
        Err(e) => {
            return TestResult::Failed {
                message: format!("Failed to read source: {}", e),
                diff: None,
            }
        }
    };

    // Parse the source
    let result = frel_compiler_core::parse_file(&source);

    if result.diagnostics.has_errors() {
        let error_msg = format_diagnostics(&result.diagnostics, &source);
        return TestResult::Failed {
            message: format!("Parse failed unexpectedly:\n{}", error_msg),
            diff: None,
        };
    }

    let ast = match result.file {
        Some(ast) => ast,
        None => {
            return TestResult::Failed {
                message: "Parse returned no AST".to_string(),
                diff: None,
            }
        }
    };

    // Check JSON format
    if format == OutputFormat::Json || format == OutputFormat::Both {
        let actual_json = match serde_json::to_string_pretty(&ast) {
            Ok(j) => j,
            Err(e) => {
                return TestResult::Failed {
                    message: format!("Failed to serialize AST: {}", e),
                    diff: None,
                }
            }
        };

        if update {
            if let Err(e) = fs::write(expected_ast, &actual_json) {
                return TestResult::Failed {
                    message: format!("Failed to update expected JSON file: {}", e),
                    diff: None,
                };
            }
        } else {
            // Read expected output
            let expected_json = match fs::read_to_string(expected_ast) {
                Ok(s) => s,
                Err(e) => {
                    return TestResult::Failed {
                        message: format!("Failed to read expected AST: {}", e),
                        diff: None,
                    }
                }
            };

            // Compare
            let actual_normalized = normalize_json(&actual_json);
            let expected_normalized = normalize_json(&expected_json);

            if actual_normalized != expected_normalized {
                let diff = generate_diff(&expected_json, &actual_json);
                return TestResult::Failed {
                    message: "AST JSON mismatch".to_string(),
                    diff: Some(diff),
                };
            }
        }
    }

    // Check DUMP format
    if format == OutputFormat::Dump || format == OutputFormat::Both {
        let actual_dump = DumpVisitor::dump(&ast);
        let dump_path = test.source_path.with_extension("ast.dump");

        if update {
            if let Err(e) = fs::write(&dump_path, &actual_dump) {
                return TestResult::Failed {
                    message: format!("Failed to update expected DUMP file: {}", e),
                    diff: None,
                };
            }
        } else if let Some(expected_dump_path) = expected_dump {
            // Read expected DUMP output
            let expected = match fs::read_to_string(expected_dump_path) {
                Ok(s) => s,
                Err(e) => {
                    return TestResult::Failed {
                        message: format!("Failed to read expected DUMP: {}", e),
                        diff: None,
                    }
                }
            };

            // Compare
            if actual_dump.trim() != expected.trim() {
                let diff = generate_diff(&expected, &actual_dump);
                return TestResult::Failed {
                    message: "AST DUMP mismatch".to_string(),
                    diff: Some(diff),
                };
            }
        }
        // If no expected_dump file exists and we're not updating, skip DUMP comparison
    }

    TestResult::Passed
}

fn run_error_test(test: &TestCase, expected_error: &Path, update: bool) -> TestResult {
    // Read source file
    let source = match fs::read_to_string(&test.source_path) {
        Ok(s) => s,
        Err(e) => {
            return TestResult::Failed {
                message: format!("Failed to read source: {}", e),
                diff: None,
            }
        }
    };

    // Parse the source (expecting failure)
    let result = frel_compiler_core::parse_file(&source);

    if !result.diagnostics.has_errors() {
        return TestResult::Failed {
            message: "Expected parse error but parsing succeeded".to_string(),
            diff: None,
        };
    }

    let actual_error = format_diagnostics(&result.diagnostics, &source);

    if update {
        // Update mode: write actual error to expected file
        if let Err(e) = fs::write(expected_error, &actual_error) {
            return TestResult::Failed {
                message: format!("Failed to update expected file: {}", e),
                diff: None,
            };
        }
        return TestResult::Passed;
    }

    // Read expected error
    let expected = match fs::read_to_string(expected_error) {
        Ok(s) => s,
        Err(e) => {
            return TestResult::Failed {
                message: format!("Failed to read expected error: {}", e),
                diff: None,
            }
        }
    };

    // Compare (normalize whitespace)
    let actual_normalized = actual_error.trim();
    let expected_normalized = expected.trim();

    if actual_normalized == expected_normalized {
        TestResult::Passed
    } else {
        let diff = generate_diff(&expected, &actual_error);
        TestResult::Failed {
            message: "Error message mismatch".to_string(),
            diff: Some(diff),
        }
    }
}

fn format_diagnostics(diagnostics: &frel_compiler_core::Diagnostics, source: &str) -> String {
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

fn normalize_json(json: &str) -> String {
    // Parse and re-serialize to normalize formatting
    match serde_json::from_str::<serde_json::Value>(json) {
        Ok(v) => serde_json::to_string(&v).unwrap_or_else(|_| json.to_string()),
        Err(_) => json.to_string(),
    }
}

fn generate_diff(expected: &str, actual: &str) -> String {
    let diff = TextDiff::from_lines(expected, actual);
    let mut result = String::new();

    result.push_str("       --- expected\n");
    result.push_str("       +++ actual\n");

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-".red(),
            ChangeTag::Insert => "+".green(),
            ChangeTag::Equal => " ".normal(),
        };
        result.push_str(&format!("       {}{}", sign, change));
    }

    result
}
