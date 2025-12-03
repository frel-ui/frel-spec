# Frel Compiler Server

The compiler server (`frel-server`) is an always-compiled daemon that watches a project directory, compiles Frel source files on change, and provides compilation results via HTTP API. It's optimized for AI-assisted workflows where fast feedback is essential.

## Design Principles

- **One server = one project**: No session management, server is tied to a single project directory
- **Always compiled**: All sources are compiled at startup and kept up-to-date
- **Immediate feedback**: File changes trigger incremental recompilation with <100ms target latency
- **Stateless startup**: Rebuilds from scratch on startup (no persistence)

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    frel-compiler-server                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────────────┐  │
│  │File Watcher │──│ Change Queue │──│ Compilation Manager    │  │
│  │  (notify)   │  │ (debounce)   │  │ (incremental engine)   │  │
│  └─────────────┘  └──────────────┘  └───────────┬────────────┘  │
│                                                  │               │
│  ┌───────────────────────────────────────────────┴─────────────┐│
│  │                     Project State                            ││
│  │  Sources │ Module Index │ Dependency Graph │ Caches         ││
│  │  SignatureRegistry (always up-to-date)                      ││
│  └──────────────────────────────────────────────────────────────┘│
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │  HTTP API    │  │    SSE       │  │  JS Output Writer    │   │
│  │  (actix)     │  │  (events)    │  │  (auto-write)        │   │
│  └──────────────┘  └──────────────┘  └──────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
compiler/frel-compiler-server/
├── Cargo.toml
└── src/
    ├── main.rs           # CLI entry point
    ├── lib.rs            # Library exports
    ├── server.rs         # HTTP server setup (actix-web)
    ├── api.rs            # API endpoint handlers
    ├── state.rs          # ProjectState, caches, ModuleIndex, DependencyGraph
    ├── compiler.rs       # Compilation logic (full_build, handle_file_change)
    ├── watcher.rs        # File watching (notify crate)
    └── events.rs         # SSE event types
```

## CLI Usage

```bash
# Start server for current directory
frel-server

# Start server for specific project
frel-server /path/to/project --port 8080

# One-shot compilation (for CI/scripts)
frel-server --once && echo "Build succeeded"

# Specify output directory
frel-server --output dist/
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `[PROJECT]` | `.` | Project directory |
| `-p, --port` | `3001` | HTTP server port |
| `-o, --output` | `build` | Build output directory |
| `--once` | - | Exit after first compilation |

## HTTP API

### Status

```
GET /status
```

Returns server and compilation status.

**Response:**
```json
{
  "initialized": true,
  "error_count": 0,
  "module_count": 3
}
```

### List Modules

```
GET /modules
```

Returns all discovered modules with their status.

**Response:**
```json
{
  "modules": [
    {
      "path": "examples.counter",
      "source_files": ["/path/to/counter.frel"],
      "has_errors": false,
      "error_count": 0,
      "warning_count": 0
    }
  ]
}
```

### Get Diagnostics

```
GET /diagnostics
GET /diagnostics/{module}
```

Returns diagnostics for all modules or a specific module.

**Response:**
```json
{
  "module": "examples.counter",
  "diagnostics": [
    {
      "severity": "error",
      "code": "E0301",
      "message": "cannot find `foo` in this scope",
      "file": "/path/to/counter.frel",
      "line": 10,
      "column": 5
    }
  ],
  "error_count": 1,
  "warning_count": 0
}
```

### Get AST

```
GET /ast/{module}
```

Returns the JSON-serialized AST for a module.

**Response:**
```json
{
  "module": "examples.counter",
  "ast": { ... }
}
```

### Get Generated Code

```
GET /generated/{module}
```

Returns generated JavaScript for a module (empty if module has errors).

**Response:**
```json
{
  "module": "examples.counter",
  "javascript": "// Generated code..."
}
```

### Push File Change

```
POST /notify
Content-Type: application/json

{
  "path": "/path/to/file.frel"
}
```

Notifies the server of a file change (alternative to file watching).

**Response:**
```json
{
  "success": true,
  "modules_rebuilt": ["examples.counter"],
  "duration_ms": 45,
  "error_count": 0
}
```

### Server-Sent Events

```
GET /events
```

SSE stream for real-time compilation events.

**Events:**
```
data: {"type": "build_started", "modules": ["examples.counter"]}

data: {"type": "build_completed", "duration_ms": 45, "modules_built": 1, "error_count": 0}

data: {"type": "file_changed", "path": "/path/to/file.frel"}

data: {"type": "module_updated", "module": "examples.counter", "has_errors": false}
```

## Compilation Flow

### Initial Build (Startup)

