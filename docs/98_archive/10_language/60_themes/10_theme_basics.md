# Themes

Themes are reusable styling configurations for UI components. They are defined using a declarative DSL and support qualifier-based variants (e.g., light/dark mode). Themes are implemented as [resources](60_resources.md) and behave as **shared sources**.

## Overview

**Purpose:** Provide component-level styling that is:
- **Declarative:** Define once, reuse everywhere
- **Reactive:** Automatically adapt to environment changes (theme, locale, etc.)
- **Composable:** Reference other themes and resources
- **Type-safe:** Compile-time validation of properties and variants

**Key Concepts:**
- **Properties:** Scalar values (numbers, colors, sizes) defined at theme level
- **Instruction Groups:** Reusable bundles of styling instructions, shared across variants
- **Variants:** Named property configurations (e.g., `success`, `error`, `primary`, `secondary`)
- **Qualifiers:** Environment-based resolution (light/dark appearance)

**Core Model:**
- Instruction groups are defined at the theme level and shared by all variants
- Variants provide property values that groups reference
- Variants can only override base properties, not add new ones or change group structure

## Theme Definition

Themes are defined in `.theme` files using a declarative DSL:

### Basic Structure

**resources/themes/badge.theme:**

```frel
theme Badge {
    // Properties (scalar values)
    prop height = 20
    prop corner_radius = 4
    prop icon_size = 14

    // Variant-specific properties (defined at base, overridden in variants)
    prop border_color = 0x000000
    prop background_color = 0xFFFFFF
    prop text_color = 0x000000
    prop icon_resource = Graphics.info
    prop icon_color = 0x000000
    prop icon_background = 0xFFFFFF

    // Instruction groups (reusable styling bundles, shared by all variants)
    group outer_container {
        height { height }
        corner_radius { corner_radius }
        background { color: background_color }
    }

    group icon_container {
        width { height }
        height { height }
        align_items_center
        background { color: icon_background }
        corner_radius { top_left: corner_radius bottom_left: corner_radius }
        border { color: border_color right: 0 }
    }

    group icon {
        width { icon_size }
        height { icon_size }
        fill { color: icon_color }
    }

    group text_base {
        height { height }
        align_self_center
        padding { horizontal: 8 }
    }

    group text_standalone {
        ..text_base
        border { color: border_color }
        corner_radius { corner_radius }
    }

    group text {
        padding { top: 2 }
        align_self_center
        font { color: text_color }
    }

    group removable_container {
        width { height }
        height { height }
        align_items_center
        background { color: border_color }
        corner_radius { top_right: corner_radius bottom_right: corner_radius }
        border { color: border_color left: 0 }
    }

    group removable_icon {
        width { icon_size - 2 }
        height { icon_size - 2 }
        fill { color: icon_color }
    }

    // Variants (named property configurations)
    variant success {
        border_color = 0x4CAF50
        background_color = 0xFFFFFF
        text_color = 0x000000
        icon_resource = Graphics.success
        icon_color = 0x2E7D32
        icon_background = 0xE8F5E9
    }

    variant error {
        border_color = 0xF44336
        background_color = 0xFFFFFF
        text_color = 0x000000
        icon_resource = Graphics.error
        icon_color = 0xC62828
        icon_background = 0xFFEBEE
    }

    variant warning {
        border_color = 0xFF9800
        background_color = 0xFFFFFF
        text_color = 0x000000
        icon_resource = Graphics.warning
        icon_color = 0xE65100
        icon_background = 0xFFF3E0
    }

    variant info {
        border_color = 0x2196F3
        background_color = 0xFFFFFF
        text_color = 0x000000
        icon_resource = Graphics.info
        icon_color = 0x1565C0
        icon_background = 0xE3F2FD
    }
}
```

### Syntax

```frel
theme <Name> {
    // Properties (scalar values)
    prop <property_name> = <value>
    prop <property_name> = <value>
    ...

    // Instruction groups (styling bundles)
    group <group_name> {
        <instruction>
        <instruction>
        ...
    }

    // Variants (named property configurations)
    variant <variant_name> {
        <property_name> = <value>
        <property_name> = <value>
        ...
    }
}
```

**Elements:**

- **`theme <Name>`**: Theme identifier (PascalCase recommended)
- **`prop`**: Property declaration keyword
- **`group`**: Instruction group declaration keyword
- **`variant`**: Variant declaration keyword
- **Properties**: Scalar values (numbers, colors, booleans, strings, resource references)
- **Instruction groups**: Blocks of instructions using `{ }` syntax, shared across all variants
- **Variants**: Named property overrides only; cannot add new properties or override groups

