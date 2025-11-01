# Fan-in Stores

Fan-in stores combine reactive and imperative state management. They automatically recompute when dependencies change (like read-only stores) but can also be directly assigned to (like writable stores). They're perfect for scenarios where state needs both reactive updates and manual overrides.

## Syntax

`fanin <id> [:<type>]? = <calc_expr> [with <reducer>]`

## Semantics

- **Kind**: Writable state that subscribes to all stores read by `<calc_expr>`.
- **Calculation**: `<calc_expr>` is re-evaluated when dependencies change to produce an input value. Must be a PHLE.
- **Reducer**: Combines the current state and new input into the next state.
- **Default reducer**: `replace(state, input) = input` - simply mirrors the dependencies.
- **Custom reducer**: User supplies a closure with signature `|state, input| → state`.
- **Writes**: `<id> = <expr2>` is allowed and directly changes the current state. Future dependency changes continue applying the reducer on top of that new state. The right-hand side `<expr2>` must be a PHLE.
- **Order/consistency**: Per drain cycle, `<calc_expr>` is evaluated once after dependencies settle; reducer is applied once (no glitches).
- **Side effects**: Reducers should be pure; side effects belong in event handlers or sources.

## Built-in Reducers

| Reducer     | Signature                    | Description                           |
|-------------|------------------------------|---------------------------------------|
| `replace`   | `(_, input) -> input`        | Replace state with input (default)    |
| `append`    | `(vec, item) -> vec + item`  | Append item to vector                 |
| `union`     | `(set, items) -> set ∪ items`| Union of sets                         |
| `max_by`    | `(state, input) -> max`      | Keep maximum value                    |
| `min_by`    | `(state, input) -> min`      | Keep minimum value                    |
| `coalesce`  | `(state, opt) -> opt or state`| Keep state if input is None          |

## Examples

### Simple Mirroring (Default)

The default `replace` reducer makes fan-in behave like a derived store with manual override capability:

```frel
fragment SelectionSync(external_selection: Option<u32>) {
    fanin selection = external_selection  // Mirrors external_selection

    column {
        text { "Selected: ${selection.unwrap_or(0)}" }

        // Automatically updates when external_selection changes
        // But can also be manually overridden:
        button { "Select Item 5" }
            .. on_click { selection = Some(5) }

        button { "Clear" }
            .. on_click { selection = None }
    }
}
```

### Accumulating Events

Collect events over time while allowing manual clear:

```frel
fragment EventLog(events: source<Event>) {
    fanin log: Vec<Event> = events.latest() with |mut state, input| {
        if let Some(event) = input {
            state.push(event);
        }
        state
    }

    column {
        text { "Events: ${log.len()}" }

        repeat on log as event {
            text { "${event.timestamp}: ${event.message}" }
        }

        button { "Clear Log" }
            .. on_click { log = vec![] }  // Manual clear
    }
}
```

### Using the `append` Reducer

Simpler syntax for accumulation:

```frel
fragment NotificationCenter() {
    source notifications = sse("/notifications")

    fanin notification_list = notifications.latest() with append

    column {
        repeat on notification_list as notif {
            row {
                text { notif.message }
                button { "×" }
                    .. on_click {
                        notification_list = notification_list
                            .iter()
                            .filter(|n| n.id != notif.id)
                            .cloned()
                            .collect()
                    }
            }
        }

        button { "Clear All" }
            .. on_click { notification_list = vec![] }
    }
}
```

### Keeping Maximum Value

Track the highest value seen:

```frel
fragment HighScore(current_score: u32) {
    fanin high_score = current_score with max_by

    column {
        text { "Current: ${current_score}" }
        text { "High Score: ${high_score}" }

        button { "Reset High Score" }
            .. on_click { high_score = 0 }
    }
}
```

### Coalesce Pattern

Keep last valid value when new value is None:

```frel
fragment LastKnownLocation(gps_signal: Option<Location>) {
    fanin last_location = gps_signal with coalesce

    column {
        select on gps_signal {
            Some(loc) => text { "Current: ${loc}" } .. font { color: Green }
            None => text { "Signal lost, last known: ${last_location}" }
                .. font { color: Orange }
        }

        button { "Clear History" }
            .. on_click { last_location = None }
    }
}
```

### Custom Reducer: Deduplication

Only add items that aren't already in the list:

```frel
fragment UniqueItemList(new_item: source<Item>) {
    fanin items: Vec<Item> = new_item.latest() with |mut state, input| {
        if let Some(item) = input {
            if !state.iter().any(|i| i.id == item.id) {
                state.push(item);
            }
        }
        state
    }

    column {
        text { "${items.len()} unique items" }

        repeat on items as item {
            text { item.name }
        }

        button { "Clear" }
            .. on_click { items = vec![] }
    }
}
```

### Custom Reducer: Sliding Window

Keep only the last N items:

```frel
fragment RecentActivity(activity: source<Activity>) {
    fanin recent: Vec<Activity> = activity.latest() with |mut state, input| {
        if let Some(act) = input {
            state.push(act);
            if state.len() > 10 {
                state.remove(0);  // Keep only last 10
            }
        }
        state
    }

    column {
        text { "Recent Activity (last 10)" }

        repeat on recent as act {
            text { "${act.timestamp}: ${act.description}" }
        }
    }
}
```

