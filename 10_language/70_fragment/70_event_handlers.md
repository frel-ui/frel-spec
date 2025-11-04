# Event Handlers

Event handlers define reactive callbacks that execute in response to user interactions, system
events, or lifecycle changes. They are the primary mechanism for introducing side effects and
imperative logic into the otherwise declarative fragment body.

## Syntax

Event handlers are instructions and follow the standard instruction syntax rules (inner or postfix).
See [Fragment Creation - Instructions](40_fragment_creation.md#4-instructions-inner-vs-postfix-syntax) for style guidelines on when to use each form.

```text
<event-handler> ::= <event-name> [ <parameter-clause> ] "{" <handler-body> "}"
<parameter-clause> ::= "|" <param-name> ":" <param-type> "|"
<handler-body> ::= <hls-statement>*
```

Where `<hls-statement>` is a Host Language Statement (HLS) from the host language's syntax.

**Forms:**

```frel
// Parameter-less handler
button { "Click" } .. on_click {
    count = count + 1           // HLS: assignment statement
}

// Handler with event parameter
button { "Click" } .. on_click |event: PointerEvent| {
    log("Clicked at ${event.x_dip}, ${event.y_dip}")  // HLS: function call
}

// Multiple statements
button { "Reset" } .. on_click {
    count = 0                   // HLS: assignment
    status = "idle"             // HLS: assignment
    log("Reset complete")       // HLS: side effect
}
```

## Semantics

### Handler Body

* The handler body contains **Host Language Statements (HLS)**, which **may have side effects**.
* Unlike Pure Host Language Expressions (PHLE) used in the DSL body, event handlers are explicitly designed for:
  - Store mutations (assignments)
  - Function calls with side effects
  - Control flow (if, match, loops in the host language)
  - Async operations (depending on host language support)
  - I/O operations
* Event handlers are the **only** place in the DSL where HLS (and side effects) are allowed.

### Execution Context

* Handlers execute synchronously when the event fires.
* Store writes trigger reactive updates (drain cycle runs after handler completes).
* Multiple store writes within a handler are batched (subscribers notified once).
* Handlers can read stores and parameters in scope.

### Statements vs Expressions

* Event handlers contain **statements** (HLS), which may include:
  - Assignments: `store = expr`
  - Function calls: `do_something()`
  - Control flow: `if`, `match`, `for`, `while` (host language dependent)
  - Return statements (if supported by host language)
* Store assignments must use **PHLE on the right-hand side**:
  - `count = count + 1` ✓ (PHLE on right side)
  - `items = items.filter(|x| x > 0)` ✓ (PHLE on right side)
  - `status = log_and_set("active")` ✗ (side effect in expression)

### Event Parameters

* Some event handlers receive an event object with event-specific data.
* Parameter type must match the event's defined type.
* Parameter-less handlers omit the parameter clause entirely.

## Examples

### Simple State Update

```frel
fragment Counter() {
    writable count = 0

    button {
        text { "Increment" }
        on_click { count = count + 1 }
    }
}
```

### Using Event Data

```frel
fragment ColorPicker() {
    writable hue = 0.0

    box {
        width { 300 }
        height { 50 }
        on_click |event: PointerEvent| {
            hue = event.x_dip / 300.0
        }
    }
}
```

### Multiple Store Updates (Batched)

```frel
button { "Reset" } .. on_click {
    count = 0
    status = "idle"
    timestamp = now()
    // All subscribers notified once after handler completes
}
```

### Calling Host Functions

```frel
button { "Log" } .. on_click {
    log("Button clicked at ${timestamp()}")
    analytics.track("button_click")
}
```

### Keyboard Shortcuts

```frel
fragment Editor() {
    writable content = ""

    column {
        text_input { content }

        on_enter { submit() }
        on_escape { cancel() }
    }
}
```

### Conditional Logic

```frel
button { "Submit" } .. on_click {
    if valid(input) {
        submit(input)
        success { "Submitted!" }
    } else {
        error { "Invalid input" }
    }
}
```

## Best Practices

### Keep Handlers Focused

Event handlers should be concise and focused. Extract complex logic into host language functions:

```frel
// Good
button { "Process" } .. on_click {
    process_data(items)
}

// Avoid
button { "Process" } .. on_click {
    // 50 lines of complex logic...
}
```

### Avoid Side Effects Outside Event Handlers

Side effects (HLS) belong only in event handlers, not in store declarations or fragment parameters (which require PHLE):

```frel
// Good - side effects in event handler (HLS allowed)
writable count = 0
button { "Click" } .. on_click {
    log("clicked")              // HLS: side effect OK here
    count = count + 1           // HLS: assignment OK here
}

// Bad - side effect in store declaration (PHLE required)
decl count = log_and_return(0)  // COMPILE ERROR: side effect not allowed in PHLE

// Good - pure expression in store declaration
decl count = 0                  // PHLE: pure expression
```

### Batch Updates

Group related store updates in a single handler for efficiency:

```frel
// Good - batched
on_click {
    x = new_x
    y = new_y
    z = new_z
}

// Inefficient - separate handlers
on_click { x = new_x }
on_click { y = new_y }
on_click { z = new_z }
```

### Error Handling

Handle errors within event handlers to prevent crashes:

```frel
button { "Fetch" } .. on_click {
    match fetch_data() {
        Ok(data) => items = data,
        Err(e) => error { "Failed: ${e}" }
    }
}
```

## See Also

- [Instructions](60_instructions.md) - Event handler syntax and available events
- [Store Declarations](../20_reactive_state/10_store_basics.md) - Store mutations in handlers
- [Detached UI](80_detached_ui.md) - Using handlers with modals and toasts
