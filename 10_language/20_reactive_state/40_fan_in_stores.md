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
    Must be a PHLE.
- **Writes**: Fan-in stores can be modified in event handlers through:
  - Direct assignment: `<id> = <expr2>` where `<expr2>` must be a PHLE
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

When `external_selection` changes, `selection` updates automatically. When you click "Select Item 5",
`selection` is set to `Some(5)`. The next time `external_selection` changes, `selection` will mirror
the new value.

### Accumulating Events

Collect events over time while allowing manual clear:

```frel
fragment EventLog() {
    source events = sse("/events")
    writable log: Vec<Event> = vec![]

    // Accumulate events as they arrive
    events .. on_value |event: Event| {
        log.push(event)
    }

    column {
        text { "Events: ${log.len()}" }

        repeat on log as event {
            text { "${event.timestamp}: ${event.message}" }
        }

        button { "Clear Log" }
            .. on_click { log = vec![] }
    }
}
```

### Notification Center with Removal

```frel
fragment NotificationCenter() {
    source notifications = sse("/notifications")
    writable notification_list: Vec<Notification> = vec![]

    notifications .. on_value |notif: Notification| {
        notification_list.push(notif)
    }

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

### Deduplication

Only add items that aren't already in the list:

```frel
fragment UniqueItemList() {
    source new_item = sse("/items")
    writable items: Vec<Item> = vec![]

    new_item .. on_value |item: Item| {
        if !items.iter().any(|i| i.id == item.id) {
            items.push(item)
        }
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

### Sliding Window

Keep only the last N items:

```frel
fragment RecentActivity() {
    source activity = sse("/activity")
    writable recent: Vec<Activity> = vec![]

    activity .. on_value |act: Activity| {
        recent.push(act)
        if recent.len() > 10 {
            recent.remove(0)  // Keep only last 10
        }
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
fragment CombinedFeed() {
    source user_actions = sse("/user-actions")
    source system_events = sse("/system-events")
    writable timeline: Vec<Event> = vec![]

    user_actions .. on_value |action: UserAction| {
        timeline.push(Event::UserAction(action))
        timeline.sort_by_key(|e| e.timestamp())
    }

    system_events .. on_value |event: SystemEvent| {
        timeline.push(Event::SystemEvent(event))
        timeline.sort_by_key(|e| e.timestamp())
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
fragment EmailInput() {
    source validator = validation_source()
    writable email = ""
    fanin is_valid = validator.latest().map(|v| v.is_ok()).unwrap_or(true)

    column {
        text_input { email }
            .. on_change |new_email: String| {
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
fragment RateLimitedSearch(query: String) {
    source debounced = debounce(query, 300)  // Wait 300ms after typing stops
    writable pending_queries: Vec<String> = vec![]

    debounced .. on_value |q: String| {
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
events .. on_value |event| { log.push(event) }
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