**Rules:**

- All properties must be declared at theme level with default values
- Variants can only override existing properties, not add new ones
- Instruction groups are defined at theme level and shared by all variants
- Groups reference properties which are resolved per-variant when accessed

### Properties

Properties are scalar values that can be referenced in instruction groups. All properties must be declared at the theme level.

```frel
theme Button {
    // Dimensions
    prop height = 32
    prop min_width = 80

    // Colors
    prop background_color = 0x2196F3
    prop text_color = 0xFFFFFF

    // Spacing
    prop padding_horizontal = 16
    prop padding_vertical = 8

    // Typography
    prop font_size = 14
    prop font_weight = 500
}
```

**Supported Types:**
- Numbers: `20`, `3.14`, `0.5`
- Colors: `0xRRGGBB`, `0xRRGGBBAA`
- Strings: `"Arial"`, `"system-ui"`
- Resource references: `Graphics.icon`, `Fonts.heading`

**Property Resolution:**
- Base theme defines all properties with default values
- Variants override property values (cannot add new properties)
- When accessing a group via a variant, property references resolve using that variant's values

### Instruction Groups

Instruction groups are reusable bundles of styling instructions defined at the theme level and shared across all variants:

```frel
theme Card {
    prop corner_radius = 8
    prop background_color = 0xFFFFFF
    prop border_color = 0xE0E0E0

    group container {
        corner_radius { corner_radius }
        background { color: background_color }
        border { color: border_color }
        padding { 16 }
        shadow { color: 0x00000020 offset_x: 0 offset_y: 2 blur: 4 }
    }

    group header {
        padding { bottom: 12 }
        border { color: border_color bottom: 1 }
        font { size: 16 weight: 600 }
    }

    group body {
        padding { vertical: 12 }
        font { size: 14 }
    }
}
```

**Instruction Group Composition:**

Use the spread operator `..` to include one group in another:

```frel
theme Input {
    prop height = 32
    prop padding_horizontal = 12
    prop border_color = 0x2196F3
    prop error_border_color = 0xF44336

    group base {
        height { height }
        padding { horizontal: padding_horizontal }
    }

    group focused {
        ..base
        border { color: border_color width: 2 }
    }

    group error {
        ..base
        border { color: error_border_color width: 2 }
    }
}
```

**Key Points:**
- Groups are defined once at theme level
- All variants share the same group definitions
- Groups reference properties which resolve based on the variant context

### Variants

Variants provide named property configurations within a theme. Variants can only override existing properties—they cannot add new properties or define instruction groups.

```frel
theme Button {
    // Base properties with defaults
    prop height = 32
    prop background_color = 0xE0E0E0
    prop text_color = 0x000000
    prop hover_background = 0xBDBDBD
    prop font_weight = 400

    // Shared instruction groups
    group container {
        background { color: background_color }
        height { height }
    }

    group text {
        font { color: text_color weight: font_weight }
    }

    // Variants override properties only
    variant primary {
        background_color = 0x2196F3
        text_color = 0xFFFFFF
        hover_background = 0x1976D2
        font_weight = 500
    }

    variant secondary {
        background_color = 0xE0E0E0
        text_color = 0x000000
        hover_background = 0xBDBDBD
        font_weight = 400
    }
}
```

**How Variants Work:**

When you access `Themes.Button.primary.container`:
1. The `container` group definition comes from the theme level
2. Property references within the group (like `background_color`) resolve using the `primary` variant's values
3. The result is the same group structure with variant-specific property values

**Rules:**
- All properties must be declared at the theme level with default values
- Variants can only override existing properties (no new properties)
- Variants cannot define or override instruction groups
- All variants share the same group definitions from the theme level

## Theme Access

Themes are accessed through the `Themes` namespace like other [resources](60_resources.md):

### Static Access

```frel
blueprint StatusBadge(status: Status, text: String) {
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
blueprint CustomButton() {
    button {
        // Access theme properties directly
        height { Themes.Button.primary.height }

        // Apply instruction group
        ..Themes.Button.primary.container

        text { "Click me" }
            ..Themes.Button.primary.text
    }
}
```

### Dynamic Variant Lookup

```frel
blueprint DynamicBadge(badge_type: String, text: String) {
    // Dynamic lookup by variant name
    decl badge_variant = Themes.Badge.variant(badge_type)
        .unwrap_or(Themes.Badge.info)

    row {
        ..badge_variant.outer_container
        text { text }
            ..badge_variant.text
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
    prop background_color = 0x2196F3
    prop text_color = 0xFFFFFF
    prop hover_background = 0x1976D2

    group container {
        background { color: background_color }
    }

    group text {
        font { color: text_color }
    }

    variant primary {
        background_color = 0x2196F3
        text_color = 0xFFFFFF
        hover_background = 0x1976D2
    }
}
```

