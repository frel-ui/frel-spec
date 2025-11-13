# Module Basics

Frel source code is organized using a simple module system. Modules provide namespacing for
top-level declarations and control which declarations are accessible from other parts of 
the application.

Top-level declarations:

- `blueprint`
- `backend`
- `contract`
- `scheme`
- `enum`
- `theme`
- `arena`

## Overview

- **Files**: Frel source code is stored in files with `.frel` extension
- **Modules**: Logical namespaces declared with the `module` keyword
- **Imports**: Declarations from other modules are imported with the `import` keyword
- **Qualified Paths**: Declarations can be referenced using their fully qualified module path

**Important**: Files are **not** organizational units from a structural point of view—they are just
the place where the code is stored. The module system defines the actual namespace structure.

## Syntax

```text
<module-decl> ::= "module" <module-path>
<module-path> ::= <identifier> { "." <identifier> }

<import-statement> ::= "import" <module-path> "." <identifier>

<qualified-path> ::= <module-path> "." <identifier>
```

## Module Declaration

Every `.frel` file **must** contain exactly one `module` declaration at the top of the file:

```frel
module frel.ui.components
```

**Rules:**

- Must be the first non-comment statement in the file
- Exactly one `module` declaration per file
- Module paths use dot notation (e.g., `frel.ui.components`)
- Module path segments use snake_case identifiers

### What Gets Namespaced

All top-level declarations in a file belong to the declared module:

```frel
module frel.ui.buttons

// All of these are in the frel.ui.buttons namespace:
blueprint PrimaryButton() { ... }     // frel.ui.buttons.PrimaryButton
blueprint SecondaryButton() { ... }   // frel.ui.buttons.SecondaryButton
backend ButtonState { ... }          // frel.ui.buttons.ButtonState
scheme ButtonConfig { ... }          // frel.ui.buttons.ButtonConfig
enum ButtonSize { small medium large } // frel.ui.buttons.ButtonSize
```

## Import Statements

The `import` keyword imports declarations from other modules, making them available without
qualification:

```frel
module frel.app

import frel.ui.buttons.PrimaryButton
import frel.ui.themes.DarkTheme
import frel.data.Color

blueprint MainScreen() {
    // Can use PrimaryButton, DarkTheme, Color directly
    PrimaryButton("Click me")
}
```

**Rules:**

- `import` statements must appear after the `module` declaration
- `import` statements must appear before any other declarations
- Each `import` imports a single declaration
- Imports are file-scoped (do not affect other files)

### Example File Structure

```frel
// File header
module frel.app.screens

// Imports
import frel.ui.components.Button
import frel.ui.components.TextInput
import frel.backends.UserBackend
import frel.data.schemes.User

// Declarations
blueprint LoginScreen() {
    with UserBackend()

    column {
        TextInput()
        Button("Login")
    }
}
```

## Fully Qualified Paths

Declarations can be referenced using their fully qualified module path without importing:

```frel
module frel.app

blueprint MainScreen {
    column {
        // Use fully qualified paths
        frel.ui.buttons.PrimaryButton("Save")
        frel.ui.buttons.SecondaryButton("Cancel")

        // Mix with imports
        LocalWidget()
    }
}
```

**When to use qualified paths:**

- One-off usage of a declaration
- Avoiding naming conflicts
- Making dependencies explicit
- Clarifying ambiguous references

## Name Resolution

When the compiler encounters an identifier, it resolves it in this order:

1. **Local declarations** in the current file
2. **Imported declarations** via `import` statements
3. **Qualified paths** (e.g., `frel.ui.Button`)

### Name Conflicts

**Shadowing is forbidden in Frel.** If an imported name conflicts with a local declaration, the compiler will reject the code.

**Invalid example** (rejected by compiler):

```frel
module frel.app

import frel.ui.Button

// ✗ ERROR: Local declaration shadows imported Button
blueprint Button() {
    // This is rejected by the compiler
}
```

**Valid alternative** - use distinct names:

```frel
module frel.app

import frel.ui.Button

// ✓ Use a distinct name for the local blueprint
blueprint AppButton() {
    // Local blueprint with distinct name
}

blueprint Example() {
    Button()     // ✓ Refers to imported Button
    AppButton()  // ✓ Refers to local blueprint
}
```

**Required**: All names within a file's scope must be unique. Choose distinct names for local declarations to avoid conflicts with imports and module-level declarations.

## Multiple Files, Same Module

Multiple files can declare the same module to split large modules across files:

```frel
// File: buttons/primary.frel
module frel.ui.buttons

blueprint PrimaryButton() { ... }
blueprint PrimaryIconButton() { ... }
```

```frel
// File: buttons/secondary.frel
module frel.ui.buttons

blueprint SecondaryButton() { ... }
blueprint SecondaryIconButton() { ... }
```

Both files contribute declarations to the `frel.ui.buttons` module. All declarations are available
to importers:

```frel
import frel.ui.buttons.PrimaryButton
import frel.ui.buttons.SecondaryButton
```

## Visibility and Access Control

**Current design**: All declarations in a module are **public** and accessible to other modules.

Private/internal declarations are not supported in this version. If you need to hide implementation
details, use naming conventions (e.g., prefix with `_`) or organize code into separate modules.

## Module-to-Filesystem Mapping

**Flexible**: The module system does not enforce a specific directory structure. Modules are logical
namespaces independent of file locations.

## Circular Dependencies

**Allowed at module level**: Circular dependencies between modules are permitted. Modules can import
from each other without restriction.

```frel
// File A
module myapp.a
import myapp.b.BlueprintB

blueprint BlueprintA {
    BlueprintB()  // Can reference B
}
```

```frel
// File B
module myapp.b
import myapp.a.BlueprintA  // ✓ Allowed - modules can reference each other

blueprint BlueprintB {
    BlueprintA()  // Can reference A
}
```

**Backend-level restrictions**: While module-level circular dependencies are allowed, circular
dependencies between specific declarations (like backends) may have restrictions.

## Future Extensions

The following features are **not** currently supported but may be added in future versions:

- **Import aliases**: `import frel.ui.Button as UIButton`
- **Grouped imports**: `import frel.ui.{ Button, TextInput, Panel }`
- **Wildcard imports**: `import frel.ui.*`
- **Re-exports**: Making imported declarations available to importers
- **Private declarations**: Hiding implementation details within a module
- **Conditional compilation**: Platform-specific module resolution