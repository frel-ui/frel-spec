# Runtime Data Model

This document describes the Frel runtime data model, including storage structures, identity
management, closures, subscriptions, and the notification system.

## Core Tenets

1. The application state is represented by datums and closures stored in global maps.
2. Reactive connections are represented by subscriptions.
3. Change propagation is push: when data changes, it notifies subscribers.
4. Value access is pull: subscribers decide when to get data (might be immediate or deferred).
5. One single thread manages the state of the application, no multi-thread synchronization is needed.

**One promise**: "You'll be notified when the truth might have changed."

**One action**: "Call `get` when you care."

## Runtime Maps

The runtime maintains five global maps:

### Shared Identity Space

The `datum` and `closures` maps share an identity space using low-bit encoding. This prevents
closure fields from accidentally holding closure identities - they are in different identity spaces.

| Map | Identity bits | Contents |
|-----|---------------|----------|
| `datum` | `...0` | Scheme/collection instances created by blueprints |
| `closures` | `...1` | Blueprint instances (structural + cleanup + fields) |

### Separate Identity Spaces

These maps have their own identity counters. They are internal runtime machinery, never stored in
fields.

| Map | Contents |
|-----|----------|
| `subscriptions` | Subscription records |
| `functions` | Callback functions |

### Static Lookup

| Map | Key | Contents |
|-----|-----|----------|
| `metadata` | Qualified name | Function tables, validation rules, type info |

## Datum

Datums are scheme and collection instances created by blueprints. They participate in the
reactive system and can be referenced from closure fields.

**Datum structure:**

```javascript
datum[2000] = {
    identity_id: 2000,              // even number (low bit = 0)
    type: "User",
    structural_rev: 1,
    carried_rev: 1,
    set_generation: 0,
    availability: "Ready",
    error: null,
    owner: null,                    // closure identity that owns this datum
    fields: {
        name: "Alice",              // intrinsic: stored directly
        age: 30,                    // intrinsic: stored directly
        profile: 2002               // composite: stored as datum identity
    },
    items: null,                    // for collections
    subscriptions: []               // subscription IDs observing this datum
}
```

**Field storage:**

- **Intrinsic types** (primitives, String, enums): stored directly as values
- **Composite types** (schemes, collections): stored as datum identities

**Collections:**

```javascript
// List<i32> - intrinsic items stored directly
datum[2004] = {
    identity_id: 2004,
    type: "List<i32>",
    // ... other fields ...
    fields: null,
    items: [10, 20, 30]
}

// List<User> - composite items stored as identities
datum[2006] = {
    identity_id: 2006,
    type: "List<User>",
    // ... other fields ...
    fields: null,
    items: [2000, 2002, 2008]       // datum identities
}
```

## Closures

Closures are blueprint instances created when a blueprint is instantiated. A closure has three
distinct parts:

```
Closure = {
    structural: { parent, children }           // unsubscribable, holds closure identities
    cleanup: { subscriptions, owned_datum }    // unsubscribable
    fields: { ... }                            // subscribable
}
```

**Critical constraint:** Closure fields can hold:
- Intrinsic values (stored directly)
- Datum identities (even numbers)

Closure fields **cannot** hold closure identities. This is enforced by the identity encoding -
closures have odd identities, datums have even identities.

**Closure structure:**

```javascript
closures[1001] = {
    closure_id: 1001,               // odd number (low bit = 1)
    blueprint: "Middle",

    // Structural (unsubscribable, holds closure identities)
    parent_closure_id: 1003,
    child_closure_ids: [1005, 1007],

    // Cleanup (unsubscribable)
    subscriptions_to_this: [4001, 4002],
    subscriptions_by_this: [4003, 4004],
    owned_datum: [2000, 2002],

    // Fields (subscribable)
    fields: {
        p1: "Hello",                // intrinsic
        p2: 2000,                   // datum identity
        count: 42                   // intrinsic
    }
}
```

**Structural vs Fields:**

The structural part (parent/children) uses closure identities to form the fragment tree. This is
separate from the reactive field system. When a closure is destroyed, its structural links are
used to tear down child closures.

The cleanup part tracks what the closure owns for proper cleanup: subscriptions it participates
in and datums it created.

## Subscriptions

Subscriptions connect data sources to callbacks. They can target either datum or closure fields.

```javascript
subscriptions[4001] = {
    subscription_id: 4001,
    source_id: 1001,                // datum or closure identity
    target_id: 1003,                // datum or closure identity
    selector: "Everything",         // or Key("fieldName"), OneOf("f1", "f2")
    callback_id: 3001               // function identity
}
```

