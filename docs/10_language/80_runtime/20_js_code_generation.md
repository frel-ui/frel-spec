# JavaScript Code Generation

This document describes what the Frel compiler generates for JavaScript targets. The generated
code works with the Frel runtime library (see [JS Runtime](30_js_runtime.md)).

## Generated vs Runtime

| Generated (per-app) | Runtime Library (shared) |
|---------------------|--------------------------|
| Subscription callbacks | Maps (`datum`, `closures`, `subscriptions`, `functions`) |
| Internal binding functions | Identity allocation |
| Call site binding functions | Notification loop (drain cycle) |
| Theme initializers | Runtime API (`get`, `set`, `subscribe`, etc.) |
| Metadata (function tables) | |

The compiler generates functions and metadata. The runtime library provides the reactive
infrastructure that executes them.

## Function Patterns

### Subscription Callbacks

Subscription callbacks are called when a subscribed field changes. They read from the source
and write to the target.

```frel
blueprint Counter(initial: u32) {
    count: u32 = initial
    doubled: u32 = count * 2
}
```

Generated:

```javascript
// Updates `doubled` when `count` changes
function Counter$doubled$callback(runtime, subscription) {
    const count = runtime.get(subscription.source_id, "count")
    runtime.set(subscription.target_id, "doubled", count * 2)
}
```

### Internal Binding Functions

Internal binding functions set up subscriptions within a closure. They are called during
blueprint instantiation.

```frel
blueprint Counter(initial: u32) {
    count: u32 = initial
    doubled: u32 = count * 2
}
```

Generated:

```javascript
function Counter$internal_binding(runtime, closure_id) {
    // Initialize fields
    runtime.set(closure_id, "count", runtime.get(closure_id, "initial"))
    runtime.set(closure_id, "doubled", runtime.get(closure_id, "count") * 2)

    // Subscribe doubled to count
    runtime.subscribe(closure_id, closure_id, Key("count"), Counter$doubled$callback)
}
```

### Call Site Binding Functions

Call site binding functions set up subscriptions between parent and child closures. Each call
site (blueprint instantiation) in the source has a unique binding function.

```frel
blueprint Parent {
    value: u32 = 10
    Child(value)        // call site #1
}

blueprint Child(p: u32) {
    // ...
}
```

Generated:

```javascript
// Callback: propagates Parent.value to Child.p
function Parent$1$p$callback(runtime, subscription) {
    runtime.set(subscription.target_id, "p", runtime.get(subscription.source_id, "value"))
}

// Binding: sets up subscription from Parent to Child at call site #1
function Parent$1$call_site_binding(runtime, parent_closure_id, child_closure_id) {
    // Initialize child parameter
    runtime.set(child_closure_id, "p", runtime.get(parent_closure_id, "value"))

    // Subscribe child to parent
    runtime.subscribe(parent_closure_id, child_closure_id, Key("value"), Parent$1$p$callback)
}
```

### Theme Initializers

Theme initializers create datum instances for theme values. They are called during application
bootstrap.

```frel
theme AppTheme {
    corner_radius: u32 = 10
    padding: u32 = 16

    variant Compact {
        padding = 8
    }
}
```

Generated:

```javascript
function AppTheme$init(runtime) {
    // Base theme
    runtime.create_datum("AppTheme", {
        corner_radius: 10,
        padding: 16
    })

    // Compact variant
    runtime.create_datum("AppTheme$Compact", {
        corner_radius: 10,
        padding: 8
    })
}
```

## Metadata Structure

Metadata is keyed by qualified blueprint/scheme name. It contains references to generated
functions.

```javascript
metadata["myapp.Counter"] = {
    // Optional: only present if there are fields to initialize
    internal_binding: Counter$internal_binding,
    // Indices into call_sites for immediate instantiation
    top_children: [0, 1],
    call_sites: {
        "0": {
            blueprint: "myapp.Display",
            binding: Counter$0$call_site_binding
        },
        "1": {
            blueprint: "myapp.Label",
            binding: Counter$1$call_site_binding
        }
    }
}

metadata["myapp.AppTheme"] = {
    init: AppTheme$init,
    variants: ["Compact"]
}
```

