# Detached UI

>> TODO review the examples in this file, they are not correct

Detached UI elements are special constructs for creating UI that renders outside the main 
fragment tree in separate channels.

Unlike normal fragments that are children of their parent, detached UI elements:

- Render in separate channels (`modal`, `snack`, `tooltip`)
- Have independent lifecycles from their creation point
- Are managed automatically by the runtime
- Don't affect the layout of their parent
- Are automatically positioned in the viewport/scene

## Overview

| Element    | Form              | Channel   | Lifecycle          |
|------------|-------------------|-----------|--------------------|
| `dialog!`  | Macro             | `modal`   | Until dismissed    |
| `snack!`   | Macro             | `snack`   | Auto-timeout       |
| `tooltip`  | Slot (not macro)  | `tooltip` | Auto-hide on leave |

## Dialog

`dialog! { <body> }`
`dialog! { runtime: <runtime-handle> <body> }`

Creates a modal dialog. Available within DSL event handlers or from native Rust code.

### Behavior

- **Non-blocking**: Execution continues immediately after dialog creation
- **Channel**: Automatically renders in `modal` channel
- **Container**: Rendered within a box sized to the full scene/viewport
- **Stacking**: Multiple dialogs stack (not queued) - all remain open simultaneously
- **Positioning**: Can be positioned and sized using standard layout instructions
- **Lifecycle**: Persists until explicitly dismissed or parent scope is destroyed

### Scope Access

**Within DSL event handlers:**

The dialog body has full access to the parent scope, including all stores. Use writable stores to handle dialog results.

**From native Rust:**

The dialog **does not** capture scope. All data must be moved into the dialog's ownership. Access to application state is done via stores through the runtime interface.

### Dismissal

A dialog can be dismissed in three ways:

1. **Explicit dismiss**: Call `dismiss()` function within the dialog
2. **Stereotype events**: `on_save` and `on_cancel` automatically dismiss after handler executes
3. **Scope destruction**: When the fragment scope that created the dialog is destroyed, the dialog is automatically dismissed

This automatic cleanup prevents orphaned dialogs when the data they refer to is removed (e.g., deleting a row from a table while its confirmation dialog is open).

### Syntax

**Within DSL event handlers:**

```dsl
on_click {
    dialog! {
        <dialog-body>
    }
    // Execution continues here immediately
}
```

**From native Rust:**

```rust
fn show_dialog(rt: &Runtime) {
    dialog! {
        runtime: rt

        <dialog-body>
    }
}
```

The runtime parameter is **optional**:
- In DSL event handlers: omit it (runtime context is implicit)
- In native Rust: provide it with `runtime: <handle>`

### Examples

**Basic confirmation dialog (DSL):**

```dsl
fragment! {
    TodoItem(item: &TodoData) {
        writable delete_confirmed = false

        row {
            text { item.title }

            button { "Delete" } .. on_click {
                dialog! {
                    column {
                        padding { 16 }
                        gap { 16 }

                        text { "Delete '${item.title}'?" }

                        row {
                            gap { 8 }
                            button { "Cancel" } .. stereotype { cancel }
                            button { "Delete" } .. stereotype { save }
                        }
                    }

                    .. align_self_center
                    .. background { color: White }
                    .. border { color: Gray 1 }
                    .. corner_radius { 8 }
                    .. shadow { color: rgba(0, 0, 0, 64) offset_x: 0 offset_y: 4 blur: 8 }

                    on_save {
                        delete_confirmed = true
                        // Dialog auto-dismisses after this handler
                    }
                }

                // Continues immediately - dialog is shown but non-blocking
            }

            when delete_confirmed {
                // Handle deletion
                emit_delete_event(item.id)
                delete_confirmed = false
            }
        }
    }
}
```

**Dialog with explicit dismiss (DSL):**

```dsl
button { "Show Info" } .. on_click {
    dialog! {
        column {
            padding { 16 }

            text { "Information" } .. font { weight: 700 size: 18 }
            text { "Some detailed information here..." }

            button { "Close" } .. on_click {
                dismiss()  // Explicit dismissal
            }
        }

        .. align_self_center
        .. background { color: White }
        .. corner_radius { 8 }
    }
}
```

**Dialog with form data (DSL):**

