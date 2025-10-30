# Themes

Themes are reusable styling configurations for UI components. They are defined using a declarative DSL and support qualifier-based variants (e.g., light/dark mode). Themes are implemented as [resources](60_resources.md) and behave as **shared sources**.

## Overview

**Purpose:** Provide component-level styling that is:
- **Declarative:** Define once, reuse everywhere
- **Reactive:** Automatically adapt to environment changes (theme, locale, etc.)
- **Composable:** Inherit from base themes, reference other resources
- **Type-safe:** Compile-time validation of properties and variants

**Key Concepts:**
- **Properties:** Scalar values (numbers, colors, sizes)
- **Instruction Groups:** Reusable bundles of styling instructions
- **Variants:** Named configurations (e.g., `success`, `error`, `primary`, `secondary`)
- **Qualifiers:** Environment-based resolution (light/dark appearance)

## Theme Definition

Themes are defined in `.theme` files using a declarative DSL:

### Basic Structure

**resources/themes/badge.theme:**

```frel
theme Badge {
    // Properties (scalar values)
    height: 20
    corner_radius: 4
    icon_size: 14

    // Instruction groups (reusable styling bundles)
    outer_container: [
        height { height }
        corner_radius { corner_radius }
        background { color: background_color }
    ]

    icon_container: [
        width { height }
        height { height }
        align_items_center
        background { color: icon_background }
        corner_radius { top_left: corner_radius bottom_left: corner_radius }
        border { color: border_color right: 0 }
    ]

    icon: [
        width { icon_size }
        height { icon_size }
        fill { color: icon_color }
    ]

    text_base: [
        height { height }
        align_self_center
        padding { horizontal: 8 }
    ]

    text_standalone: [
        ..text_base
        border { color: border_color }
        corner_radius { corner_radius }
    ]

    text: [
        padding { top: 2 }
        align_self_center
        font { color: text_color }
    ]

    removable_container: [
        width { height }
        height { height }
        align_items_center
        background { color: border_color }
        corner_radius { top_right: corner_radius bottom_right: corner_radius }
        border { color: border_color left: 0 }
    ]

    removable_icon: [
        width { icon_size - 2 }
        height { icon_size - 2 }
        fill { color: icon_color }
    ]

    // Variants (named configurations)
    variants {
        success {
            border_color: 0x4CAF50
            background_color: 0xFFFFFF
            text_color: 0x000000
            icon_resource: Graphics.success
            icon_color: 0x2E7D32
            icon_background: 0xE8F5E9
        }

        error {
            border_color: 0xF44336
            background_color: 0xFFFFFF
            text_color: 0x000000
            icon_resource: Graphics.error
            icon_color: 0xC62828
            icon_background: 0xFFEBEE
        }

        warning {
            border_color: 0xFF9800
            background_color: 0xFFFFFF
            text_color: 0x000000
            icon_resource: Graphics.warning
            icon_color: 0xE65100
            icon_background: 0xFFF3E0
        }

        info {
            border_color: 0x2196F3
            background_color: 0xFFFFFF
            text_color: 0x000000
            icon_resource: Graphics.info
            icon_color: 0x1565C0
            icon_background: 0xE3F2FD
        }
    }
}
```

### Syntax

```frel
theme <Name> [: <BaseTheme>] {
    // Properties (scalar values)
    <property_name>: <value>
    <property_name>: <value>
    ...

    // Instruction groups (styling bundles)
    <group_name>: [
        <instruction>
        <instruction>
        ...
    ]

    // Variants (named configurations)
    variants {
        <variant_name> {
            <property_name>: <value>
            ...

            <group_name>: [
                <instruction>
                ...
            ]
        }

        <variant_name> {
            ...
        }
    }
}
```

**Elements:**

- **`<Name>`**: Theme identifier (PascalCase recommended)
- **`: <BaseTheme>`**: Optional inheritance from another theme
- **Properties**: Scalar values (numbers, colors, booleans, strings)
- **Instruction groups**: Arrays of instructions using `[]` syntax
- **Variants**: Named configurations with their own properties and instruction groups

### Properties

