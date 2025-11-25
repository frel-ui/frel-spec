# Frel Development Tools

This directory contains development tools and editor integrations for Frel.

## Structure

```
dev/
├── lsp/          # Language Server Protocol implementation
└── vscode/       # VS Code extension
```

## Language Server (LSP)

The Frel Language Server provides IDE features like:
- Syntax error highlighting
- Autocomplete
- Go-to-definition
- Hover information
- Code formatting

**Location:** `dev/lsp/frel-lsp/`

**Status:** Placeholder (to be implemented)

See [dev/lsp/frel-lsp/README.md](lsp/frel-lsp/README.md) for details.

## VS Code Extension

VS Code extension providing:
- Syntax highlighting (TextMate grammar)
- LSP integration
- Code snippets

**Location:** `dev/vscode/`

**Status:** Basic syntax highlighting implemented

See [dev/vscode/README.md](vscode/README.md) for installation and development.

## Future Tools

Planned additions:

- **Formatter**: Standalone code formatter
- **Playground**: Web-based interactive Frel playground
- **Debugger**: Debugging tools for Frel applications
- **Benchmarks**: Performance benchmarking suite
- **IntelliJ Plugin**: JetBrains IDE integration

## Contributing

When adding new development tools:

1. Create a subdirectory under `dev/`
2. Add a README.md explaining the tool
3. Update this file with a link
4. Add to `.gitignore` if needed