```dsl
fragment! {
    UserList() {
        writable new_user_name = ""
        writable show_form = false

        button { "Add User" } .. on_click {
            show_form = true

            dialog! {
                column {
                    padding { 16 }
                    gap { 12 }

                    text { "New User" } .. font { weight: 700 }

                    input {
                        value: new_user_name
                        placeholder: "Enter name"
                    } .. on_input |event| {
                        new_user_name = event.character
                    }

                    row {
                        gap { 8 }
                        button { "Cancel" } .. stereotype { cancel }
                        button { "Add" } .. stereotype { save }
                    }
                }

                .. align_self_center
                .. background { color: White }
                .. corner_radius { 8 }

                on_cancel {
                    show_form = false
                    new_user_name = ""
                }

                on_save {
                    add_user(new_user_name)
                    show_form = false
                    new_user_name = ""
                }
            }
        }
    }
}
```

**Dialog from native Rust:**

```rust
pub fn show_error_dialog(rt: &Runtime, error_msg: String) {
    dialog! {
        runtime: rt

        column {
            padding { 16 }
            gap { 12 }

            text { "Error" } .. font { weight: 700 size: 18 color: Red }
            text { error_msg }

            button { "OK" } .. on_click {
                dismiss()
            }
        }

        .. align_self_center
        .. width { 400 }
        .. background { color: White }
        .. border { color: Red 2 }
        .. corner_radius { 8 }
    }
}
```

**Dialog from native Rust with store access:**

```rust
pub fn show_confirmation(rt: &Runtime, message: String, item_id: u64) {
    // Access writable store through runtime
    let confirmed_store = rt.create_writable(false);

    dialog! {
        runtime: rt

        column {
            padding { 16 }
            gap { 16 }

            text { message }

            row {
                gap { 8 }
                button { "Cancel" } .. stereotype { cancel }
                button { "Confirm" } .. stereotype { save }
            }
        }

        .. align_self_center
        .. background { color: White }

        on_save {
            // Write to store - caller can observe this
            rt.write_store(confirmed_store, true)
            dismiss()
        }
    }
}
```

### Positioning and Styling

Dialogs are rendered in a box sized to the full scene. Use standard layout and decoration instructions:

```dsl
dialog! {
    column { /* content */ }

    // Positioning
    .. align_self_center                    // Center in viewport
    .. align_self_end                       // Right side
    .. position { top: 100 left: 100 }      // Absolute position

    // Sizing
    .. width { 400 }                        // Fixed width
    .. height { content }                   // Height from content
    .. width { content min: 300 max: 600 }  // Constrained

    // Styling
    .. background { color: White }
    .. border { color: Gray 1 }
    .. corner_radius { 8 }
    .. shadow { color: rgba(0, 0, 0, 64) offset_x: 0 offset_y: 4 blur: 8 }
}
```

### Multiple Dialogs

Multiple dialogs can be open simultaneously. They stack in the order they were created:

```dsl
button { "Open Two Dialogs" } .. on_click {
    dialog! {
        text { "First dialog" }
        .. align_self_center
        .. background { color: White }
    }

    dialog! {
        text { "Second dialog" }
        .. align_self_center
        .. background { color: White }
    }

    // Both dialogs are now visible, stacked
}
```

### Automatic Cleanup

When a dialog's parent scope is destroyed, the dialog is automatically dismissed:

```dsl
fragment! {
    UserTable() {
        writable users = fetch_users()

        repeat on users by user.id as user {
            row {
                text { user.name }

                button { "Delete" } .. on_click {
                    dialog! {
                        text { "Delete ${user.name}?" }
                        button { "Confirm" } .. stereotype { save }

                        on_save {
                            users.remove(user)
                        }
                    }
                }
            }
        }

        // If `users` is updated and this user is removed:
        // 1. The row's scope is destroyed (no longer in repeat)
        // 2. The dialog is automatically dismissed
        // This prevents orphaned dialogs for deleted data
    }
}
```

## Snack

`snack! { <body> }`
`snack! { runtime: <runtime-handle> <body> }`

Creates a temporary notification message. Available within DSL event handlers or from native Rust code.

### Behavior

- **Non-blocking**: Execution continues immediately
- **Channel**: Automatically renders in `snack` channel
- **Stacking**: Multiple snacks stack and queue
- **Auto-dismiss**: Automatically dismissed after timeout (default: 3000ms)
- **Default positioning**: Bottom-center of viewport

### Dismissal

A snack can be dismissed in two ways:

1. **Auto-timeout**: Automatically dismissed after duration (default 3000ms)
2. **Explicit dismiss**: Call `dismiss()` function within the snack

### Syntax

**Within DSL event handlers:**

```dsl
on_click {
    snack! {
        <snack-body>
    }
    // Execution continues here immediately
}
```

**From native Rust:**

```rust
fn show_snack(rt: &Runtime) {
    snack! {
        runtime: rt

        <snack-body>
    }
}
```

