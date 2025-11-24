# Runtime Data Model

1. The application state is represented by a (possibly high) number of datum.
2. Reactive connections between datum are represented by subscriptions.
3. `Datum` = the sole source of truth.
4. Change propagation is push: when a datum changes, it notifies subscribers.
5. Value access is pull: subscribers decide when to get data from the datum (might be immediate, but could be deferred).

Core tenets:

- **Single abstraction**: `Datum`.
- **One promise**: “You’ll be notified when the truth might have changed.”
- **One action**: “Call `get` when you care.”

**Notes**

One single thread manages the state of the application, no multi-thread synchronization is needed.

## Subscriptions

Subscriptions describe what happens when a datum changes.

| Property            | Description                                          |
|---------------------|------------------------------------------------------|
| **Subscription Id** | A unique identifier of this **subscription**.        |
| **Datum Identity**  | The datum this subscription belongs to.              |
| **Selector**        | Selects which changes should trigger a notification. |
| **Callback Id**     | Id of the function that handles the notification.    |

```javascript
subscriptions[1001] = {
    subscription_id: 1001,
    datum_id: 2001,
    selector: "Everything",
    callback_id: 3001
}
```

## 1. Core Storage Strategy

The runtime uses **two-tier storage** based on type category:

### 1.1 Intrinsic Types (Optimized)

**Intrinsic types** (primitives, String, enums) are **stored directly as values** without
the full datum structure. This is an optimization that is possible because:

- Intrinsic datum is immutable - changing means creating a new value
- Compiler knows types statically
- Identity is the value itself
- Change tracking is provided by the datum that contains the intrinsic datum

```javascript
fields = {
    count: 42,
    name: "Alice",
    is_active: true
}
```

### 1.2 Composite Types (Full Datum Structure)

**Composite types** (schemes, collections, arenas, backends) need full reactive tracking.

**Datum structure for composite types:**

```javascript
datum[2001] = {
  identity_id: 2001,
  type: "User",
  structural_rev: 0,
  carried_rev: 5,
  set_generation: 0,
  availability: "Ready",
  error: null,
  owner: null,
  fields: {
    id: 100,
    name: "Alice",
    profile: 3001
  },
  items: null,
  entries: null,
  subscriptions: [ 10, 11 ]
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

```javascript
datum[2001] = {
  identity_id: 2001,
  type: "UserWithProfile",
  structural_rev: 0,
  carried_rev: 0,
  set_generation: 0,
  availability: "Ready",
  error: null,
  owner: null,
  fields: {
    name: "Alice",
    age: 30,
    profile: 3001
  },
  items: null,
  entries: null,
  subscriptions: []
}
```

- `name`: Intrinsic, stored directly as string
- `age`: Intrinsic, stored directly as number
- `profile`: Composite, stored as identityId
- `owner`: null (this is a root datum, not owned by another)

**The Profile scheme (separate datum):**

```javascript
datum[3001] = {
  identity_id: 3001,
  type: "Profile",
  structural_rev: 0,
  carried_rev: 0,
  set_generation: 0,
  availability: "Ready",
  error: null,
  owner: 2001,
  fields: {
    bio: "Software engineer",
    avatar: "https://..."
  },
  items: null,
  entries: null,
  subscriptions: []
}
```

- `owner`: 2001 (owned by userDatum - changes propagate up the ownership chain)

## 3. Collections

**List<i32> with values [10, 20, 30]:**

```javascript
datum[3001] = {
  identity_id: 3001,
  type: "List<i32>",
  structural_rev: 0,
  carried_rev: 0,
  set_generation: 0,
  availability: "Ready",
  error: null,
  owner: null,
  fields: null,
  items: [
    10,
    20,
    30
  ],
  entries: null,
  subscriptions: []
}
```

- `items`: Plain numbers, no wrappers!
- `owner`: null (root datum)

**List<User> with composite items:**

```javascript
datum[3002] = {
  identity_id: 3002,
  type: "List<User>",
  structural_rev: 0,
  carried_rev: 0,
  set_generation: 0,
  availability: "Ready",
  error: null,
  owner: null,
  fields: null,
  items: [
    2001,
    2002,
    2003
  ],
  entries: null,
  subscriptions: []
}
```

- `items`: identityIds of User datums (each User datum has `owner: 3002`)
- `owner`: null (root datum)