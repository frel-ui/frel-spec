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
│   ├── frel-core/                 # Core: parser, AST, type checker
│   ├── frel-plugin-javascript/    # JavaScript code generation
│   └── frel-cli/                  # CLI binary
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
├── docs/                          # Documentation
│   ├── 00_overview/               # Overview and setup guides
│   └── 10_language/               # Language specification
│
└── frel.pest                      # PEST grammar
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
cd compiler/frel-core
# Edit src/parser.rs
cargo test
```

**Working on code generation:**
```bash
cd compiler/frel-plugin-javascript
# Edit src/codegen.rs
cargo test
```

**Testing the CLI:**
```bash
cd compiler
cargo run --bin frelc -- compile ../examples/counter/src/counter.frel
```

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

## Key Files to Understand

### Language Specification

Start with these documents in `docs/10_language/`:

1. [Language Overview](docs/10_language/00_overview/00_language_overview.md) - High-level concepts
2. [Data Model Basics](docs/10_language/20_data_model/01_data_model_basics.md) - Core data types
3. [Reactivity](docs/10_language/20_data_model/03_reactivity.md) - Reactive system
4. [Blueprint Basics](docs/10_language/70_blueprints/10_blueprint_basics.md) - UI components
5. [Runtime Data Model](docs/10_language/80_runtime/10_runtime_data_model.md) - Runtime behavior

### Compiler Implementation

- `compiler/frel-core/src/frel.pest` - Grammar definition
- `compiler/frel-core/src/parser.rs` - Parser implementation
- `compiler/frel-core/src/ast.rs` - AST structure
- `compiler/frel-plugin-javascript/src/codegen.rs` - JS code generation

### Runtime Implementation

- `host/javascript/runtime/src/runtime.ts` - Main runtime class
- `host/javascript/runtime/src/datum.ts` - Reactive data unit
- `host/javascript/runtime/src/subscription.ts` - Reactive dependencies
- `host/javascript/runtime/src/fragment.ts` - Blueprint instances

### Adapter Implementation

- `host/javascript/adapters/browser/src/adapter.ts` - Browser integration
- `host/javascript/adapters/browser/src/renderer.ts` - DOM rendering
- `host/javascript/adapters/browser/src/blueprints/` - Standard blueprints

## Next Steps

### For Language Design

1. Read the language specification thoroughly
2. Provide feedback on design decisions
3. Propose improvements via GitHub issues/discussions

### For Compiler Development

1. Implement AST builder from PEST parse tree
2. Add semantic analysis and type checking
3. Complete JavaScript code generation
4. Add tests for all compiler phases

### For Runtime Development

1. Complete subscription callback mechanisms
2. Add more comprehensive tests
3. Optimize change propagation
4. Profile and improve performance

### For Adapter Development

1. Complete standard blueprint implementations
2. Implement instruction application (layout, styling)
3. Add event handler binding
4. Test with real applications

### For Library Development

1. Write higher-level widgets in Frel
2. Build common UI patterns
3. Create component library

## Getting Help

- **Language Questions**: See [Language Documentation](../10_language/)
- **Architecture**: See [Architecture](20_architecture.md)
- **Issues**: GitHub Issues
- **Discussions**: GitHub Discussions