The runtime parameter is **optional**:
- In DSL event handlers: omit it (runtime context is implicit)
- In native Rust: provide it with `runtime: <handle>`

### Examples

**Simple notification (DSL):**

```dsl
button { "Save" } .. on_click {
    save_data()

    snack! {
        text { "Saved successfully!" }
        .. padding { 12 }
        .. background { color: Green }
        .. corner_radius { 4 }
    }
}
```

**Notification with icon (DSL):**

```dsl
button { "Copy" } .. on_click {
    copy_to_clipboard()

    snack! {
        row {
            padding { 12 }
            gap { 8 }

            icon { "check" } .. color { White }
            text { "Copied to clipboard" } .. font { color: White }
        }

        .. background { color: rgba(0, 0, 0, 200) }
        .. corner_radius { 4 }
    }
}
```

**Error notification (DSL):**

```dsl
button { "Submit" } .. on_click {
    when validation_failed {
        snack! {
            row {
                padding { 12 }
                gap { 8 }

                icon { "error" } .. color { White }
                text { "Please fill all required fields" } .. font { color: White }
            }

            .. background { color: Red }
            .. corner_radius { 4 }
        }
    }
}
```

**Snack with dismiss button (DSL):**

```dsl
button { "Start Process" } .. on_click {
    start_long_process()

    snack! {
        row {
            padding { 12 }
            gap { 8 }

            text { "Processing..." }
            button { "×" } .. on_click {
                dismiss()  // Explicit dismissal
            }
        }

        .. background { color: Blue }
        .. corner_radius { 4 }
    }
}
```

**Snack from native Rust:**

```rust
pub fn notify_save_success(rt: &Runtime) {
    snack! {
        runtime: rt

        row {
            padding { 12 }
            gap { 8 }

            icon { "check" } .. color { White }
            text { "Changes saved" } .. font { color: White }
        }

        .. background { color: Green }
        .. corner_radius { 4 }
    }
}

pub fn notify_error(rt: &Runtime, error_message: String) {
    snack! {
        runtime: rt

        row {
            padding { 12 }
            gap { 8 }

            icon { "error" } .. color { White }
            text { error_message } .. font { color: White }
        }

        .. background { color: Red }
        .. corner_radius { 4 }
    }
}
```

### Stacking and Queueing

Multiple snacks stack vertically and queue if too many are shown:

```dsl
button { "Multiple Notifications" } .. on_click {
    snack! { text { "First" } }
    snack! { text { "Second" } }
    snack! { text { "Third" } }

    // All three will be shown, stacked vertically
    // Each auto-dismisses after 3 seconds
}
```

### Future Customization

**Note**: These instructions are planned but not yet implemented. For POC, use defaults.

```dsl
// Planned for future versions:

snack! {
    text { "Important message" }

    .. duration { 5000 }              // Custom timeout
    .. position { top }               // Top of viewport instead of bottom
}
```

## Tooltip

`at tooltip: { <body> }`

Pre-defined slot available on all fragments. Creates a tooltip that appears on hover.

**Note**: Unlike `dialog!` and `snack!`, tooltip is **not a macro**. It's a built-in slot because tooltips are semantically **attached** to their parent element, not detached.

### Behavior

- **Channel**: Automatically renders in `tooltip` channel
- **Trigger**: Shows on `on_pointer_enter` with delay (default: 500ms)
- **Hide**: Hides on `on_pointer_leave`
- **Positioning**: Positioned relative to parent using `align_relative`
- **Smart repositioning**: Automatically adjusts position if insufficient viewport space

### Default Positioning

`align_relative { horizontal: center vertical: below }`

The positioning is treated as a **suggestion**. If the tooltip would overflow the viewport boundaries, the renderer automatically adjusts the position to keep it visible.

### Syntax

```dsl
<fragment> {
    <content>
    at tooltip: { <tooltip-body> }
}

// Or with template reference:
<fragment> {
    <content>
    at tooltip: TooltipTemplate
}
```

### Examples

**Basic tooltip:**

```dsl
button {
    "Save"

    at tooltip: {
        text { "Ctrl+S" }
        .. padding { 4 }
        .. background { color: rgba(0, 0, 0, 200) }
        .. font { color: White size: 12 }
        .. corner_radius { 2 }
    }
}
```

**Rich tooltip:**

```dsl
icon { "help" }
    at tooltip: {
        column {
            gap { 4 }

            text { "Help Information" } .. font { weight: 700 color: White }
            text { "Click for more details" } .. font { color: White size: 12 }
        }

        .. padding { 8 }
        .. background { color: rgba(0, 0, 0, 220) }
        .. corner_radius { 4 }
    }
```

