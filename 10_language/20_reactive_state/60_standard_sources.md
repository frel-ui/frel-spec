# Standard Sources

## Environment

### Environment Context

`source env = environment()`

The `environment()` source provides access to the current runtime environment context. 
It is a **shared global source** that is initialized during application startup and updates when
environment properties change.

**Properties:**

```rust
struct Environment {
    language: String,        // ISO 639-1/639-2 language code (e.g., "en", "hu")
    region: Option<String>,  // ISO 3166-1-alpha-2 country code (e.g., "US", "GB")
    appearance: Appearance,  // Light | Dark appearance
    density: Density,        // Ldpi | Mdpi | Hdpi | Xhdpi | Xxhdpi | Xxxhdpi
    locale: String,          // Full locale string (e.g., "en-US", "hu-HU")
}

enum Appearance {
    Light,
    Dark,
}

enum Density {
    Ldpi,    // 0.75x
    Mdpi,    // 1.0x
    Hdpi,    // 1.5x
    Xhdpi,   // 2.0x
    Xxhdpi,  // 3.0x
    Xxxhdpi, // 4.0x
}
```

**Usage:**

```frel
fragment AppearanceAwareIcon(icon_name: String) {
    source env = environment()

    decl current_appearance = env.latest().map(|e| e.appearance).unwrap_or(Appearance::Light)

    select on current_appearance {
        Appearance::Light => icon { Graphics.lookup(icon_name + "_light") }
        Appearance::Dark => icon { Graphics.lookup(icon_name + "_dark") }
    }
}
```

**Initialization:**

The environment source must be initialized during application startup:

```rust
fn main() {
    // Initialize environment with current system settings
    Environment::init(EnvironmentConfig {
        language: detect_system_language(),
        region: detect_system_region(),
        appearance: detect_system_appearance(),
        density: detect_screen_density(),
    });

    Application::run();
}
```

**Dynamic Updates:**

The environment can be updated at runtime (e.g., when user changes preferences):

```rust
// Reactively updates all fragments using environment()
Environment::update(|env| {
    env.appearance = Appearance::Dark;
});
```

**Shared Source Behavior:**

- **Global lifecycle:** Created once during initialization, not per-fragment
- **Shared subscriptions:** All fragments using `environment()` subscribe to the same source
- **Reactive propagation:** Changes propagate to all subscribers automatically
- **Always ready:** Status is always `Ready` after initialization (no loading state)

**Resource Integration:**

The environment source drives [resource qualification](../60_resources.md#qualifier-resolution). 
When the environment changes, all qualified resources are re-resolved automatically.

## UI State-Related

### Focus

`source focused = focus()`

The `focus()` source produces a `bool` value that indicates whether the fragment or
**any of its children** has the focus.

**Lifecycle:** Created per-fragment, tracks focus state for that specific fragment instance.

**Example:**

```frel
fragment Input() {
    source focused = focus()
    writable text = ""

    decl border_color = if focused.latest().unwrap_or(false) { Blue } else { Gray }

    text_input { text }
        .. border { color: border_color }
}
```

### Hover

`source hovered = hover()`

The `hover()` source produces a `bool` value that indicates whether the fragment or
**any of its children** is hovered.

**Lifecycle:** Created per-fragment, tracks hover state for that specific fragment instance.

**Example:**

```frel
fragment Button() {
    source hovered = hover()

    decl bg_color = if hovered.latest().unwrap_or(false) { LightGray } else { White }

    box {
        background { color: bg_color }
        padding { 8 }
        text { "Hover me" }
    }
}
```
