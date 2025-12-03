# Frel Compiler Architecture

The Frel compiler is implemented in Rust and transforms Frel source code into host language code (JavaScript, Kotlin, etc.) through a two-phase, module-based compilation pipeline.

## Workspace Structure

```
compiler/
├── frel-compiler-core/           # Core compiler library
│   └── src/
│       ├── lib.rs                # Public API
│       ├── lexer/                # Tokenization
│       ├── parser/               # Recursive descent parser
│       ├── ast/                  # Abstract Syntax Tree
│       ├── semantic/             # Semantic analysis
│       ├── diagnostic/           # Error reporting
│       └── source.rs             # Span and source tracking
├── frel-compiler-cli/            # Command-line tool (frelc)
├── frel-compiler-server/         # HTTP compilation server (planned)
├── frel-compiler-test/           # Test runner
└── frel-compiler-plugin-javascript/  # JS code generation
```

## Core Concept: Module-Based Two-Phase Compilation

### Compilation Unit = Module

- Multiple `.frel` files can contribute to the same module (same `module` declaration)
- Files are grouped by module path before compilation
- Cross-file name clashes within a module are detected during analysis
- The cacheable unit is the **module signature**, not the file

### Two-Phase Compilation Flow

```
                    ┌──────────────────────────────────────────┐
                    │           Source Files                    │
                    │  test/data/user.frel  (module test.data)  │
                    │  test/data/order.frel (module test.data)  │
                    │  test/main.frel       (module test.main)  │
                    └──────────────────┬───────────────────────┘
                                       │
                                       ▼
                    ┌──────────────────────────────────────────┐
                    │         Group by Module Path              │
                    │  test.data: [user.frel, order.frel]       │
                    │  test.main: [main.frel]                   │
                    └──────────────────┬───────────────────────┘
                                       │
         ┌─────────────────────────────┼─────────────────────────────┐
         │                             │                             │
         ▼                             ▼                             ▼
┌─────────────────┐          ┌─────────────────┐          ┌─────────────────┐
│   PHASE 1       │          │   PHASE 1       │          │  EXTERNAL       │
│   test.data     │          │   test.main     │          │  MODULE SIG     │
│                 │          │                 │          │  (from cache    │
│ - Parse files   │          │ - Parse files   │          │   or remote)    │
│ - Build scopes  │          │ - Build scopes  │          │                 │
│ - Build symbols │          │ - Build symbols │          │                 │
│ - Extract exports│         │ - Extract exports│         │                 │
└────────┬────────┘          └────────┬────────┘          └────────┬────────┘
         │                             │                             │
         ▼                             ▼                             ▼
┌─────────────────┐          ┌─────────────────┐          ┌─────────────────┐
│ ModuleSignature │          │ ModuleSignature │          │ ModuleSignature │
│   test.data     │          │   test.main     │          │   external.lib  │
└────────┬────────┘          └────────┬────────┘          └────────┬────────┘
         │                             │                             │
         └─────────────────────────────┼─────────────────────────────┘
                                       │
                                       ▼
                    ┌──────────────────────────────────────────┐
                    │            Signature Registry             │
                    │  (all module signatures available)        │
                    └──────────────────┬───────────────────────┘
                                       │
         ┌─────────────────────────────┼─────────────────────────────┐
         │                             │                             │
         ▼                             ▼                             ▼
┌─────────────────┐          ┌─────────────────┐                    ...
│   PHASE 2       │          │   PHASE 2       │
│   test.data     │          │   test.main     │
│                 │          │                 │
│ - Import symbols│          │ - Import symbols│
│ - Resolve types │          │ - Resolve types │
│ - Type check    │          │ - Type check    │
└────────┬────────┘          └────────┬────────┘
         │                             │
         ▼                             ▼
┌─────────────────┐          ┌─────────────────┐
│ SemanticResult  │          │ SemanticResult  │
│   test.data     │          │   test.main     │
└─────────────────┘          └─────────────────┘
         │                             │
         ▼                             ▼
┌─────────────────┐          ┌─────────────────┐
│ Code Generation │          │ Code Generation │
└─────────────────┘          └─────────────────┘
```

