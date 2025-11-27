# Frel Architecture

This document describes the architecture of the Frel project and how the different components work together.

## Overview

Frel follows a three-layer architecture that separates concerns between compilation, runtime, and rendering.

```
┌─────────────────────────────────────────────────────────────┐
│                     Frel Source Code                        │
│                      (.frel files)                          │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                  Compiler (Rust)                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │  frel-core   │─>│    Plugin    │─>│    frelc     │       │
│  │ Parser, AST  │  │ (per target) │  │              │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
└─────────────────────┬───────────────────────────────────────┘
                      │ generates
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                  Host Language Code                         │
│           (JavaScript, Kotlin, Swift, etc.)                 │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                      Runtime                                │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  • Datum storage and identity management            │    │
│  │  • Subscription and reactive propagation            │    │
│  │  • Event and notification queues                    │    │
│  │  • Fragment lifecycle                               │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                      Adapter                                │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  • Platform UI rendering                            │    │
│  │  • Event handling (click, input, etc.)              │    │
│  │  • Standard Blueprints:                             │    │
│  │    - text, image, icon                              │    │
│  │    - box, column, row                               │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Components

### 1. Compiler (Rust)

Located in `compiler/`

**frel-core**
- hand-written (well, AI written) lexer/parser
- AST construction from parse tree
- Semantic analysis and type checking
- Language-agnostic IR

**Code Generation Plugins**
- One plugin per target host language
- Examples: `frel-plugin-javascript`, `frel-plugin-kotlin`
- Produces runtime-compatible code

**frelc** (CLI)
- Command-line interface
- File compilation
- Error reporting

### 2. Runtime (Host Language)

Located in `host/{language}/runtime/`

The runtime is written once per host language and is platform-independent.

**Core Responsibilities:**
- **Datum Management**: Storage for all reactive data with identity tracking
- **Subscriptions**: Reactive dependencies between data
- **Change Propagation**: Push-based notification system with generation tracking
- **Event Queue**: Thread-safe event ingestion
- **Notification Queue**: Batched change propagation with cycle detection

**Key Classes:**
- `Runtime`: Main coordinator
- `Datum`: Fundamental unit of reactive data
- `Subscription`: Reactive dependency
- `Fragment`: Runtime blueprint instance
- `Arena`: Collection container for schemes

**Platform Independence:**
The runtime has NO knowledge of:
- Platform UI (DOM, UIKit, Android View, etc.)
- Rendering implementation
- Platform-specific event formats

### 3. Adapter (Host Language + Platform)

Located in `host/{language}/adapters/{platform}/`

One adapter is written per platform (browser, iOS, Android, desktop, etc.).

**Core Responsibilities:**
- Platform-specific UI rendering
- Event handling and normalization
- Standard blueprint implementations

**Adapter Components:**

**PlatformAdapter**
- Entry point for the specific platform
- Sets up platform event listeners
- Normalizes events and feeds them into the runtime

**Renderer**
- Manages the platform UI tree
- Applies styles and layout
- Node creation/destruction

**Standard Blueprints**
- `TextFragment`: Text rendering
- `ImageFragment`: Raster images
- `IconFragment`: Vector icons
- `BoxFragment`: Positional container
- `ColumnFragment`: Vertical layout
- `RowFragment`: Horizontal layout

### Platform Examples

| Host Language | Platform | UI Layer | Adapter Location |
|---------------|----------|----------|------------------|
| JavaScript    | Browser  | DOM      | `host/javascript/adapters/browser/` |
| Kotlin        | Android  | Android View | `host/kotlin/adapters/android/` |
| Swift         | iOS      | UIKit    | `host/swift/adapters/ios/` |
| Kotlin        | Desktop  | Skia     | `host/kotlin/adapters/desktop/` |

## Data Flow

### Compilation Flow

```
.frel source
    ↓
PEST Parser (frel.pest)
    ↓
Parse Tree
    ↓
AST Builder
    ↓
Semantic Analysis & Type Checking
    ↓
Code Generator Plugin
    ↓
Host Language Module
```

### Runtime Flow

Details:

- [Events and Notifications](/docs/10_language/80_runtime/20_events_and_notifications.md)

```
User Interaction (platform event)
    ↓
Adapter normalizes to Frel event
    ↓
Runtime.putEvent()
    ↓
Event Queue
    ↓
Runtime.drainEvents()
    ↓
Process Event → Update Datum
    ↓
Datum.setField() → Notification generation
    ↓
Queue Notifications for Subscriptions
    ↓
Runtime.drainNotifications()
    ↓
For each Notification:
    - Execute Callback
    - May trigger more changes
    ↓
Render cycle (if needed)
    ↓
Platform UI updates
```

### Reactive Propagation

Details:

- [Data Model Basics](/docs/10_language/20_data_model/01_data_model_basics.md)
- [Type System](/docs/10_language/20_data_model/02_type_system.md)
- [Reactivity](/docs/10_language/20_data_model/03_reactivity.md)
- [Events and Notifications](/docs/10_language/80_runtime/20_events_and_notifications.md)

When a field changes:

1. **Datum marks generation**: `datum.setGeneration = currentGeneration`
2. **Notifications queued**: For all subscriptions on that datum
3. **Drain loop**:
   - Take all current notifications
   - Clear queue, increment generation
   - Execute callbacks (may queue more notifications)
   - Repeat until queue is empty

This ensures:
- All dependent computations update
- Changes propagate in waves (generations)
- Diamond dependencies resolved correctly
- Infinite loops detected (generation limit)
