# Enums

Enums define a fixed set of named variants, providing type-safe alternatives to string constants
or numeric codes. They are commonly used for state machines, status values, and categorical data.

## Syntax

```
enum <Name> { <variant1> <variant2> <variant3> ... }
```

## Semantics

- **Variants**: Space-separated identifiers representing the possible values
- **Scope**: Top-level declarations, available throughout the module
- **Naming**: Enum names use PascalCase, variants use snake_case
- **Ordering**: Variants maintain declaration order
- **Usage**: Can be used as types in schemes, blueprints, and function signatures

## Basic Examples

### Status Enums

```frel
enum TaskStatus { pending in_progress completed cancelled }

enum Priority { low medium high urgent }

enum Visibility { public private unlisted }
```

### Usage in Schemes

```frel
enum ArticleStatus { draft published archived }

scheme Article {
    title .. String .. blank { false }
    status .. ArticleStatus .. default { ArticleStatus::draft }
    created_at .. Instant .. default { Instant::now() }
}
```

### Usage in Blueprints

```frel
enum ViewMode { list grid table }

blueprint ContentView() {
    writable mode = ViewMode::list

    row {
        button { "List" }
            .. on_click { mode = ViewMode::list }
            .. active { mode == ViewMode::list }

        button { "Grid" }
            .. on_click { mode = ViewMode::grid }
            .. active { mode == ViewMode::grid }

        button { "Table" }
            .. on_click { mode = ViewMode::table }
            .. active { mode == ViewMode::table }
    }

    select on mode {
        ViewMode::list => ListView()
        ViewMode::grid => GridView()
        ViewMode::table => TableView()
    }
}
```

## Inline Enums

For single-use enums, you can define them inline within scheme fields:

```frel
scheme Meeting {
    title .. String
    frequency { enum { once daily weekly monthly } }
        .. default { frequency::once }
}
```

**Note:** Inline enums are scoped to the field and accessed via `field_name::variant`.

## Auto-Generated Methods

Enums automatically provide several methods:

### Variant Access

```frel
enum Status { draft published archived }

// Variant constructors (singleton values)
let s = Status::draft
```

### String Conversion

```frel
// Convert to string
status.to_string()  // "draft"

// Parse from string
Status::from_string("draft")      // Ok(Status::draft)
Status::from_string("invalid")    // Err(ParseError)
```

### Comparison

```frel
if status == Status::published {
    show_badge()
}

// Pattern matching
select on status {
    Status::draft => text { "Draft" }
    Status::published => text { "Published" }
    Status::archived => text { "Archived" }
}
```

### Enumeration

```frel
// Get all variants
Status::all()  // [Status::draft, Status::published, Status::archived]

// Iterate over variants
repeat on Status::all() as variant {
    text { variant.to_string() }
}
```

## Validation Instructions

Enums can have validation rules when used in schemes:

### Default Values

```frel
scheme Task {
    status .. TaskStatus .. default { TaskStatus::pending }
    priority .. Priority .. default { Priority::medium }
}
```

### State Transitions

Define valid state transitions to enforce state machine logic:

```frel
enum OrderStatus {
    pending
    confirmed
    shipped
    delivered
    cancelled
}

scheme Order {
    status .. OrderStatus
        .. default { OrderStatus::pending }
        .. transitions {
            pending => [confirmed cancelled]
            confirmed => [shipped cancelled]
            shipped => [delivered]
            delivered => []
            cancelled => []
        }
}
```

**Usage:**
```frel
writable order = Order()

// Check if transition is allowed
if order.status.can_transition_to(OrderStatus::shipped) {
    order.status = OrderStatus::shipped
}

// Get valid next states
let next_states = order.status.valid_transitions()
// Returns: [OrderStatus::shipped, OrderStatus::cancelled]
```

## UI Integration

### Automatic Rendering

When bound to form inputs, enums automatically render as appropriate widgets:

```frel
scheme Settings {
    theme { enum { light dark auto } }
    language { enum { en es fr de } }
}

blueprint SettingsForm() {
    writable settings = Settings()

    column {
        // Automatically renders as dropdown/select
        enum_select { settings.theme }

        // Automatically renders as radio buttons (for small sets)
        enum_radio { settings.language }
    }
}
```

