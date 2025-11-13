# Reactivity Model

This document describes Frel's foundational reactivity model. Understanding this model helps explain
how changes are tracked, when updates propagate, and how subscriptions work at a conceptual level.

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
| **Availability**        | The availability state (Loading/Ready/Error)                       |
| **Structural Revision** | Increments when the identities of contained data change            |
| **Carried Revision**    | Increments when changes propagate from contained data              |
| **Metadata**            | Validation rules, constraints, etc. (not relevant to reactivity)   |

The key insight is that **identity and revisions depend on the type category**.

## Types

Frel has several type categories with different reactivity semantics:

- intrinsic types
- composite types
- nullable types
- reference types
- draft types
- resource types

### Intrinsic Types

**Intrinsic types** are atomic values where identity is determined by type and value:

- **Primitives**: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `bool`
- **Specialized primitives**: `String`, `Uuid`, `Url`, `Decimal`, `Color`, `Graphics`, `Secret`, `Blob`
- **Temporal types**: `Instant`, `LocalDate`, `LocalTime`, `LocalDateTime`, `Timezone`, `Duration`
- **Enums**: All enum variants

For intrinsic types:

```
identity = type + value
structural revision = constant 1
carried revision = constant 1
```

**Key property**: Intrinsic values are immutable. Each distinct value is a distinct identity.

**Example**: The intrinsic `i32(5)` has identity `i32(5)`. The intrinsic `i32(7)` has identity
`i32(7)`. These are two different, immutable values with different identities.

For more information see [Intrinsic Types](20_intrinsic_types.md).

### Composite Types

**Composite types** are containers that hold other pieces of data:

- **Collections**: `List<T>`, `Set<T>`, `Map<K,V>`, `Tree<T>`
- **User-defined types**: Schemes, Arenas, Backends, Themes, Blueprints

For composite types:

```
identity = type + system-assigned-number
structural revision = starts at 1, increments on structural changes
carried revision = starts at 1, increments on carried changes
```

**Key property**: A composite's identity is stable even when its contents change.

**Example**: A `List<i32>` gets identity `List<i32>#42` (where `#42` is system-assigned). This
identity remains the same whether the list contains `[1, 2, 3]` or `[5, 6, 7]`.

For more information see [Composite Types](30_composite_types.md).

### Nullable Types

**Nullable types** allow a value to be either a valid instance of the type or `null`.

A nullable type is created by adding `?` to any intrinsic or composite type:

```frel
name: String?        // nullable String
count: i32?          // nullable i32
items: List<Item>?   // nullable List
user: User?          // nullable scheme
```

**Key properties:**

- **`null` value**: Represents the explicit absence of a value
- **Type safety**: A nullable type `T?` is distinct from non-nullable `T`
- **Works with any type**: Both intrinsic types (`String?`, `i32?`) and composite types (`List<T>?`, `User?`) can be nullable

**Identity and reactivity**: When a nullable field changes between `null` and a non-null value (or between different non-null values), this is treated as an identity change:

```frel
scheme User {
    name: String
    bio: String?
}

user.bio = null              // bio field contains null
user.bio = "Hello world"     // bio field contains String("Hello world") - structural change
user.bio = "Updated bio"     // bio field contains String("Updated bio") - structural change
```

**Important distinction**: `null` is different from `undefined` (when status is not Ready). See the Status section for details.

### Reference Types

**Reference types** represent references to scheme instances that exist in arenas. A reference type
is created by adding the `ref` modifier before a scheme type.

**Syntax**:

```frel
scheme Thermometer {
    id : UUID .. identity
    location : ref Location    // Reference to Location scheme
    name : String
}
```

**Key properties:**

- **Arena resolution**: References are resolved by looking up the referenced entity in its arena
- **Identity storage**: A `ref T` field stores the identity value of the referenced entity (e.g.,
  the UUID)
- **Transitive access**: Fields and virtual fields of the referenced entity can be accessed directly
  through the reference
- **Type requirement**: The referenced type must have an identity field (marked with `.. identity`)

**Availability semantics:**

When accessing a referenced entity, availability is determined by the arena lookup:

- **Ready**: The referenced entity exists in the arena and is ready
- **Loading**: The referenced entity is not yet loaded in the arena
- **Error**: The referenced entity does not exist or failed to load

**Example**:

```frel
scheme Location {
    id : UUID .. identity
    name : String
}

scheme Thermometer {
    id : UUID .. identity
    location : ref Location

    virtual locationName : String = location.name
}

// When accessing thermometer.locationName:
// 1. Resolve thermometer.location reference via LocationArena
// 2. Access the name field of the resolved Location
// 3. Propagate availability if location is Loading or Error
```

**Reactivity:**

>> TODO these reactivity rules are probably wrong, too easy to create circular references
>> actually, circular references are quite normal in data structures
>> we should probably not propagate changes in references, but we should subscribe
>> driectly from virtual fields

- Changing a reference field (e.g., `thermometer.location = newLocationId`) is a **structural change**
- Changes to the referenced entity (e.g., updating `location.name`) propagate as **carried changes**
  through virtual fields that depend on the reference
- The reactive system automatically tracks dependencies across arena boundaries

**Nullable references:**

References can be nullable:

```frel
location : ref Location?    // Optional reference
```

When a nullable reference is `null`, its availability is `Ready` (the field is defined, its value is
explicitly `null`). This is different from a non-null reference that cannot be resolved (which has
availability `Error` or `Loading`).

### Draft Types

**Draft types** are editable copies of scheme instances, primarily used for form editing and
temporary data manipulation. A draft type is created by adding the `draft` modifier to a scheme
type.

**Purpose**: Draft types solve the form editing problem by providing an isolated, editable copy of
data that:

