# Closures

Closures in Frel define what declarations and bindings are accessible at any given point in the
code. Frel has two distinct concepts that work together:

1. **Scope**: Defines what can be referenced at compile time
2. **Runtime Closures**: Provide actual data bindings during execution

Understanding closures is essential for working with Frel's reactive system, as they determine how
data flows through your application.

## Overview

**Scopes** determine name resolution during compilation. When you write `count` or
`message` in your code, the compiler uses the scope to determine what that name refers
to.

**Runtime closures** are created dynamically as fragments are instantiated from blueprints. They
bind names to actual reactive identities, enabling Frel's reactive system to track dependencies and
propagate changes.

## Scope

A scope defines what declarations are available at any point in the source code. The
scope is built from three layers:

1. **Module-level declarations**: Top-level declarations in the current module
2. **File-level imports**: Declarations imported via `import` statements
3. **Local declarations**: Declarations within the current scope

### Module-Level Declarations

All top-level declarations in a module are available throughout that module:

```frel
module myapp.ui

// These are available throughout the module
blueprint Header() { ... }
backend AppBackend { ... }
scheme User { ... }
enum Status { pending active completed }
```

### File-Level Imports

Import statements make declarations from other modules available within the current file only:

```frel
module myapp.screens

import myapp.ui.Header
import myapp.backends.UserBackend
import myapp.data.User

// Header, UserBackend, and User are now in this file's scope
blueprint LoginScreen() {
    Header()  // Available via import
}
```

**Important**: Imports are **file-scoped**, not module-scoped. If multiple files declare the same module, each file must have its own `import` statements. Imports in one file do not affect other files, even if they belong to the same module.

### Local Declarations

Declarations within a top-level declaration create a local scope. Each declaration type has its
own scoping rules.

#### Backend Local Declarations

Fields in a backend are available to other fields and commands within that backend:

```frel
backend ExampleBackend {
    i : u32 = 12
    j : u32 = i * 2      // 'i' is in the local scope
    k : u32 = i + j      // Both 'i' and 'j' are available
}
```

#### Blueprint Local Declarations

Blueprints create nested scopes based on their structure:

```frel
blueprint Chat(
    message : String,
    names : List<String>
) {
    count = names.length

    if message.is_not_empty {
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

#### Slot Scopes

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

### Name Resolution Order

When the compiler encounters an identifier, it resolves it in this order:

1. **Local declarations** in the current scope
2. **Parent scope declarations** (for nested scopes)
3. **File-level imports** via `import` statements
4. **Module-level declarations** in the current module
5. **Qualified paths** (e.g., `myapp.ui.Button`)

### Shadowing

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

## Runtime Closures

Runtime closures are created dynamically as fragments are instantiated from blueprints. Unlike
scopes which exist at compile time, runtime closures exist during program execution
and bind names to actual reactive data.

### Fragment Closures

When a blueprint is instantiated, the Frel runtime creates a **fragment** with its own **fragment
closure**. The fragment closure:

- Binds each name to a **reactive identity** (see [Reactivity Model](../10_data_model/10_reactivity_model.md))
- Tracks dependencies between reactive computations
- Enables automatic updates when data changes
- Lives as long as the fragment exists

### Bindings

Each binding in a fragment closure has:

- **Name**: The identifier used in source code
- **Type**: The Frel type of the bound data
- **Reactive Identity**: A unique identifier for the data instance

Binding names come from:

- **Parameter names**: From blueprint parameters
- **Local value names**: From declarations like `count = names.length`
- **Anonymous bindings**: System-generated names for expressions (e.g., `anonymous#123`)

### Example

Consider this blueprint:

```frel
blueprint Counter(initial : u32) {
    count : u32 = initial
    doubled : u32 = count * 2

    text { "$doubled" }
}
```

**Scope** (compile time):

- Contains: `initial`, `count`, `doubled`
- Determines that `count * 2` can reference `count`
- Validates that all names are defined

**Runtime closure** (execution time):

- Binds `initial` to reactive identity (e.g., `u32(10)`)
- Binds `count` to reactive identity (e.g., `u32#42`)
- Binds `doubled` to a computation that subscribes to `count`
- Binds `anonymous#123` to the string interpolation expression
- Updates `doubled` when `count` changes

### Nested Fragment Closures

Each nested blueprint or control structure creates its own fragment closure that inherits from its
parent:

```frel
blueprint Parent(x : u32) {
    y : u32 = x * 2

    column {
        Child(z: y)
    }
}

blueprint Child(z : u32) {
    result : u32 = z + 1
}
```

At runtime:

- Parent fragment closure: binds `x` and `y`
- Child fragment closure: binds `z` and `result`, separate from parent
- Changes to `x` propagate to `y`, which propagates to `z`, which propagates to `result`

## Implementation Notes

The exact implementation of runtime closures is host-platform specific and not defined by the
language specification. However, all implementations must:

- Create bindings that support reactive subscriptions
- Maintain proper scoping and shadowing semantics
- Enable dependency tracking for reactive updates
- Clean up closures when fragments are destroyed

The reactive identity system ensures that implementations can optimize closure storage and updates
while maintaining correct reactive behavior.

## Relationship to Reactivity

Runtime closures are fundamental to Frel's reactivity model:

1. **Dependency Tracking**: When a computation references a name, the runtime subscribes to that
   binding's reactive identity
2. **Change Propagation**: When a binding's value changes, all subscribers are notified
3. **Automatic Updates**: The UI automatically updates when reactive data changes
4. **Fine-Grained Reactivity**: Each binding can have independent subscribers

For more details on how reactivity works, see
the [Reactivity Model](../10_data_model/10_reactivity_model.md) documentation.