### Key Properties

1. **Phase 1 is independent per module** - Can run in parallel, no cross-module dependencies
2. **Module signatures are the cache boundary** - Phase 1 results can be cached/serialized
3. **External modules just provide signatures** - From cache, remote, or stubs
4. **Phase 2 uses signature registry** - All imported module signatures must be available

## Module and Signature Types

### Module

```rust
/// A module compilation unit (one or more files)
pub struct Module {
    /// Module path (e.g., "test.data")
    pub path: String,
    /// Files that make up this module
    pub files: Vec<ast::File>,
}
```

### Module Signature

The signature contains everything needed to compile code that imports from this module:

```rust
pub struct ModuleSignature {
    /// Format version for compatibility
    pub version: u32,
    /// Module path (e.g., "test.data")
    pub path: String,
    /// Exported declarations (public API)
    pub exports: Vec<ExportedDecl>,
    /// Scope graph for member lookup
    pub scopes: SerializableScopeGraph,
    /// Symbol table
    pub symbols: SerializableSymbolTable,
}

pub struct ExportedDecl {
    pub name: String,
    pub kind: SymbolKind,
    pub symbol_id: SymbolId,
    pub body_scope: Option<ScopeId>,
}
```

### Signature Registry

```rust
pub struct SignatureRegistry {
    signatures: HashMap<String, ModuleSignature>,
}

impl SignatureRegistry {
    fn register(&mut self, signature: ModuleSignature);
    fn get(&self, module_path: &str) -> Option<&ModuleSignature>;
    fn resolve_import(&self, module: &str, name: &str) -> Option<&ExportedDecl>;
}
```

## Compilation Phases Detail

### Phase 1a: Lexical Analysis

**Location:** `src/lexer/`

The lexer transforms source text into a stream of tokens.

#### Key Types

- **`Token`**: A lexical unit with kind, span, and optional value
- **`TokenKind`**: Enum of all token types (keywords, operators, literals, etc.)
- **`Span`**: Byte offset range `(start, end)` in source

#### Features

- Hand-written scanner (not generated)
- Contextual keywords (e.g., `module`, `blueprint` are identifiers unless at declaration position)
- Layout block lexing for grid syntax
- String interpolation support
- Color literals (`#fff`, `#rgba`)
- Comprehensive error recovery

```rust
// Example token stream for: module test.app
[
    Token { kind: Identifier, text: "module" },
    Token { kind: Identifier, text: "test" },
    Token { kind: Dot },
    Token { kind: Identifier, text: "app" },
]
```

### Phase 1b: Parsing

**Location:** `src/parser/`

The parser builds an Abstract Syntax Tree from tokens using recursive descent with Pratt parsing for expressions.

#### Parser Structure

```
parser/
├── mod.rs        # Parser state and top-level parsing
├── expr.rs       # Expression parsing (Pratt algorithm)
├── types.rs      # Type expression parsing
├── blueprint.rs  # Blueprint declarations
├── backend.rs    # Backend declarations
├── scheme.rs     # Scheme declarations
├── contract.rs   # Contract declarations
├── enum_decl.rs  # Enum declarations
├── theme.rs      # Theme declarations
├── arena.rs      # Arena declarations
├── layout.rs     # Layout grid parsing
└── common.rs     # Shared utilities
```

#### Key Types

- **`Parser`**: Stateful parser with token cursor
- **`ParseResult`**: Contains `Option<ast::File>` and `Diagnostics`

#### Error Recovery

The parser continues after errors by synchronizing to recovery points:
- Top-level declaration keywords
- Closing braces
- End of file

### Phase 1c: Scope and Symbol Building

**Location:** `src/semantic/resolve.rs`, `src/semantic/signature_builder.rs`

Build the scope graph and symbol table, then extract exports for the module signature.

```rust
/// Phase 1: Build module signature (no cross-module deps)
pub fn build_signature(module: &Module) -> SignatureResult {
    // 1. Parse each file
    // 2. Build scope graph
    // 3. Build symbol table
    // 4. Extract exports
    // Returns: ModuleSignature + Diagnostics
}
```

### Phase 2: Cross-Module Analysis

