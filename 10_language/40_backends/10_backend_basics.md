# Backends

Backends are **reactive state containers with behavior**. They encapsulate related stores (decl, writable, fanin, source) along with commands and lifecycle hooks that operate on those stores. Backends separate business logic from UI declarations, following the Model-View-Controller pattern where the fragment is the view and the backend is the controller + model.

Conceptually, a backend is not a single store with a value, but rather a structured collection of reactive state with associated operations. When backend stores change, fragments that depend on them automatically update.

## Syntax

```text
<backend> ::= "backend" <name> [ "(" <param-list> ")" ] "{" <backend-body> "}"
<param-list> ::= <param> { "," <param> }
<param> ::= [<store-kind>] <param-name> ":" <param-type> [ "=" <default-expr> ]
<store-kind> ::= "writable" | "source" | "decl" | "fanin"

<backend-body> ::= { <uses-clause> | <include-clause> | <store-decl> | <lifecycle-hook> | <command-decl> }

<uses-clause>    ::= "uses" <contract-name>
<include-clause> ::= "include" <backend-name> [ "{" <include-config> "}" ]
<include-config> ::= <name> "=" <expr> { "," <name> "=" <expr> }
<lifecycle-hook> ::= "on_init" | "on_cleanup"
<command-decl>   ::= "command" <name> "(" [ <param-list> ] ")" [ "->" <return-type> ]
```

Store declarations (`<store-decl>`) use the same syntax as in fragments. See [Reactive State](../20_reactive_state/10_store_basics.md) for the complete definition.

## Semantics

