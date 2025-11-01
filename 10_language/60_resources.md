# Resources

The resource subsystem provides declarative access to application assets such as documents, files,
fonts, images, graphics, and strings. Resources are implemented as **shared sources** that integrate
seamlessly with the reactive store system and support qualifiers for multi-environment deployment.

## Goals

1. **Declarative Definition:** Resources are declared once and accessed through type-safe identifiers.
2. **Platform Independence:** Resource handling is separated from platform-specific loading mechanisms.
3. **Qualifier Support:** Resources adapt to runtime context (language, density, theme, etc.).
4. **Reactive Integration:** Resources are shared sources that integrate naturally with stores and expressions.
5. **Compile-Time Safety:** Invalid resource references produce compile-time errors.
6. **Dynamic Access:** Resources can be accessed both statically (compile-time safe) and dynamically (runtime lookup).

## Conceptual Model

```text
  Source Files       Compile Time         Runtime (Shared Sources)

  ┌──────────┐       ┌──────────┐          ┌────────────────────┐
  │ Resource │   →   │ Resource │     →    │ Shared Sources +   │
  │ Files    │       │ Registry │          │ Qualifier Resolver │
  └──────────┘       └──────────┘          └────────────────────┘
                                                      ↓
├─────────────────────────────────┤     ├────────────────────────┤
  Processed by resource compiler          Reactive in fragments
```

**Resource Processing:**

