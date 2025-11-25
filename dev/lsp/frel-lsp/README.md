# Frel Language Server

Language Server Protocol (LSP) implementation for Frel, providing IDE features.

## Status

**Placeholder** - To be implemented once the compiler is stable.

## Planned Features

### Phase 1: Diagnostics
- Parse errors and syntax highlighting
- Semantic analysis errors
- Type checking errors

### Phase 2: Basic Navigation
- Go-to-definition
- Find references
- Document symbols

### Phase 3: Code Intelligence
- Autocomplete (keywords, identifiers, snippets)
- Hover information (types, documentation)
- Signature help

### Phase 4: Code Actions
- Quick fixes
- Refactoring actions
- Code formatting

## Architecture

```
frel-lsp (Rust binary)
    ↓
Uses tower-lsp for LSP protocol
    ↓
Reuses frel-core for parsing/analysis
    ↓
Communicates with editors via stdio/socket
```

## Building

```bash
cd dev/lsp/frel-lsp
cargo build --release

# Binary will be at: target/release/frel-lsp
```

## Testing Locally

Once implemented, test with any LSP-compatible editor:

**VS Code:**
```json
// .vscode/settings.json
{
  "frel.languageServer": {
    "path": "/path/to/frel-lsp"
  }
}
```

**Neovim:**
```lua
-- After building frel-lsp
require'lspconfig'.frel.setup{
  cmd = { '/path/to/frel-lsp' }
}
```

## Implementation References

- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [tower-lsp](https://github.com/ebkalderon/tower-lsp) - Rust LSP framework
- [rust-analyzer](https://github.com/rust-lang/rust-analyzer) - Reference implementation

## Dependencies

The LSP server reuses `frel-core` from the compiler, ensuring:
- Consistent parsing behavior
- Same error messages
- No duplication of logic
