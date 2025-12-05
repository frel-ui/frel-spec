# JavaScript Runtime Tracing

This document describes the tracing and debugging capabilities of the Frel JavaScript runtime.

## Overview

The runtime includes a configurable tracing system for:

- **Testing**: Capture trace events and compare against baselines
- **Debugging**: Log runtime operations to console
- **Analysis**: Understand reactive propagation and performance

## Tracer Architecture

### Components

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Runtime   │────▶│   Tracer    │────▶│    Sink     │
└─────────────┘     └─────────────┘     └─────────────┘
                          │
                          ▼
                    Category Filter
```

- **Runtime**: Emits trace events at key operation points
- **Tracer**: Filters by category and timestamps events
- **Sink**: Receives and stores/outputs events

### Trace Events

```typescript
interface TraceEvent {
    timestamp: number;        // High-resolution timestamp
    category: TraceCategory;  // Event category
    event: string;            // Event name within category
    data: Record<string, unknown>;  // Event-specific data
}
```

## Trace Categories

| Category | Events | Use Case |
|----------|--------|----------|
| `datum` | create, destroy | Datum lifecycle tracking |
| `closure` | create, destroy, instantiate | Fragment lifecycle |
| `field` | set | Value change tracking |
| `subscription` | subscribe, unsubscribe, notify | Reactive flow analysis |
| `notification` | drain_start, drain_end, generation, callback | Drain loop analysis |
| `rendering` | (future) | Visual output tracking |

### Event Details

#### datum.create
```json
{ "id": 0, "type": "User", "fields": {"name": "Alice"}, "owner": 1 }
```

#### datum.destroy
```json
{ "id": 0 }
```

#### closure.create
```json
{ "id": 1, "blueprint": "myapp.Counter", "parent": null }
```

#### closure.instantiate
```json
{ "id": 1, "blueprint": "myapp.Counter", "params": {"initial": 0} }
```

#### field.set
```json
{ "id": 1, "field": "count", "old": 0, "new": 1, "generation": 5 }
```

#### subscription.subscribe
```json
{ "id": 0, "source": 1, "target": 3, "selector": "Key(count)" }
```

#### subscription.notify
```json
{ "sub_id": 0, "source": 1, "selector": "Key(count)" }
```

#### notification.drain_start
```json
{ "queue_size": 3 }
```

#### notification.generation
```json
{ "generation": 5, "callbacks": 3 }
```

#### notification.callback
```json
{ "sub_id": 0, "callback_id": 2 }
```

#### notification.drain_end
```json
{ "generations": 2 }
```

## Sink Types

### MemorySink

Stores events in an array for later analysis:

```typescript
import { MemorySink } from '@frel/runtime';

const sink = new MemorySink();
// After execution...
console.log(sink.events.length);
sink.clear();  // Reset for next test
```

### ConsoleSink

Pretty-prints events with colors:

```typescript
import { ConsoleSink } from '@frel/runtime';

const sink = new ConsoleSink();
// Events are logged as they occur:
// [closure] create id=1 blueprint="myapp.Main" parent=null
```

Color coding:
- Cyan: datum events
- Magenta: closure events
- Yellow: field events
- Green: subscription events
- Blue: notification events

### CompositeSink

Writes to multiple sinks:

```typescript
import { CompositeSink, MemorySink, ConsoleSink } from '@frel/runtime';

const memorySink = new MemorySink();
const consoleSink = new ConsoleSink();
const sink = new CompositeSink([memorySink, consoleSink]);
```

## Usage

### Basic Setup

```typescript
import { Runtime, Tracer, MemorySink, ALL_CATEGORIES } from '@frel/runtime';

const sink = new MemorySink();
const tracer = new Tracer({
    enabled: true,
    categories: ALL_CATEGORIES,
    sink,
});

const runtime = new Runtime({ tracer });
```

### Convenience Functions

```typescript
import { createTestTracer, createDebugTracer } from '@frel/runtime';

// For testing: memory sink with all categories
const testTracer = createTestTracer();

// For debugging: console sink with optional category filter
const debugTracer = createDebugTracer();
const filteredTracer = createDebugTracer(new Set(['field', 'subscription']));
```

### Category Filtering

Only capture specific categories:

```typescript
const tracer = new Tracer({
    enabled: true,
    categories: new Set(['field', 'subscription']),
    sink: new MemorySink(),
});
```

### Runtime Entry Point

The `run()` method provides a convenient entry point:

```typescript
// Finds blueprint ending with ".Main"
const rootId = runtime.run();

// Or specify explicitly
const rootId = runtime.run('mymodule.CustomEntry');
```

### State Snapshots

Capture runtime state for verification:

```typescript
const snapshot = runtime.snapshot();

// snapshot.datums: Map<DatumIdentity, DatumSnapshotData>
// snapshot.closures: Map<ClosureIdentity, ClosureSnapshotData>

for (const [id, closure] of snapshot.closures) {
    console.log(`${closure.blueprint}: ${JSON.stringify(closure.fields)}`);
}
```

## Zero Overhead

When tracing is disabled, there is minimal overhead:

```typescript
// Trace calls use optional chaining
this.tracer?.trace('field', 'set', {...});

// When tracer is undefined or disabled, the call short-circuits
```

The tracer also checks `enabled` flag before any processing:

```typescript
trace(category, event, data) {
    if (!this.enabled || !this.categories.has(category)) {
        return;  // Early exit
    }
    // ...
}
```

## Example: Debugging Reactive Flow

```typescript
import { Runtime, createDebugTracer } from '@frel/runtime';

const tracer = createDebugTracer(new Set(['field', 'subscription', 'notification']));
const runtime = new Runtime({ tracer });

// Register your metadata...
runtime.register_metadata('example.Counter', {
    internal_binding: (rt, id) => {
        rt.set(id, 'count', 0);
        rt.set(id, 'doubled', 0);
        rt.subscribe(id, id, Key('count'), (rt, sub) => {
            rt.set(sub.target_id, 'doubled', rt.get(sub.target_id, 'count') * 2);
        });
    },
    top_children: [],
    call_sites: {}
});

runtime.run('example.Counter');
```

Output:
```
[field] set id=1 field="count" new=0 generation=0
[field] set id=1 field="doubled" new=0 generation=0
[subscription] subscribe id=0 source=1 target=1 selector="Key(count)"
[notification] drain_start queue_size=1
[notification] generation generation=1 callbacks=1
[notification] callback sub_id=0 callback_id=0
[notification] drain_end generations=1
```

## See Also

- [Testing Framework](../../../00_overview/30_testing.md) - How to use tracing for tests
- [JS Runtime](30_js_runtime.md) - Runtime API reference
- [Runtime Data Model](10_runtime_data_model.md) - Conceptual model
