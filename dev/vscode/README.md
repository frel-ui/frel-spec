# Frel VS Code Extension

Visual Studio Code extension for the Frel language.

## Features

- ✅ **Syntax Highlighting**: TextMate grammar for Frel syntax
- ✅ **Bracket Matching**: Auto-closing and highlighting
- ✅ **Comment Support**: Line and block comments
- 🚧 **LSP Integration**: Language server features (planned)

## Installation

### From Source

```bash
cd dev/vscode
npm install
npm run compile
npm run package

# Install extension
code --install-extension frel-0.1.0.vsix
```

### Development Mode

1. Open this directory in VS Code
2. Press `F5` to launch Extension Development Host
3. Open a `.frel` file to test syntax highlighting

## Syntax Highlighting

The extension provides syntax highlighting for:

- **Keywords**: `blueprint`, `backend`, `scheme`, `when`, `repeat`, etc.
- **Types**: `i32`, `String`, `List`, `Blueprint`, etc.
- **Strings**: Including template interpolation `${...}`
- **Numbers**: Decimal, hex, binary, octal, floats
- **Comments**: `//` and `/* */`
- **Operators**: Arithmetic, logical, comparison

## Language Configuration

- Auto-closing pairs for `{}`, `[]`, `()`, `""`
- Block comment toggling with `Ctrl+/`
- Auto-indentation based on braces

## Future Features

Once the LSP server is implemented:

- Error diagnostics
- Autocomplete
- Go-to-definition
- Hover type information
- Code formatting
- Refactoring actions

## Building

```bash
# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Package extension
npm run package
```

This creates `frel-0.1.0.vsix` which can be installed in VS Code.

## Testing

Open any `.frel` file to test syntax highlighting. Try:

```frel
// Test file
module test

blueprint Counter {
    count : i32 = 0

    column {
        text { "Count: ${count}" }

        button { "Click" }
            .. on_click { count = count + 1 }
    }
}
```

## Configuration

Currently no configuration options. Future LSP integration will add:

- `frel.languageServer.path` - Path to frel-lsp binary
- `frel.trace.server` - LSP trace level

## Contributing

The extension source will be in `src/extension.ts` once LSP integration is added.

For now, the TextMate grammar in `syntaxes/frel.tmLanguage.json` defines syntax highlighting rules.
