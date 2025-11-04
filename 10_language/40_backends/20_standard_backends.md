# Standard Backends

Standard backends are built-in backend definitions provided by the Frel standard library. Like standard fragments and standard sources, they provide common functionality that applications can use or extend.

## Workspace Backend

The **Workspace** backend provides application-level composition and management. It serves as the root container for applications, managing the top-level fragment, shared services, and global application state.

### Overview

The Workspace backend follows the same pattern as standard fragments and standard sources - it's a built-in backend definition that applications can use or extend to create their application root.

**Key responsibilities:**
- Root fragment management
- Theme and locale management
- Global application lifecycle
- Shared backend registry (via `include`)
- Application-wide state coordination

### Basic Usage

#### Using the Workspace Backend

```frel
backend MyApp {
    include Workspace {
        root_fragment = AppShell
        active_theme = Themes.default
        locale = "en"
    }
}

fragment AppShell {
    column {
        text { "Hello, World!" }
    }
}

// In main.rs
fn main() {
    let app = MyApp::new();
    Application::run(app);
}
```

#### Extending with Application Services

```frel
backend MyApp {
    include Workspace {
        root_fragment = AppShell
        active_theme = Themes.dark
        locale = "en"
    }

    // Include application-level backends
    include AuthBackend
    include RouterBackend
    include NotificationBackend

    // Application-specific state
    writable user_session: Option<UserSession>

    on_init {
        self.check_saved_session();
    }

    command check_saved_session()
    command logout()
}
```

### Workspace State

The Workspace backend provides the following state fields:

#### root_fragment

```frel
writable root_fragment: Fragment
```

The top-level fragment that will be rendered as the application UI. This fragment is instantiated when the workspace is created and destroyed when the workspace is destroyed.

**Usage:**
```frel
include Workspace {
    root_fragment = MainApp
}
```

#### active_theme

```frel
writable active_theme: Theme
```

The currently active theme for the application. Changing this value will reactively update all themed components.

**Usage:**
```frel
include Workspace {
    active_theme = Themes.dark
}

// Change theme at runtime
workspace.switch_theme(Themes.light);
```

#### locale

```frel
writable locale: String
```

The current locale identifier for internationalization (e.g., "en", "es", "fr-CA").

**Usage:**
```frel
include Workspace {
    locale = "en"
}

// Change locale at runtime
workspace.change_locale("es");
```

### Workspace Commands

#### set_root

```frel
command set_root(fragment: Fragment)
```

Dynamically replaces the root fragment. The old fragment is destroyed and the new one is instantiated.

**Example:**
```frel
button { "Switch to Admin View" }
    .. on_click { workspace.set_root(AdminPanel) }
```

#### switch_theme

```frel
command switch_theme(theme: Theme)
```

Changes the active theme. All themed components will reactively update.

**Example:**
```frel
button { "Toggle Theme" }
    .. on_click {
        let next_theme = if workspace.active_theme == Themes.dark {
            Themes.light
        } else {
            Themes.dark
        };
        workspace.switch_theme(next_theme);
    }
```

#### change_locale

```frel
command change_locale(locale: String)
```

Changes the application locale. This triggers re-rendering of all localized content.

**Example:**
```frel
select { workspace.locale }
    .. options { ["en", "es", "fr", "de"] }
    .. on_change |new_locale: String| {
        workspace.change_locale(new_locale);
    }
```

### Composition Patterns

#### Multi-Backend Application

```frel
backend MyApp {
    include Workspace {
        root_fragment = AppShell
        active_theme = Themes.default
        locale = "en"
    }

    // Authentication
    include AuthBackend

    // Routing
    include RouterBackend {
        initial_route = Routes.home
    }

    // Feature modules
    include UserManagementBackend
    include ProjectsBackend
    include SettingsBackend
}

fragment AppShell {
    with MyApp()

    column {
        // Header with theme toggle
        row {
            text { "My App" }
            button { "Toggle Theme" }
                .. on_click { app.switch_theme_toggle() }
        }

        // Main content driven by router
        when app.router.current_route == Routes.home {
            HomePage()
        }
        when app.router.current_route == Routes.users {
            UsersPage()
        }
    }
}
```

#### Multiple Workspaces (Testing)

You can create multiple workspace instances for different contexts:

```frel
backend TestWorkspace {
    include Workspace {
        root_fragment = TestHarness
        active_theme = Themes.test
        locale = "en"
    }

    // Mock backends for testing
    include MockAuthBackend
    include MockDataBackend
}

#[test]
fn test_user_flow() {
    let workspace = TestWorkspace::new();
    // Test with isolated workspace
}
```

#### Sub-Workspaces for Features