### Combining Multiple Sources

Fan-in can combine data from multiple reactive sources:

```frel
fragment CombinedFeed() {
    source user_actions = sse("/user-actions")
    source system_events = sse("/system-events")

    // Merge both streams into one timeline
    fanin timeline = match (user_actions.latest(), system_events.latest()) {
        (Some(action), _) => Some(Event::UserAction(action)),
        (_, Some(event)) => Some(Event::SystemEvent(event)),
        _ => None,
    } with |mut state: Vec<Event>, input| {
        if let Some(event) = input {
            state.push(event);
            state.sort_by_key(|e| e.timestamp());
        }
        state
    }

    column {
        repeat on timeline as event {
            EventCard(event)
        }

        button { "Clear Timeline" }
            .. on_click { timeline = vec![] }
    }
}
```

### Form Validation with Override

Track validation results but allow manual override:

```frel
fragment EmailInput(validator: source<ValidationResult>) {
    writable email = ""
    fanin is_valid = validator.latest().map(|v| v.is_ok()).unwrap_or(true)

    column {
        text_input { email }
            .. on_change |new_email: String| {
                email = new_email;
                trigger_validation(email.clone());
            }

        when !is_valid {
            text { "Invalid email format" } .. font { color: Red }
        }

        // Admin can override validation
        button { "Force Accept" }
            .. visible { is_admin() }
            .. on_click { is_valid = true }
    }
}
```

### Rate Limiting with Accumulation

Accumulate requests but allow manual flush:

```frel
fragment RateLimitedSearch(query: String) {
    source debounced = debounce(query, 300)  // Wait 300ms after typing stops

    fanin pending_queries = debounced.latest() with |mut state: Vec<String>, input| {
        if let Some(q) = input {
            state.push(q);
        }
        state
    }

    column {
        text { "${pending_queries.len()} searches pending" }

        when !pending_queries.is_empty() {
            button { "Search Now" }
                .. on_click {
                    for query in &pending_queries {
                        perform_search(query.clone());
                    }
                    pending_queries = vec![];
                }
        }
    }
}
```

## Comparison with Other Stores

| Aspect          | Read-only (`decl`)  | Writable          | Fan-in                  |
|-----------------|---------------------|-------------------|-------------------------|
| Dependencies    | Auto-subscribed     | None              | Auto-subscribed         |
| Assignment      | ✗ Not allowed       | ✓ Full control    | ✓ Overrides reducer     |
| Updates         | Auto on dep change  | Manual only       | Both auto and manual    |
| Use case        | Pure computation    | User input state  | Hybrid reactive/manual  |

## Best Practices

### Choose the Right Reducer

```frel
// For mirroring with override capability - use default
fanin selection = external.selection

// For accumulation - use append
fanin log = events.latest() with append

// For last-valid-value - use coalesce
fanin last_gps = gps_signal with coalesce

// For custom logic - write a reducer
fanin unique_items = new_items.latest() with |state, input| { /* custom */ }
```

### Keep Reducers Pure

Reducers should not have side effects:

```frel
// Bad - side effect in reducer
fanin items = new_item with |mut state, input| {
    log("Adding item");  // Side effect!
    state.push(input);
    state
}

// Good - side effects in event handler
fanin items = new_item with append

button { "Add" }
    .. on_click {
        log("Adding item");  // Side effect in handler
        trigger_add();
    }
```

### Manual Override Semantics

Manual assignments replace the current state and become the new base for future reducer applications:

```frel
fanin count = external_count  // Uses replace reducer by default

button { "Override to 100" }
    .. on_click { count = 100 }

// After this click:
// - count is now 100
// - When external_count next changes (e.g., to 50):
//   - Reducer is called: replace(100, 50) = 50
//   - count becomes 50
// - The reducer uses the manually set value (100) as the state

// With accumulation:
fanin items: Vec<Item> = source.latest() with append

button { "Clear" }
    .. on_click { items = vec![] }

// After clear:
// - items is vec![]
// - When source next emits an item "foo":
//   - Reducer is called: append(vec![], "foo") = vec!["foo"]
//   - items becomes vec!["foo"]
// - Future emissions continue appending to this new cleared base
```

**Key principle:** Manual assignment sets a new state value. The reducer continues to operate, using this new value as the state parameter in future applications.

### Type Annotations for Reducers

**When a custom reducer is used, type annotation is mandatory:**

```frel
// Required - explicit type with custom reducer
fanin items: Vec<Item> = source.latest() with |state, input| {
    // Compiler knows state is Vec<Item>
    // ...
}

// Optional - type can be inferred without reducer
fanin selection = external.selection  // Type inferred from external.selection

// Optional - type can be inferred with built-in reducer
fanin log = events.latest() with append  // Type inferred from events
```

**Rationale:** While the type could theoretically be inferred from the reducer's return type, explicit annotation improves code clarity and helps catch type mismatches early.