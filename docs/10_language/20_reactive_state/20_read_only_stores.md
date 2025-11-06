# Read-only Stores

Read-only stores hold immutable values that either remain constant or automatically recompute when their dependencies change. They use the `decl` keyword.

## Syntax

`decl <id> [:<type>]? = <expr>`

## Semantics

- **Kind**: Subscribes to all stores used in `<expr>`. If no stores are referenced, it behaves as a constant.
- **Initializer**: `<expr>` must be a pure Frel expression - no side effects allowed. See [Frel Expressions](../15_expressions/10_expression_basics.md).
- **Writes**: Not assignable. Read-only stores cannot be modified after creation.
- **Updates**: Automatically recomputed when any dependency changes (glitch-free; one recompute per drain cycle).
- **Guards**: Dependency graphs must be acyclic. Cycles are detected at runtime and cause an error.

## Constants vs Derived

From the DSL perspective, constants and derived stores are the same type - the difference is simply whether they reference other stores:

```frel
decl theme = "dark"              // Constant (no dependencies)
decl doubled = count * 2         // Derived (depends on 'count')
```

> [!NOTE]
>
> From the DSL perspective, read-only stores with and without dependencies are the same, the latter
> is just a specific case where the dependency set is empty. This provides a clear mental model:
> the important characteristic is that these stores are read-only.
>
> From an implementation perspective, stores with no dependencies can be optimized while stores with
> dependencies need bookkeeping, subscriptions, and notification propagation. However, that is
> purely an implementation detail, not a DSL concern.
>
> We intentionally use a single keyword to avoid unnecessary complexity.

## Examples

### Constants

Simple unchanging values:

```frel
blueprint AppConfig() {
    decl app_name = "My Application"
    decl version = "1.0.0"
    decl max_items = 100

    text { "${app_name} v${version}" }
}
```

### Simple Derivations

Values computed from other stores:

```frel
blueprint PriceCalculator() {
    writable price = 100.0
    writable quantity = 1

    decl subtotal = price * quantity
    decl tax = subtotal * 0.1
    decl total = subtotal + tax

    column {
        text { "Subtotal: $${subtotal}" }
        text { "Tax: $${tax}" }
        text { "Total: $${total}" }
    }
}
```

### Complex Computations

Derived stores can use any Pure HLE, including method calls and transformations:

```frel
blueprint UserList(users: List<User>) {
    writable search = ""

    // TODO: List filter operation not yet specified
    decl filtered_users = filter_users(users, search)

    // TODO: List .len() operation not yet specified
    decl user_count = count_users(filtered_users)
    decl has_results = user_count > 0

    column {
        text { "Found ${user_count} users" }

        when has_results {
            repeat on filtered_users as user {
                text { user.name }
            }
        }
    }
}
```

### Working with Options

Derived stores work naturally with Option types:

```frel
blueprint UserProfile(user_id: Option<u32>) {
    decl has_user = user_id.is_some()
    decl user_display = user_id
        .map(|id| format!("User #{}", id))
        .unwrap_or_else(|| "No user selected".to_string())

    text { user_display }
}
```

### Chaining Derivations

Derived stores can depend on other derived stores:

```frel
blueprint TemperatureDisplay() {
    writable celsius = 20.0

    decl fahrenheit = celsius * 9.0 / 5.0 + 32.0
    decl kelvin = celsius + 273.15

    decl temp_status = if celsius < 0.0 {
        "Freezing"
    } else if celsius < 20.0 {
        "Cold"
    } else if celsius < 30.0 {
        "Comfortable"
    } else {
        "Hot"
    }

    column {
        text { "${celsius}°C = ${fahrenheit}°F = ${kelvin}K" }
        text { "Status: ${temp_status}" }
    }
}
```

### Collections and Aggregations

```frel
blueprint Statistics(values: List<f64>) {
    // TODO: List operations for aggregation not yet specified
    decl count = count_values(values)
    decl sum = sum_values(values)
    decl average = if count > 0 { sum / count } else { 0.0 }
    decl max = max_value(values)
    decl min = min_value(values)

    column {
        text { "Count: ${count}" }
        text { "Average: ${average:.2}" }
        text { "Min: ${min:.2}, Max: ${max:.2}" }
    }
}
```

### Conditional Logic

```frel
blueprint StatusBadge(status: String) {
    decl badge_color = match status.as_str() {
        "active" => Green,
        "pending" => Yellow,
        "error" => Red,
        _ => Gray,
    }

    decl badge_text = status.to_uppercase()

    text { badge_text }
        .. background { color: badge_color }
        .. padding { horizontal: 8 vertical: 4 }
        .. corner_radius { 4 }
}
```

## Reactivity Behavior

When a dependency changes, all derived stores that depend on it recompute automatically:

```frel
blueprint ReactivityDemo() {
    writable count = 0

    decl doubled = count * 2          // Recomputes when count changes
    decl tripled = count * 3          // Recomputes when count changes
    decl sum = doubled + tripled      // Recomputes when doubled OR tripled change

    // When count changes from 0 to 1:
    // 1. count updates to 1
    // 2. doubled recomputes to 2
    // 3. tripled recomputes to 3
    // 4. sum recomputes to 5 (only once, not twice)

    button { "Increment" } .. on_click { count = count + 1 }
}
```

## Type Inference

Types are usually inferred, but can be explicitly annotated when needed:

```frel
decl count = 0                    // Inferred as i32
decl count: u32 = 0               // Explicit type
decl items: List<String> = []  // Helpful for empty collections
```

## Best Practices

### Keep Computations Pure

Derived stores must use pure expressions only:

```frel
// Good - pure computation
decl total = items.iter().map(|i| i.price).sum()

// Bad - side effect in expression (will not compile)
decl total = {
    log("Computing total");  // Side effect!
    items.iter().map(|i| i.price).sum()
}
```

### Extract Complex Logic

For complex computations, define helper functions in the host language:

```rust
fn calculate_discount(price: f64, quantity: u32) -> f64 {
    if quantity > 10 { price * 0.9 } else { price }
}
```

```frel
decl discounted_price = calculate_discount(price, quantity)
```

### Avoid Deep Nesting

Break complex expressions into multiple stores for clarity:

```frel
// Less clear
decl result = if items.iter().filter(|i| i.active).count() > 0 {
    items.iter().filter(|i| i.active).map(|i| i.value).sum()
} else {
    0
}

// Better
// TODO: List filter operation not yet specified
decl active_items = filter_active(items)
decl has_active = count_items(active_items) > 0
decl total = if has_active {
    sum_values(active_items)
} else {
    0
}
```