**Custom positioning:**

```dsl
button {
    "Options"

    at tooltip: {
        text { "Click to see options" }
        .. padding { 6 }
        .. background { color: Black }
        .. font { color: White }

        // Position above and to the right
        .. align_relative { horizontal: end vertical: above }
    }
}
```

**Template reference:**

```dsl
fragment! {
    SaveButton() {
        button {
            "Save"
            at tooltip: SaveTooltip
        }
    }
}

fragment! {
    SaveTooltip() {
        text { "Save changes (Ctrl+S)" }
        .. padding { 6 }
        .. background { color: rgba(0, 0, 0, 200) }
        .. font { color: White size: 12 }
        .. corner_radius { 2 }
    }
}
```

**Tooltip on any element:**

```dsl
// Works on any fragment
text {
    "Hover for info"

    at tooltip: {
        text { "This is helpful information" }
        .. padding { 6 }
        .. background { color: Black }
        .. font { color: White }
    }
}

row {
    "Complex content"

    at tooltip: {
        text { "Tooltip on a row" }
        .. padding { 6 }
        .. background { color: Black }
        .. font { color: White }
    }

    button { "Click" }
    text { "More stuff" }
}
```

### Smart Repositioning

The tooltip automatically adjusts its position to stay within the viewport:

```dsl
// Near top of viewport
button {
    "Top Button"

    at tooltip: {
        text { "This will show below" }
        .. align_relative { horizontal: center vertical: above }
        // If there's no space above, shows below instead
    }
}

// Near right edge of viewport
button {
    "Right Button"

    at tooltip: {
        text { "This will adjust leftward" }
        .. align_relative { horizontal: after vertical: below }
        // If there's no space to the right, shifts left
    }
}
```

The `align_relative` positioning is a **suggestion**. The renderer will:
1. Try the suggested position first
2. If it would overflow the viewport, try alternative positions
3. Choose the position that best fits within the viewport

## Dismiss Function

`dismiss()`

Explicitly closes the current detached UI element. Available only within `dialog!` and `snack!` bodies.

**Note**: This is a regular **function**, not a macro.

### Scope

The `dismiss()` function is only available within the body of `dialog!` or `snack!`. Using it elsewhere is a compile-time error.

### Behavior

- Immediately closes the detached UI element
- For dialogs with stereotype events, `dismiss()` is called automatically after the handler
- For snacks, overrides the auto-timeout

### Examples

**Explicit dismiss in dialog:**

```dsl
dialog! {
    column {
        text { "Information" }

        button { "Close" } .. on_click {
            dismiss()
        }
    }
}
```

**Not needed with stereotypes:**

```dsl
dialog! {
    button { "Save" } .. stereotype { save }

    on_save {
        save_data()
        // dismiss() is called automatically after this handler
    }
}
```

**Conditional dismiss:**

```dsl
dialog! {
    button { "Submit" } .. on_click {
        when validation_passes {
            submit_data()
            dismiss()
        }
        // If validation fails, dialog stays open
    }
}
```

**Dismiss snack early:**

```dsl
snack! {
    row {
        text { "Processing..." }
        button { "×" } .. on_click {
            dismiss()  // Close before auto-timeout
        }
    }
}
```

## Channel Management

All detached UI elements automatically manage their channel assignment:

| Element   | Channel   | Purpose                          |
|-----------|-----------|----------------------------------|
| `dialog!` | `modal`   | Modal dialogs, blocking content  |
| `snack!`  | `snack`   | Temporary notifications          |
| `tooltip` | `tooltip` | Contextual help                  |

You **cannot** manually set the channel for detached UI elements. This is managed by the runtime to ensure correct stacking and behavior.

## Scope and Lifecycle

### Scope Capture

**Within DSL event handlers:**

Detached UI elements capture their parent scope, including:

- All stores (decl, writable, fanin, source)
- Parameters
- Local variables from control statements (e.g., `repeat` items)

```dsl
fragment! {
    ItemList() {
        writable items = vec![...]

        repeat on items by item.id as item {
            button { "Delete ${item.name}" } .. on_click {
                dialog! {
                    // Captures: items (writable), item (repeat local)
                    text { "Delete ${item.name}?" }

                    on_save {
                        items.remove(item)
                        // Both items and item are in scope
                    }
                }
            }
        }
    }
}
```

**From native Rust:**

Detached UI **does not capture scope**. All data must be moved into the dialog's ownership:

