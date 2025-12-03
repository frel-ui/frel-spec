# Grove - Frel IDE

Grove is a web-based IDE for developing Frel applications. It connects to the `frel-compiler-server` to provide real-time compilation feedback as you edit source files.

## Features

- **Module Browser**: View all modules in your project with error counts
- **Source Editor**: Edit Frel source files with auto-save
- **Live Compilation**: See diagnostics, AST, and generated JavaScript update in real-time
- **Compiler Dev Mode**: Save and compare expected compilation results for compiler development

## Getting Started

### Prerequisites

1. Build the compiler server:
   ```bash
   cd compiler
   cargo build -p frel-compiler-server
   ```

2. Install Grove dependencies:
   ```bash
   cd grove
   npm install
   ```

### Running Grove

1. Start the compiler server for your project:
   ```bash
   cargo run -p frel-compiler-server -- /path/to/project
   ```
   The server runs on port 3001 by default.

2. Start Grove development server:
   ```bash
   cd grove
   npm run dev
   ```
   Grove runs on port 5173 by default.

3. Open http://localhost:5173 in your browser.

## User Interface

```
┌──────────────────────────────────────────────────────────────────────────┐
│                              Grove                                        │
├─────────────┬────────────────────────────┬───────────────────────────────┤
│   Modules   │         Editor             │          Output               │
│             │                            │                               │
│ ▼ examples  │  module examples.counter   │  [Diagnostics] [AST] [JS]     │
│   counter   │                            │                               │
│   ▶ utils   │  store count: i32 = 0      │  No diagnostics               │
│             │                            │                               │
│             │  increment = () {          │                               │
│             │    count += 1              │                               │
│             │  }                         │                               │
│             │                            │                               │
├─────────────┴────────────────────────────┴───────────────────────────────┤
│ Connected │ 3 modules │ 0 errors │        [✓] Compiler Dev │ Save as... │
└──────────────────────────────────────────────────────────────────────────┘
```

### Sidebar (Modules)

- Lists all modules discovered in the project
- Click to expand module and see source files
- Error badges show count of compilation errors per module

### Editor

- Displays selected source file content
- Edit directly - changes auto-save after 500ms debounce
- Dirty indicator (*) shows unsaved changes
- "Compiling..." indicator shows when recompilation is in progress

### Output Panels

Four tabs showing compilation results for the selected module:

- **Diagnostics**: Errors and warnings with file/line locations
- **AST**: JSON representation of the parsed abstract syntax tree
- **Scope**: Scope graph and symbol table from semantic analysis
- **Generated JS**: JavaScript output (empty if module has errors)

### Status Bar

Shows:
- Connection status (click to refresh)
- Module count
- Total error count
- Compiler Dev mode toggle
- Diff status (in dev mode)
- Save as Expected button (in dev mode)

## Compiler Development Mode

Compiler dev mode is designed for working on the Frel compiler itself. It allows you to:

1. **Save expected results**: Capture current AST, diagnostics, and generated JS as baseline
2. **See differences**: Visual indicator when compilation output differs from expected
3. **Accept changes**: Save new results when compiler behavior changes intentionally

### Workflow

1. Enable "Compiler Dev" checkbox in status bar
2. Select a module
3. Click "Save as Expected" to capture current compilation results
4. Make changes to the compiler code
5. Rebuild and restart the compiler server
6. Refresh Grove - diff indicators show what changed:
   - **AST**: Parser or AST structure changed
   - **Diag**: Diagnostic messages changed
   - **JS**: Code generation changed
7. If changes are intentional, click "Save as Expected" to accept

### Storage

Expectations are stored in `.grove-expectations/` in your project root:
```
.grove-expectations/
├── examples.counter.json
├── examples.utils.json
└── ...
```

Each file contains:
```json
{
  "module": "examples.counter",
  "ast": { ... },
  "diagnostics": [],
  "generated_js": "// Generated code..."
}
```

## Project Structure

```
grove/
├── package.json
├── tsconfig.json
├── vite.config.ts       # Vite config with API proxy
├── index.html
└── src/
    ├── main.tsx         # React entry point
    ├── App.tsx          # Main UI components
    ├── App.css          # Styles
    ├── api.ts           # HTTP client for compiler server
    ├── types.ts         # TypeScript type definitions
    └── hooks/
        └── useCompiler.ts   # Main state management hook
```

## API Proxy

Grove uses Vite's proxy to forward API requests to the compiler server:

```typescript
// vite.config.ts
server: {
  port: 5173,
  proxy: {
    '/api': {
      target: 'http://localhost:3001',
      changeOrigin: true,
      rewrite: (path) => path.replace(/^\/api/, ''),
    },
  },
}
```

This means:
- `fetch('/api/status')` in Grove → `http://localhost:3001/status`
- No CORS issues during development

## Building for Production

```bash
cd grove
npm run build
```

Output is written to `grove/dist/`. The built files can be served statically, but you'll need to configure your production server to proxy `/api` requests to the compiler server.

## Technology Stack

- **React 18**: UI framework
- **TypeScript**: Type safety
- **Vite**: Build tool and dev server
- **Plain CSS**: Styling (VS Code-inspired dark theme)