Large applications can use multiple workspace-like backends for feature areas:

```frel
backend EditorWorkspace {
    writable active_document: Document
    writable tool_panels: Vec<ToolPanel>
    writable layout_mode: LayoutMode

    include FileSystemBackend
    include SyntaxHighlightingBackend
    include AutocompleteBackend
}

backend MyApp {
    include Workspace {
        root_fragment = AppShell
    }

    include EditorWorkspace
    include TerminalWorkspace
    include DebugWorkspace
}
```

### Accessing Workspace from Fragments

Fragments can access the workspace through backend declarations:

```frel
fragment UserProfile {
    with MyApp()

    column {
        text { "Current locale: ${locale}" }
        text { "Theme: ${active_theme.name}" }

        button { "Change Language" }
            .. on_click { change_locale("es") }
    }
}
```

### Lifecycle Hooks

The workspace participates in application lifecycle:

```frel
backend MyApp {
    include Workspace {
        root_fragment = AppShell
    }

    include AuthBackend

    on_init {
        // Called when application starts
        self.restore_session();
        self.load_preferences();
    }

    on_cleanup {
        // Called when application exits
        self.save_preferences();
        self.cleanup_resources();
    }

    command restore_session()
    command load_preferences()
    command save_preferences()
    command cleanup_resources()
}
```

### Best Practices

#### Single Workspace per Application

```frel
// Good - one workspace for the application
backend MyApp {
    include Workspace {
        root_fragment = AppShell
    }
}

// Avoid - multiple workspaces in production
// (Only use multiple workspaces for testing)
```

#### Organize Backends Hierarchically

```frel
// Good - clear organization
backend MyApp {
    include Workspace

    // Core services
    include AuthBackend
    include RouterBackend

    // Feature modules
    include UserModule
    include ProjectModule
}

// Avoid - flat structure with too many includes
```

#### Use Include Configuration

```frel
// Good - configure included backends
backend MyApp {
    include Workspace {
        root_fragment = AppShell
        active_theme = Themes.dark
        locale = "en"
    }

    include RouterBackend {
        initial_route = Routes.dashboard
        not_found_handler = NotFoundPage
    }
}
```

#### Separate UI from Workspace

```frel
// Good - workspace manages state, fragment handles UI
backend MyApp {
    include Workspace {
        root_fragment = AppShell
    }

    writable sidebar_visible: bool = true

    command toggle_sidebar()
}

fragment AppShell {
    with MyApp()

    row {
        when sidebar_visible {
            Sidebar()
        }

        MainContent()
    }
}

// Avoid - putting UI logic in workspace
```

### Integration with Application Runtime

The workspace is the entry point for the application runtime:

```rust
// In main.rs
fn main() {
    // Create workspace instance
    let workspace = MyApp::new();

    // Register contracts (if needed)
    register_contract::<UserAPI>(http_transport);
    register_contract::<DataAPI>(websocket_transport);

    // Run application with workspace as root
    Application::run(workspace);
}
```

### Example: Full Application

```frel
// Define application workspace
backend TodoApp {
    include Workspace {
        root_fragment = TodoAppShell
        active_theme = Themes.light
        locale = "en"
    }

    uses TodoAPI

    writable todos: List<Todo> = List::new()
    writable filter: TodoFilter = TodoFilter::All

    // Derived state
    decl filtered_todos = filter_todos(todos, filter)

    on_init {
        self.load_todos();
    }

    command load_todos()
    command add_todo(text: String)
    command toggle_todo(id: u32)
    command delete_todo(id: u32)
    command set_filter(filter: TodoFilter)
}

// Main application UI
fragment TodoAppShell {
    with TodoApp()

    column {
        padding { 20 }
        gap { 16 }

        // Header
        row {
            text { "Todo App" } .. font { size: 24 weight: 700 }
            button { "Toggle Theme" }
                .. on_click {
                    let next = if active_theme == Themes.light {
                        Themes.dark
                    } else {
                        Themes.light
                    };
                    switch_theme(next);
                }
        }

        // Add todo
        TodoInput(on_add: add_todo)

        // Filter buttons
        TodoFilters(
            current: filter,
            on_change: set_filter
        )

        // Todo list
        repeat on filtered_todos as todo {
            TodoItem(
                todo: todo,
                on_toggle: toggle_todo,
                on_delete: delete_todo
            )
        }
    }
}
```

### See Also

- [Backend Basics](10_backend_basics.md) - Understanding backend definitions
- [Standard Fragments](../70_fragment/30_standard_fragments.md) - Built-in UI components
- [Standard Sources](../20_reactive_state/60_standard_sources.md) - Built-in reactive data sources
