# Event Handlers

Event handlers define reactive callbacks that execute in response to user interactions, system
events, or lifecycle changes. They are the primary mechanism for introducing side effects and
imperative logic into the otherwise declarative blueprint body.

## Syntax

Event handlers are instructions and follow the standard instruction syntax rules (inner or postfix).
See [Fragment Creation - Instructions](40_fragment_creation.md#4-instructions-inner-vs-postfix-syntax) for style guidelines on when to use each form.

```text
<event-handler> ::= <event-name> [ <param-list> "->" ] "{" <handler-body> "}"
<param-list> ::= <param-spec> | <param-spec> { "," <param-spec> }
<param-spec> ::= <param-name> [ ":" <param-type> ]
<handler-body> ::= <handler-statement>*
<handler-statement> ::= <store-assignment> | <command-call>
<store-assignment> ::= <store-name> "=" <frel-expr>
<command-call> ::= <command-name> "(" [ <frel-expr> { "," <frel-expr> } ] ")"
```

Event handlers contain a sequence of statements that perform side effects.

**Parameter syntax:**
- When a handler has no parameters, omit the parameter list entirely
- When a handler has one parameter and no name is specified, it defaults to `it`
- Type annotations are optional when the type can be inferred from the event definition

**Forms:**

```frel
// Parameter-less handler
button { "Click" } .. on_click {
    count = count + 1           // Store mutation
}

// Handler with event parameter (explicit name)
button { "Click" } .. on_click { event: PointerEvent ->
    log_event("Clicked at ${event.x_dip}, ${event.y_dip}")  // Command call
}

// Handler with event parameter (using default 'it')
button { "Click" } .. on_click {
    log_event("Clicked at ${it.x_dip}, ${it.y_dip}")  // 'it' is the PointerEvent
}

// Handler with inferred type
button { "Click" } .. on_click { event ->
    log_event("Clicked at ${event.x_dip}, ${event.y_dip}")  // Type inferred
}

// Multiple statements
button { "Reset" } .. on_click {
    count = 0                   // Store mutation
    status = "idle"             // Store mutation
    reset_data()                // Command call
}
```

## Semantics

### Handler Body

* The handler body contains a **sequence of statements** that perform side effects.
* Each statement is either:
  1. **Store assignment**: `count = count + 1` - assigns a pure Frel expression to a writable store
  2. **Command call**: `save()` - calls a backend command
* All expressions in event handlers are **pure Frel expressions** - same as elsewhere in the DSL.
* Event handlers are the **only** place in the Frel DSL where side effects (mutations, command calls) are allowed.

### Execution Context

* Handlers execute synchronously when the event fires.
* Store writes trigger reactive updates (drain cycle runs after handler completes).
* Multiple store writes within a handler are batched (subscribers notified once).
* Handlers can read stores and parameters in scope.

### Allowed Statements

Event handlers support exactly two types of statements:

#### 1. Store Assignments

```frel
// Direct assignment
count = count + 1
name = "Alice"
items = [1, 2, 3]

// Using expressions
total = price * quantity
isValid = name.length > 0 && email.includes("@")

// Note: collection mutations happen via reassignment
items = items.map(x => x * 2)  // Creates new list
```

The right-hand side must be a pure Frel expression. No direct mutation of collections.

#### 2. Command Calls

```frel
// No arguments
save()
reset()

// With arguments (all must be Frel expressions)
load_user(userId)
update_settings(theme, fontSize)

// Commands are async but appear synchronous in handlers
validate()  // Runs asynchronously
```

### What's NOT Allowed

Event handlers do **not** support:

- **Control flow statements**: No `if`, `for`, `while`, `match` inside handlers
- **Direct collection mutation**: No `.push()`, `.insert()`, `.remove()` on stores
- **Variable declarations**: No `let` or `const` - only store assignments
- **Host language syntax**: Handlers are pure Frel DSL

### Control Flow

If you need conditional logic or loops, use these patterns:

**Option 1: Use Frel control flow outside the handler**

```frel
// ❌ Don't do this - no if statements in handlers
button { "Click" } .. on_click {
    if (count > 10) {
        reset()
    } else {
        count = count + 1
    }
}

// ✅ Do this instead - split into multiple handlers
when count > 10 {
    button { "Click" } .. on_click { reset() }
}

when count <= 10 {
    button { "Click" } .. on_click { count = count + 1 }
}
```

**Option 2: Implement logic in backend command**

```frel
// Backend declaration
backend Counter {
    writable count: i32 = 0

    command increment_or_reset()
}

// Backend implementation (in host language)
impl Counter {
    async fn increment_or_reset(&mut self) {
        if self.count.get() > 10 {
            self.count.set(0);
        } else {
            self.count.set(self.count.get() + 1);
        }
    }
}

// Fragment - simple call
blueprint CounterView() {
    with Counter

    button { "Click" } .. on_click { increment_or_reset() }
}
```

### Event Parameters

* Some event handlers receive an event object with event-specific data.
* Parameter type must match the event's defined type.
* Parameter-less handlers omit the parameter clause entirely.

## Examples

### Simple State Update

```frel
blueprint Counter() {
    writable count = 0

    button {
        text { "Increment" }
        on_click { count = count + 1 }
    }
}
```

### Using Event Data

```frel
blueprint ColorPicker() {
    writable hue = 0.0

    box {
        width { 300 }
        height { 50 }
        on_click { event: PointerEvent ->
            hue = event.x_dip / 300.0
        }
    }
}

// Or using default 'it' parameter
blueprint ColorPicker() {
    writable hue = 0.0

    box {
        width { 300 }
        height { 50 }
        on_click {
            hue = it.x_dip / 300.0
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

### Calling Backend Commands

```frel
backend Analytics {
    command log_event(message: String)
    command track(event_name: String)
}

blueprint App() {
    with Analytics

    button { "Log" } .. on_click {
        log_event("Button clicked")
        track("button_click")
    }
}
```

### Keyboard Shortcuts

```frel
backend Editor {
    writable content: String = ""

    command submit()
    command cancel()
}

blueprint EditorView() {
    with Editor

    column {
        text_input { content }

        on_enter { submit() }
        on_escape { cancel() }
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

Side effects belong only in event handlers, not in store declarations or blueprint parameters:

```frel
// ✅ Good - side effects in event handler
backend Logger {
    command log(message: String)
}

blueprint Counter() {
    with Logger

    writable count = 0

    button { "Click" } .. on_click {
        log("clicked")        // Command call OK here
        count = count + 1     // Store mutation OK here
    }
}

// ❌ Bad - side effect in store declaration
decl count = log_and_return(0)  // COMPILE ERROR: command calls not allowed in expressions

// ✅ Good - pure expression in store declaration
decl count = 0                  // Pure Frel expression
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

Error handling is done in backend commands or through sources:

```frel
// Backend handles errors
backend DataLoader {
    writable items: List<Item> = []
    writable error: String? = null

    command fetch_data()
}

impl DataLoader {
    async fn fetch_data(&mut self) {
        match api.fetch_items().await {
            Ok(data) => {
                self.items.set(data);
                self.error.set(None);
            }
            Err(e) => {
                self.error.set(Some(e.to_string()));
            }
        }
    }
}

// Fragment just calls command
blueprint DataView() {
    with DataLoader

    button { "Fetch" } .. on_click { fetch_data() }

    when error.is_some() {
        text { "Error: ${error.unwrap()}" } .. color { Red }
    }
}
```

## See Also

- [Instructions](60_instructions.md) - Event handler syntax and available events
- [Store Declarations](../20_reactive_state/10_store_basics.md) - Store mutations in handlers
- [Detached UI](80_detached_ui.md) - Using handlers with modals and toasts
