use clap::Parser;
use colored::Colorize;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "frel-compiler-test")]
#[command(about = "Testing framework for the Frel compiler")]
struct Cli {
    /// Optional filter pattern (e.g., "parser/blueprint" or "simple")
    filter: Option<String>,

    /// Update expected output files with actual results
    #[arg(long)]
    update: bool,

    /// Show detailed diff on failures
    #[arg(long)]
    verbose: bool,
}

#[derive(Debug)]
enum TestKind {
    Success { expected_ast: PathBuf },
    Error { expected_error: PathBuf },
    Wip,
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
    Skipped { reason: String },
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

    // Discover test cases
    let test_cases = discover_tests(&tests_dir, cli.filter.as_deref());

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
    let mut skipped = 0;

    for test in &test_cases {
        let result = run_test(test, cli.update);

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
            TestResult::Skipped { reason } => {
                println!("  {} {} ({})", "SKIP".yellow(), test.name, reason);
                skipped += 1;
            }
        }
    }

    // Print summary
    println!();
    println!(
        "Results: {} passed, {} failed, {} skipped",
        passed.to_string().green(),
        failed.to_string().red(),
        skipped.to_string().yellow()
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

        // Determine test kind
        let ast_path = source_path.with_extension("ast.json");
        let error_path = source_path.with_extension("error.txt");

        let kind = if ast_path.exists() {
            TestKind::Success {
                expected_ast: ast_path,
            }
        } else if error_path.exists() {
            TestKind::Error {
                expected_error: error_path,
            }
        } else {
            TestKind::Wip
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

fn run_test(test: &TestCase, update: bool) -> TestResult {
    match &test.kind {
        TestKind::Wip => {
            if update {
                // In update mode, try to create the expected file
                run_wip_test(test)
            } else {
                TestResult::Skipped {
                    reason: "no expected output file".to_string(),
                }
            }
        }
        TestKind::Success { expected_ast } => run_success_test(test, expected_ast, update),
        TestKind::Error { expected_error } => run_error_test(test, expected_error, update),
    }
}

fn run_wip_test(test: &TestCase) -> TestResult {
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

    match result {
        Ok(ast) => {
            // Serialize AST to JSON and write to .ast.json
            let ast_path = test.source_path.with_extension("ast.json");
            match serde_json::to_string_pretty(&ast) {
                Ok(json) => {
                    if let Err(e) = fs::write(&ast_path, &json) {
                        return TestResult::Failed {
                            message: format!("Failed to write AST file: {}", e),
                            diff: None,
                        };
                    }
                    TestResult::Passed
                }
                Err(e) => TestResult::Failed {
                    message: format!("Failed to serialize AST: {}", e),
                    diff: None,
                },
            }
        }
        Err(e) => {
            // Write error to .error.txt
            let error_path = test.source_path.with_extension("error.txt");
            let error_msg = format!("{}", e);
            if let Err(e) = fs::write(&error_path, &error_msg) {
                return TestResult::Failed {
                    message: format!("Failed to write error file: {}", e),
                    diff: None,
                };
            }
            TestResult::Passed
        }
    }
}

fn run_success_test(test: &TestCase, expected_ast: &Path, update: bool) -> TestResult {
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

    match result {
        Ok(ast) => {
            // Serialize AST to JSON
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
                // Update mode: write actual output to expected file
                if let Err(e) = fs::write(expected_ast, &actual_json) {
                    return TestResult::Failed {
                        message: format!("Failed to update expected file: {}", e),
                        diff: None,
                    };
                }
                return TestResult::Passed;
            }

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

            if actual_normalized == expected_normalized {
                TestResult::Passed
            } else {
                let diff = generate_diff(&expected_json, &actual_json);
                TestResult::Failed {
                    message: "AST mismatch".to_string(),
                    diff: Some(diff),
                }
            }
        }
        Err(e) => TestResult::Failed {
            message: format!("Parse failed unexpectedly: {}", e),
            diff: None,
        },
    }
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

    match result {
        Ok(_) => TestResult::Failed {
            message: "Expected parse error but parsing succeeded".to_string(),
            diff: None,
        },
        Err(e) => {
            let actual_error = format!("{}", e);

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
    }
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
