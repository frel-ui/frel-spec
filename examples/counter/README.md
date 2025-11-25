# Counter Example

A simple counter application demonstrating Frel's reactive system, backends, and blueprints.

## Features

- Reactive counter state
- Backend commands (increment, decrement, reset)
- Blueprint composition
- Layout with column and row
- Event handlers
- String interpolation

## Source

See [src/counter.frel](src/counter.frel) for the complete Frel source code.

## Running

Once the compiler is complete:

```bash
# Install dependencies
npm install

# Compile Frel source
npm run compile

# Run development server
npm run dev
```

## Structure

```
counter/
├── src/
│   └── counter.frel      # Frel source code
├── index.html            # HTML entry point
├── main.js               # JavaScript bootstrap
└── package.json          # Dependencies and scripts
```
