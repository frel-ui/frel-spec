# Parser Testing Framework

This document describes the testing framework for the Frel parser, located in `compiler/frel-compiler-test`.

## Overview

The parser testing framework validates that the Frel parser correctly handles both valid and invalid source files. Tests are organized as `.frel` files in `compiler/test-data/`.

## Test Categories

Tests are categorized based on two factors:
1. **Expected outcome**: Whether parsing should succeed or fail
2. **Verification state**: Whether the output has been verified (locked) or is work-in-progress (WIP)

| Category | Detection | Pass Condition |
|----------|-----------|----------------|
| Locked Success | has `.ast.json` | Parse succeeds AND output matches |
| Locked Error | has `.error.txt` | Parse fails AND error matches |
| WIP Success | no output files, not in `errors/` | Parse succeeds |
| WIP Error | no output files, in `errors/` directory | Parse fails |

## Directory Convention

The expected outcome for WIP tests is determined by directory structure:

- Tests in a directory named `errors` (direct parent) expect parsing to **fail**
- All other tests expect parsing to **succeed**

```
compiler/test-data/parser/
├── scheme/
│   ├── simple_scheme.frel           # WIP Success - expects parse to succeed
│   ├── scheme_complex.frel          # WIP Success
│   └── errors/
│       ├── empty_scheme.frel        # WIP Error - expects parse to fail
│       └── missing_colon.frel       # WIP Error
├── layout/
│   ├── grid_basic.frel              # WIP Success
│   └── errors/
│       └── unclosed.frel            # WIP Error
└── ...
```

## Output Files

When a test is "locked" (verified), it has associated output files:

### Success Tests

- `test.ast.json` - Expected AST in JSON format (required)
- `test.ast.dump` - Human-readable AST dump (optional)

### Error Tests

- `test.error.txt` - Expected error message

## Running Tests

```bash
# Run all tests
cargo run -p frel-compiler-test

# Filter tests by pattern
cargo run -p frel-compiler-test "scheme"
cargo run -p frel-compiler-test "layout/errors"

# Update expected outputs (lock tests)
cargo run -p frel-compiler-test --update

# Show verbose diff on failures
cargo run -p frel-compiler-test --verbose

# Control output format
cargo run -p frel-compiler-test --update --format json   # Only .ast.json
cargo run -p frel-compiler-test --update --format dump   # Only .ast.dump
cargo run -p frel-compiler-test --update --format both   # Both files
```

## HTML Report

Generate a visual HTML report of all test cases:

```bash
cargo run -p frel-compiler-test report

# Custom output path
cargo run -p frel-compiler-test report -o /path/to/report.html
```

The report shows:
- Source code with syntax highlighting
- AST dump (human-readable)
- AST JSON (expandable)
- Error messages for error tests

Test status is color-coded:
- **Green (Success)**: Locked test passing
- **Red (Error)**: Locked error test passing
- **Orange (WIP)**: Work-in-progress test passing
- **Red (WIP FAIL)**: WIP test failing (outcome doesn't match expectation)

## Workflow

### Adding a New Test

1. Create a `.frel` file in the appropriate directory:
   - For tests that should parse successfully: `parser/<category>/test_name.frel`
   - For tests that should fail: `parser/<category>/errors/test_name.frel`

2. Run the tests to verify the expected outcome:
   ```bash
   cargo run -p frel-compiler-test "test_name"
   ```

3. When ready to lock the test output:
   ```bash
   cargo run -p frel-compiler-test --update "test_name"
   ```

### WIP Development

During active parser development:

1. Create test files in the appropriate directories
2. Run tests to see which pass/fail based on expected outcomes
3. Tests that don't match expectations will show as failures
4. Use the HTML report for visual inspection of parser output
5. Lock tests once the output is verified correct

### Detecting Parser Regressions

After locking tests with `--update`:

1. Any parser change that affects output will cause locked tests to fail
2. Run with `--verbose` to see the diff
3. Either fix the regression or update the expected output if the change is intentional

## Test Result States

| State | Meaning |
|-------|---------|
| PASS | Test outcome matches expectation |
| FAIL | Test outcome doesn't match expectation |

For WIP tests:
- **WIP Success + parse succeeds** → PASS
- **WIP Success + parse fails** → FAIL (parser error unexpected)
- **WIP Error + parse fails** → PASS
- **WIP Error + parse succeeds** → FAIL (error expected but didn't occur)

For locked tests:
- **Locked Success + parse succeeds + output matches** → PASS
- **Locked Success + parse fails OR output differs** → FAIL
- **Locked Error + parse fails + error matches** → PASS
- **Locked Error + parse succeeds OR error differs** → FAIL