The `top_children` array contains indices of `call_sites` that should be instantiated
immediately when the blueprint is instantiated. Children inside control statements
(`when`, `repeat`, `select`) are not top-level - they are instantiated by those control
blueprints when their conditions are met.

## Complete Example

**Frel source:**

```frel
module myapp

blueprint TodoApp {
    items: List<TodoItem> = []

    TodoList(items)
    AddButton(items)
}

blueprint TodoList(items: List<TodoItem>) {
    repeat on items { item ->
        TodoRow(item)
    }
}

blueprint TodoRow(item: TodoItem) {
    text { item.title }
}

blueprint AddButton(items: List<TodoItem>) {
    button { "Add" } .. on_click { add_item(items) }
}
```

**Generated JavaScript:**

```javascript
// ============================================
// Subscription Callbacks
// ============================================

function TodoList$items$callback(runtime, subscription) {
    runtime.set(subscription.target_id, "items", runtime.get(subscription.source_id, "items"))
}

function TodoRow$item$callback(runtime, subscription) {
    runtime.set(subscription.target_id, "item", runtime.get(subscription.source_id, "item"))
}

function AddButton$items$callback(runtime, subscription) {
    runtime.set(subscription.target_id, "items", runtime.get(subscription.source_id, "items"))
}

// ============================================
// Internal Binding Functions
// ============================================

function TodoApp$internal_binding(runtime, closure_id) {
    runtime.set(closure_id, "items", [])
}

function TodoList$internal_binding(runtime, closure_id) {
    // items is a parameter, initialized by call site binding
}

function TodoRow$internal_binding(runtime, closure_id) {
    // item is a parameter, initialized by call site binding
}

function AddButton$internal_binding(runtime, closure_id) {
    // items is a parameter, initialized by call site binding
}

// ============================================
// Call Site Binding Functions
// ============================================

// TodoApp -> TodoList (call site #0)
function TodoApp$0$call_site_binding(runtime, parent_id, child_id) {
    runtime.set(child_id, "items", runtime.get(parent_id, "items"))
    runtime.subscribe(parent_id, child_id, Key("items"), TodoList$items$callback)
}

// TodoApp -> AddButton (call site #1)
function TodoApp$1$call_site_binding(runtime, parent_id, child_id) {
    runtime.set(child_id, "items", runtime.get(parent_id, "items"))
    runtime.subscribe(parent_id, child_id, Key("items"), AddButton$items$callback)
}

// TodoList -> TodoRow (call site #0, inside repeat)
function TodoList$0$call_site_binding(runtime, parent_id, child_id) {
    runtime.set(child_id, "item", runtime.get(parent_id, "item"))
    runtime.subscribe(parent_id, child_id, Key("item"), TodoRow$item$callback)
}

// ============================================
// Metadata
// ============================================

metadata["myapp.TodoApp"] = {
    internal_binding: TodoApp$internal_binding,
    top_children: [0, 1],  // TodoList and AddButton
    call_sites: {
        "0": { blueprint: "myapp.TodoList", binding: TodoApp$0$call_site_binding },
        "1": { blueprint: "myapp.AddButton", binding: TodoApp$1$call_site_binding }
    }
}

metadata["myapp.TodoList"] = {
    // No internal_binding (no fields)
    top_children: [],  // TodoRow is inside repeat, not a top child
    call_sites: {
        "0": { blueprint: "myapp.TodoRow", binding: TodoList$0$call_site_binding }
    }
}

metadata["myapp.TodoRow"] = {
    // No internal_binding (no fields)
    top_children: [],
    call_sites: {}
}

metadata["myapp.AddButton"] = {
    // No internal_binding (no fields)
    top_children: [],
    call_sites: {}
}
```

## Cross-Language Transformation

JavaScript serves as the canonical output format. Other target languages (TypeScript, Kotlin,
Rust) can transform from the JS structure since the concepts map directly:

- Maps → `Map`/`HashMap`/`Dict` in all languages
- Functions → lambdas/closures in all languages
- Identity numbers → integers in all languages

No JavaScript-specific features are used in the generated code.
