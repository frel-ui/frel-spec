# Fan-in Stores

Fan-in stores combine reactive and imperative state management. They automatically recompute when 
dependencies change (like read-only stores) but can also be directly assigned to (like writable
stores). They're perfect for scenarios where state needs both reactive updates and manual overrides.

## Syntax

`fanin <id> [:<type>]? = <calc_expr>`

## Semantics

- **Kind**: Writable state that subscribes to all stores read by `<calc_expr>`.
- **Calculation**:
    `<calc_expr>` is re-evaluated when dependencies change, and the result becomes the new value.
    Must be a pure Frel expression. See [Frel Expressions](../15_expressions/10_expression_basics.md).
- **Writes**: Fan-in stores can be modified in event handlers through:
  - Direct assignment: `<id> = <expr2>` (in host language statement context)
  - In-place mutation: `<id>.push(x)`, `<id>.insert(k, v)`, etc.
- **Reactive continuation**:
    After a manual write, the store continues tracking dependencies. The next time dependencies
    change, `<calc_expr>` is re-evaluated and overwrites the manually set value.
- **Order/consistency**:
    Per drain cycle, `<calc_expr>` is evaluated once after dependencies settle (no glitches). If
    both a dependency update and manual write occur in the same cycle, the manual write wins.
- **Reactivity**: 
    When the value changes (either through calculation or manual write), dependent stores are
    notified. See [Mutation Detection](10_store_basics.md#mutation-detection) for details on how 
    the runtime detects changes.
- **Type**: Can be inferred from `<calc_expr>` or explicitly annotated.

## Examples

### Simple Mirroring

```frel
blueprint SelectionSync(external_selection: Option<u32>) {
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

When `external_selection` changes, `selection` updates automatically. When you click "Select Item 5",
`selection` is set to `Some(5)`. The next time `external_selection` changes, `selection` will mirror
the new value.

### Accumulating Events

Collect events over time while allowing manual clear:

```frel
blueprint EventLog() {
    source events = sse("/events")
    writable log: List<Event> = []

    // Accumulate events as they arrive
    events .. on_value { event: Event ->
        // TODO: list.append(event) - need to specify list append operation
        log = log  // placeholder
    }

    column {
        text { "Events: ${log.length}" }

        repeat on log as event {
            text { "${event.timestamp}: ${event.message}" }
        }

        button { "Clear Log" }
            .. on_click { log = [] }
    }
}
```

### Notification Center with Removal

```frel
blueprint NotificationCenter() {
    source notifications = sse("/notifications")
    writable notification_list: List<Notification> = []

    notifications .. on_value { notif: Notification ->
        // TODO: list.append(notif) - need to specify list append operation
        notification_list = notification_list  // placeholder
    }

    column {
        repeat on notification_list as notif {
            row {
                text { notif.message }
                button { "×" }
                    .. on_click {
                        notification_list = notification_list.filter(n => n.id != notif.id)
                    }
            }
        }

        button { "Clear All" }
            .. on_click { notification_list = [] }
    }
}
```

### Deduplication

Only add items that aren't already in the list:

```frel
blueprint UniqueItemList() {
    source new_item = sse("/items")
    writable items: List<Item> = []

    new_item .. on_value { item: Item ->
        // TODO: conditional append - if item not in list, append it
        // Need: list.contains() or list.any() and list.append()
        items = items  // placeholder
    }

    column {
        text { "${items.length} unique items" }

        repeat on items as item {
            text { item.name }
        }

        button { "Clear" }
            .. on_click { items = [] }
    }
}
```

### Sliding Window

Keep only the last N items:

```frel
blueprint RecentActivity() {
    source activity = sse("/activity")
    writable recent: List<Activity> = []

    activity .. on_value { act: Activity ->
        // TODO: append and trim list
        // Need: list.append() and list.slice() or list.takeLast()
        recent = recent  // placeholder
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

Merge multiple event streams into one timeline:

```frel
blueprint CombinedFeed() {
    source user_actions = sse("/user-actions")
    source system_events = sse("/system-events")
    writable timeline: List<Event> = []

    user_actions .. on_value { action: UserAction ->
        // TODO: append and sort list
        // Need: list.append() and list.sort()
        timeline = timeline  // placeholder
    }

    system_events .. on_value { event: SystemEvent ->
        // TODO: append and sort list
        // Need: list.append() and list.sort()
        timeline = timeline  // placeholder
    }

    column {
        repeat on timeline as event {
            EventCard(event)
        }

        button { "Clear Timeline" }
            .. on_click { timeline = [] }
    }
}
```

### Form Validation with Override

Track validation results but allow manual override:

```frel
blueprint EmailInput() {
    source validator = validation_source()
    writable email = ""
    fanin is_valid = validator.latest().map(|v| v.is_ok()).unwrap_or(true)

    column {
        text_input { email }
            .. on_change { new_email: String -> {
                email = new_email
                trigger_validation(email.clone())
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
blueprint RateLimitedSearch(query: String) {
    source debounced = debounce(query, 300)  // Wait 300ms after typing stops
    writable pending_queries: Vec<String> = vec![]

    debounced .. on_value { q: String -> {
        pending_queries.push(q)
    }

    column {
        text { "${pending_queries.len()} searches pending" }

        when !pending_queries.is_empty() {
            button { "Search Now" }
                .. on_click {
                    for query in &pending_queries {
                        perform_search(query.clone())
                    }
                    pending_queries = vec![]
                }
        }
    }
}
```

## Comparison with Other Stores

| Aspect          | Read-only (`decl`)  | Writable          | Fan-in                  |
|-----------------|---------------------|-------------------|-------------------------|
| Dependencies    | Auto-subscribed     | None              | Auto-subscribed         |
| Assignment      | ✗ Not allowed       | ✓ Full control    | ✓ Temporarily overrides |
| Updates         | Auto on dep change  | Manual only       | Both auto and manual    |
| Use case        | Pure computation    | User input state  | Hybrid reactive/manual  |

## Best Practices

### Choose the Right Store Type

```frel
// For pure derived values - use decl
decl total = price * quantity

// For user input and independent state - use writable
writable email = ""

// For reactive mirroring with override capability - use fanin
fanin selection = external.selection

// For accumulation from sources - use writable + on_value
writable log: Vec<Event> = vec![]
events .. on_value { event -> { log.push(event) }
```

### Manual Override Semantics

Manual assignments temporarily override the reactive expression:

```frel
fanin selection = external.selection

button { "Override to 5" }
    .. on_click { selection = Some(5) }

// After this click:
// - selection is now Some(5)
// - When external.selection next changes (e.g., to Some(10)):
//   - selection becomes Some(10)
//   - The manual override is replaced by the reactive value
```

**Key principle:** Manual assignment temporarily sets a value. The next time dependencies change, 
the reactive expression takes over again.

### When to Use Fan-in vs Writable

Use **fan-in** when:
- You want the value to track a reactive expression most of the time
- But occasionally need to override it manually
- And want reactive tracking to resume after the override

Use **writable** when:
- The state is primarily controlled by user input or event handlers
- You don't want automatic updates from dependencies
- You need full manual control