```rust
pub fn show_item_dialog(rt: &Runtime, item_name: String, item_id: u64) {
    // item_name and item_id are moved into the dialog
    dialog! {
        runtime: rt

        column {
            text { format!("Item: {}", item_name) }  // item_name is moved/cloned

            button { "Delete" } .. on_click {
                // Access to stores must be through runtime interface
                delete_item_via_store(rt, item_id)
                dismiss()
            }
        }
    }
}
```

Use stores through the runtime interface to communicate between native Rust and dialog handlers.

### Lifecycle Management

Detached UI elements are automatically cleaned up when their parent scope is destroyed:

**Scenario: Dynamic list with dialogs**

```dsl
repeat on users by user.id as user {
    button { "Edit ${user.name}" } .. on_click {
        dialog! {
            // Dialog for this specific user
            text { "Edit ${user.name}" }
        }
    }
}

// If users list is updated and a user is removed:
// 1. The repeat iteration for that user is destroyed
// 2. Any open dialogs for that user are automatically dismissed
```

This prevents orphaned UI elements that reference deleted or invalid data.

### Non-blocking Execution

All detached UI creation is non-blocking:

```dsl
button { "Show" } .. on_click {
    dialog! { text { "Dialog" } }
    snack! { text { "Snack" } }
    log("This executes immediately")
    // Dialog and snack are shown, but execution continues
}
```

## Data Handling

### Results via Stores (DSL)

Use writable stores to capture dialog results:

```dsl
fragment! {
    App() {
        writable user_confirmed = false

        button { "Delete All" } .. on_click {
            dialog! {
                text { "Delete all items?" }
                button { "Cancel" } .. stereotype { cancel }
                button { "Delete" } .. stereotype { save }

                on_save {
                    user_confirmed = true
                }
            }
        }

        when user_confirmed {
            delete_all_items()
            user_confirmed = false
        }
    }
}
```

### Results via Stores (Native Rust)

Use the runtime's store interface to communicate:

```rust
pub fn request_confirmation(rt: &Runtime, message: String) -> StoreHandle<bool> {
    // Create a writable store to hold the result
    let result_store = rt.create_writable(false);
    let result_handle = result_store.clone();

    dialog! {
        runtime: rt

        column {
            text { message }

            row {
                button { "No" } .. stereotype { cancel }
                button { "Yes" } .. stereotype { save }
            }
        }

        on_save {
            rt.write_store(result_store, true)
            dismiss()
        }
    }

    // Caller can observe this store for the result
    result_handle
}

// Usage:
let confirmed = request_confirmation(&rt, "Delete?".to_string());
rt.observe(confirmed, |value| {
    if value {
        delete_items();
    }
});
```

## Error Conditions

| Condition                                    | Kind         | Description                                    |
|----------------------------------------------|--------------|------------------------------------------------|
| `dialog!` outside event handler or Rust      | Compile-time | Dialog only allowed in event handlers or Rust  |
| `snack!` outside event handler or Rust       | Compile-time | Snack only allowed in event handlers or Rust   |
| `dismiss()` outside detached UI              | Compile-time | Dismiss only allowed in dialog/snack bodies    |
| Multiple `tooltip` slots on same fragment    | Compile-time | Only one tooltip slot allowed                  |
| Missing runtime parameter in native Rust     | Compile-time | Runtime required when called from Rust         |
| Parent scope destroyed while dialog open     | Runtime      | Dialog auto-dismissed (not an error)           |

## Notes

**Why macros for dialog and snack?**

Using procedural macro syntax (`dialog!`, `snack!`) makes it clear these are special compile-time constructs that:
- Work seamlessly in both DSL and native Rust contexts
- Follow Rust conventions (like `vec!`, `println!`)
- Signal compile-time magic with the `!` sigil
- Allow optional runtime parameter based on context

**Why slot syntax for tooltip?**

Tooltips are semantically **attached** to their parent element, not detached like dialogs. The slot syntax (`at tooltip:`) reflects this relationship more accurately than a macro would.

**Why detached?**

Dialogs, snacks, and tooltips are fundamentally different from normal UI elements:

- They render in different channels
- Their position is relative to the viewport, not their parent
- Their lifecycle is independent of tree structure
- They need special stacking and management

Treating them as special constructs rather than normal fragments makes their behavior explicit and prevents incorrect usage patterns.

**Future enhancements:**

Planned improvements include:
- Custom duration for snacks
- Custom positioning strategies for snacks (top/bottom)
- Backdrop/overlay styling for dialogs
- Animation/transitions for show/hide
- Custom show/hide delays for tooltips
- Multiple tooltips (different triggers)
