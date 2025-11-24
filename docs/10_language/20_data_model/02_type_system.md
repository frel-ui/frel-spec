# Type system

This document defines Frel's type system and the reactive semantics of each type category. 
Understanding these type-specific semantics is essential for working with Frel's reactive data model.

For foundational concepts (datum, field, type categories), see [Data Model Basics](01_data_model_basics.md). For 
system-level reactive behavior (ownership, availability, propagation), see [Reactivity](03_reactivity.md).

Frel has several type categories with different reactivity semantics:

- intrinsic types
- composite types
- nullable types
- reference types
- draft types
- asset types
- closure-bound types
- synthetic types

## Intrinsic Types

**Intrinsic types** are immutable values where identity is determined by type and value:

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

## Composite Types

**Composite types** are mutable types that contain other data through fields or items:

- **Field-based composite types**: Schemes, Arenas, Backends, Themes, Blueprints
- **Collection types**: `List<T>`, `Set<T>`, `Map<K,V>`, `Tree<T>`

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

## Nullable Types

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

**Important distinction**: `null` is different from `undefined` (when availability is not Ready).

## Reference Types

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

## Draft Types

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
- **Ownership**: Drafts are owned by their declaring closure (scope-bound), not by field assignments

**Assignment semantics:**

When a draft is assigned to a field or used in a scheme constructor:

1. A **deep copy** is created of the entire draft structure
2. The copy becomes a **non-draft** composite datum
3. The original draft remains **unchanged and usable**

```frel
blueprint FormBackend {
    addr : draft Address = Address { street = "Main St" }

    // Assignment creates deep copy, addr remains usable
    user : User = User {
        name = "Alice",
        address = addr  // Deep copy of addr, becomes non-draft Address
    }

    // addr can still be edited
    addr.street = "Oak Ave"  // Does not affect user.address
}
```

This enables **iterative editing with controlled commits** - you can assign a draft multiple
times while continuing to edit it.

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

## Asset Types

**Asset types** are externally-loaded values that represent UI assets such as colors, strings,
and graphics. An asset type is created by adding the `asset` modifier to an intrinsic type.

**Purpose**: Asset types solve the asset loading problem by:

- Decoupling semantic names from actual file names
- Supporting asynchronous loading with availability tracking
- Enabling theme-based asset organization
- Allowing platform-specific asset resolution

**Syntax**:

```frel
theme MessageTheme {
    self_background : asset Color      // Asset-loaded color
    new_message : asset String         // Asset-loaded localized string
    send : asset Graphics              // Asset-loaded icon/image
    corner_radius : u32 = 10              // Computed value (not an asset)
}
```

**Key properties:**

- **No initial value**: Asset fields cannot have initial values (they are loaded externally)
- **Availability semantics**: Assets have Loading/Ready/Error states during the loading process
- **Semantic binding**: The field name represents semantic usage (e.g., `send`), while the actual
  asset file is bound separately (e.g., `ic_arrow_forward_24dp.png`)
- **Type requirement**: Assets typically use intrinsic types suitable for assets (Color, String,
  Graphics, etc.)

**Availability semantics:**

When accessing an asset field, availability reflects the loading state:

- **Loading**: The asset is being loaded asynchronously
- **Ready**: The asset has been successfully loaded and is available
- **Error**: The asset failed to load (file not found, invalid format, etc.)

**Example**:

```frel
theme AppTheme {
    primary_color : asset Color
    app_name : asset String
    logo : asset Graphics

    padding : u32 = 16  // Not an asset, just a computed value
}

// When accessing theme.primary_color:
// 1. Asset loader resolves the semantic name to actual file
// 2. File is loaded asynchronously
// 3. Availability propagates through dependent expressions
// 4. Once loaded, color value becomes available
```

**Reactivity:**

- Asset fields do not change once loaded (they represent immutable assets)
- Switching themes (changing a `theme : ref Theme` field) is a structural change that triggers
  reloading of all asset fields
- Asset loading does not affect structural or carried revisions, only availability

## Closure-bound Types

Closure-bound types may carry closure when passed between fragments. This enables higher-order 
blueprints and bidirectional data access.

Categories:

- `Blueprint<P1, P2, ..., Pn>` - Blueprints
- `Accessor<T>` - Bidirectional data accessor for value editors

### Blueprint

Blueprint types represent blueprints enabling composition patterns. Syntax:

There are two main use patters for blueprint types:

1. storing a blueprint for later use
2. anonymous blueprints

**Storing a blueprint**

```frel
scheme Config {
    renderer : BluePrint<i32>
}

blueprint heading(section_number : i32, content : Blueprint<i32>) {
    content(section_number) // here we create a fragment with the `content` blueprint
}

blueprint title_renderer(section_number : i32) {
    text { "${section_number}. ${title}" }
}

bluprint document {
    config : Config = Config { renderer = title_renderer }
    heading(1, config.renderer)
}    
```

**Using an anonymous blueprint**

```frel
blueprint heading(section_number : i32, content : Blueprint<i32>) {
    content(section_number) // here we create a fragment with the `content` blueprint
}

bluprint document {
    title : String = "First section:
    heading(1) { section_number -> // here we define an anonymous blueprint
        text { "${section_number}. ${title}" } // here we use the carried closure
    }
}    
```

>> TODO identity and reactivity semantics of blueprints

### Accessor

Accessor types provide bidirectional access to data fields, enabling form editors to both read and
write values with full context.

```frel
blueprint text_editor(accessor : Accessor<String>) {
    /* ... */
}

blueprint example() {
    message : String = ""
    text_editor { message }
}
```

Accessor implementations carry:

- Path to the target datum
- Metadata (validation constraints, etc.)
- Bidirectional read/write access

```frel
blueprint example(original : User) {
    user : draft User = original
    text_editor { user.personal.name.family_name }
}
```

The accessor contains the full path and field constraints for family_name.

>> TODO identity and reactivity semantics of blueprints

## Synthetic Types

Synthetic types are generated by the compiler to store runtime data. These types are
not accessible from Frel code but share identity and reactivity semantics with other
Frel types.

Categories:

- `Fragment`
- `Closure`

### Closure

Closures are synthetic schemes. Each defines what is accessible at a given point of
a fragment.

See [Closures](../80_runtime/30_fragments_and_closures.md) for more information.

### Fragment

Fragments are synthetic schemes generated by the compiler:

```frel
scheme Fragment {
    id : u64 .. identifier
    parent : ref Fragment
    children : List<Fragment>
    closure : Closure<u32,String>
}
```

### Expression

>> TODO