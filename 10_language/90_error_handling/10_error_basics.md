# Error Handling

**Errors do not exist at the Frel language level.**

Frel is a declarative UI language. There is no special error handling syntax - no try/catch blocks, no throw statements, no error propagation. Instead, errors are simply application state that gets rendered like any other data.

## Core Principle

Errors are just state:
- **Sources** have built-in error states (Status::Error)
- **Commands** set store values to indicate failures
- **Fragments** render state declaratively

## Sources with Error States

Sources represent async operations that can fail. Pattern match on their status:

```frel
backend UserProfile {
    source userData: User = fetch_user_data()
}

fragment UserView() {
    with UserProfile

    match userData.status {
        Status::Loading => text { "Loading..." }
        Status::Ready => UserCard { userData.value }
        Status::Error => text { "Could not load user" } .. color { Red }
    }
}
```

## Commands that Report Errors

Commands update stores to communicate failures:

```frel
backend DataEditor {
    writable saveStatus: String = ""
    command save(item: Item)
}
```

Host language implementation:

```rust
impl DataEditor {
    async fn save(&mut self, item: Item) {
        match api.save_item(&item).await {
            Ok(_) => self.save_status.set("Saved successfully".to_string()),
            Err(e) => self.save_status.set(format!("Save failed: {}", e)),
        }
    }
}
```

Fragment renders the state:

```frel
fragment Editor() {
    with DataEditor

    button { "Save" } .. on_click { save(currentItem) }

    when saveStatus.length > 0 {
        text { saveStatus }
            .. color { saveStatus.starts_with("Save failed") ? Red : Green }
    }
}
```

## What Frel Does NOT Have

- `try`/`catch` blocks
- `throw` statements
- `Error` types or exceptions
- `Result<T, E>` return types
- Error propagation operators

## Common Patterns

### State Machine with Error Variant

```frel
enum RequestStatus {
    Idle
    InProgress
    Success
    Failed(message: String)
}

backend ApiClient {
    writable status: RequestStatus = RequestStatus::Idle
    command fetch_data()
}

fragment ApiView() {
    with ApiClient

    match status {
        RequestStatus::Idle => button { "Fetch" } .. on_click { fetch_data() }
        RequestStatus::InProgress => text { "Loading..." }
        RequestStatus::Success => text { "Success!" } .. color { Green }
        RequestStatus::Failed(msg) => column {
            text { "Failed: ${msg}" } .. color { Red }
            button { "Retry" } .. on_click { fetch_data() }
        }
    }
}
```

### Validation Errors

```frel
backend FormValidator {
    writable email: String = ""
    writable errors: List<String> = []
    command validate()
}

fragment ValidationForm() {
    with FormValidator

    text_input { email }
    button { "Validate" } .. on_click { validate() }

    when errors.length > 0 {
        column {
            repeat on errors as error {
                text { error } .. color { Red }
            }
        }
    }
}
```

### Multiple Independent Sources

```frel
backend Dashboard {
    source user: User = fetch_user()
    source stats: Stats = fetch_stats()
}

fragment DashboardView() {
    with Dashboard

    column {
        match user.status {
            Status::Ready => UserCard { user.value }
            Status::Error => text { "Could not load user" } .. color { Red }
            Status::Loading => text { "Loading..." }
        }

        match stats.status {
            Status::Ready => StatsPanel { stats.value }
            Status::Error => text { "Could not load stats" } .. color { Red }
            Status::Loading => text { "Loading..." }
        }
    }
}
```

## Design Rationale

1. **Simplicity**: UI layer stays purely declarative
2. **Separation of Concerns**: Error recovery logic (retries, backoff) belongs in host language
3. **Consistency**: Everything is state - successful results and errors are treated uniformly
4. **Composability**: State flows naturally through the component tree
5. **Clarity**: What can fail (sources/commands) and how failures are represented is explicit

## Boundary Between Frel and Host Language

**Host language**: Rich error handling (Result, Option, try/catch, etc.)
**Frel boundary**: Convert errors to state (set store values)
**Frel UI**: Render state declaratively

```rust
impl MyBackend {
    async fn complex_operation(&mut self) -> Result<(), MyError> {
        // Traditional error handling
        let result = self.try_something().await?;

        // Convert to state for Frel
        match result {
            Ok(data) => self.data_store.set(Some(data)),
            Err(e) => self.error_store.set(Some(e.to_string())),
        }
        Ok(())
    }
}
```

## See Also

- [Sources](../20_reactive_state/50_sources.md) - Built-in error state tracking
- [Commands](../40_backends/10_backend_basics.md#commands) - Reporting errors from commands
- [Enums](../10_data_modeling/50_enums.md) - Modeling states with variants
- [Pattern Matching](../15_expressions/60_pattern_matching.md) - Matching on state variants
