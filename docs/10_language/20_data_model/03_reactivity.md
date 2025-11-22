# Reactivity

This document describes Frel's foundational reactivity model, building on the foundational concepts
introduced in [Data model basics](01_data_model_basics.md) and in [Type system](02_type_system.md).
Understanding this model helps explain how changes are tracked, when updates propagate, and how 
subscriptions work at a conceptual level.

## Ownership and Assignment

Frel enforces controlled isolation through strict ownership and assignment rules. These rules prevent
ambiguous ownership and ensure data flows are explicit and intentional.

### Ownership Tree Structure

Each composite datum has an **owner field** that establishes a tree structure:

- **Value**: `null` for root datums, or the identity of the owning datum
- **Invariant**: Each composite datum has at most one owner
- **Purpose**: Enables carried revision propagation and prevents ownership ambiguity

**When ownership is established:**

- **Scheme field contains composite type** → field value is owned by the scheme datum
- **Collection contains composite items** → each item is owned by the collection datum
- **Reference types (`ref T`)** → do NOT establish ownership (refs are intrinsic)
- **Draft types** → owned by their declaring closure (scope-bound)

**Example ownership relationships:**

```frel
scheme User {
    name : String           // String is intrinsic, no ownership
    address : Address       // Address datum owned by this User
    location : ref Location // ref is intrinsic, Location not owned by User
}

items : List<TodoItem>      // Each TodoItem owned by the list
```

**Ownership prevents cycles:**

Since ownership forms a tree structure (no datum can own itself, directly or indirectly),
carried revision propagation is guaranteed to terminate. Cycles in the data graph are
possible through reference types, but those do not establish ownership relationships.

### Assignment Rules

Frel restricts which types can be assigned to prevent implicit sharing of composite datums.

**Allowed assignments:**

1. **Intrinsic types** (including `ref T`) - Can be assigned freely
2. **Draft types** - Assignment creates a deep copy (see below)

**Forbidden assignments:**

3. **Non-draft composite types** - Cannot be assigned directly (compile error)

**Rationale**: Direct assignment of composite types would create ownership ambiguity.
The following example is **forbidden**:

```frel
// ❌ COMPILE ERROR - direct composite assignment forbidden
addr : Address = Address { }
user : User = User { address = addr }  // Error: cannot assign non-draft composite type
```

To fix this, use either `draft` (for mutable copies) or `ref` (for shared access):

```frel
// ✅ CORRECT - draft creates controlled copy
addr : draft Address = Address { }
user : User = User { address = addr }  // Deep copy, addr remains usable

// ✅ CORRECT - ref for shared access
addr : Address = Address { }
user : User = User { address = ref addr }  // Shared reference, no ownership
```

### Draft Assignment Semantics

When a draft datum is assigned to a field:

1. **Deep copy is created** - All nested composite datums are recursively copied
2. **Copy becomes non-draft** - The assigned value is a regular (non-draft) composite datum
3. **Original draft unchanged** - The draft datum remains usable for further edits
4. **Draft owned by closure** - Drafts are owned by their declaring closure, not by field assignments

**Example:**

```frel
blueprint UserEditorBackend {
    original : User

    // Create draft for editing
    userDraft : draft User = original

    // Assign draft to field - creates deep copy, userDraft still usable
    updated : User = User {
        name = "Updated",
        profile = userDraft.profile  // Deep copy of profile, becomes non-draft
    }

    // userDraft can still be edited
    userDraft.name = "Another edit"
}
```

**Benefits of this model:**

- **No ownership ambiguity** - Compiler prevents multiple owners
- **Explicit data flow** - `draft` and `ref` make intent clear
- **Controlled isolation** - Mutations happen only in drafts
- **Iterative editing** - Draft remains usable after assignment for multiple commits

### Carried Revision Propagation

When a datum's structural or carried revision changes, the runtime walks up the ownership
chain (following owner fields) and increments each ancestor's carried revision. This
continues until reaching root datums (where owner is `null`).

This propagation mechanism ensures that changes deep in a data structure are visible to
subscribers at higher levels.

## Revision Semantics

Both revision types track different kinds of changes to composite data:

### Structural Revision

The **structural revision** increments when:

1. **The identity of contained data changes**
2. **New data is added to the composite**
3. **Data is removed from the composite**
4. **Order of contained data changes (for List and Tree)**

For collection types, "contained data" means the items. For field-based composite types, it means the field payloads.

### Carried Revision

The **carried revision** increments when:

1. **The structural revision of contained data changes** (carries structural changes upward)
2. **The carried revision of contained data changes** (carries deep changes upward)

In other words, carried revision tracks "changes propagated from within", while structural revision
tracks "which pieces are present".

## Availability

Every datum in Frel has an associated availability that represents its availability state, one of:

- **Loading**: Data is being fetched or computed but not yet available
- **Ready**: Data is available and can be used
- **Error**: An error occurred; data is not available

### Availability and Revisions

Availability transitions are treated as **structural changes**:

- When availability changes (e.g., `Loading` → `Ready`, `Ready` → `Error`), the structural revision increments
- This ensures subscribers are notified when data becomes available or encounters errors

### Payload Availability

The availability determines whether a payload is defined:

