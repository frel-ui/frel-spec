# Reactivity Model

This document describes the foundational reactivity model that underlies Frel's reactive state
system. Understanding this model helps explain how stores track changes, when updates propagate, and
how subscriptions work at a conceptual level.

## Overview

Frel's reactivity is built on three core concepts:

- **Identity**: What uniquely identifies a piece of data
- **Revisions**: When and how data changes
- **Subscriptions**: What kinds of changes we care about

These concepts work together to enable fine-grained reactivity - reacting only to the changes that
matter for a given use case.

## Data Properties

Conceptually (before optimization), each piece of data in Frel has these properties:

| Property                | Description                                                        |
|-------------------------|--------------------------------------------------------------------|
| **Type**                | The Frel type (e.g., `i32`, `String`, `List<User>`, `scheme Todo`) |
| **Value**               | The actual data content                                            |
| **Identity**            | What uniquely identifies this piece of data                        |
| **Structural Revision** | Increments when the identities of contained data change            |
| **Carried Revision**    | Increments when changes propagate from contained data              |
| **Metadata**            | Validation rules, constraints, etc. (not relevant to reactivity)   |

The key insight is that **identity and revisions depend on the type category**.

## Type Categories

Frel types fall into two categories that have fundamentally different reactivity semantics:

### Intrinsic Types

**Intrinsic types** are atomic values where identity is determined by type and value:

- **Primitives**: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `bool`
- **Specialized primitives**: `String`, `Uuid`, `Url`, `Decimal`, `Color`, `Secret`, `Blob`
- **Temporal types**: `Instant`, `LocalDate`, `LocalTime`, `LocalDateTime`, `Timezone`, `Duration`
- **Enums**: All enum variants

For intrinsic types:

```
identity = type + value
structural revision = constant 1
carried revision = constant 1
```

**Key property**: Changing the value of an intrinsic creates a new identity.

**Example**: An `i32` with value `5` has identity `i32(5)`. Changing it to `7` creates a new
identity `i32(7)`.

### Composite Types

**Composite types** are containers that hold other pieces of data:

- **Collections**: `List<T>`, `Set<T>`, `Map<K,V>`, `Tree<T>`
- **Schemes**: User-defined structured types

For composite types:

```
identity = type + system-assigned-number
structural revision = starts at 1, increments on structural changes
carried revision = starts at 1, increments on carried changes
```

**Key property**: A composite's identity is stable even when its contents change.

**Example**: A `List<i32>` gets identity `List<i32>#42` (where `#42` is system-assigned). This
identity remains the same whether the list contains `[1, 2, 3]` or `[5, 6, 7]`.

## Revision Semantics

Both revision types track different kinds of changes to composite data:

### Structural Revision

The **structural revision** increments when:

1. **The identity of contained data changes**
2. **New data is added to the composite**
3. **Data is removed from the composite**

For collections, "contained data" means the elements. For schemes, it means the field values.

### Carried Revision

The **carried revision** increments when:

1. **The structural revision of contained data changes** (carries structural changes upward)
2. **The carried revision of contained data changes** (carries deep changes upward)

In other words, carried revision tracks "changes propagated from within", while structural revision
tracks "which pieces are present".

## Propagation Examples

### Example 1: Primitive Value Change

```frel
scheme Counter {
    count: i32
}

decl counter: Counter = Counter { count: 5 }
```

Initial state:

- `counter` identity: `Counter#1`
- `counter.count` identity: `i32(5)`
- `counter` structural revision: 1
- `counter` carried revision: 1

Change `count` from `5` to `7`:

```frel
counter.count = 7
```

After change:

- `counter` identity: `Counter#1` (unchanged)
- `counter.count` identity: `i32(7)` (**changed** - new identity!)
- `counter` structural revision: 2 (**incremented** - contained identity changed)
- `counter` carried revision: 2 (**incremented** - structural change carried upward)

**Why both revisions increment**: For primitives, identity includes value. Changing the value
changes the identity, which is a structural change.

### Example 2: List of Primitives

```frel
decl numbers: List<i32> = [1, 2, 3]
```

Initial state:

- `numbers` identity: `List<i32>#10`
- `numbers` contains identities: `[i32(1), i32(2), i32(3)]`
- `numbers` structural revision: 1
- `numbers` carried revision: 1

Change first element from `1` to `5`:

```frel
// TODO: list mutation syntax not yet specified
// Conceptually: numbers[0] = 5
```

After change:

- `numbers` identity: `List<i32>#10` (unchanged)
- `numbers` contains identities: `[i32(5), i32(2), i32(3)]` (**changed**)
- `numbers` structural revision: 2 (**incremented** - contained identity changed)
- `numbers` carried revision: 2 (**incremented**)

**Why both revisions increment**: Changing an `i32` value changes its identity, so this is a
structural change to the list.

### Example 3: List of Schemes

```frel
scheme TodoItem {
    text: String
    done: bool
}

decl items: List<TodoItem> = [
    TodoItem { text: "Buy milk", done: false },
    TodoItem { text: "Walk dog", done: false }
]
```

Initial state:

- `items` identity: `List<TodoItem>#20`
- `items` contains: `[TodoItem#30, TodoItem#31]`
- `items` structural revision: 1
- `items` carried revision: 1

Change first item's `done` field from `false` to `true`:

```frel
// TODO: scheme field mutation syntax not yet specified
// Conceptually: items[0].done = true
```

After change:

