# Scope

A scope defines what declarations are available at any point in the source code during compilation.
When you write `count` or `message` in your code, the compiler uses the scope to determine what that
name refers to.

## Overview

Scope is a compile-time concept that determines name resolution. The scope is built from three
layers:

1. **Module-level declarations**: [Top-level declarations](10_top_level.md) in the current module
2. **File-level imports**: Declarations imported via `import` statements
3. **Local declarations**: Declarations within the current scope

## Module-Level Declarations

All top-level declarations in a module are available throughout that module:

```frel
module myapp.ui

// These are available throughout the module
blueprint Header { ... }
backend AppBackend { ... }
scheme User { ... }
enum Status { pending active completed }
```

See [Top-Level Declarations](10_top_level.md) for details on what can be declared at module level.

## File-Level Imports

Import statements make declarations from other modules available within the current file only:

```frel
module myapp.screens

import myapp.ui.Header
import myapp.backends.UserBackend
import myapp.data.User

// Header, UserBackend, and User are now in this file's scope
blueprint LoginScreen {
    Header()  // Available via import
}
```

**Important**: Imports are **file-scoped**, not module-scoped. If multiple files declare the same
module, each file must have its own `import` statements. Imports in one file do not affect other
files, even if they belong to the same module.

See [Modules](20_modules.md) for details on the module system and imports.

## Local Declarations

Declarations within a top-level declaration create a local scope. Each declaration type has its
own scoping rules.

### Backend Local Declarations

Fields in a backend are available to other fields and commands within that backend:

```frel
backend ExampleBackend {
    i : u32 = 12
    j : u32 = i * 2      // 'i' is in the local scope
    k : u32 = i + j      // Both 'i' and 'j' are available
}
```

### Blueprint Local Declarations

Blueprints create nested scopes based on their structure:

```frel
blueprint Chat(
    message : String,
    names : List<String>
) {
    count = names.length

    when message.is_not_empty {
        column {
            repeat on names { name ->
                text { "$message $name!" }
            }
        }
    } else {
        row {
            text { "nothing to say" }
        }
    }

    text { "$count recipient(s)" }
}
```

In this example, there are multiple nested scopes:

1. **Blueprint scope** (outermost): Contains `message`, `names`, and `count`
2. **Column scope**: Inherits blueprint scope, adds nothing new
3. **Loop scope**: Inherits column + blueprint scopes, adds `name` from the loop iterator
4. **Row scope**: Inherits blueprint scope, adds nothing new

Each nested scope can access declarations from its parent scopes, creating a scope chain.

### Slot Scopes

When a blueprint has parameters of type `Blueprint`, the use site creates anonymous scopes for
each slot:

```frel
blueprint Article {
    header : Blueprint
    content : Blueprint
    footer : Blueprint
}

blueprint Page {
    Article {
        at header : text { "Header" }
        at content : {
            text { "Content line 1" }
            text { "Content line 2" }
        }
        at footer : text { "Footer" }
    }
}
```

Each slot (`at header`, `at content`, `at footer`) creates its own scope that has access to the
`Page` blueprint's scope.

## Name Resolution Order

When the compiler encounters an identifier, it resolves it in this order:

1. **Local declarations** in the current scope
2. **Parent scope declarations** (for nested scopes)
3. **File-level imports** via `import` statements
4. **Module-level declarations** in the current module
5. **Qualified paths** (e.g., `myapp.ui.Button`)

## Shadowing

**Shadowing is forbidden in Frel.** All names within a scope must be unique. The compiler will
reject any code that attempts to shadow a name from an outer scope.

This applies to all levels:

- Local declarations cannot shadow imported declarations
- Local declarations cannot shadow module-level declarations
- Nested scope declarations cannot shadow parent scope declarations
- Parameters cannot shadow any declaration in the scope

**Invalid example** (rejected by compiler):

```frel
module myapp

import external.Counter

backend AppBackend {
    Counter : u32 = 0  // ✗ ERROR: shadows imported Counter
}
```

**Valid alternative** - use distinct names:

```frel
module myapp

import external.Counter

backend AppBackend {
    counter_value : u32 = 0          // ✓ Distinct name
    external_counter : external.Counter  // ✓ Can use imported Counter
}
```

If you need to reference both an imported declaration and a local one, use distinct names. This
makes code more explicit and prevents confusion about which declaration is being referenced.

## Relationship to Runtime Closures

Scope determines what *can* be referenced at compile time. At
runtime, [closures](../80_runtime/30_fragments_and_closures.md) bind these names to actual reactive identities,
enabling Frel's reactive system to track dependencies and propagate changes.