- Has its own reactive identity (separate from the original)
- Can be validated without affecting the original
- Can be discarded or committed back to the original
- Lives only as long as its containing backend

**Syntax**:

```frel
backend UserEditorBackend {
    original : User                          // Reference to arena instance
    user : draft User = original             // Draft copy for editing
}
```

**Key properties:**

- **Separate identity**: `draft#456 User#123` has a different reactive identity than `User#123`,
  preventing arena updates from affecting the draft
- **Independent validation**: Validation rules apply to drafts, but errors don't block editing 
  (non-blocking validation)
- **Explicit lifecycle**: Drafts exist only within their backend scope and are automatically cleaned
  up when the backend is destroyed
- **Mutability**: Draft instances are mutable, following Frel's general mutability philosophy

**Common operations**:

```frel
// Create draft from original
user : draft User = original
```

**Identity and reactivity**:

* Each draft gets a unique identity
* Identity is stable for the draft's lifetime
* Identity is independent of the original's identity

The type modifier is part of the reactive identity:

- `User#123` - Arena instance with identity `User#123`
- `draft#456 User#123` - Draft instance with identity `draft#456 User#123`

This ensures that:

- Changes to the arena instance don't propagate to the draft
- Changes to the draft don't propagate to the arena
- Each has independent reactive subscribers

**Relationship with arenas**:

Draft types work seamlessly with arenas for the common pattern of "edit and save back":

1. User selects an entity from an arena (read-only display)
2. Backend creates a draft copy for editing
3. User edits the draft in a form
4. Validation runs on the draft
5. If valid, draft is committed back to the original, which updates the arena
6. Arena propagates the update to all subscribers

### Resource Types

**Resource types** are externally-loaded values that represent UI assets such as colors, strings,
and graphics. A resource type is created by adding the `resource` modifier to an intrinsic type.

**Purpose**: Resource types solve the asset loading problem by:

- Decoupling semantic names from actual file names
- Supporting asynchronous loading with availability tracking
- Enabling theme-based resource organization
- Allowing platform-specific resource resolution

**Syntax**:

```frel
theme MessageTheme {
    self_background : resource Color      // Resource-loaded color
    new_message : resource String         // Resource-loaded localized string
    send : resource Graphics              // Resource-loaded icon/image
    corner_radius : u32 = 10              // Computed value (not a resource)
}
```

**Key properties:**

- **No initial value**: Resource fields cannot have initial values (they are loaded externally)
- **Availability semantics**: Resources have Loading/Ready/Error states during the loading process
- **Semantic binding**: The field name represents semantic usage (e.g., `send`), while the actual
  resource file is bound separately (e.g., `ic_arrow_forward_24dp.png`)
- **Type requirement**: Resources typically use intrinsic types suitable for assets (Color, String,
  Graphics, etc.)

**Availability semantics:**

When accessing a resource field, availability reflects the loading state:

- **Loading**: The resource is being loaded asynchronously
- **Ready**: The resource has been successfully loaded and is available
- **Error**: The resource failed to load (file not found, invalid format, etc.)

**Example**:

```frel
theme AppTheme {
    primary_color : resource Color
    app_name : resource String
    logo : resource Graphics

    padding : u32 = 16  // Not a resource, just a computed value
}

// When accessing theme.primary_color:
// 1. Resource loader resolves the semantic name to actual file
// 2. File is loaded asynchronously
// 3. Availability propagates through dependent expressions
// 4. Once loaded, color value becomes available
```

**Reactivity:**

- Resource fields do not change once loaded (they represent immutable assets)
- Switching themes (changing a `theme : ref Theme` field) is a structural change that triggers
  reloading of all resource fields
- Resource loading does not affect structural or carried revisions, only availability

## Revision Semantics

Both revision types track different kinds of changes to composite data:

### Structural Revision

The **structural revision** increments when:

1. **The identity of contained data changes**
2. **New data is added to the composite**
3. **Data is removed from the composite**
4. **Order of contained data changes (for List and Tree)**

For collections, "contained data" means the elements. For schemes, it means the field values.

### Carried Revision

The **carried revision** increments when:

1. **The structural revision of contained data changes** (carries structural changes upward)
2. **The carried revision of contained data changes** (carries deep changes upward)

In other words, carried revision tracks "changes propagated from within", while structural revision
tracks "which pieces are present".

## Availability

Every piece of data in Frel has an associated availability that represents its availability state, one of:

- **Loading**: Data is being fetched or computed but not yet available
- **Ready**: Data is available and can be used
- **Error**: An error occurred; data is not available

### Availability and Revisions

Availability transitions are treated as **structural changes**:

- When availability changes (e.g., `Loading` → `Ready`, `Ready` → `Error`), the structural revision increments
- This ensures subscribers are notified when data becomes available or encounters errors

### Value Availability

The availability determines whether a value is defined:

- **Ready**: Value is defined and accessible
- **Loading**: Value is undefined (not yet available)
- **Error**: Value is undefined (error prevented availability)

**Important distinction**: When availability is not Ready, the value is **undefined**. This is different from a nullable field being `null`:

```
// Value undefined due to availability
user_data with availability=Loading  // value is undefined, cannot access fields

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

When availability propagates to `Loading` or `Error`, the dependent's structural revision increments (due to availability change), and its value becomes undefined.

## Propagation Examples

### Example 1: Availability Transition and Primitive Value Change

```frel
scheme Counter {
    count: i32
}

// Initially loading from API
decl counter: Counter  // fetched from API
```

**Initial state (loading)**:

- `counter` identity: `Counter#1`
- `counter` availability: `Loading`
- `counter` value: undefined
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
decl numbers: List<i32> = [1, 2, 3]
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