Properties are scalar values that can be referenced in instruction groups:

```frel
theme Button {
    // Dimensions
    height: 32
    min_width: 80

    // Colors
    background_color: 0x2196F3
    text_color: 0xFFFFFF

    // Spacing
    padding_horizontal: 16
    padding_vertical: 8

    // Typography
    font_size: 14
    font_weight: 500
}
```

**Supported Types:**
- Numbers: `20`, `3.14`, `0.5`
- Colors: `0xRRGGBB`, `0xRRGGBBAA`
- Strings: `"Arial"`, `"system-ui"`
- Resource references: `Graphics.icon`, `Fonts.heading`

### Instruction Groups

Instruction groups are reusable bundles of styling instructions:

```frel
theme Card {
    corner_radius: 8

    container: [
        corner_radius { corner_radius }
        background { color: 0xFFFFFF }
        border { color: 0xE0E0E0 }
        padding { 16 }
        shadow { color: 0x00000020 offset_x: 0 offset_y: 2 blur: 4 }
    ]

    header: [
        padding { bottom: 12 }
        border { color: 0xE0E0E0 bottom: 1 }
        font { size: 16 weight: 600 }
    ]

    body: [
        padding { vertical: 12 }
        font { size: 14 }
    ]
}
```

**Instruction Group Composition:**

Use the spread operator `..` to include one group in another:

```frel
theme Input {
    base: [
        height { 32 }
        padding { horizontal: 12 }
    ]

    focused: [
        ..base
        border { color: 0x2196F3 width: 2 }
    ]

    error: [
        ..base
        border { color: 0xF44336 width: 2 }
    ]
}
```

### Variants

Variants provide named configurations within a theme:

```frel
theme Button {
    height: 32

    variants {
        primary {
            background_color: 0x2196F3
            text_color: 0xFFFFFF
            hover_background: 0x1976D2

            container: [
                background { color: background_color }
                height { height }
            ]

            text: [
                font { color: text_color weight: 500 }
            ]
        }

        secondary {
            background_color: 0xE0E0E0
            text_color: 0x000000
            hover_background: 0xBDBDBD

            container: [
                background { color: background_color }
                height { height }
            ]

            text: [
                font { color: text_color weight: 400 }
            ]
        }
    }
}
```

**Variant Inheritance:**

Variants inherit properties and instruction groups from the base theme and can override them:

```frel
theme Badge {
    height: 20

    base_container: [
        height { height }
        corner_radius { 4 }
    ]

    variants {
        success {
            // Inherits height and base_container
            border_color: 0x4CAF50

            // Override base_container
            container: [
                ..base_container
                border { color: border_color }
            ]
        }
    }
}
```

## Theme Access

Themes are accessed through the `Themes` namespace like other [resources](60_resources.md):

### Static Access

```frel
fragment StatusBadge(status: Status, text: String) {
    // Select theme variant based on status
    decl badge_variant = select on status {
        Status::Success => Themes.Badge.success
        Status::Error => Themes.Badge.error
        Status::Warning => Themes.Badge.warning
        else => Themes.Badge.info
    }

    row {
        // Apply theme instruction groups using the spread operator
        ..badge_variant.outer_container

        // Icon container
        box {
            ..badge_variant.icon_container
            icon { badge_variant.icon_resource }
                ..badge_variant.icon
        }

        // Text
        text { text }
            ..badge_variant.text
    }
}
```

### Direct Property Access

```frel
fragment
    CustomButton() {
        button {
            // Access theme properties directly
            height { Themes.Button.primary.height }

            // Apply instruction group
            ..Themes.Button.primary.container

            text { "Click me" }
                ..Themes.Button.primary.text
        }
    }
}
```

### Dynamic Variant Lookup

```frel
fragment
    DynamicBadge(badge_type: String, text: String) {
        // Dynamic lookup by variant name
        decl badge_variant = Themes.Badge.variant(badge_type)
            .unwrap_or(Themes.Badge.info)

        row {
            ..badge_variant.outer_container
            text { text }
                ..badge_variant.text
        }
    }
}
```

## Theme Qualifiers