- **Ready**: Payload is defined and accessible
- **Loading**: Payload is undefined (not yet available)
- **Error**: Payload is undefined (error prevented availability)

**Important distinction**: When availability is not Ready, the payload is **undefined**. This is different from a nullable field being `null`:

```
// Payload undefined due to availability
user_data with availability=Loading  // payload is undefined, cannot access fields

// Nullable field that is null
scheme User {
    name: String
    bio: String?  // nullable field
}

user with availability=Ready, bio=null  // user is defined, bio field is explicitly null
```

### Availability Propagation

Availability propagates through data dependencies using the rule: **Error > Loading > Ready**

- If ANY dependency has availability `Error` → result availability is `Error`
- Else if ANY dependency has availability `Loading` → result availability is `Loading`
- Else if ALL dependencies have availability `Ready` → result availability is `Ready`

When availability propagates to `Loading` or `Error`, the dependent's structural revision increments (due to availability change), and its payload becomes undefined.

## Propagation Examples

### Example 1: Availability Transition and Primitive Value Change

```frel
scheme Counter {
    count: i32
}

// Initially loading from API
counter: Counter  // fetched from API
```

**Initial state (loading)**:

- `counter` identity: `Counter#1`
- `counter` availability: `Loading`
- `counter` payload: undefined
- `counter` structural revision: 1
- `counter` carried revision: 1

**After data loads**:

```frel
// counter receives: Counter { count: 5 }
```

- `counter` identity: `Counter#1` (unchanged)
- `counter` availability: `Ready` (**changed** - availability transition)
- `counter.count` identity: `i32(5)`
- `counter` structural revision: 2 (**incremented** - availability change)
- `counter` carried revision: 2 (**incremented**)

**Later, replace `count` field with a different value**:

```frel
counter.count = 7
```

- `counter` identity: `Counter#1` (unchanged)
- `counter` availability: `Ready` (unchanged)
- `counter.count` identity: `i32(7)` (**replaced** - different intrinsic!)
- `counter` structural revision: 3 (**incremented** - contained identity changed)
- `counter` carried revision: 3 (**incremented** - structural change carried upward)

**Why both revisions increment**: The `count` field now contains `i32(7)` instead of `i32(5)`. These are different intrinsic values with different identities, so this is a structural change to the `counter` scheme.

### Example 2: List of Primitives

```frel
numbers: List<i32> = [1, 2, 3]
```

Initial state:

- `numbers` identity: `List<i32>#10`
- `numbers` contains identities: `[i32(1), i32(2), i32(3)]`
- `numbers` structural revision: 1
- `numbers` carried revision: 1

Replace first element with a different value:

```frel
// TODO: list mutation syntax not yet specified
// Conceptually: numbers[0] = 5
```

After replacement:

- `numbers` identity: `List<i32>#10` (unchanged)
- `numbers` contains identities: `[i32(5), i32(2), i32(3)]` (**replaced** first element)
- `numbers` structural revision: 2 (**incremented** - contained identity changed)
- `numbers` carried revision: 2 (**incremented**)

**Why both revisions increment**: The list now contains `i32(5)` instead of `i32(1)` at position 0.
These are different intrinsic values with different identities, so this is a structural change to the list.

### Example 3: List of Schemes

```frel
scheme TodoItem {
    text: String
    done: bool
}

items: List<TodoItem> = [
    TodoItem { text: "Buy milk", done: false },
    TodoItem { text: "Walk dog", done: false }
]
```

Initial state:

- `items` identity: `List<TodoItem>#20`
- `items` contains: `[TodoItem#30, TodoItem#31]`
- `items` structural revision: 1
- `items` carried revision: 1

Replace first item's `done` field with a different value:

```frel
// TODO: scheme field mutation syntax not yet specified
// Conceptually: items[0].done = true
```

After replacement:

- `items` identity: `List<TodoItem>#20` (unchanged)
- `items` contains: `[TodoItem#30, TodoItem#31]` (unchanged - same identities)
- `TodoItem#30` structural revision: 2 (`done` field now contains `bool(true)` instead of `bool(false)`)
- `TodoItem#30` carried revision: 2
- `items` structural revision: 1 (**unchanged** - still contains same TodoItem identities)
- `items` carried revision: 2 (**incremented** - structural change within contained item propagates)

**Key insight**: Replacing a field within a scheme item is a **structural change to the item** (the field
contains a different intrinsic identity), which propagates as a **carried change** to the list. The list
still contains the same `TodoItem` identities, so the list's structural revision doesn't increment.

### Example 4: Nested Schemes

```frel
scheme User {
    name: String
    friends: List<UserId>
}

user: User = User {
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

## Use Cases

### Use Case 1: Render List Structure Only

**Scenario**: Display count of items, but don't care about item contents.

```frel
scheme TodoItem {
    text: String
    done: bool
}

items: List<TodoItem> = load_todos()
item_count: i32 = items.length

text { "You have ${item_count} items" }
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

user: User = fetch_user()
greeting: String = "Hello, ${user.name}"

text { greeting }
```

**Subscription**: `user` identity with key selector `"name"`

- **Triggers when**: `name` field changes
- **Does NOT trigger when**: `age` or `email` changes