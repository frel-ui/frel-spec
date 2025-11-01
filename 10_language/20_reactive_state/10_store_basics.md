# Store Basics

Stores are named reactive variables that form the foundation of Frel's state management system.
They participate in automatic dependency tracking and notification propagation, enabling 
declarative reactive UIs.

## Overview

Every store has:
- **Identity**: A name used to reference it within the fragment
- **Type**: The host language type of the value it holds
- **Reactivity**: How it responds to changes in dependencies
- **Mutability**: Whether it can be written to (and how)
- **Lifetime**: How long the store persists

## Store Types

Frel provides five kinds of stores, each optimized for different use cases:

| Store Type | Keyword      | Mutability | Dependencies | Use Case                            |
|------------|--------------|------------|--------------|-------------------------------------|
| Read-only  | `decl`       | Immutable  | Automatic    | Constants and derived values        |
| Writable   | `writable`   | Mutable    | None         | Local component state               |
| Session    | `session`    | Mutable    | None         | Survives fragment recreation        |
| Persistent | `persistent` | Mutable    | None         | Survives app restart                |
| Fan-in     | `fanin`      | Hybrid     | Automatic    | Combine reactive + imperative state |
| Source     | `source`     | Producer   | N/A          | Async data and external events      |

## Common Features

### Type Annotation

All stores support optional type annotations:

```frel
decl count = 0              // Type inferred as i32
decl count: u32 = 0         // Explicit type annotation
```

Type inference uses the host language's type system. If inference fails, the compiler
will request an explicit annotation.

### Dependency Tracking

Stores automatically track dependencies by analyzing expressions:

```frel
decl price = 100
decl quantity = 5
decl total = price * quantity  // Automatically subscribes to price and quantity
```

When `price` or `quantity` changes, `total` recomputes automatically.

### Glitch Freedom

The reactive system ensures **glitch-free updates**: derived stores recompute exactly once per
dependency change, even when multiple dependencies change simultaneously.

```frel
decl a = 1
decl b = a + 1
decl c = a + b  // Recomputes once when 'a' changes, not twice

button { "Update" } .. on_click { a = a + 1 }
```

### Cyclic Dependencies

Dependency graphs must be acyclic. Cycles are detected at **runtime** and cause an error.

**Why runtime detection?**

Compile-time cycle detection is not feasible due to conditional dependencies:

```frel
decl a = if some_condition { b } else { 0 }
decl b = if other_condition { a } else { 0 }
// Whether this cycles depends on runtime values of the conditions
```

The runtime detects cycles during drain notification propagation by limiting the notification cycle
count. When a cycle is detected, the system raises an error to prevent infinite loops.

This applies to both read-only stores (`decl`) and fan-in stores (`fanin`), as both participate in
automatic dependency tracking.

## Initialization Order

Stores are initialized in **declaration order** within a fragment:

```frel
fragment Example() {
    decl doubled = count * 2   // Error! 'count' not declared yet
    writable count = 0
}
```

The host language compiler enforces this ordering. Forward references are not allowed.

**Best practice:** Declare stores in dependency order - dependencies before dependents:

```frel
fragment Example() {
    writable count = 0         // Declared first
    decl doubled = count * 2   // Can reference count ✓
    decl quadrupled = doubled * 2  // Can reference doubled ✓
}
```

**Parameters are available from the start:**

```frel
fragment Example(initial: i32) {
    decl doubled = initial * 2  // ✓ Parameters available before any declarations
    writable count = initial
}
```

## Store Lifecycle and Cleanup

### Fragment-Scoped Stores (Default)

Stores without lifetime modifiers are automatically disposed when the fragment is destroyed:

```frel
fragment Child() {
    writable count = 0
    decl doubled = count * 2
    source data = fetch(|| api::get_data())

    // All stores disposed when fragment is destroyed
}

when show_child {
    Child()  // Created and destroyed as show_child changes
}
```

**Cleanup behavior:**
- **Read-only stores:** Unsubscribe from dependencies, free memory
- **Writable stores:** Free memory
- **Sources:** Dropped, which may cancel ongoing operations (source-specific)
- **Fan-in stores:** Unsubscribe from dependencies, free memory

### Session Stores

Session stores persist beyond fragment destruction but are cleared on app restart:

```frel
fragment Panel() {
    session width: "panel.width" = 300
}

// Fragment destroyed - width persists in session registry
// Fragment recreated - width restored from session registry
// App restarted - width cleared, initialized to 300
```

### Persistent Stores

Persistent stores survive app restart:

```frel
fragment Settings() {
    persistent theme: "app.theme" = "dark"
}

// Fragment destroyed - theme persists in platform storage
// App restarted - theme restored from platform storage
```

### Source Cancellation

Sources are automatically dropped when their owning fragment is destroyed. Whether ongoing operations are cancelled depends on the source implementation:

```frel
fragment DataLoader() {
    source data = fetch(|| expensive_api_call())
    // Fragment destroyed before fetch completes
    // → fetch source dropped, may cancel HTTP request
}
```

**Note:** Shared sources (like `environment()`) are never dropped as they're global.

## Detailed Documentation

- [**Read-only Stores**](20_read_only_stores.md) - Constants and derived values
- [**Writable Stores**](30_writable_stores.md) - Mutable state with lifetimes
- [**Fan-in Stores**](40_fan_in_stores.md) - Combine reactive and imperative updates
- [**Sources**](50_sources.md) - Async data producers
- [**Standard Sources**](60_standard_sources.md) - Built-in sources (environment, focus, hover)
