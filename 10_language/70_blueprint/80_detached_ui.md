# Detached UI

Detached UI elements are special constructs for creating UI that renders outside the main
blueprint hierarchy in separate channels.

Unlike normal fragments that are children of their parent, detached UI elements:

- Render in separate channels (`modal`, `toast`, `tooltip`)
- Have independent lifecycles from their creation point
- Are managed automatically by the runtime
- Don't affect the layout of their parent
- Are automatically positioned in the viewport/scene

## Overview

| Element   | Form             | Channel   | Lifecycle          |
|-----------|------------------|-----------|--------------------|
| `modal!`  | Macro            | `modal`   | Until dismissed    |
| `toast!`  | Macro            | `toast`   | Auto-timeout       |
| `tooltip` | Slot (not macro) | `tooltip` | Auto-hide on leave |

## Modal

```frel
modal ConfirmDelete(item: &TodoData) {
    column { /* ... */ }
}

// Usage from DSL
button { "Delete" } .. on_click { 
    show ConfirmDelete(item)
}
```

```rust
// Usage from native Rust
fn show_delete_modal(rt: &RuntimeHandle, item: &TodoData) {
    ConfirmDelete::show(rt, item)
}  
```

### Behavior

`show` is:

- **Non-blocking**: Execution continues immediately after modal creation
- **Channel**: Automatically renders in `modal` channel
- **Container**: Rendered within a box sized to the full scene/viewport
- **Stacking**: Multiple modals stack (not queued) - all remain open simultaneously
- **Positioning**: Can be positioned and sized using standard layout instructions
- **Lifecycle**: Persists until explicitly dismissed

### Dismissal

A modal can be dismissed by these ways:

1. **Explicit dismiss**: Call `dismiss()` function within the modal
2. **Stereotype events**: `on_save` and `on_cancel` automatically dismiss after handler executes

The `dismiss()` function can be called from within the modal body.

## Toast Notifications

Toast notifications are temporary messages that appear briefly to provide feedback to the user. They auto-dismiss after a timeout and render in the `toast` channel.

### Semantic Macros (Recommended)

For common notification types, use semantic macros that automatically apply appropriate styling:

```frel
info { "Processing complete" }
success { "Saved successfully!" }
warning { "Unsaved changes" }
error { "Connection failed" }
```

**Behavior:**
- **Non-blocking**: Execution continues immediately
- **Channel**: Automatically renders in `toast` channel
- **Stacking**: Multiple toasts stack and queue
- **Auto-dismiss**: Automatically dismissed after timeout (default: 3000ms)
- **Default positioning**: Bottom-center of viewport
- **Automatic styling**: Each type gets appropriate colors and icons

**Examples:**

```frel
// Simple messages
button { "Save" } .. on_click {
    save_data()
    success { "Changes saved" }
}

button { "Submit" } .. on_click {
    when validation_failed {
        error { "Please fill all required fields" }
    }
}

// With rich content
info! {
    markdown { "See [documentation](https://...) for details" }
}

// Using template references
warning { UnsavedChangesMessage }
```

**From native Rust:**

```rust
pub fn notify_save_success(rt: &Runtime) {
    success_rt!(rt, "Changes saved");
}

pub fn notify_error(rt: &Runtime, message: String) {
    error_rt!(rt, message);
}
```

### Custom Toast Macro

For custom-styled toast notifications that don't fit the semantic types, use the `toast!` macro to define reusable templates:

`toast <name>(<params>) { <body> }`

**Definition (top-level only):**

```frel
toast Achievement(text: String) {
    row {
        padding { 12 }
        gap { 8 }
        background { color: Purple }
        corner_radius { 4 }
        shadow { color: rgba(0, 0, 0, 128) offset_y: 2 blur: 4 }

        icon { "star" } .. color { Gold }
        text { text } .. font { color: White weight: 700 }
    }
}
```

**Usage:**

```frel
// From DSL event handlers
button { "Complete Level" } .. on_click {
    show Achievement(text: "Level 10 reached!")
}
```

```rust
// From native Rust
pub fn show_achievement(rt: &Runtime, text: String) {
    Achievement::show(rt, text)
}
```

### Custom Toast with Dismiss Button

```frel
toast ProcessingNotification(message: String) {
    row {
        padding { 12 }
        gap { 8 }
        background { color: Blue }
        corner_radius { 4 }

        text { message }
        button { "×" } .. on_click {
            dismiss()  // Explicit dismissal
        }
    }
}

button { "Start Process" } .. on_click {
    start_long_process()
    show ProcessingNotification(message: "Processing...")
}
```

### Stacking and Queueing

Multiple toasts stack vertically and queue if too many are shown:

```frel
button { "Multiple Notifications" } .. on_click {
    info { "First" }
    info { "Second" }
    info { "Third" }

    // All three will be shown, stacked vertically
    // Each auto-dismisses after 3 seconds
}
```

### Future Customization

**Note**: These instructions are planned but not yet implemented. For POC, use defaults.

```frel
// Planned for future versions:

toast ImportantMessage(text: String) {
    text { text }

    .. duration { 5000 }              // Custom timeout
    .. position { top }               // Top of viewport instead of bottom
}
```

## Tooltip

`at tooltip: { <body> }`

Pre-defined slot available on all fragments. Creates a tooltip that appears on hover.

**Note**: Unlike `modal!` and `toast!`, tooltip is **not a macro**. It's a built-in slot because 
tooltips are semantically **attached** to their parent element, not detached.

### Behavior

- **Channel**: Automatically renders in `tooltip` channel
- **Trigger**: Shows on `on_pointer_enter` with delay (default: 500ms)
- **Hide**: Hides on `on_pointer_leave`
- **Positioning**: Positioned relative to parent using `align_relative`
- **Smart repositioning**: Automatically adjusts position if insufficient viewport space

### Default Positioning

`align_relative { horizontal: center vertical: below }`

The positioning is treated as a **suggestion**. If the tooltip would overflow the viewport boundaries,
the renderer automatically adjusts the position to keep it visible.

### Syntax

```frel
<blueprint> {
    <content>
    at tooltip: { <tooltip-body> }
}

// Or with template reference:
<blueprint> {
    <content>
    at tooltip: TooltipTemplate
}
```