# Data model basics

## Datum

In Frel, the **conceptual unit of runtime application data** is called **datum**.

A **datum** can be very simple such as a standalone number or very complex such as a tree of
deeply nested data.

Conceptually (before optimization), each **datum** has these properties:

| Property                | Description                                                        |
|-------------------------|--------------------------------------------------------------------|
| **Identity**            | A unique identifier of this **datum**.                             |
| **Type**                | The Frel type (e.g., `i32`, `String`, `List<User>`, `scheme Todo`) |
| **Payload**             | The actual data content.                                           |
| **Availability**        | The availability state (Loading/Ready/Error).                      |
| **Structural Revision** | Increments when the identities of contained data change.           |
| **Carried Revision**    | Increments when changes propagate from the payload.                |
| **Owner**               | The identity of the owning datum, or `null` for root datums.       |
| **Derivation**          | The function that can derive the datum from its dependencies.      |
| **Subscriptions**       | Identities of dependent data.                                      |
| **Metadata**            | Validation rules, constraints, etc. (not relevant to reactivity).  |

### Core tenets

1. Datum identity is unique, there cannot be two different payloads with the same identity.
2. The identity of a datum is **type-dependent**.
3. The reactivity behavior of a datum is **type-dependent**.
4. Each datum serves as the single source of truth for its identity.
5. Dependencies between data are expressed through subscriptions. When datum A depends on datum B, A
   subscribes to B.

## Type

The type system sorts types into two main categories:

1. **Intrinsic types**
   - Pre-defined by the language
   - Cannot have user-defined fields
   - Immutable

2. **Composite types**
   - Defined by the user or synthesized by the compiler
   - Composed of either fields (schemes, backends, etc.) or items (collections)
   - Mutable

In addition to the main categories, types can be modified with type qualifiers:

- `nullable` (suffix `?`) - the datum may be absent
- `ref` - reference to another datum
- `draft` - isolated editable copy
- `asset` - externally-loaded, environment dependent value

## Field

A **field** is a declaration that defines containment between a composite type A and a type B.

Fields are the primary mechanism for composing complex data structures from simpler ones.

Each field has these properties:

| Property    | Description                                                 |
|-------------|-------------------------------------------------------------|
| **Name**    | A container(type A)-unique name of the field.               |
| **Type**    | The Frel type (e.g., `i32`, `String`, `List<User>`, `Todo`) |

### Core tenets

1. Fields are **declarations** - they are part of the type definition, not runtime data.
2. Fields belong to **composite, non-collection types only** - intrinsic types cannot have fields.

### Fields vs. Collection items

Fields should not be confused with collection containment:
- A **field** is a named, declared slot in a composite type (e.g., `User.name`)
- A **collection item** is an element in a List, Set, Map, or Tree (e.g., `list[0]`)

Both represent containment, but fields are structural (part of the type), while collection items are
dynamic (part of the runtime data).