**Location:** `src/semantic/module_analysis.rs`

With all signatures in the registry, perform full semantic analysis.

```rust
/// Phase 2: Full semantic analysis (needs imported signatures)
pub fn analyze_module(
    module: &Module,
    registry: &SignatureRegistry,
) -> ModuleAnalysisResult;
```

#### Unified Symbol Approach

External symbols are imported into the local `SymbolTable` during name resolution:

```rust
// In resolve.rs
if let Some(export) = registry.resolve_import(&import.module, &import.name) {
    symbols.define_external(
        &export.name,
        export.kind,
        ScopeId::ROOT,
        import.span,
        import.module.clone(),  // source_module marks it as external
    );
}
```

This allows type checking to use uniform symbol lookup for both local and imported types.

## Abstract Syntax Tree

**Location:** `src/ast/`

### Top-Level Structure

```rust
struct File {
    module: String,              // Module path (e.g., "test.app")
    source_path: Option<String>, // File path for diagnostics
    imports: Vec<Import>,        // import statements
    declarations: Vec<TopLevelDecl>,
}

enum TopLevelDecl {
    Blueprint(Blueprint),
    Backend(Backend),
    Contract(Contract),
    Scheme(Scheme),
    Enum(Enum),
    Theme(Theme),
    Arena(Arena),
}
```

### Expression AST

```rust
enum Expr {
    // Literals
    Null, Bool(bool), Int(i64), Float(f64), String(String),
    Color(u32), List(Vec<Expr>), Object(Vec<(String, Expr)>),

    // Identifiers
    Identifier(String),

    // Operators
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Unary { op: UnaryOp, expr: Box<Expr> },
    Ternary { condition, then_expr, else_expr },

    // Access
    FieldAccess { base: Box<Expr>, field: String },
    OptionalChain { base: Box<Expr>, field: String },
    Call { callee: Box<Expr>, args: Vec<Expr> },
}
```

### AST Visitor

The `Visitor` trait enables AST traversal:

```rust
trait Visitor {
    fn visit_file(&mut self, file: &File);
    fn visit_blueprint(&mut self, bp: &Blueprint);
    fn visit_expr(&mut self, expr: &Expr);
    // ... more visit methods
}
```

## Semantic Analysis

**Location:** `src/semantic/`

### Submodules

```
semantic/
├── mod.rs              # analyze() entry point
├── scope.rs            # Scope graph construction
├── symbol.rs           # Symbol table
├── resolve.rs          # Name resolution
├── typecheck.rs        # Type checking
├── types.rs            # Type system
├── signature.rs        # Module signatures
├── signature_builder.rs # Build signatures from AST
├── module_analysis.rs  # Cross-module analysis
└── dump.rs             # Debug output
```

### Scope Graph

```rust
struct ScopeGraph {
    scopes: Vec<Scope>,
}

struct Scope {
    id: ScopeId,
    kind: ScopeKind,     // Module, Backend, Blueprint, etc.
    parent: Option<ScopeId>,
    children: Vec<ScopeId>,
    name: Option<String>,
    span: Span,
}
```

### Symbol Table

```rust
struct SymbolTable {
    symbols: Vec<Symbol>,
    name_lookup: HashMap<(ScopeId, String), SymbolId>,
    scope_symbols: HashMap<ScopeId, Vec<SymbolId>>,
}

struct Symbol {
    id: SymbolId,
    name: String,
    kind: SymbolKind,     // Backend, Blueprint, Scheme, Field, etc.
    scope: ScopeId,
    def_span: Span,
    body_scope: Option<ScopeId>,
    source_module: Option<String>,  // For imported symbols (None = local)
}
```

### Type System

```rust
enum Type {
    // Intrinsic types
    Null, Bool, I32, I64, F32, F64, String,
    Color, Duration, Instant, Uuid,

    // Type constructors
    Nullable(Box<Type>),
    List(Box<Type>),
    Set(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Tree(Box<Type>),
    Ref(Box<Type>),
    Draft(Box<Type>),
    Asset(Box<Type>),
    Accessor(Box<Type>),

    // Named types (resolved to symbol)
    Scheme(SymbolId),
    Backend(SymbolId),
    Blueprint(SymbolId),
    Enum(SymbolId),

    // Special
    Unknown,
    Error,
}
```

