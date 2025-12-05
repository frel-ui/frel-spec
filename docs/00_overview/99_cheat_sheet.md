# Frel Command Cheat Sheet

Quick reference for common development commands.

## Compiler Server (frel-server)

The always-compiled daemon for development workflows. Watches files and provides HTTP API.

### Running

```bash
# Start server in current directory (default port 3001)
cargo run -p frel-compiler-server

# Specify project directory and port
cargo run -p frel-compiler-server /path/to/project -p 3001

# Custom build output directory
cargo run -p frel-compiler-server -o build

# One-shot mode (compile once and exit, for CI)
cargo run -p frel-compiler-server --once
```

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/status` | GET | Server status, error count, module count |
| `/modules` | GET | List all modules with metadata |
| `/diagnostics` | GET | All diagnostics across all modules |
| `/diagnostics/{module}` | GET | Diagnostics for specific module |
| `/ast/{module}` | GET | AST (JSON + dump) for a module |
| `/generated/{module}` | GET | Generated JavaScript for a module |
| `/scope/{module}` | GET | Scope/symbol information |
| `/source/{path}` | GET | Source file content |
| `/notify` | POST | Notify of file change (triggers rebuild) |
| `/write` | POST | Write file and trigger rebuild |
| `/expectations/{module}` | GET | Get expected results (dev mode) |
| `/expectations/{module}/save` | POST | Save current as expected |
| `/compare/{module}` | GET | Compare current vs expected |

### Quick API Access

```bash
# Check status
curl http://localhost:3001/status

# List modules
curl http://localhost:3001/modules

# Get diagnostics
curl http://localhost:3001/diagnostics

# Get generated JS for a module
curl http://localhost:3001/generated/myapp.main

# Get AST
curl http://localhost:3001/ast/myapp.main
```

---

## Grove IDE

Web-based IDE for compiler development. Connects to frel-server.

### Running

```bash
cd grove

# Install dependencies (first time)
npm install

# Start development server (port 5173)
npm run dev
```

Then open http://localhost:5173 in your browser.

**Note:** Grove proxies `/api/*` requests to frel-server on port 3001. Start frel-server first!

### Typical Development Session

```bash
# Terminal 1: Start compiler server
cargo run -p frel-compiler-server /path/to/project

# Terminal 2: Start Grove IDE
cd grove && npm run dev

# Open browser to http://localhost:5173
```

---

## Compiler

### Building

```bash
# Build all compiler crates
cargo build

# Build in release mode
cargo build --release

# Build specific crate
cargo build -p frel-compiler-core
cargo build -p frel-compiler-plugin-javascript
cargo build -p frel-compiler-server
```

### Testing

```bash
# Run all parser tests
cargo run -p frel-compiler-test

# Filter by pattern
cargo run -p frel-compiler-test "scheme"
cargo run -p frel-compiler-test "blueprint/errors"

# Update baselines (lock test outputs)
cargo run -p frel-compiler-test --update

# Update specific test
cargo run -p frel-compiler-test --update "simple_scheme"

# Verbose output (show diffs)
cargo run -p frel-compiler-test --verbose

# Generate HTML report
cargo run -p frel-compiler-test report
cargo run -p frel-compiler-test report -o report.html
```

### Running the CLI Compiler

```bash
# Compile a Frel file to JavaScript
cargo run -p frel-compiler-cli -- compile input.frel -o output.js

# With verbose output
cargo run -p frel-compiler-cli -- compile input.frel -o output.js --verbose
```

---

## JavaScript Runtime

All runtime commands are run from `host/javascript/runtime/`.

### Setup

```bash
cd host/javascript/runtime

# Install dependencies
npm install
```

### Building

```bash
# Build TypeScript to JavaScript
npm run build

# Type-check without emitting
npx tsc --noEmit

# Watch mode (rebuild on changes)
npx tsc --watch
```

### Testing

```bash
# Run all tests
npm test

# Run specific test file
npm test -- tests/runtime.test.ts
```

### Development

```bash
# Quick trace verification
node -e "
import { Runtime, createDebugTracer, Key } from './dist/index.js';
const runtime = new Runtime({ tracer: createDebugTracer() });
// ... your test code
"
```

---

## Common Workflows

### Adding a Parser Test

```bash
# 1. Create test file
echo 'module test\nblueprint Foo {}' > compiler/test-data/parser/blueprint/my_test.frel

# 2. Verify it parses
cargo run -p frel-compiler-test "my_test"

# 3. Lock the output
cargo run -p frel-compiler-test --update "my_test"

# 4. Check generated files
ls compiler/test-data/parser/blueprint/my_test.*
```

### Adding a Runtime Test

```bash
# 1. Create test directory
mkdir -p compiler/test-data/runtime/tests/my_test

# 2. Create Frel source
cat > compiler/test-data/runtime/tests/my_test/my_test.frel << 'EOF'
module my_test

blueprint Main {
    // test content
}
EOF

# 3. Compile to JavaScript (using Frel compiler)
# 4. Run and generate baseline
# 5. Save baseline.ndjson
```

### Debugging Runtime Behavior

```typescript
// In a test file or Node REPL
import { Runtime, createDebugTracer, Key } from '@frel/runtime';

const runtime = new Runtime({ tracer: createDebugTracer() });
runtime.register_metadata('test.Main', { /* ... */ });
runtime.run();
```

### Live Development with Grove

```bash
# Terminal 1: Compiler server watching your project
cargo run -p frel-compiler-server ~/myproject

# Terminal 2: Grove IDE
cd grove && npm run dev

# Browser: http://localhost:5173
# - View modules, AST, generated code
# - See diagnostics in real-time
# - Save expectations for regression testing
```

---

## Directory Structure Reference

```
frel/
├── compiler/
│   ├── frel-compiler-cli/       # CLI tool
│   ├── frel-compiler-core/      # Core compiler (parser, analysis)
│   ├── frel-compiler-server/    # Always-compiled daemon
│   ├── frel-compiler-test/      # Parser test framework
│   ├── frel-compiler-plugin-javascript/  # JS codegen
│   └── test-data/
│       ├── parser/              # Parser tests
│       │   ├── blueprint/
│       │   ├── scheme/
│       │   └── */errors/        # Error tests
│       └── runtime/             # Runtime tests
│           ├── testing.frel     # Shared test fragments
│           ├── build/           # Compiled shared modules
│           └── tests/           # Individual test cases
│
├── grove/                       # Web IDE
│   ├── src/
│   └── package.json
│
├── host/
│   └── javascript/
│       └── runtime/             # JS runtime library
│           ├── src/
│           │   ├── runtime.ts   # Core runtime
│           │   ├── tracer.ts    # Tracing system
│           │   ├── testing.ts   # Test utilities
│           │   └── types.ts     # Type definitions
│           └── dist/            # Compiled output
│
└── docs/
    ├── 00_overview/
    └── 10_language/
```

---

## Environment Setup

### Prerequisites

```bash
# Rust (for compiler)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (for runtime and grove)
# Use your preferred method (nvm, brew, etc.)
nvm install 20
nvm use 20
```

### First Time Setup

```bash
# Clone and enter
git clone <repo-url> frel
cd frel

# Build compiler
cargo build

# Setup runtime
cd host/javascript/runtime
npm install
npm run build

# Setup grove
cd ../../grove
npm install
```

---

## Quick Checks

```bash
# Compiler builds?
cargo build

# Parser tests pass?
cargo run -p frel-compiler-test

# Runtime builds?
cd host/javascript/runtime && npm run build

# Runtime type-checks?
cd host/javascript/runtime && npx tsc --noEmit

# Grove builds?
cd grove && npm run build
```