**resources/themes/button-dark.theme:**

```frel
theme Button {
    prop background_color = 0x90CAF9
    prop text_color = 0x000000
    prop hover_background = 0x64B5F6

    group container {
        background { color: background_color }
    }

    group text {
        font { color: text_color }
    }

    variant primary {
        background_color = 0x90CAF9
        text_color = 0x000000
        hover_background = 0x64B5F6
    }
}
```

### Automatic Resolution

The appropriate theme is automatically selected based on the [environment](../20_reactive_state/60_standard_sources.md#environment):

```frel
blueprint ThemedButton(text: String) {
    // Automatically resolves to light or dark variant
    button {
        ..Themes.Button.primary.container
        text { text }
            ..Themes.Button.primary.text
    }

    // When environment.appearance changes, the theme automatically updates
}
```

### Reactive Theme Switching

```frel
blueprint App() {
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
```

## Theme Composition

Themes can reference other themes and resources for consistency.

### Referencing Other Themes

```frel
theme DesignSystem {
    prop control_height = 32
    prop border_radius = 4
    prop spacing_unit = 8
}

theme TextInput {
    // Reference properties from another theme
    prop height = Themes.DesignSystem.control_height
    prop corner_radius = Themes.DesignSystem.border_radius
    prop padding_horizontal = Themes.DesignSystem.spacing_unit * 1.5

    // Input-specific properties
    prop font_size = 14
    prop border_color = 0xE0E0E0

    group container {
        height { height }
        corner_radius { corner_radius }
        padding { horizontal: padding_horizontal }
        border { color: border_color }
    }

    group text {
        font { size: font_size }
    }

    variant default {
        border_color = 0xE0E0E0
    }
}
```

### Referencing Resources

```frel
theme Input {
    // Reference graphics
    prop clear_icon = Graphics.close
    prop search_icon = Graphics.search

    // Reference colors (if you have a color resource system)
    prop border_color = Colors.outline
    prop focus_color = Colors.primary

    group container {
        border { color: border_color }
    }

    group prefix_icon {
        width { 16 }
        height { 16 }
    }

    variant default {
        border_color = Colors.outline
    }
}
```

### Usage

```frel
blueprint SearchInput(query: String) {
    row {
        ..Themes.Input.default.container

        // Use theme's icon reference
        icon { Themes.Input.search_icon }
            ..Themes.Input.default.prefix_icon

        text_input(value: query) { }
    }
}
```

## Themes as Shared Sources

Like all [resources](60_resources.md#resources-as-shared-sources), themes are **shared sources**:

- **Global lifecycle:** Created once during application initialization, not per-fragment
- **Shared subscriptions:** All fragments accessing the same theme subscribe to the same underlying source
- **Reactive updates:** When the [environment](../20_reactive_state/60_standard_sources.md#environment) changes (e.g., appearance switch), themes automatically re-resolve and notify subscribers
- **Status tracking:** Themes have `Loading`, `Ready`, or `Error` status (accessible via `.status()`)
- **Optional values:** Themes provide `.latest()` to access the current value

### Explicit Source Handling

```frel
blueprint ThemedComponent() {
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
```

### Status Checking

```frel
blueprint SafeThemedComponent() {
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
```

## Generated Theme API

The theme compiler generates a type-safe API for each theme.

### For a Theme Definition

**Input (badge.theme):**

```frel
theme Badge {
    prop height = 20
    prop corner_radius = 4
    prop border_color = 0x000000

    group outer_container { ... }
    group icon { ... }
    group text { ... }

    variant success { ... }
    variant error { ... }
    variant warning { ... }
}
```

### Generated Module Structure

**Output (generated code):**

```rust
// Top-level theme namespace
pub mod Badge {
    // Default variant (first variant)
    pub fn default() -> &'static BadgeThemeVariant {
        &success
    }

    // Base theme properties (defaults)
    pub const BASE_HEIGHT: DPixel = 20.dp;
    pub const BASE_CORNER_RADIUS: DPixel = 4.dp;

    // Named variant accessors (static)
    pub mod success {
        // Variant properties (resolved with defaults)
        pub const height: DPixel = 20.dp;
        pub const corner_radius: DPixel = 4.dp;
        pub const border_color: Color = 0x4CAF50;

        // Instruction groups (evaluated with variant properties)
        pub static outer_container: InstructionGroup = evaluate_with_props(...);
        pub static icon: InstructionGroup = evaluate_with_props(...);
        pub static text: InstructionGroup = evaluate_with_props(...);
    }

    pub mod error {
        pub const height: DPixel = 20.dp;
        pub const corner_radius: DPixel = 4.dp;
        pub const border_color: Color = 0xF44336;

        pub static outer_container: InstructionGroup = evaluate_with_props(...);
        pub static icon: InstructionGroup = evaluate_with_props(...);
        pub static text: InstructionGroup = evaluate_with_props(...);
    }

    // Dynamic variant lookup
    pub fn variant(name: &str) -> Option<&'static BadgeThemeVariant> {
        match name {
            "success" => Some(&success),
            "error" => Some(&error),
            "warning" => Some(&warning),
            _ => None,
        }
    }

    // Shared source implementation
    impl Resource for BadgeThemeVariant {
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
    group container { ... }
    group header { ... }
    group body { ... }
    group footer { ... }

    // Avoid: kitchen sink group
    group everything { /* too many unrelated instructions */ }
}
```

### 3. Define Semantic Variants

```frel
theme Alert {
    prop background_color = 0xFFFFFF
    prop border_color = 0x000000
    prop icon = Graphics.info

    // Good: semantic names
    variant success { ... }
    variant error { ... }
    variant warning { ... }
    variant info { ... }

    // Good: hierarchy names
    variant primary { ... }
    variant secondary { ... }
    variant tertiary { ... }
}
```

### 4. Compose and Reuse

```frel
theme DesignSystem {
    prop control_height = 32
    prop border_radius = 4
}

theme Button {
    // Reuse DesignSystem properties
    prop height = Themes.DesignSystem.control_height
    prop corner_radius = Themes.DesignSystem.border_radius
}

theme Input {
    // Reuse DesignSystem properties
    prop height = Themes.DesignSystem.control_height
    prop corner_radius = Themes.DesignSystem.border_radius
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
    prop background_color = ...
    prop text_color = ...
    prop hover_background = ...
}

theme Input {
    prop background_color = ...
    prop text_color = ...
    prop focus_background = ...
}

// Good: consistent instruction group names
theme Card {
    group container { ... }
    group header { ... }
    group body { ... }
}

theme Modal {
    group container { ... }
    group header { ... }
    group body { ... }
}
```

### 7. Declare All Properties at Theme Level

```frel
// Good: all properties declared at theme level
theme Button {
    prop height = 32
    prop background_color = 0xE0E0E0
    prop text_color = 0x000000

    variant primary {
        background_color = 0x2196F3  // override only
        text_color = 0xFFFFFF        // override only
    }
}

// Bad: variant tries to add new property
theme Button {
    prop height = 32

    variant primary {
        height = 32
        new_prop = 123  // ERROR: not declared at theme level
    }
}
```

## Complete Example

**resources/themes/input-light.theme:**

```frel
theme Input {
    // Dimensions
    prop height = 32
    prop border_radius = 4
    prop padding_horizontal = 12
    prop font_size = 14

    // Colors (base defaults)
    prop background_color = 0xFFFFFF
    prop border_color = 0xE0E0E0
    prop text_color = 0x000000
    prop placeholder_color = 0x9E9E9E
    prop focus_border_color = 0x2196F3
    prop error_border_color = 0xF44336

    // Resources
    prop clear_icon = Graphics.close

    // Instruction groups (shared by all variants)
    group container {
        height { height }
        corner_radius { border_radius }
        padding { horizontal: padding_horizontal }
        background { color: background_color }
        border { color: border_color }
    }

    group text {
        font { size: font_size color: text_color }
    }

    group placeholder {
        font { size: font_size color: placeholder_color }
    }

    group clear_button {
        width { 16 }
        height { 16 }
        cursor { pointer }
    }

    // Variants (property overrides only)
    variant default {
        border_color = 0xE0E0E0
    }

    variant focused {
        border_color = 0x2196F3
    }

    variant error {
        border_color = 0xF44336
    }
}
```

**Usage in blueprint:**

```frel
blueprint TextInput(value: String, error: Option<String>) {
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
```

## See Also

- [Resources](60_resources.md) - Resource system overview
- [Standard Sources](../20_reactive_state/60_standard_sources.md#environment) - Environment source for theme qualification
- [Instructions](../../../10_language/70_blueprints/60_instructions.md) - Available styling instructions
- [Store Declarations](../20_reactive_state/10_store_basics.md#data-source) - Source system