## Code Generation

**Location:** `frel-compiler-plugin-javascript/`

Code generation is plugin-based, with one plugin per target host language.

### Plugin Interface

```rust
pub fn generate(file: &ast::File) -> String {
    codegen::generate_file(file)
}
```

### JavaScript Plugin

Generates ES6 modules:
- Backends → JavaScript classes
- Blueprints → Fragment factory functions
- Schemes → TypeScript interfaces (optional)
- Enums → JavaScript objects

## Diagnostic System

**Location:** `src/diagnostic/`

Structured, machine-readable diagnostics compatible with LSP.

### Diagnostic Structure

```rust
struct Diagnostic {
    severity: Severity,        // Error, Warning, Info, Hint
    code: Option<String>,      // E0301, W0101, etc.
    message: String,
    span: Span,
    labels: Vec<Label>,        // Additional source locations
    suggestions: Vec<Suggestion>, // Quick fixes
    help: Option<String>,
    related: Vec<RelatedInfo>, // Cross-file references
    tags: Vec<DiagnosticTag>,  // Unnecessary, Deprecated
}
```

### Error Codes

Organized by category:
- `E01xx`: Lexer errors
- `E02xx`: Parser errors
- `E03xx`: Name resolution errors
- `E04xx`: Type errors
- `W0xxx`: Warnings

## Compiler Server

**Location:** `frel-compiler-server/`

An always-compiled daemon that watches a project directory, compiles Frel source files on change, and provides compilation results via HTTP API. Optimized for AI-assisted workflows.

For full documentation, see [Compiler Server](10_compiler_server.md).

### Quick Start

```bash
# Start server
frel-server /path/to/project --port 3001

# One-shot compilation (for CI)
frel-server /path/to/project --once
```

### Key Features

- **One server = one project**: No session management
- **Always compiled**: Full build on startup, incremental on change
- **File watching**: Automatic recompilation with 50ms debounce
- **HTTP API**: Status, modules, diagnostics, AST, generated code
- **Fast feedback**: <100ms target for single-file changes

### HTTP API Summary

| Endpoint | Description |
|----------|-------------|
| `GET /status` | Server status (initialized, error_count, module_count) |
| `GET /modules` | List all modules with status |
| `GET /diagnostics` | All diagnostics |
| `GET /diagnostics/{module}` | Module diagnostics |
| `GET /ast/{module}` | JSON-serialized AST |
| `GET /generated/{module}` | Generated JavaScript |
| `POST /notify` | Push file change notification |
| `GET /events` | SSE stream for compilation events |

## CLI Tool (frelc)

**Location:** `frel-compiler-cli/`

```bash
# Compile to JavaScript
frelc compile input.frel -o output.js

# Check without generating code
frelc check input.frel

# Show version
frelc version
```

## Testing

**Location:** `frel-compiler-test/`

Test runner for parser and semantic analysis tests with HTML report generation.

## Design Decisions

1. **Module path from source**: The module path comes solely from the `module` declaration in source code, not from file paths. File organization is up to the user.

2. **Two-phase compilation**: Phase 1 (scope building) is independent per module and produces cacheable signatures. Phase 2 (type checking) uses the signature registry for cross-module resolution.

3. **No circular imports**: Cross-module circular imports are forbidden. This enforces a clean dependency tree and enables simple topological sort for compilation order. Circular references *within* a module are allowed.

4. **Unified symbol registry**: External symbols are imported into the local SymbolTable with a `source_module` marker. Type checking doesn't distinguish between local and imported types.

5. **Versioned signatures**: Module signatures include a version number. Incompatible versions are rejected, forcing recompilation. This enables safe caching and external module distribution.

6. **Hand-written lexer/parser**: Maximum control over error recovery and diagnostics.

7. **Contextual keywords**: Better identifier flexibility while maintaining clean syntax.

8. **Span-based tracking**: Every AST node knows its source location for precise error reporting.

9. **Plugin-based code generation**: Clean separation between analysis and output, enabling multiple target languages.
