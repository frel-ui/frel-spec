# Runtime Data Model

This document explores how the Frel reactive system is represented as data structures in a
runtime, focusing on **what the data looks like at a given moment**.

## 1. Core Storage Strategy

The runtime uses **two-tier storage** based on type category:

### 1.1 Intrinsic Types (Optimized)

**Intrinsic types** (primitives, String, enums) are **stored directly as values** without
the full datum structure. This is an optimization that is possible because:

- Intrinsic datum is immutable - changing means creating a new value
- Compiler knows types statically
- Identity is the value itself
- Change tracking is provided by the datum that contains the intrinsic datum

```json
{
    "count": 42,
    "name": "Alice",
    "isActive": true
}
```

### 1.2 Composite Types (Full Datum Structure)

**Composite types** (schemes, collections, arenas, backends) need full reactive tracking.

**Datum structure for composite types:**

```json
{
    "identityId": 2001,
    "type": "User",
    "structuralRev": 0,
    "carriedRev": 5,
    "availability": "Ready",
    "error": null,
    "owner": null,
    "fields": {
        "id": 100,
        "name": "Alice",
        "profile": 3001
    },
    "items": null,
    "entries": null,
    "subscribers": [
        {"subscriberId": 10, "selector": "Everything"},
        {"subscriberId": 11, "selector": "Key:name"}
    ]
}
```

The meaning of fields is described in these documents:

- [Data Model Basics](/docs/10_language/20_data_model/01_data_model_basics.md)
- [Reactivity Model](/docs/10_language/20_data_model/03_reactivity.md)

For information about ownership semantics, see the [Ownership and Assignment](../20_data_model/02_type_system.md#ownership-and-assignment) section in the Reactivity Model.

## 2. Schemes: Mixed Intrinsic and Composite Fields

```frel
scheme UserWithProfile {
    name : String              // Intrinsic
    age : i32                  // Intrinsic
    profile : Profile          // Composite (another scheme)
}
```

**UserWithProfile instance:**

```json
{
    "identityId": 2001,
    "type": "UserWithProfile",
    "structuralRev": 0,
    "carriedRev": 0,
    "availability": "Ready",
    "error": null,
    "owner": null,
    "fields": {
        "name": "Alice",
        "age": 30,
        "profile": 3001
    },
    "items": null,
    "entries": null,
    "subscribers": []
}
```

- `name`: Intrinsic, stored directly as string
- `age`: Intrinsic, stored directly as number
- `profile`: Composite, stored as identityId
- `owner`: null (this is a root datum, not owned by another)

**The Profile scheme (separate datum):**

```json
{
    "identityId": 3001,
    "type": "Profile",
    "structuralRev": 0,
    "carriedRev": 0,
    "availability": "Ready",
    "error": null,
    "owner": 2001,
    "fields": {
        "bio": "Software engineer",
        "avatar": "https://..."
    },
    "items": null,
    "entries": null,
    "subscribers": []
}
```

- `owner`: 2001 (owned by userDatum - changes propagate up the ownership chain)

## 3. Collections

**List<i32> with values [10, 20, 30]:**

```json
{
    "identityId": 3001,
    "type": "List<i32>",
    "structuralRev": 0,
    "carriedRev": 0,
    "availability": "Ready",
    "error": null,
    "owner": null,
    "fields": null,
    "items": [10, 20, 30],
    "entries": null,
    "subscribers": []
}
```

- `items`: Plain numbers, no wrappers!
- `owner`: null (root datum)

**List<User> with composite items:**

```json
{
    "identityId": 3002,
    "type": "List<User>",
    "structuralRev": 0,
    "carriedRev": 0,
    "availability": "Ready",
    "error": null,
    "owner": null,
    "fields": null,
    "items": [2001, 2002, 2003],
    "entries": null,
    "subscribers": []
}
```

- `items`: identityIds of User datums (each User datum has `owner: 3002`)
- `owner`: null (root datum)

## 4. Fragment Structure

Fragments form a parent-child hierarchy, each representing an instantiated blueprint.

```json
{
    "fragmentId": 100,
    "blueprintName": "UserCard",
    "parentFragmentId": 50,
    "childFragmentIds": [101, 102, 103],
    "closureId": 1000
}
```

## 5. Closure Storage

Each fragment has a closure that binds names to values (intrinsic) or identityIds (composite).

```json
{
    "closureId": 1000,
    "fragmentId": 100,
    "blueprintName": "UserCard",
    "closure": {
        "userId": 100,
        "displayName": "Alice",
        "user": 2001,
        "$anon_0": 42,
        "count": 0,
        "doubled": 0
    },
    "parentClosure": null
}
```