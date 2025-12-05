# Testing Framework

This document describes the testing frameworks for Frel components.

## Overview

Frel has two primary testing frameworks:

| Component | Location | Approach |
|-----------|----------|----------|
| Parser | `compiler/frel-compiler-test` | AST comparison with locked baselines |
| Runtime | `host/javascript/runtime` | Trace-based verification with NDJSON baselines |

---

# Parser Testing

The parser testing framework validates that the Frel parser correctly handles both valid and invalid source files. Tests are organized as `.frel` files in `compiler/test-data/parser/`.

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

---

# Runtime Testing

The runtime testing framework validates the reactive runtime behavior using trace-based verification.
Tests are organized in `compiler/test-data/runtime/tests/`.

## Overview

Runtime tests work by:

1. Running a Frel application with tracing enabled
2. Capturing all trace events during execution
3. Comparing the captured trace against an expected baseline (NDJSON format)

This approach provides deterministic, reproducible tests that verify the exact sequence of
runtime operations.

## Trace Categories

The tracer captures events in these categories:

| Category | Events | Description |
|----------|--------|-------------|
| `datum` | create, destroy | Datum lifecycle |
| `closure` | create, destroy, instantiate | Closure/fragment lifecycle |
| `field` | set | Field value changes (with old/new values) |
| `subscription` | subscribe, unsubscribe, notify | Reactive subscriptions |
| `notification` | drain_start, drain_end, generation, callback | Drain loop execution |

## Test Structure

Each test has its own directory:

```
compiler/test-data/runtime/
├── testing.frel              # Reusable test fragments (T0, T1, HO, etc.)
├── build/
│   └── testing.js            # Compiled testing module
└── tests/
    ├── simple_instantiate/
    │   ├── simple_instantiate.frel   # Test source
    │   ├── simple_instantiate.js     # Compiled output
    │   └── baseline.ndjson           # Expected trace
    └── reactive_field/
        ├── reactive_field.frel
        ├── reactive_field.js
        └── baseline.ndjson
```

### Convention

- Each test has a `Main` blueprint as the entry point
- Tests can import from `testing.frel` for reusable fragments
- Baseline files use NDJSON format (one JSON object per line)

## NDJSON Baseline Format

Baselines are stored as newline-delimited JSON for git-friendly diffs:

```json
{"timestamp":0,"category":"closure","event":"create","data":{"id":1,"blueprint":"test.Main","parent":null}}
{"timestamp":0,"category":"closure","event":"instantiate","data":{"id":1,"blueprint":"test.Main","params":{}}}
{"timestamp":0,"category":"field","event":"set","data":{"id":1,"field":"count","new":0,"generation":0}}
```

Benefits:
- Each event is on its own line
- Git diffs show exactly which events changed
- Easy to read and edit manually
- Streamable for large traces

## Testing API

The runtime provides testing utilities in `@frel/runtime`:

```typescript
import {
    runTest,
    parseNdjson,
    compareTraces,
    assertTracesMatch,
    createTestRuntime,
} from '@frel/runtime';

// Run a test with a register function
const result = runTest(registerMetadata, {
    categories: new Set(['closure', 'field']),  // Optional filter
    entryBlueprint: 'test.Main',                // Optional override
});

// Result contains:
// - result.events: TraceEvent[]
// - result.snapshot: RuntimeSnapshot
// - result.rootId: number

// Compare with baseline
const baseline = parseNdjson(baselineContent);
const diffs = compareTraces(result.events, baseline, {
    ignoreTimestamps: true,
});

// Or use assertion
assertTracesMatch(result.events, baseline);
```

## Running Tests

```bash
# From host/javascript/runtime directory
cd host/javascript/runtime

# Build the runtime
npm run build

# Run tests (using Node.js test runner)
npm test
```

## Workflow

### Adding a New Runtime Test

1. Create a new directory in `compiler/test-data/runtime/tests/`:
   ```
   mkdir tests/my_new_test
   ```

2. Create the Frel source file with a `Main` blueprint:
   ```frel
   module my_new_test

   blueprint Main {
       // Test content
   }
   ```

3. Compile to JavaScript (using the Frel compiler)

4. Run once to generate initial trace, then save as baseline:
   ```typescript
   const result = runTest(registerMetadata);
   const ndjson = serializeToNdjson(result.events);
   // Save to baseline.ndjson
   ```

5. Verify the baseline is correct, then commit

### Updating Baselines

When runtime behavior changes intentionally:

1. Run the test to get new trace
2. Review the diff to ensure changes are expected
3. Update `baseline.ndjson` with the new trace
4. Commit the updated baseline

## Debugging with Traces

Enable console tracing for debugging:

```typescript
import { Runtime, createDebugTracer } from '@frel/runtime';

const tracer = createDebugTracer();
const runtime = new Runtime({ tracer });

// All operations are logged to console with colors
runtime.run('myapp.Main');
```

Output:
```
[closure] create id=1 blueprint="myapp.Main" parent=null
[closure] instantiate id=1 blueprint="myapp.Main" params={}
[field] set id=1 field="count" new=0 generation=0
[subscription] subscribe id=0 source=1 target=1 selector="Key(count)"
```

## Snapshot Testing

Capture final state for additional verification:

```typescript
const result = runTest(registerMetadata);

// Check final state
const snapshot = result.snapshot;
expect(snapshot.closures.size).toBe(2);

const main = snapshot.closures.get(1);
expect(main.fields.count).toBe(0);
expect(main.fields.doubled).toBe(0);
```