### Custom Rendering

Override default rendering with custom UI:

```frel
enum Theme { light dark auto }

blueprint ThemeSelector(current: Theme) {
    row {
        repeat on Theme::all() as theme {
            button { theme.to_string() }
                .. on_click { current = theme }
                .. active { current == theme }
        }
    }
}
```

## Localization

Enum variant labels can be localized:

```frel
enum Priority { low medium high urgent }

// In en.yml
priority:
  low: "Low Priority"
  medium: "Medium Priority"
  high: "High Priority"
  urgent: "Urgent!"

// In es.yml
priority:
  low: "Prioridad Baja"
  medium: "Prioridad Media"
  high: "Prioridad Alta"
  urgent: "¡Urgente!"
```

**Usage:**
```frel
blueprint PriorityBadge(priority: Priority) {
    text { i18n("priority.${priority.to_string()}") }
}
```

## Best Practices

### Naming Conventions

```frel
// Good - descriptive, lowercase variants
enum Status { pending active completed }

// Good - clear semantic meaning
enum AccessLevel { read_only read_write admin }

// Avoid - unclear abbreviations
enum Status { pnd act cmp }
```

### When to Use Enums

**Use enums for:**
- Finite, predefined sets of values
- State machines with defined transitions
- Categorical data with type safety
- Configuration options

**Don't use enums for:**
- Open-ended sets (use String instead)
- Large sets (> 20 variants, consider database lookup)
- Values that change frequently

### Enum vs String

```frel
// Prefer enum for fixed sets
enum UserRole { guest user moderator admin }

// Use String for open-ended content
scheme Post {
    title .. String  // Not enum - unbounded
    author .. String  // Not enum - many users
    status .. PostStatus  // Enum - fixed states
}
```

## Advanced Examples

### Multi-Stage State Machine

```frel
enum TicketStatus {
    new
    triaged
    assigned
    in_progress
    code_review
    qa_testing
    ready_for_deploy
    deployed
    closed
}

scheme Ticket {
    id .. u64 .. readonly { true }
    title .. String .. blank { false }
    status .. TicketStatus
        .. default { TicketStatus::new }
        .. transitions {
            new => [triaged closed]
            triaged => [assigned closed]
            assigned => [in_progress closed]
            in_progress => [code_review closed]
            code_review => [in_progress qa_testing closed]
            qa_testing => [code_review ready_for_deploy closed]
            ready_for_deploy => [deployed closed]
            deployed => [closed]
            closed => []
        }
}
```

### Enum with Associated Metadata

```frel
enum LogLevel { trace debug info warn error }

// In host language, extend with methods
impl LogLevel {
    fn color(&self) -> Color {
        match self {
            LogLevel::trace => Gray,
            LogLevel::debug => Blue,
            LogLevel::info => Green,
            LogLevel::warn => Orange,
            LogLevel::error => Red,
        }
    }

    fn icon(&self) -> &str {
        match self {
            LogLevel::trace => "•",
            LogLevel::debug => "◆",
            LogLevel::info => "ℹ",
            LogLevel::warn => "⚠",
            LogLevel::error => "✗",
        }
    }
}
```

**Usage:**
```frel
blueprint LogEntry(level: LogLevel, message: String) {
    row {
        text { level.icon() }
            .. font { color: level.color() }
        text { message }
    }
}
```

## Type System Integration

Enums integrate with the host language type system:

```frel
enum Result { success failure }

// Used in function signatures
fn process_item(item: Item) -> Result {
    if item.is_valid() {
        Result::success
    } else {
        Result::failure
    }
}

// Used in conditionals
if process_item(item) == Result::success {
    show_success_message()
}
```

## Serialization

Enums serialize to strings by default:

```frel
enum Status { draft published }

scheme Article {
    status .. Status
}

// JSON serialization
{
    "status": "draft"
}

// Deserialization
Article::from_json(json)  // Parses "draft" → Status::draft
```