1. Discover all `.frel` files in project (glob `**/*.frel`)
2. Read and parse all files
3. Build module index from `module` declarations
4. Build dependency graph from imports
5. **Phase 1**: Build signatures for all modules (parallel)
6. Register all signatures in registry
7. **Phase 2**: Analyze all modules (parallel)
8. Generate JavaScript for error-free modules
9. Write `.js` files to build directory
10. Mark `initialized = true`

### Incremental Rebuild (File Change)

1. Read new content, compute hash
2. Quick exit if content unchanged
3. Re-parse changed file
4. Update module index if `module` declaration changed
5. Rebuild signature for affected module
6. Compare exports hash:
   - **Unchanged**: Only re-analyze this module
   - **Changed**: Re-analyze this module + all transitive importers
7. Regenerate JavaScript for affected modules
8. Write updated `.js` files
9. Broadcast events to SSE clients

### Output Mapping

```
Source:  {project}/src/counter.frel
Module:  examples.counter (from `module examples.counter` declaration)
Output:  {project}/build/examples/counter.js
```

## Core Data Structures

### ProjectState

The main in-memory state container:

```rust
pub struct ProjectState {
    pub root: PathBuf,           // Project root directory
    pub build_dir: PathBuf,      // Build output directory
    pub sources: HashMap<PathBuf, FileState>,
    pub module_index: ModuleIndex,
    pub dependencies: DependencyGraph,
    pub parse_cache: HashMap<PathBuf, ParseCacheEntry>,
    pub signature_cache: HashMap<String, SignatureCacheEntry>,
    pub analysis_cache: HashMap<String, AnalysisCacheEntry>,
    pub registry: SignatureRegistry,
    pub generation: u64,         // Cache invalidation counter
    pub initialized: bool,
}
```

### ModuleIndex

Maps files to modules and vice versa:

```rust
pub struct ModuleIndex {
    file_to_module: HashMap<PathBuf, String>,
    module_to_files: HashMap<String, Vec<PathBuf>>,
}
```

### DependencyGraph

Tracks import relationships for invalidation:

```rust
pub struct DependencyGraph {
    importers: HashMap<String, HashSet<String>>,  // reverse deps
    imports: HashMap<String, HashSet<String>>,    // forward deps
}
```

### Cache Entries

```rust
pub struct ParseCacheEntry {
    pub file: ast::File,
    pub diagnostics: Diagnostics,
    pub content_hash: u64,
}

pub struct SignatureCacheEntry {
    pub result: SignatureResult,
    pub exports_hash: u64,     // For detecting export changes
    pub generation: u64,
}

pub struct AnalysisCacheEntry {
    pub result: ModuleAnalysisResult,
    pub generated_js: String,
    pub generation: u64,
}
```

## Incremental Compilation Strategy

The server uses smart invalidation to minimize recompilation:

1. **Content hashing**: Files are only re-parsed if their content hash changes
2. **Exports hashing**: Module signatures track an exports hash to detect API changes
3. **Dependency tracking**: The dependency graph knows which modules import from which
4. **Transitive invalidation**: When exports change, all transitive importers are re-analyzed

### Invalidation Rules

| Change Type | Phase 1 (Signature) | Phase 2 (Analysis) |
|-------------|---------------------|-------------------|
| File content unchanged | Skip | Skip |
| Internal change (exports same) | Rebuild | Rebuild this module only |
| Export changed | Rebuild | Rebuild this module + all importers |

## Performance Targets

| Operation | Target |
|-----------|--------|
| Single file change (no export change) | <50ms |
| Single file change (export changed, few deps) | <100ms |
| Initial build (small project, ~10 files) | <500ms |
| Initial build (medium project, ~50 files) | <2s |

## Integration Examples

### AI Workflow Integration

```python
import requests

# Start compilation
def compile_project(project_path):
    # Server should already be running
    response = requests.get(f"http://localhost:3001/status")
    status = response.json()

    if not status["initialized"]:
        raise Exception("Server not ready")

    if status["error_count"] > 0:
        diags = requests.get(f"http://localhost:3001/diagnostics").json()
        return {"success": False, "diagnostics": diags}

    return {"success": True, "module_count": status["module_count"]}

# Notify of file change (if not using file watcher)
def notify_change(file_path):
    response = requests.post(
        "http://localhost:3001/notify",
        json={"path": file_path}
    )
    return response.json()
```

### CI Integration

```bash
#!/bin/bash
# Build and check for errors
frel-server /path/to/project --once
exit_code=$?

if [ $exit_code -eq 0 ]; then
    echo "Build successful"
else
    echo "Build failed"
    exit 1
fi
```

## Future Enhancements

- WebSocket alternative to SSE
- LSP integration
- Disk-based signature caching
- Hot module replacement
- Multi-project support
- External module dependencies (stdlib, packages)