1. **Source Files:** Raw assets organized by type and qualifiers
2. **Resource Compiler:** Processes resources into registry during build
3. **Resource Registry:** Backend-neutral catalog of available resources with metadata
4. **Shared Sources:** Resources are global sources shared across all fragments
5. **Qualifier Resolver:** Selects appropriate resource variant based on [environment context](20_reactive_state/60_standard_sources.md#environment)

## Resources as Shared Sources

Resources are implemented as **shared sources** (see [Store Declarations - Data source](20_reactive_state/10_store_basics.md#data-source)):

- **Global lifecycle:** Resources are created once during application initialization, not per-fragment
- **Shared subscriptions:** All fragments accessing the same resource subscribe to the same underlying source
- **Status tracking:** Resources have `Loading`, `Ready`, or `Error` status (accessible via `.status()`)
- **Optional values:** Resources provide `.latest()` to access the current value
- **Reactive updates:** When the [environment](20_reactive_state/60_standard_sources.md#environment) changes (e.g., appearance or locale), resources automatically re-resolve and notify subscribers
- **Async loading:** Resources may load asynchronously (network fetch, file I/O) without blocking fragments

**Usage as sources:**

```frel
fragment ProfileImage(user_id: u32) {
    // Resource as a source - can check status and value
    decl avatar_status = Images.avatar.status()
    decl avatar = Images.avatar.latest()

    select on avatar_status {
        Status::Loading => spinner { }
        Status::Error(e) => text { "Failed to load image" }
        Status::Ready => {
            when avatar.is_some() {
                image { avatar.unwrap() }
            }
        }
    }
}
```

## Resource Types

| Type       | Description                         | Typical Extensions      | Example Usage                       |
|------------|-------------------------------------|-------------------------|-------------------------------------|
| Document   | Structured text (Markdown)          | `.md`                   | Help pages, documentation           |
| File       | Raw binary or text files            | `.pdf`, `.json`, `.csv` | Terms of service, config            |
| Font       | Font files                          | `.ttf`, `.woff2`        | Custom typography                   |
| Image      | Raster graphics                     | `.png`, `.jpg`, `.webp` | Photos, backgrounds                 |
| Graphic    | Vector graphics                     | `.svg`                  | Icons, illustrations                |
| String     | Localized text                      | `.xml`                  | UI labels, messages                 |
| Theme      | Component styling configurations    | `.theme` (DSL)          | Button themes, input field themes   |

> [!NOTE]
>
> For detailed information about theme definitions, qualifiers, and usage, see [Themes](70_themes.md).
>

## Directory Structure

Resources are organized in a `resources` directory within each source module:

```text
src/
  resources/
    documents/
      help.md
    files/
      terms-en.pdf
      terms-hu.pdf
    fonts/
      custom_font.ttf
    images/
      background.png
      background-hdpi.png
    graphics/
      icon-light.svg
      icon-dark.svg
    strings/
      strings-en.xml
      strings-hu.xml
    themes/
      badge.theme
      button.theme
      input-light.theme
      input-dark.theme
```

### Organization Strategies

**By Type (Recommended for Small Projects):**

```text
resources/
  documents/
    intro.md
  files/
    terms-en.pdf
    terms-hu.pdf
  fonts/
    custom_font.ttf
  images/
    background-hdpi.jpg
    background-mdpi.jpg
  graphics/
    icon-light.svg
    icon-dark.svg
  strings/
    strings-en.xml
    strings-hu.xml
```

**By Environment (Recommended for Large Projects):**

```text
resources/
  en/
    files/
      terms.pdf
    strings/
      strings.xml
  hu/
    files/
      terms.pdf
    strings/
      strings.xml
  hdpi/
    images/
      background.jpg
  mdpi/
    images/
      background.jpg
  graphics/  # Resolution independent
    icon.svg
```

## Resource Access

Resources can be accessed in two ways: **statically** (compile-time safe) or **dynamically** (runtime lookup).

### Static Access

Static access provides compile-time safety and is the primary way to use resources:

```frel
fragment AppMain() {
    column {
        // Document resource
        doc { Documents.help }

        // External link to file
        link(url: Files.terms) { "Terms of Service" }

        // Font is loaded through font instruction
        text { "Custom Font" } .. font { name: Fonts.custom_font }

        // Vector graphic
        icon { Graphics.add }

        // Raster image
        image { Images.background }

        // Localized string
        text { Strings.welcome }
    }
}
```

**Resource Namespaces:**

The namespaces provide type-safe access:

```text
Documents.<name>
Files.<name>
Fonts.<name>
Graphics.<name>
Images.<name>
Strings.<name>
Themes.<name>
```

Each resource identifier (e.g., `Images.background`) is a **shared source** that:
- Resolves to the appropriate qualified variant based on current [environment](20_reactive_state/60_standard_sources.md#environment)
- Can be used directly in expressions (automatically uses `.latest().unwrap()`)
- Can be explicitly checked for status via `.status()` and `.latest()`

### Dynamic Access

Dynamic access allows runtime resource lookup by string key:

```frel
fragment CountryFlag(country_code: String) {
    // Dynamic lookup by key
    decl flag_resource = Images.lookup("flag_" + country_code.to_lowercase())

    // Check if resource exists and is ready
    decl flag_status = flag_resource.status()
    decl flag = flag_resource.latest()

    select on flag_status {
        Status::Ready => {
            when flag.is_some() {
                image { flag.unwrap() }
            }
            else {
                image { Images.flag_default }  // Fallback
            }
        }
        Status::Loading => spinner { }
        Status::Error(_) => image { Images.flag_default }
    }
}
```

**Lookup Signature:**

```rust
<ResourceType>.lookup(key: &str) -> Resource<T>
```

Where `<ResourceType>` is one of: `Documents`, `Files`, `Fonts`, `Graphics`, `Images`, `Strings`, `Themes`.

**Behavior:**

- Returns a `Resource<T>` (shared source) for the given key
- Applies qualifier resolution based on current environment
- Status is `Ready` with `Some(value)` if resource exists
- Status is `Ready` with `None` if resource key doesn't exist (not an error)
- Status is `Error` only if loading fails for an existing resource

**Common Use Cases:**

1. **Dynamic icon/image selection:**
   ```rust
   decl icon = Graphics.lookup(icon_name + "_" + theme)
   ```

2. **User-generated content:**
   ```rust
   decl avatar = Images.lookup("avatar_" + user_id.to_string())
   ```

3. **Country/language-specific assets:**
   ```rust
   decl flag = Images.lookup("flag_" + country_code)
   ```

**Static vs Dynamic Comparison:**

| Aspect           | Static (`Images.avatar`)       | Dynamic (`Images.lookup("avatar")`)     |
|------------------|--------------------------------|-----------------------------------------|
| Safety           | Compile-time checked           | Runtime checked                         |
| Performance      | Direct access (no lookup)      | Hash map lookup                         |
| Existence        | Guaranteed to exist            | May not exist (returns `None`)          |
| Use Case         | Known resources at build time  | Dynamic resource selection              |
| Autocomplete     | Full IDE support               | String-based (no autocomplete)          |

## Qualifiers

Qualifiers enable context-aware resource selection. Resources without qualifiers serve as fallbacks.

### Qualifier Syntax

Qualifiers can be specified in:

1. **Directory names:** `resources/en/strings/`
2. **File names:** `resources/strings/strings-en.xml`
3. **Combinations:** `resources/en-US/files/terms-hdpi.pdf`

Qualifiers are separated by `-` (hyphen) in file names and by `/` (directory separator) in paths.

### Supported Qualifiers

#### Language and Region

- **Language:** ISO 639-1 (2-letter) or ISO 639-2 (3-letter) code
  - Examples: `en`, `hu`, `eng`
- **Region:** ISO 3166-1-alpha-2 with `r` prefix
  - Examples: `en-rUS`, `en-rGB`, `hu-rHU`

**Note:** Language and region codes are case-sensitive.

#### Appearance

- `light` → Light appearance
- `dark` → Dark appearance

#### Density (for raster images)

| Qualifier | DPI | Density |
|-----------|-----|---------|
| `ldpi`    | 120 | 0.75x   |
| `mdpi`    | 160 | 1.0x    |
| `hdpi`    | 240 | 1.5x    |
| `xhdpi`   | 320 | 2.0x    |
| `xxhdpi`  | 480 | 3.0x    |
| `xxxhdpi` | 640 | 4.0x    |

**Note:** Vector graphics (SVG) are resolution-independent and do not need density qualifiers.

### Qualifier Examples

```text
resources/
  strings/
    strings.xml           # Default fallback
    strings-en.xml        # English
    strings-hu.xml        # Hungarian
    strings-en-rUS.xml    # US English
    strings-en-rGB.xml    # British English

  graphics/
    icon.svg              # Default
    icon-light.svg        # Light theme
    icon-dark.svg         # Dark theme

  images/
    background.png        # Default fallback
    background-mdpi.png   # Medium density
    background-hdpi.png   # High density

  files/
    terms.pdf             # Default
    terms-en.pdf          # English version
    terms-hu.pdf          # Hungarian version
```

### Qualifier Resolution

Qualifier resolution is driven by the [environment source](20_reactive_state/60_standard_sources.md#environment):

1. The [environment](20_reactive_state/60_standard_sources.md#environment) provides **qualifier context** (language, theme, density, etc.)
2. Resource system selects the **best match** based on available resources and current environment
3. If no qualified resource matches, the **default (unqualified) resource** is used
4. If no default exists, a **compile-time error** is produced

**Reactive Resolution:**

When the environment changes (e.g., user switches from light to dark appearance), all resources automatically re-resolve and notify their subscribers:

```frel
fragment ThemedBackground() {
    // Automatically updates when environment.appearance changes
    box { } .. background { image: Images.background }
}
```

**Resolution Order Example:**

Environment: `language: "en"`, `region: Some("US")`, `appearance: Dark`, `density: Xhdpi`

```text
1. background-en-rUS-dark-xhdpi.png    ← Most specific
2. background-en-rUS-dark.png
3. background-en-dark-xhdpi.png
4. background-en-dark.png
5. background-dark-xhdpi.png
6. background-dark.png
7. background-xhdpi.png
8. background.png                      ← Fallback (unqualified)
```

**Resolution Algorithm:**

1. **Exact match:** All qualifiers match
2. **Partial match:** Most qualifiers match (scored by number of matches)
3. **Default fallback:** No qualifiers specified
4. **Compile error:** No default exists (caught at build time)

## Strings

String resources are stored in XML files with the same format as Android string resources:

```xml
<resources>
    <string name="app_name">Application Name</string>
    <string name="welcome">Welcome!</string>
    <string name="greeting">Hello, ${name}!</string>
    <string name="item_count">You have ${count} items</string>
</resources>
```

### String Interpolation

Strings support variable interpolation using `${variable}` syntax:

```frel
fragment Greeting(username: String) {
    decl count = 42

    // Simple string
    text { Strings.welcome }

    // String with interpolation
    text { Strings.greeting.format(name: username) }

    // Multiple variables
    text { Strings.item_count.format(count: count) }
}
```

**String Accessor Signature:**

```rust
// Simple string (as shared source)
Strings.<name>: Resource<&str>

// String with interpolation
Strings.<name>.format(param1: T1, param2: T2, ...) -> String
```

**String Loading:**

Like all resources, strings are **shared sources** that load during application initialization. The resource system handles loading automatically based on the current environment:

```rust
fn main() {
    // Initialize environment (which triggers resource loading)
    Environment::init(EnvironmentConfig {
        language: "en",
        region: Some("US"),
        appearance: Appearance::Light,
        density: Density::Hdpi,
    });

    // Strings are now loaded and ready
    Application::run();
}
```

**Reactive String Updates:**

When the environment language/region changes, string resources automatically reload and notify subscribers:

```frel
fragment LanguageSwitcher() {
    // Automatically updates when environment.language changes
    text { Strings.welcome }

    button {
        text { "Switch to Hungarian" }
        on_click {
            Environment::update(|env| {
                env.language = "hu";
            });
        }
    }
}
```

**Direct Usage:**

For most cases, you can use strings directly (the `.latest().unwrap()` is implicit):

```frel
fragment SomeFragment() {
    text { Strings.welcome }  // Automatically unwraps
}
```

**Explicit Status Checking:**

If you need to handle loading states explicitly:

```frel
fragment SomeFragment() {
    decl welcome_status = Strings.welcome.status()
    decl welcome_text = Strings.welcome.latest()
  
    select on welcome_status {
        Status::Loading => text { "Loading..." }
        Status::Ready => text { welcome_text.unwrap_or("Welcome") }
        Status::Error(_) => text { "Welcome" }  // Fallback
    }
}
```

## Resource Configuration

Resource processing can be configured in the build system:

```text
// build.rs or equivalent configuration

ResourceConfig {
    // Base directory for resources (relative to module root)
    resource_dir: "resources",

    // Whether to embed resources in the binary
    embed: true,

    // Whether file names may contain qualifiers
    // true: "icon-dark.svg" → qualifiers in name
    // false: "icon-dark.svg" → literal name
    with_file_qualifiers: true,

    // Generate public or internal accessors
    public_accessors: false,

    // Target directory in output artifact
    output_dir: "assets",
}
```

### Configuration Options

| Option                 | Description                                        | Default      |
|------------------------|----------------------------------------------------|--------------|
| `resource_dir`         | Source directory for resources                     | `resources`  |
| `embed`                | Embed resources in binary vs. external files       | `true`       |
| `with_file_qualifiers` | Parse qualifiers from file names                   | `true`       |
| `public_accessors`     | Make resource accessors public                     | `false`      |
| `output_dir`           | Output directory in artifact (when not embedded)   | `assets`     |

## Platform-Specific Considerations

### WebAssembly

Resources are embedded in the Wasm binary or fetched via HTTP. The resource system abstracts these details:

- **Embedded:** Resources compiled directly into Wasm module
- **External:** Resources fetched asynchronously with proper caching

### Native (Desktop/Mobile)

Resources may be:

- Embedded in the executable
- Bundled alongside the executable
- Loaded from application data directories

### Resource URIs

Resources can be referenced by URI for external linking:

```frel
fragment TermsLink() {
    // Generates a URI to the resource
    link(url: Files.terms.uri()) { "Terms of Service" }
}
```

**URI Format:**

- **Embedded:** `resource://<namespace>/<type>/<name>`
- **External:** Platform-specific path or URL

## Error Handling

### Compile-Time Errors

| Condition                          | Error                                                |
|------------------------------------|------------------------------------------------------|
| Reference to non-existent resource | `Resource not found: Images.foo`                     |
| No default for qualified resource  | `Missing default resource: background.png`           |
| Conflicting resources              | `Duplicate resource: icon.svg in multiple locations` |
| Invalid qualifier syntax           | `Invalid qualifier: icon-invalidQual.svg`            |

### Runtime Errors

Runtime resource loading is generally error-free due to compile-time validation. Exceptions:

- **String loading failure:** Handled by `Strings.load()` return value
- **Network failure (external resources):** Retry logic or fallback to embedded resources
- **Corrupted resources:** Platform-specific error handling

## Example Usage

### Complete Fragment with Resources

```frel
fragment UserProfile(user_id: u32) {
    // Load user data
    source user = fetch(|| api.user(user_id))

    decl username = user.latest().map(|u| u.name).unwrap_or_default()
    decl is_loading = user.status() == Status::Loading

    column {
        // Header with localized string and interpolation
        text { Strings.welcome.format(name: username) }
            .. font {
                name: Fonts.heading
                size: 24
                weight: 700
            }

        // Profile image with density-aware loading
        when !is_loading {
            image { Images.avatar }
                .. width { 100 }
                .. height { 100 }
                .. corner_radius { 50 }
        }

        // Action buttons with icons
        row {
            button {
                icon { Graphics.edit }
                text { Strings.edit }
            }

            button {
                icon { Graphics.delete }
                text { Strings.delete }
            }
        }

        // Terms link
        link(url: Files.terms.uri()) {
            text { Strings.terms_of_service }
                .. underline
                .. font { color: Blue }
        }
    }
}
```

### Multi-Language Support

```text
resources/
  en/
    strings/
      strings.xml
  hu/
    strings/
      strings.xml
```

**en/strings/strings.xml:**

```xml
<resources>
    <string name="welcome">Welcome, ${name}!</string>
    <string name="edit">Edit</string>
    <string name="delete">Delete</string>
</resources>
```

**hu/strings/strings.xml:**

```xml
<resources>
    <string name="welcome">Üdvözöljük, ${name}!</string>
    <string name="edit">Szerkesztés</string>
    <string name="delete">Törlés</string>
</resources>
```

Fragment code remains unchanged; the resource system automatically selects the appropriate strings based on runtime locale.

## Implementation Notes

### Resource Compiler

The resource compiler (build-time tool):

1. **Scans** the `resources` directory recursively
2. **Parses** qualifiers from directory and file names
3. **Validates** resource completeness (default fallbacks, no conflicts)
4. **Generates** type-safe accessor code in the `Resources` namespace
5. **Packages** resources for the target platform (embedded or external)

### Resource Registry Format

The resource registry is a compile-time generated structure that provides shared sources:

```rust
// Top-level namespaces (generated by resource compiler)
pub mod Images {
    // Static resource accessor (shared source)
    pub fn background() -> &'static SharedResource<ImageData> {
        &BACKGROUND_RESOURCE
    }

    // Dynamic lookup
    pub fn lookup(key: &str) -> SharedResource<ImageData> {
        RESOURCE_REGISTRY.get(key).unwrap_or(&EMPTY_RESOURCE)
    }

    // Internal: compile-time generated shared resources
    static BACKGROUND_RESOURCE: SharedResource<ImageData> = SharedResource {
        variants: &[
            (QualifierSet { theme: Some(Theme::Dark), .. }, ImageData { .. }),
            (QualifierSet { theme: Some(Theme::Light), .. }, ImageData { .. }),
            (QualifierSet::default(), ImageData { .. }),  // Fallback
        ],
        env_subscription: OnceCell::new(),
    };

    static RESOURCE_REGISTRY: HashMap<&'static str, &'static SharedResource<ImageData>> = ..;
}

// Shared resource type (implements source interface)
pub struct SharedResource<T> {
    variants: &'static [(QualifierSet, T)],
    env_subscription: OnceCell<EnvSubscription>,
}

impl<T> SharedResource<T> {
    // Source interface methods
    pub fn latest(&self) -> Option<&T> {
        let env = Environment::current();
        Some(self.select_variant(env))
    }

    pub fn status(&self) -> Status {
        Status::Ready  // Resources loaded during init
    }

    fn select_variant(&self, env: &Environment) -> &T {
        // Qualifier matching algorithm (see below)
    }
}

pub struct QualifierSet {
    language: Option<&'static str>,
    region: Option<&'static str>,
    appearance: Option<Appearance>,
    density: Option<Density>,
}
```

### Qualifier Matching Algorithm

```rust
fn select_variant<T>(
    variants: &[(QualifierSet, T)],
    env: &Environment
) -> &T {
    // 1. Find exact match
    for (qualifiers, resource) in variants {
        if qualifiers.matches_exactly(env) {
            return resource;
        }
    }

    // 2. Find best partial match (score by number of matching qualifiers)
    let mut best_match = None;
    let mut best_score = 0;

    for (qualifiers, resource) in variants {
        let score = qualifiers.match_score(env);
        if score > best_score {
            best_score = score;
            best_match = Some(resource);
        }
    }

    if let Some(resource) = best_match {
        return resource;
    }

    // 3. Find default (no qualifiers)
    for (qualifiers, resource) in variants {
        if qualifiers.is_default() {
            return resource;
        }
    }

    // 4. Panic (should never happen due to compile-time validation)
    panic!("No default resource found")
}
```

### Shared Source Lifecycle

1. **Initialization:** During `Environment::init()`, all resource registries are created
2. **Subscription:** When a fragment accesses a resource, it subscribes to the shared source
3. **Environment Change:** When environment updates, all shared resources re-resolve and notify subscribers
4. **Lazy Loading:** Resources may load content lazily (on first access) but share the loading state
5. **Cleanup:** Shared resources persist for the application lifetime (not per-fragment)