- `items` identity: `List<TodoItem>#20` (unchanged)
- `items` contains: `[TodoItem#30, TodoItem#31]` (unchanged - same identities)
- `TodoItem#30` structural revision: 2 (done field identity changed)
- `TodoItem#30` carried revision: 2
- `items` structural revision: 1 (**unchanged** - identities same)
- `items` carried revision: 2 (**incremented** - contained item's carried change propagates)

**Key insight**: Changing a field within a scheme item is a **carried change** to the list, not a
structural change. The list still contains the same `TodoItem` identities.

### Example 4: Nested Schemes

```frel
scheme User {
    name: String
    friends: List<UserId>
}

decl user: User = User {
    name: "Alice",
    friends: [uuid1, uuid2]
}
```

Add a friend to the `friends` list:

```frel
// TODO: list append syntax not yet specified
// Conceptually: user.friends.append(uuid3)
```

After change:

- `user` identity: `User#40` (unchanged)
- `user.friends` identity: `List<UserId>#41` (unchanged)
- `user.friends` contains: `[uuid1, uuid2, uuid3]` (**changed**)
- `user.friends` structural revision: 2 (**incremented** - new identity added)
- `user.friends` carried revision: 2 (**incremented**)
- `user` structural revision: 1 (**unchanged** - friends field identity same)
- `user` carried revision: 2 (**incremented** - carried from friends)

**Result**: The `user` scheme has a carried change (because the `friends` field changed
structurally), but not a structural change (because the `friends` field's identity didn't change).

## Subscriptions

Frel's reactive system allows subscribing to an **identity** with an optional **selector** that
specifies what kinds of changes matter.

### Selector Types

| Selector       | Triggers When                                      |
|----------------|----------------------------------------------------|
| **Everything** | Structural revision OR carried revision increments |
| **Structural** | Structural revision increments                     |
| **Carried**    | Carried revision increments (but NOT structural)   |
| **Key-based**  | Changes to a specific field or map key             |

**Important**: "Carried" selector means carried-only changes. Structural changes do NOT trigger
carried-only subscriptions.

### Type-Specific Selectors

Different types support different key-based selectors:

| Type                | Key-Based Selectors                                |
|---------------------|----------------------------------------------------|
| **Intrinsic types** | None - no keys available                           |
| **Schemes**         | Field names (e.g., `"name"`, `"age"`, `"friends"`) |
| **List\<T\>**       | None - positions are unstable                      |
| **Set\<T\>**        | None - unordered collection                        |
| **Map\<K,V\>**      | Map keys (e.g., actual K values)                   |
| **Tree\<T\>**       | None - tree positions not exposed as keys          |

### Why Lists Don't Support Positional Keys

Positional indices (e.g., `list[0]`, `list[1]`) are intentionally **not supported** as subscription
keys for lists.

**Rationale**:

- List positions are **unstable** across insertions, deletions, and reorderings
- Subscribing to a position creates fragile dependencies that break when the list structure changes
- Index-based subscriptions are a common source of bugs in reactive systems
- Using item identities (not positions) for rendering produces more robust UIs

**Alternative**: Subscribe to structural changes to detect list membership changes, then use item
identities for granular rendering.

## Use Cases

### Use Case 1: Render List Structure Only

**Scenario**: Display count of items, but don't care about item contents.

```frel
scheme TodoItem {
    text: String
    done: bool
}

decl items: List<TodoItem> = load_todos()
decl item_count: i32 = items.length

text { "You have " + item_count + " items" }
```

**Subscription**: `items` identity with "structural" selector

- **Triggers when**: Items added, removed, or replaced
- **Does NOT trigger when**: An item's `text` or `done` field changes (carried changes only)

### Use Case 2: Render List Contents

**Scenario**: Display each todo item, reacting to field changes.

```frel
repeat on items as item {
    checkbox { item.done }
    text { item.text }
}
```

**Subscription**: `items` identity with "everything" selector

- **Triggers when**: Items added/removed/replaced (structural) OR any item field changes (carried)

Internally, the framework can optimize by also subscribing to each `item` identity individually for
fine-grained updates.

### Use Case 3: Field-Specific Reactivity

**Scenario**: Display just the user's name, ignoring other fields.

```frel
scheme User {
    name: String
    age: i32
    email: String
}

decl user: User = fetch_user()
decl greeting: String = "Hello, " + user.name

text { greeting }
```

**Subscription**: `user` identity with key selector `"name"`

- **Triggers when**: `name` field changes
- **Does NOT trigger when**: `age` or `email` changes

## Implementation Notes

### Optimization: Intrinsic Types

In actual implementations, intrinsic types typically **do not store** identity or revision
information explicitly:

- Identity is computed on-demand from type and value
- Revisions are conceptually constant and don't need storage
- This optimization saves significant memory for primitive-heavy data structures

### Optimization: Structural Sharing

For immutable operations, composites can use structural sharing:

- Multiple composite instances can share the same contained data
- Changes create new composite nodes but reuse unchanged subtrees
- Identity and revision tracking remain consistent across shared structures

## Summary

Frel's reactivity model distinguishes between:

1. **Intrinsic types**: Identity includes value; changing value = new identity = structural change
2. **Composite types**: Identity is stable; contents change via structural (which identities) or
   carried (propagated changes) revisions

This design enables:

- **Fine-grained reactivity**: React only to relevant changes (structural vs carried)
- **Natural semantics**: Primitives behave intuitively (value change = identity change)
- **Efficient updates**: Collections can distinguish membership changes from propagated changes
- **Type-safe subscriptions**: Key-based selectors work with type structure

The model forms the conceptual foundation for how stores track dependencies and propagate updates
throughout the reactive system.
