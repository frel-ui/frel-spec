# Getting Started with Frel

This guide will help you set up the Frel development environment and understand the project structure.

## Quick Start

### 1. Clone and Explore

```bash
git clone https://github.com/frel-lang/frel.git
cd frel

# View the structure
ls -la
```

### 2. Build the Compiler

```bash
cd compiler
cargo build --release

# The compiler binary will be at: target/release/frelc
```

### 3. Set up JavaScript Runtime

```bash
cd host/javascript/runtime
npm install
npm run build
```

### 4. Set up Browser Adapter

```bash
cd host/javascript/adapters/browser
npm install
npm run build
```

### 5. Try the Example

```bash
cd examples/counter
npm install
npm run dev
```

The example will open in your browser at `http://localhost:5173`

### 6. Install VS Code Extension (Optional)

For syntax highlighting when browsing Frel code:

```bash
cd dev/vscode
npm install
npm run compile
npm run package
code --install-extension frel-0.1.0.vsix
```

This provides:
- Syntax highlighting for `.frel` files
- Bracket matching and auto-closing
- Comment toggling support

## Project Structure

```
frel/
├── compiler/                      # Rust compiler workspace
│   ├── frel-compiler-core/        # Core: parser, AST, type checker
│   ├── frel-compiler-plugin-javascript/  # JavaScript code generation
│   ├── frel-compiler-cli/         # CLI binary (frelc)
│   ├── frel-compiler-server/      # Compiler server (frel-server)
│   ├── frel-compiler-test/        # Compiler testing tool
│   └── test-data/                 # Compiler test data
│
├── host/                          # Host language implementations
│   └── javascript/
│       ├── runtime/               # Reactive runtime (platform-independent)
│       └── adapters/
│           └── browser/           # Browser DOM adapter
│
├── libraries/                     # Frel libraries
│   └── stdlib/                    # Standard library widgets
│
├── examples/                      # Example applications
│   └── counter/                   # Counter app
│
├── dev/                           # Development tools
│   ├── lsp/                       # Language Server Protocol
│   └── vscode/                    # VS Code extension
│
└── docs/                          # Documentation
    ├── 00_overview/               # Overview and setup guides
    └── 10_language/               # Language specification
```

## Development Workflow

### Writing Frel Code

1. Create a `.frel` file in your project
2. Use the language specification in `docs/10_language/`
3. Compile with `frelc compile your-file.frel`

### Compiler Development

Located in `compiler/`

**Working on the parser:**
```bash
cd compiler/frel-compiler-core
# Edit src/parser.rs
cargo test
```

**Working on code generation:**
```bash
cd compiler/frel-compiler-plugin-javascript
# Edit src/codegen.rs
cargo test
```

**Testing the CLI:**
```bash
cd compiler
cargo run --bin frelc -- compile ../examples/counter/src/counter.frel
```

### Compiler Server

The compiler server (`frel-server`) provides an always-compiled daemon with HTTP API, optimized for AI-assisted workflows.

**Starting the server:**
```bash
cd compiler
cargo run --package frel-compiler-server -- ../examples/counter
```

**One-shot compilation (for CI):**
```bash
cd compiler
cargo run --package frel-compiler-server -- ../examples/counter --once
```

**With custom port and output:**
```bash
cargo run --package frel-compiler-server -- /path/to/project --port 8080 --output dist/
```

**HTTP API endpoints:**
- `GET /status` - Server status (initialized, error_count, module_count)
- `GET /modules` - List all modules with status
- `GET /diagnostics` - All diagnostics
- `GET /diagnostics/{module}` - Module-specific diagnostics
- `GET /ast/{module}` - JSON-serialized AST
- `GET /generated/{module}` - Generated JavaScript
- `POST /notify` - Push file change notification

For full documentation, see [Compiler Server](/docs/30_compiler/10_compiler_server.md).

### Compiler Testing

The compiler has a dedicated testing framework (`frel-compiler-test`) that uses file-based tests. Test data is located in `compiler/test-data/`.

For detailed documentation, see [Parser Testing Framework](30_testing.md).

**Running compiler tests:**
```bash
cd compiler
cargo run --package frel-compiler-test
```

**Test categories:**
- **Success tests**: Expect parsing to succeed (tests not in `errors/` directory)
- **Error tests**: Expect parsing to fail (tests in an `errors/` directory)
- **WIP tests**: No output files yet, validates expected outcome only
- **Locked tests**: Have `.ast.json` or `.error.txt`, validates exact output

**Test directory structure:**
```
compiler/test-data/
└── parser/
    ├── blueprint/           # Success tests
    │   └── errors/          # Error tests for blueprints
    ├── scheme/              # Success tests
    │   └── errors/          # Error tests for schemes
    └── layout/
        └── errors/          # Error tests for layout
```

**Useful commands:**
```bash
# Run all tests
cargo run --package frel-compiler-test

# Run tests matching a pattern
cargo run --package frel-compiler-test "blueprint"

# Update expected output files (lock tests)
cargo run --package frel-compiler-test --update

# Show detailed diff on failures
cargo run --package frel-compiler-test --verbose

# Generate HTML report
cargo run --package frel-compiler-test report
```

**Adding new tests:**
1. Create a `.frel` file:
   - For success tests: `parser/<category>/test_name.frel`
   - For error tests: `parser/<category>/errors/test_name.frel`
2. Run tests to verify expected outcome
3. When ready to lock: run with `--update` to generate output files
4. Commit both `.frel` and output files

### Runtime Development

Located in `host/javascript/runtime/`

```bash
cd host/javascript/runtime

# Make changes to src/
npm run build
npm test
```

### Adapter Development

Located in `host/javascript/adapters/browser/`

```bash
cd host/javascript/adapters/browser

# Make changes to src/
npm run build

# Test with examples
cd ../../../examples/counter
npm run dev
```