# Composite Types

Frel provides composite types for building complex data structures from simpler components. These
include collections for grouping values and user-defined types for structured data.

Frel provides platform-independent collection types (`List`, `Set`, `Map`, `Tree`).

## List - Ordered Collection

Ordered sequence of values.

### Characteristics

- **Ordered**: Elements maintain their insertion order.
- **Duplicates are allowed**: The same identity can appear multiple times.
- **Type homogeneous**: All elements must be of the same type `T`

### Key Capabilities

- Add elements:
    - to the beginning
    - to the end
    - insert after a specific identity
- Remove elements:
    - from beginning
    - from the end
    - by identity
- Query length, check if empty, test membership by identity
- Iterate over elements

### Semantics

**Duplicates policy**: Duplicates are allowed, but only first‑occurrence semantics are defined
for mutating ops, except bulk (`*_all`) operations where all occurrences are affected. (In 
practice duplicates and positional mutations are rarely used together.)

**Reordering guidance**: If you need drag‑and‑drop, ensure items have unique identities (composite
types). Avoid duplicated identities in reorderable lists.

**Not‑found behavior**: Insert/move/remove/replace with missing anchors/targets are no‑ops
(never silently placed elsewhere).

**Replace semantics**: Same identity → no-op; different identity → structural change (equivalent to
remove+insert).

**Identity-anchored operations**: All positional operations use identity anchors (not numeric
indices). This provides stable references that work correctly with reactivity subscriptions. Numeric
indices are intentionally not supported (see Reactivity Model: Why Lists Don't Support Positional
Keys).

**Reactivity**: Query operations (length, is_empty, contains) react to structural changes (
add/remove/replace). See Reactivity Model for subscription semantics.

**Reordering and reactivity**: Reordering operations (`move_*`) increment the structural revision
because they change the list's order, which is part of its structure. The set of contained
identities remains the same, but their order changes.

### API

All positional operations (with `anchor_id` parameter) work on the first element with the specified
identity.

Insert:
- `push_start(v)`
- `push_end(v)`
- `insert_before(anchor_id, v)`
- `insert_after(anchor_id, v)`

Remove:
- `pop_start()`
- `pop_end()`
- `remove_first(id)`
- `remove_all(id)`
- `clear()`

Replace:
- `replace(old_id, v)`

Reorder:
- `move_to_start(id)`
- `move_to_end(id)`
- `move_before(anchor_id, id)`
- `move_after(anchor_id, id)`

Query:
- `length`
- `is_empty`
- `contains(id)`

## Set - Unique Values, Unordered

Unordered collection of unique values.

### Characteristics

- **Unordered**: No guaranteed order of elements
- **Unique**: Each value can only appear once; duplicates are automatically ignored
- **Type homogeneous**: All elements must be of the same type `T`

### Key Capabilities

- Add elements (duplicates automatically ignored, tested by identity)
- Remove elements by identity
- Query length, check if empty, test membership by identity
- Iterate over elements

### Semantics

**Uniqueness policy**: Duplicates are automatically ignored. Adding an element that already exists (
by identity) is a no-op.

**Not-found behavior**: Remove operations with missing identities are no-ops.

**Identity-based operations**: All operations use identity comparison (not value equality or other
criteria).

**Reactivity**: Query operations (`length`, `is_empty`, `contains`) react to structural changes (
add/remove). Since Set
is unordered, there are no reordering operations. See Reactivity Model for subscription semantics.

### API

Insert:
- `insert(v)`

Remove:
- `remove(id)`
- `clear()`

Query:
- `length()`
- `is_empty()`
- `contains(id)`

## Map - Key-Value Pairs

Key-value mapping with hashable key types.

> > TODO Map will be specified later

## Tree - Hierarchical Structure

First-class hierarchical collection with automatic node ID management, efficient updates, and
fine-grained reactivity. Trees are stored in a flat structure internally for optimal performance
and scalability.

> > TODO Tree will be specified later

## User Defined Types

- [**Schemes**](35_schemes.md)
- **Arenas**
- **Backend**
- **Themes**
- **Blueprints**