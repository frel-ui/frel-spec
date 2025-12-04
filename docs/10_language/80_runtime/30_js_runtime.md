# JavaScript Runtime

This document describes the Frel JavaScript runtime library API. The runtime manages reactive
data, subscriptions, and the notification system.

## Overview

The runtime is a singleton object that:

- Manages the global maps (`datum`, `closures`, `subscriptions`, `functions`)
- Allocates identities with proper encoding
- Executes the notification drain cycle
- Provides the API used by generated code

## Runtime API

### Identity Allocation

```javascript
runtime.alloc_datum_id()     // Returns even number (low bit = 0)
runtime.alloc_closure_id()   // Returns odd number (low bit = 1)
runtime.alloc_subscription_id()
runtime.alloc_function_id()
```

### Datum Operations

```javascript
// Create a new datum
runtime.create_datum(type, fields)
// Returns: datum identity (even number)

// Example:
const user_id = runtime.create_datum("User", {
    name: "Alice",
    age: 30
})
```

### Closure Operations

```javascript
// Create a new closure
runtime.create_closure(blueprint_name, parent_closure_id)
// Returns: closure identity (odd number)

// Example:
const child_id = runtime.create_closure("myapp.TodoRow", parent_id)
```

### Field Access

```javascript
// Get field value from datum or closure
runtime.get(identity, field_name)

// Set field value on datum or closure
runtime.set(identity, field_name, value)
```

The runtime determines whether to access `datum` or `closures` map based on the identity's
low bit.

```javascript
// Example:
const count = runtime.get(closure_id, "count")    // closure_id is odd
runtime.set(closure_id, "doubled", count * 2)

const name = runtime.get(datum_id, "name")        // datum_id is even
runtime.set(datum_id, "name", "Bob")
```

### Subscriptions

```javascript
// Create subscription
runtime.subscribe(source_id, target_id, selector, callback)
// Returns: subscription identity

// Remove subscription
runtime.unsubscribe(subscription_id)
```

**Selectors:**

```javascript
Everything                  // Any change
Structural                  // Structural revision changes
Carried                     // Carried revision changes (not structural)
Key("fieldName")            // Specific field
OneOf("field1", "field2")   // Any of listed fields
```

**Example:**

```javascript
runtime.subscribe(
    parent_id,
    child_id,
    Key("count"),
    function(runtime, subscription) {
        runtime.set(subscription.target_id, "p", runtime.get(subscription.source_id, "count"))
    }
)
```

### Function Registration

```javascript
// Register a callback function
runtime.register_function(fn)
// Returns: function identity

// Get function by identity
runtime.get_function(function_id)
```

### Instantiation

```javascript
// Instantiate a blueprint (creates closure, runs bindings)
runtime.instantiate(blueprint_name, parent_closure_id, params)
// Returns: closure identity
```

This is the high-level entry point that:

1. Creates a closure with `create_closure`
2. Sets parameter values from `params`
3. Calls `metadata[blueprint_name].internal_binding`
4. For each call site, recursively instantiates child blueprints and calls their bindings

### Destruction

```javascript
// Destroy a closure and all its owned resources
runtime.destroy(closure_id)
```

This:

1. Unsubscribes all `subscriptions_to_this`
2. Unsubscribes all `subscriptions_by_this`
3. Destroys all `owned_datum`
4. Recursively destroys all `child_closure_ids`
5. Removes the closure from the `closures` map
6. Removes from parent's `child_closure_ids`

## Event System

```javascript
// Queue an event (thread-safe)
runtime.put_event(event)

// Process all pending events (main thread only)
runtime.drain_events()
```

## Notification System

```javascript
// Queue a notification (called internally when data changes)
runtime.put_notification(subscription_id)

// Process all pending notifications (called from drain_events)
runtime.drain_notifications()
```

The drain cycle is described in [Runtime Data Model](10_runtime_data_model.md#drain-cycle).

## Internal State

```javascript
runtime = {
    // Maps
    datum: {},              // identity -> datum structure
    closures: {},           // identity -> closure structure
    subscriptions: {},      // identity -> subscription structure
    functions: {},          // identity -> callback function

    // Identity counters
    next_datum_id: 0,       // increments by 2 (stays even)
    next_closure_id: 1,     // increments by 2 (stays odd)
    next_subscription_id: 0,
    next_function_id: 0,

    // Event/notification queues
    event_queue: [],
    notification_queue: [],
    current_generation: 0,

    // Constants
    GEN_LIMIT: 1000
}
```

## Bootstrap

Application startup:

```javascript
// 1. Initialize runtime
const runtime = create_runtime()

// 2. Initialize themes
AppTheme$init(runtime)

// 3. Instantiate root blueprint
const root_id = runtime.instantiate("myapp.App", null, {})

// 4. Start event loop (platform-specific)
platform.on_frame(() => {
    runtime.drain_events()
    platform.render(root_id)
})
```

## Example: Runtime Execution

Given this Frel code:

```frel
blueprint Counter {
    count: u32 = 0
    doubled: u32 = count * 2
}
```

And user clicks increment:

```javascript
// 1. Event handler sets field
runtime.set(counter_closure_id, "count", 1)

// 2. set() internally:
//    - Updates closures[counter_closure_id].fields.count = 1
//    - Updates set_generation
//    - Queues notification for subscription watching "count"

// 3. drain_notifications() processes the queue:
//    - Finds subscription: source=counter, target=counter, selector=Key("count")
//    - Calls Counter$doubled$callback(runtime, subscription)

// 4. Callback executes:
//    - runtime.get(source_id, "count") returns 1
//    - runtime.set(target_id, "doubled", 2)

// 5. set() for "doubled":
//    - Updates field
//    - Queues notifications for any subscribers to "doubled"

// 6. drain_notifications() continues until queue is empty
```