- **Parameters**: Constructor parameters for backend initialization
- **Uses**: Declares dependency on contracts (external services)
- **Include**: Composes other backends for reusable logic
- **Store declarations**: Reactive stores exposed to the fragment (see [Store Types](#store-types))
- **Lifecycle hooks**: Declarations indicating the backend has initialization/cleanup behavior (implemented in host language)
- **Commands**: Async method signatures callable from fragment event handlers (implemented in host language)

### Parameters

**Definition**

Each `<param>` declares a reactive store that the backend receives at construction time.
Parameters enable reactive data flow and store sharing between backends.

**Store Kind**

Parameters can specify what kind of store they accept. For detailed information
see [Store Basics](../20_reactive_state/10_store_basics.md).

When no store kind is specified, `decl` is assumed (read-only reactive).

All parameters are reactive - changes propagate automatically to/from the backend.

## Store Types

Backends support four types of reactive stores. Each store type has the same semantics as in fragments:

- **`decl`** - [Read-only stores](../20_reactive_state/20_read_only_stores.md): Derived values that automatically update when dependencies change
- **`writable`** - [Writable stores](../20_reactive_state/30_writable_stores.md): Mutable state modified by commands
- **`fanin`** - [Fan-in stores](../20_reactive_state/40_fan_in_stores.md): Reactive mirroring with manual override capability
- **`source`** - [Sources](../20_reactive_state/50_sources.md): Async data producers and external event streams

See the [Reactive State](../20_reactive_state/10_store_basics.md) documentation for detailed semantics, rules, and usage patterns.

## Declaration

```frel
backend UserEditor(user_id: u32) {
    uses UserAPI

    include DirtyTrackingBackend

    writable name: String = ""
    writable email: String = ""
    writable original_name: String = ""
    writable original_email: String = ""

    // Derived state automatically updates (pure Frel expression)
    decl has_changes = name != original_name || email != original_email

    // Async data source
    source user_data = fetch(|| api.get_user(user_id))

    // Lifecycle hooks (declarations only)
    on_init
    on_cleanup

    // Commands (declarations only - implementation in host language)
    command load_user()
    command save()
    command cancel()
}
```

Host language implementation (example in Rust):

```rust
#[async_trait]
impl UserEditor for UserEditorImpl {
    async fn on_init(&mut self) {
        self.load_user().await;
    }

    async fn on_cleanup(&mut self) {
        if self.has_changes.get() {
            // Maybe prompt user to save?
        }
    }

    async fn load_user(&mut self) {
        let api = get_contract::<UserAPI>();
        if let Ok(user) = api.get_user(self.user_id).await {
            self.name.set(user.name.clone());
            self.email.set(user.email.clone());
            self.original_name.set(user.name);
            self.original_email.set(user.email);
        }
    }

    async fn save(&mut self) {
        let api = get_contract::<UserAPI>();
        // Implementation...
    }

    async fn cancel(&mut self) {
        self.name.set(self.original_name.get());
        self.email.set(self.original_email.get());
    }
}
```

**Rules:**

- Backend names use PascalCase
- Parameters use the same syntax as fragment parameters
- Store type keyword (`decl`, `writable`, `fanin`, `source`) is required
- Store initializers use pure Frel expressions
- Type annotations can be inferred for `decl`, required for others
- Commands declare only the signature (implementation in host language)
- Lifecycle hooks are declarations only (implementation in host language)

## Store Exposure

All store declarations become reactive stores accessible from the fragment:

```frel
fragment UserProfile(user_id: u32) {
    with UserEditor(user_id)

    // Access writable stores
    text_editor { name }
    text_editor { email }

    // Access derived stores
    when has_changes {
        text { "Unsaved changes" } .. color { Orange }
    }

    // Access source stores
    select on user_data.status() {
        Status::Loading => spinner { }
        Status::Ready => text { user_data.latest().unwrap().display_name }
        Status::Error(e) => text { "Error: ${e}" }
    }
}
```

## Commands

Commands are async methods callable from event handlers:

```frel
fragment UserProfile(user_id: u32) {
    with UserEditor(user_id)

    button { "Save" }
        .. enabled { has_changes }
        .. on_click { save() }
}
```

## Lifecycle Hooks

Lifecycle hooks are **declarations only** - they indicate that the backend has lifecycle behavior, but the implementation is written entirely in the host language.

### on_init

Declares that the backend has initialization logic (executed when fragment is mounted):

```frel
backend UserEditor(user_id: u32) {
    writable data: User

    // Declaration only - no body in Frel
    on_init

    command load_user()
}
```

Host language implementation:

```rust
impl UserEditor {
    async fn on_init(&mut self) {
        self.load_user().await;
    }

    async fn load_user(&mut self) {
        // Implementation...
    }
}
```

### on_cleanup

Declares that the backend has cleanup logic (executed when fragment is unmounted):

```frel
backend UserEditor(user_id: u32) {
    writable is_dirty: bool = false

    // Declaration only - no body in Frel
    on_cleanup

    command save_draft()
}
```

Host language implementation:

```rust
impl UserEditor {
    async fn on_cleanup(&mut self) {
        if self.is_dirty.get() {
            self.save_draft().await;
        }
    }

    async fn save_draft(&mut self) {
        // Implementation...
    }
}
```

## Using Contracts

Backends declare contract dependencies with `uses`:

```frel
backend UserEditor(user_id: u32) {
    uses UserAPI
    uses ValidationService

    writable user: User

    command save()
}
```

In the implementation, contracts are accessed via the global registry:

```rust
async fn save(&self) {
    let api = get_contract::<UserAPI>();
    let result = api.update_user(self.user_id, self.user.get()).await;
    // ...
}
```

## Including Other Backends

Backends can compose other backends using `include`. This allows for reusable behavior and automatic registry creation.

### Basic Include

```frel
backend DirtyTrackingBackend {
    writable is_dirty: bool = false

    command mark_dirty()
    command mark_clean()
}

backend UserEditor(user_id: u32) {
    uses UserAPI

    include DirtyTrackingBackend

    writable name: String = ""

    command update_name(value: String)
}
```

Included backends are accessible from the fragment:

```frel
fragment UserProfile(user_id: u32) {
    with UserEditor(user_id)

    // Access main backend state
    text { name }

    // Access included backend state
    when is_dirty {
        text { "Unsaved changes" }
    }
}
```

### Include with Configuration

Backends can be included with initial configuration values:

```frel
backend ToolPane {
    writable position: PanePosition
    writable icon: Graphic
    writable title: String
    writable visible: bool = true

    writable content_fragment: Fragment

    command toggle()
}

backend GitPane {
    include ToolPane {
        position = PanePosition::Right
        icon = Graphics.git
        title = Strings.git_panel
        content_fragment = GitContent
    }

    // Git-specific state
    source branches = fetch(|| api.get_branches())

    command refresh()
}
```

The configuration block sets the initial values for the included backend's state fields.

### Multiple Includes

Backends can include multiple other backends to compose orthogonal behaviors:

```frel
backend NavTarget {
    writable route: Route
    writable title: String
    writable icon: Graphic
}

backend EditorPane(file_path: String) {
    include ToolPane {
        position = PanePosition::Right
        icon = Graphics.file
        title = file_path
        content_fragment = EditorContent
    }

    include NavTarget {
        route = Route::editor(file_path)
        title = file_path
        icon = Graphics.file
    }

    // Editor-specific state
    writable content: String = ""
    writable language: String = "text"

    command save()
}
```

### Auto-Generated Collections

When a backend includes multiple backends of the same type, the compiler automatically generates collection methods:

```frel
backend AppWorkspace {
    include GitPane
    include TerminalPane
    include DebugPane

    // Compiler auto-generates:
    // fn tool_panes(&self) -> Vec<&dyn ToolPane>
    // Returns all included backends that implement ToolPane
}

fragment ToolPanelArea {
    backend workspace = AppWorkspace()

    // Iterate over all tool panes automatically
    column {
        repeat on workspace.tool_panes() as tool {
            when tool.visible {
                ToolPaneChrome(
                    title: tool.title,
                    icon: tool.icon
                ) {
                    tool.content_fragment
                }
            }
        }
    }
}
```

This pattern enables registry creation through composition without explicit registration code.

## Generated Code

The backend declaration generates a trait with typed store accessors:

```rust
#[async_trait]
pub trait UserEditor: Backend {
    // Parameters
    fn user_id(&self) -> u32;

    // Store accessors - typed by store kind
    fn name(&self) -> &WritableStore<String>;
    fn email(&self) -> &WritableStore<String>;
    fn has_changes(&self) -> &ReadOnlyStore<bool>;
    fn user_data(&self) -> &Source<User>;

    // Included backends
    fn dirty_tracking(&self) -> &DirtyTrackingBackendImpl;

    // Lifecycle
    async fn on_init(&self);
    async fn on_cleanup(&self);

    // Commands
    async fn load_user(&self);
    async fn save(&self);
    async fn cancel(&self);
}
```

**Store type mapping:**
- `writable` → `WritableStore<T>`
- `decl` → `ReadOnlyStore<T>`
- `fanin` → `FanInStore<T>`
- `source` → `Source<T>`

User provides the implementation in the host language.

## Example

```frel
contract PointAPI {
    get_point(id: u32) -> PointConfig
    update_point(id: u32, data: PointConfig) -> PointConfig
}

backend PointConfigBackend(point_id: u32, mode: EditMode) {
    uses PointAPI

    include DirtyTrackingBackend

    writable current_page: u32 = 1
    writable point_data: PointConfig = PointConfig::default()

    // Native async data sources
    source load_result = fetch(|| api.get_point(point_id))
    source save_result = fetch(|| api.update_point(point_id, point_data))

    // Derived state
    decl is_edit_mode = mode == EditMode::Edit
    decl can_save = is_dirty && !save_result.is_pending()

    on_init {
        when is_edit_mode {
            self.load_point();
        }
    }

    command load_point()
    command save()
    command next_page()
    command previous_page()
}

fragment PointConfigDialog(point_id: u32, mode: EditMode) {
    with PointConfigBackend(point_id, mode)

    column {
        text { "Page ${current_page}" }

        button { "Next" } .. on_click { next_page() }
        button { "Save" }
            .. enabled { can_save }
            .. on_click { save() }

        when save_result.is_pending() {
            spinner { }
        }
    }
}
```
