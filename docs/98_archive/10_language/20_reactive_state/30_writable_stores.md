# Writable Stores

Writable stores hold mutable state that can be updated through explicit assignments in event handlers. Unlike read-only stores, they don't automatically recompute when dependencies change - they only update through direct assignment.

## Syntax

`[<lifetime>] writable <id> [: <key_expr>]? [:<type>]? = <expr>`

## Semantics

- **Kind**: Writable stores hold mutable state with no automatic subscriptions to other stores.
- **Initializer**: `<expr>` is evaluated once at store creation. Even if it mentions other stores, there's no ongoing subscription. Must be a pure Frel expression. See [Frel Expressions](../15_expressions/10_expression_basics.md).
- **Writes**: Writable stores can be modified in event handlers through:
  - Direct assignment: `<id> = <expr2>` (must be a pure Frel expression)
- **Updates**: Only through explicit writes (no automatic recomputation from dependencies).
- **Reactivity**: When the value changes, dependent stores are notified and recompute. See [Mutation Detection](10_store_basics.md#mutation-detection) for details on how the runtime detects changes.
- **Persistence**: Implemented by the adapter of the host platform.

`<lifetime>` is one of:
- (omitted) — fragment-scoped (default)
- `session` — survives fragment destruction, cleared on app restart
- `persistent` — survives app restart

`<key_expr>` (Key Expression) is:
- Required for `session` and `persistent` stores
- String expression (literal or interpolated) that uniquely identifies the store
- Must be a pure Frel expression, evaluated once at store creation time

**Shorthands:**

| Shorthand                             | Full form                                    |
|---------------------------------------|----------------------------------------------|
| `writable <id> = <expr>`              | `writable <id> = <expr>` (fragment-scoped)   |
| `session <id>: <key> = <expr>`        | `session writable <id>: <key> = <expr>`      |
| `persistent <id>: <key> = <expr>`     | `persistent writable <id>: <key> = <expr>`   |

### Lifetime scopes

| Lifetime     | Survives                   | Storage           | Key required |
|--------------|----------------------------|-------------------|--------------|
| (default)    | Fragment instance only     | Memory            | No           |
| `session`    | Fragment destroy           | Session registry  | Yes          |
| `persistent` | App restart                | Platform storage  | Yes          |

### Key expressions

**Requirements:**
- Must be unique across all stores with the same lifetime in the app
- For `persistent` stores, keys must be stable across app restarts
- Use stable identifiers (user IDs, document IDs) not transient values (indices, positions)

**Key Collision Behavior:**

Key uniqueness is the developer's responsibility. The system cannot enforce uniqueness at compile-time, especially with interpolated keys.

If two stores use the same key:
- **Session stores:** Last-writer-wins - the most recently created store's value prevails
- **Persistent stores:** Last-writer-wins - the most recently written value is persisted

**Example of collision:**
```frel
blueprint UserTable(scope: String) {
    session filter: "${scope}.filter" = ""
}

// Both instances use the same key "users.filter"
UserTable(scope: "users")
UserTable(scope: "users")  // Collision! They share the same session state
```

To avoid collisions, ensure keys are unique by including instance-specific identifiers.

**Examples:**
```frel
session writable split_pos: "app.split" = 300
session writable filter: "UserTable.${table_id}.filter" = ""
persistent writable theme: "app.theme" = "dark"
persistent writable settings: "user.${user_id}.settings" = default_settings()
```

### Type constraints

- Default and `session` lifetime: any type
- `persistent` lifetime: type must be serializable (implement appropriate traits for the host platform)

### Reusable blueprints pattern

When the same blueprint is instantiated multiple times, pass a scope parameter:

```frel
blueprint UserTable(users: List<User>, scope: String) {
    session writable filter: "${scope}.filter" = ""
    session writable sort_column: "${scope}.sort" = "name"
    // ...
}

blueprint DocumentEditor(doc: Document, scope: String) {
    persistent writable font_size: "${scope}.fontSize" = 14
    persistent writable zoom: "${scope}.zoom" = 1.0
    // ...
}

blueprint App() {
    column {
        UserTable(active_users, scope: "active")
        UserTable(archived_users, scope: "archived")
    }
}
```

## Examples

### Fragment-Scoped Writable

Basic mutable state that lives as long as the fragment:

```frel
blueprint Counter() {
    writable count = 0

    column {
        text { "Count: ${count}" }

        row {
            button { "+" } .. on_click { count = count + 1 }
            button { "-" } .. on_click { count = count - 1 }
            button { "Reset" } .. on_click { count = 0 }
        }
    }
}
```

### Form State

Managing multiple writable stores for form inputs:

```frel
blueprint LoginForm() {
    writable username = ""
    writable password = ""
    writable remember_me = false

    // TODO: String validation operations not yet fully specified
    decl is_valid = username != "" && password != ""

    column {
        gap { 12 }

        text_input { username } .. placeholder { "Username" }
        password_input { password } .. placeholder { "Password" }

        row {
            checkbox { remember_me }
            text { "Remember me" }
        }

        button { "Log in" }
            .. enabled { is_valid }
            .. on_click {
                login(username, password, remember_me)
            }
    }
}
```

### Toggle State

Simple boolean toggles:

```frel
blueprint CollapsibleSection(title: String) {
    writable is_expanded = false

    column {
        row {
            text { title }
            button { if is_expanded { "▼" } else { "▶" } }
                .. on_click { is_expanded = !is_expanded }
        }

        when is_expanded {
            box {
                padding { 16 }
                text { "Section content here..." }
            }
        }
    }
}
```

### Session Writable

State that survives fragment recreation but not app restart:

```frel
blueprint SplitView() {
    session split_position: "app.split" = 300

    row {
        box {
            width { split_position }
            background { color: LightGray }
            text { "Sidebar" }
        }

        // Draggable divider
        box {
            width { 4 }
            background { color: Gray }
            cursor { resize_horizontal }

            on_drag { event: DragEvent ->
                split_position = split_position + event.delta_x
            }
        }

        box {
            width { expand }
            text { "Main content" }
        }
    }
}
```

### Session with Scoping

Using session stores in reusable blueprints:

```frel
blueprint DataTable(data: List<Row>, scope: String) {
    session sort_column: "${scope}.sort" = "name"
    session sort_direction: "${scope}.direction" = "asc"
    session page: "${scope}.page" = 0
    session page_size: "${scope}.pageSize" = 25

    decl sorted_data = sort_data(data, sort_column, sort_direction)
    decl page_data = paginate(sorted_data, page, page_size)

    column {
        // Table headers with sorting
        row {
            repeat on ["name", "age", "email"] as col {
                button { col }
                    .. on_click {
                        // Event handlers cannot contain control flow
                        // Use expressions instead
                        sort_direction = if sort_column == col {
                            if sort_direction == "asc" { "desc" } else { "asc" }
                        } else {
                            "asc"
                        }
                        sort_column = col
                    }
            }
        }

        // Table rows
        repeat on page_data as row {
            DataRow(row)
        }

        // Pagination
        row {
            button { "Previous" }
                .. enabled { page > 0 }
                .. on_click { page = page - 1 }

            text { "Page ${page + 1}" }

            button { "Next" }
                .. on_click { page = page + 1 }
        }
    }
}

// Usage - each instance maintains separate state
blueprint App() {
    column {
        DataTable(users, scope: "users")
        DataTable(orders, scope: "orders")
    }
}
```

### Persistent Writable

State that survives app restart:

```frel
blueprint Settings() {
    persistent theme: "app.theme" = "light"
    persistent font_size: "app.fontSize" = 14
    persistent auto_save: "app.autoSave" = true

    column {
        gap { 16 }

        row {
            text { "Theme:" }
            button { "Light" }
                .. on_click { theme = "light" }
            button { "Dark" }
                .. on_click { theme = "dark" }
        }

        row {
            text { "Font Size:" }
            button { "-" }
                .. on_click { font_size = (font_size - 1).max(10) }
            text { "${font_size}" }
            button { "+" }
                .. on_click { font_size = (font_size + 1).min(24) }
        }

        row {
            checkbox { auto_save }
            text { "Auto-save" }
                .. on_click { auto_save = !auto_save }
        }
    }
}
```

### Persistent with User Scope

Separate persistent state per user:

```frel
blueprint UserPreferences(user_id: u32) {
    persistent notifications: "user.${user_id}.notifications" = true
    persistent language: "user.${user_id}.language" = "en"
    persistent recent_files: "user.${user_id}.recentFiles" = []

    column {
        row {
            checkbox { notifications }
            text { "Enable notifications" }
                .. on_click { notifications = !notifications }
        }

        row {
            text { "Language:" }
            select { language }
                .. options { ["en", "es", "fr", "de"] }
                .. on_change { lang: String -> language = lang }
        }

        column {
            text { "Recent files:" }
            repeat on recent_files as file {
                text { file }
            }
        }
    }
}
```

### Complex State Updates

```frel
blueprint ShoppingCart() {
    writable items: List<CartItem> = []
    writable coupon_code = ""

    // TODO: List operations for sum/reduce not yet specified
    decl subtotal = calculate_subtotal(items)
    decl item_count = calculate_item_count(items)

    column {
        text { "${item_count} items - $${subtotal:.2}" }

        repeat on items as item {
            row {
                text { item.name }
                text { "${item.quantity} × $${item.price}" }

                button { "+" }
                    .. on_click {
                        // TODO: List update operations not yet specified
                        // Need to define how to update items in a list
                        increment_item_quantity(item.id)
                    }

                button { "Remove" }
                    .. on_click {
                        // TODO: List filter operations not yet specified
                        remove_item(item.id)
                    }
            }
        }

        row {
            text_input { coupon_code }
                .. placeholder { "Coupon code" }

            button { "Apply" }
                .. on_click {
                    // Event handlers cannot contain control flow
                    // Move validation logic to backend command
                    apply_coupon(coupon_code)
                }
        }
    }
}
```

## Initialization from Parameters

Writable stores can be initialized from blueprint parameters, but won't track changes:

```frel
blueprint EditableText(initial: String) {
    writable text = initial  // Initialized once, no subscription to 'initial'

    text_input { text }
        .. on_change { new_text: String -> text = new_text }
}

// When used:
blueprint Parent() {
    writable source = "Hello"

    EditableText(source)  // EditableText gets "Hello" initially
                          // But won't update if 'source' changes later
}
```

## Best Practices

### Initialize Carefully

The initializer runs only once at creation:

```frel
// Good - simple initialization
writable count = 0

// Good - initialization from parameter
writable text = initial_value

// Beware - no ongoing subscription
writable cached = expensive_computation()  // Only computed once
```

### Use Derived Stores for Computed Values

Don't manually sync writable stores - use derived stores:

```frel
// Bad - manual sync prone to bugs
writable count = 0
writable doubled = 0

button { "+" } .. on_click {
    count = count + 1
    doubled = count * 2  // Easy to forget!
}

// Good - automatic sync
writable count = 0
decl doubled = count * 2  // Always correct

button { "+" } .. on_click {
    count = count + 1     // doubled updates automatically
}
```

### Choose the Right Lifetime

| Lifetime      | Use When                                      |
|---------------|-----------------------------------------------|
| (default)     | State belongs to one fragment instance        |
| `session`     | State should persist across navigation        |
| `persistent`  | State should survive app restart              |

### Batch Related Updates

Multiple assignments in one event handler are efficient:

```frel
button { "Reset All" } .. on_click {
    username = ""
    password = ""
    remember_me = false
    // All dependent stores update once after this handler completes
}
```