Themes support [environment qualifiers](60_resources.md#qualifiers) just like other resources, enabling automatic light/dark appearance resolution.

### Light/Dark Themes

Define separate theme files with qualifiers:

**resources/themes/button-light.theme:**

```frel
theme Button {
    variants {
        primary {
            background_color: 0x2196F3
            text_color: 0xFFFFFF
            hover_background: 0x1976D2

            container: [
                background { color: background_color }
            ]
        }
    }
}
```

**resources/themes/button-dark.theme:**

```frel
theme Button {
    variants {
        primary {
            background_color: 0x90CAF9
            text_color: 0x000000
            hover_background: 0x64B5F6

            container: [
                background { color: background_color }
            ]
        }
    }
}
```

### Automatic Resolution

The appropriate theme is automatically selected based on the [environment](standard_sources.md#environment):

```frel
fragment
    ThemedButton(text: String) {
        // Automatically resolves to light or dark variant
        button {
            ..Themes.Button.primary.container
            text { text }
                ..Themes.Button.primary.text
        }

        // When environment.appearance changes, the theme automatically updates
    }
}
```

### Reactive Theme Switching

```frel
fragment
    App() {
        column {
            button {
                text { "Toggle Theme" }
                on_click {
                    Environment::update(|env| {
                        env.appearance = match env.appearance {
                            Appearance::Light => Appearance::Dark,
                            Appearance::Dark => Appearance::Light,
                        };
                    });
                    // All themed components automatically update
                }
            }

            // Uses current theme automatically
            box {
                ..Themes.Card.default.container
                text { "Themed content" }
            }
        }
    }
}
```

## Theme Composition

Themes can reference other themes and resources for consistency.

### Referencing Other Themes

```frel
theme InputBase {
    height: 32
    border_radius: 4
    padding_horizontal: 12

    variants {
        default {
            container: [
                height { height }
                corner_radius { border_radius }
                padding { horizontal: padding_horizontal }
            ]
        }
    }
}

theme TextInput {
    // Reference base theme properties
    base_height: Themes.InputBase.default.height
    base_radius: Themes.InputBase.default.border_radius

    // Additional properties
    font_size: 14

    variants {
        default {
            container: [
                // Include base theme instructions
                ..Themes.InputBase.default.container

                // Add additional styling
                font { size: font_size }
            ]
        }
    }
}
```

### Referencing Resources

```frel
theme Input {
    // Reference graphics
    clear_icon: Graphics.close
    search_icon: Graphics.search

    // Reference colors (if you have a color resource system)
    border_color: Colors.outline
    focus_color: Colors.primary

    variants {
        default {
            container: [
                border { color: border_color }
            ]

            // Use referenced resources
            prefix_icon: [
                width { 16 }
                height { 16 }
            ]
        }
    }
}
```

### Usage

```frel
fragment
    SearchInput(query: String) {
        row {
            ..Themes.Input.default.container

            // Use theme's icon reference
            icon { Themes.Input.search_icon }
                ..Themes.Input.default.prefix_icon

            text_input(value: query) { }
        }
    }
}
```

## Theme Inheritance

Themes can inherit from base themes to share common properties and instruction groups.

### Basic Inheritance

```frel
theme InputBase {
    height: 32
    border_radius: 4
    padding_horizontal: 12

    variants {
        default {
            container: [
                height { height }
                corner_radius { border_radius }
                padding { horizontal: padding_horizontal }
                border { color: 0xE0E0E0 }
            ]
        }
    }
}

theme TextInput : InputBase {
    // Inherits all properties from InputBase
    // Can add new properties
    font_size: 14

    variants {
        default {
            // Inherits InputBase.default properties
            // Can override or extend instruction groups
            container: [
                ..InputBase.default.container
                font { size: font_size }
            ]
        }
    }
}

theme NumberInput : InputBase {
    // Inherits from InputBase
    text_align: "right"

    variants {
        default {
            container: [
                ..InputBase.default.container
                align_items_end
            ]
        }
    }
}
```

### Override Properties

```frel
theme ButtonBase {
    height: 32
    padding_horizontal: 16

    variants {
        default {
            container: [
                height { height }
                padding { horizontal: padding_horizontal }
            ]
        }
    }
}

theme LargeButton : ButtonBase {
    // Override inherited property
    height: 48
    padding_horizontal: 24

    variants {
        default {
            // Uses overridden height and padding
            container: [
                ..ButtonBase.default.container
            ]
        }
    }
}
```

## Themes as Shared Sources

Like all [resources](60_resources.md#resources-as-shared-sources), themes are **shared sources**:

- **Global lifecycle:** Created once during application initialization, not per-fragment
- **Shared subscriptions:** All fragments accessing the same theme subscribe to the same underlying source
- **Reactive updates:** When the [environment](standard_sources.md#environment) changes (e.g., appearance switch), themes automatically re-resolve and notify subscribers
- **Status tracking:** Themes have `Loading`, `Ready`, or `Error` status (accessible via `.status()`)
- **Optional values:** Themes provide `.latest()` to access the current value

### Explicit Source Handling

```frel
fragment
    ThemedComponent() {
        // Access environment to react to theme changes
        source env = environment()

        decl current_appearance_name = env.latest()
            .map(|e| match e.appearance {
                Appearance::Light => "light",
                Appearance::Dark => "dark",
            })
            .unwrap_or("light")

        // Automatically updates when environment.appearance changes
        box {
            ..Themes.Panel.default.container

            text { "Current appearance: ${current_appearance_name}" }
        }
    }
}
```

### Status Checking

```frel
fragment
    SafeThemedComponent() {
        // Check theme status
        decl theme_status = Themes.Button.primary.status()

        select on theme_status {
            Status::Loading => text { "Loading theme..." }
            Status::Error(e) => text { "Failed to load theme" }
            Status::Ready => {
                button {
                    ..Themes.Button.primary.container
                    text { "Themed Button" }
                }
            }
        }
    }
}
```

## Generated Theme API

The theme compiler generates a type-safe API for each theme.

### For a Theme Definition

**Input (badge.theme):**

```frel
theme Badge {
    height: 20
    corner_radius: 4

    outer_container: [ ... ]
    icon: [ ... ]
    text: [ ... ]

    variants {
        success { ... }
        error { ... }
        warning { ... }
    }
}
```

### Generated Module Structure

**Output (generated code):**

```rust
// Top-level theme namespace
pub mod Badge {
    // Default variant (first variant or base)
    pub fn default() -> &'static BadgeTheme {
        &success
    }

    // Named variant accessors (static)
    pub mod success {
        pub const height: DPixel = 20.dp;
        pub const corner_radius: DPixel = 4.dp;
        pub static outer_container: InstructionGroup = ...;
        pub static icon: InstructionGroup = ...;
        pub static text: InstructionGroup = ...;
    }

    pub mod error {
        pub const height: DPixel = 20.dp;
        pub const corner_radius: DPixel = 4.dp;
        pub static outer_container: InstructionGroup = ...;
        pub static icon: InstructionGroup = ...;
        pub static text: InstructionGroup = ...;
    }

    pub mod warning {
        pub const height: DPixel = 20.dp;
        pub const corner_radius: DPixel = 4.dp;
        pub static outer_container: InstructionGroup = ...;
        pub static icon: InstructionGroup = ...;
        pub static text: InstructionGroup = ...;
    }

    // Dynamic variant lookup
    pub fn variant(name: &str) -> Option<&'static BadgeTheme> {
        match name {
            "success" => Some(&success),
            "error" => Some(&error),
            "warning" => Some(&warning),
            _ => None,
        }
    }

    // Shared source implementation
    impl Resource for BadgeTheme {
        fn status(&self) -> Status { Status::Ready }
        fn latest(&self) -> Option<&Self> { Some(self) }
    }
}
```

### Usage Examples

```frel
// Static access (compile-time safe)
..Themes.Badge.success.outer_container

// Direct property access
height { Themes.Badge.success.height }

// Dynamic variant lookup
decl variant = Themes.Badge.variant("success").unwrap()
..variant.outer_container

// Default variant
..Themes.Badge.default().outer_container
```

## Theme Best Practices

### 1. Use Qualifiers for Light/Dark Themes

**Good:**
```text
resources/themes/
  button-light.theme
  button-dark.theme
  card-light.theme
  card-dark.theme
```

Automatically resolves based on `environment().appearance`.

### 2. Group Related Styling

Keep instruction groups focused and well-named:

```frel
theme Card {
    // Good: focused groups
    container: [ ... ]
    header: [ ... ]
    body: [ ... ]
    footer: [ ... ]

    // Avoid: kitchen sink group
    everything: [ /* too many unrelated instructions */ ]
}
```

### 3. Define Semantic Variants

```frel
theme Alert {
    variants {
        // Good: semantic names
        success { ... }
        error { ... }
        warning { ... }
        info { ... }

        // Good: hierarchy names
        primary { ... }
        secondary { ... }
        tertiary { ... }
    }
}
```

### 4. Compose and Reuse

```frel
theme Base {
    control_height: 32
    border_radius: 4
}

theme Button : Base {
    // Reuse Base.control_height
    height: Base.control_height
}

theme Input : Base {
    // Reuse Base.control_height and border_radius
    height: Base.control_height
    border_radius: Base.border_radius
}
```

### 5. Theme Components, Not Primitives

**Good:**
- `ButtonTheme`
- `InputTheme`
- `CardTheme`
- `BadgeTheme`
- `SelectTheme`

**Avoid:**
- `BoxTheme` (too generic)
- `TextTheme` (too primitive)
- `RowTheme` (layout primitive)

### 6. Use Consistent Naming

```frel
// Good: consistent property names across themes
theme Button {
    background_color: ...
    text_color: ...
    hover_background: ...
}

theme Input {
    background_color: ...
    text_color: ...
    focus_background: ...
}

// Good: consistent instruction group names
theme Card {
    container: [ ... ]
    header: [ ... ]
    body: [ ... ]
}

theme Modal {
    container: [ ... ]
    header: [ ... ]
    body: [ ... ]
}
```

## Complete Example

**resources/themes/input-light.theme:**

```frel
theme Input {
    height: 32
    border_radius: 4
    padding_horizontal: 12
    font_size: 14

    background_color: 0xFFFFFF
    border_color: 0xE0E0E0
    text_color: 0x000000
    placeholder_color: 0x9E9E9E
    focus_border_color: 0x2196F3
    error_border_color: 0xF44336

    clear_icon: Graphics.close

    variants {
        default {
            container: [
                height { height }
                corner_radius { border_radius }
                padding { horizontal: padding_horizontal }
                background { color: background_color }
                border { color: border_color }
            ]

            text: [
                font { size: font_size color: text_color }
            ]

            placeholder: [
                font { size: font_size color: placeholder_color }
            ]

            clear_button: [
                width { 16 }
                height { 16 }
                cursor { pointer }
            ]
        }

        focused {
            container: [
                ..default.container
                border { color: focus_border_color width: 2 }
            ]
        }

        error {
            container: [
                ..default.container
                border { color: error_border_color width: 2 }
            ]
        }
    }
}
```

**Usage in fragment:**

```frel
fragment
    TextInput(value: String, error: Option<String>) {
        source focused = focus()

        decl has_error = error.is_some()
        decl is_focused = focused.latest().unwrap_or(false)

        decl input_variant = when has_error {
            Themes.Input.error
        } else when is_focused {
            Themes.Input.focused
        } else {
            Themes.Input.default
        }

        row {
            ..input_variant.container

            text_input(value: value) { }
                ..input_variant.text

            when !value.is_empty() {
                button {
                    icon { Themes.Input.clear_icon }
                        ..Themes.Input.default.clear_button

                    on_click { value = "" }
                }
            }
        }

        when has_error {
            text { error.unwrap() }
                ..Themes.Input.error.text
        }
    }
}
```

## See Also

- [Resources](60_resources.md) - Resource system overview
- [Standard Sources](standard_sources.md#environment) - Environment source for theme qualification
- [Instructions](40_instructions.md) - Available styling instructions
- [Store Declarations](10_store_declarations.md#data-source) - Source system