**Selectors:**

| Selector | Triggers when |
|----------|---------------|
| `Everything` | Any change (structural or carried) |
| `Structural` | Structural revision increments |
| `Carried` | Carried revision increments (but NOT structural) |
| `Key("name")` | Specific field changes |
| `OneOf("a", "b")` | Any of the listed fields change |

## Functions

Functions are callbacks registered by the generated code. They are called during notification
processing.

```javascript
functions[3001] = function(runtime, subscription) {
    runtime.set(
        subscription.target_id,
        "sum",
        runtime.get(subscription.source_id, "p1") + runtime.get(subscription.source_id, "p2")
    )
}
```

## Metadata

Metadata is keyed by qualified name (not identity). It contains static, compile-time information:

- Function tables (internal binding, call site bindings)
- Validation rules
- Type information

```javascript
metadata["myapp.Counter"] = {
    internal_binding: Counter$internal_binding,
    call_sites: {
        "42": { blueprint: "myapp.Display", binding: Counter$42$call_site_binding }
    }
}
```

## Events and Notifications

The runtime is event-driven. Events come from:
- User actions (mouse, keyboard, gestures)
- Network traffic
- Timers

### Event Queue

The event queue provides two functions:

- `put_event`: Thread-safe, can be called from any thread
- `drain_events`: **NOT** thread-safe, called only from the main UI thread

### Notification Queue

When an event changes data, notifications are created for each affected subscription.

Both `put_notification` and `drain_notifications` are **NOT** thread-safe - they run from
`drain_events` on the main thread.

**Key properties:**

- Notifications are subscription IDs
- Processing does **NOT** preserve subscription order
- Notifications are processed in batches called *generations*

### Drain Cycle

`drain_notifications` works in a loop:

1. **Sanity check**: Loop must finish in ≤ `GEN_LIMIT` cycles (1000 by default)
2. **Take**: All pending notifications from queue → the *processed generation*
3. **Reset**: Empty queue and increment generation counter
4. **Execute**: Call callback functions of the processed generation
   - May change data → those changes belong to the *next generation*
   - May queue a re-render (processed outside the data subsystem)
5. **Stop**: If the next generation has no notifications

This ensures all cascading changes are applied before the cycle completes.

### Generation Tracking

When a field changes, the runtime saves the current generation into `set_generation` of the
datum/closure structure.

### Subscribe During Drain

When subscribing during drain (the common case):

- If `set_generation == current_generation`: queue a notification immediately
- Otherwise: wait for normal change mechanism to trigger notification

### Unsubscribe During Drain

Unsubscribing simply removes the subscription. If the subscription ID is reached later in the
drain cycle, it's skipped (subscription no longer exists).

## Example: Blueprint Instantiation

```frel
blueprint Outer {
    Middle("Hello")
}

blueprint Middle(p1: String) {
    count: u32 = 42
    Inner(p1, count)
}

blueprint Inner(p1: String, p2: u32) {
    text { "${p1}: ${p2}" }
}
```

**Runtime state after instantiation:**

```javascript
// Outer closure
closures[1001] = {
    closure_id: 1001,
    blueprint: "Outer",
    parent_closure_id: null,
    child_closure_ids: [1003],
    subscriptions_to_this: [],
    subscriptions_by_this: [],
    owned_datum: [],
    fields: {}
}

// Middle closure
closures[1003] = {
    closure_id: 1003,
    blueprint: "Middle",
    parent_closure_id: 1001,
    child_closure_ids: [1005],
    subscriptions_to_this: [4001, 4002],
    subscriptions_by_this: [],
    owned_datum: [],
    fields: {
        p1: "Hello",
        count: 42
    }
}

// Inner closure
closures[1005] = {
    closure_id: 1005,
    blueprint: "Inner",
    parent_closure_id: 1003,
    child_closure_ids: [],
    subscriptions_to_this: [],
    subscriptions_by_this: [4001, 4002],
    owned_datum: [],
    fields: {
        p1: "Hello",
        p2: 42
    }
}

// Subscription: Middle.p1 -> Inner.p1
subscriptions[4001] = {
    subscription_id: 4001,
    source_id: 1003,
    target_id: 1005,
    selector: Key("p1"),
    callback_id: 3001
}

// Subscription: Middle.count -> Inner.p2
subscriptions[4002] = {
    subscription_id: 4002,
    source_id: 1003,
    target_id: 1005,
    selector: Key("count"),
    callback_id: 3002
}
```
