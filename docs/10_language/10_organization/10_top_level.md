# Top-Level Declarations

Frel source code is organized using top-level declarations. A top-level declaration is any
declaration that appears directly within a module, not nested inside another declaration.

## What is a Top-Level Declaration?

A **top-level declaration** is a named entity defined at the module level in a `.frel` file. These
declarations:

- Must appear after the `module` statement and any `import` statements
- Cannot be nested inside other declarations
- Are automatically namespaced by their containing module
- Can be imported by other modules

## File Structure

Every `.frel` file must follow this structure:

```frel
// 1. Module declaration (required, exactly one)
module myapp.feature

// 2. Import statements (optional, zero or more)
import other.module.SomeDeclaration
import another.module.AnotherDeclaration

// 3. Top-level declarations (optional, zero or more)
blueprint MyBlueprint { ... }
backend MyBackend { ... }
scheme MyScheme { ... }
```

## Top-Level Declaration Types

Frel supports these top-level declaration types:

### `blueprint`

Declares a reusable UI component template.

```frel
blueprint Button(label: String) {
    text { label }
}
```

See: Blueprint documentation (TODO: add link when available)

### `backend`

Declares a reactive state container with behavior.

```frel
backend AppBackend {
    count : u32 = 0

    command increment()
}
```

See: Backend documentation (TODO: add link when available)

### `contract`

Declares an interface to external services and remote APIs.

```frel
contract UserAPI {
    call fetch_user(id: UUID) : User
    call update_user(user: User)
}
```

See: [Contracts](../40_contract/10_contracts.md)

### `scheme`

Declares a structured data type with validation and constraints.

```frel
scheme User {
    id : UUID .. identity
    name : String
    email : String .. pattern { email_regex }
}
```

See: [Schemes](../20_data_model/35_schemes.md)

### `enum`

Declares a fixed set of named variants for type-safe categorical data.

```frel
enum Status { pending active completed cancelled }
```

See: [Enums](../20_data_model/20_intrinsic_types.md#enums)

### `theme`

Declares a reusable styling configuration for blueprints.

```frel
theme DarkTheme {
    background : asset Color
    foreground : asset Color
    padding : u32 = 16
}
```

See: Theme documentation (TODO: add link when available)

### `arena`

Declares a collection for scheme instances with identity-based storage.

```frel
arena UserArena {
    for User
}
```

See: Arena documentation (TODO: add link when available)

## Rules

### Uniqueness

All top-level declaration names within a module must be unique:

```frel
module myapp

blueprint Button { ... }  // ✓ OK
scheme Button { ... }        // ✗ ERROR: duplicate name 'Button'
```

### No Nesting

Top-level declarations cannot be nested inside other declarations:

```frel
blueprint Outer {
    // ✗ ERROR: Cannot declare blueprint inside another blueprint
    blueprint Inner() { ... }
}
```

### Visibility

All top-level declarations are **public** and accessible to other modules via import:

```frel
// File: myapp/widgets.frel
module myapp.widgets

blueprint CustomButton { ... }

// File: myapp/screens.frel
module myapp.screens

import myapp.widgets.CustomButton  // ✓ Can import

blueprint HomeScreen {
    CustomButton()  // ✓ Can use
}
```

### Shadowing

Top-level declarations cannot shadow imported declarations. See [Scope](30_scope.md#shadowing) for
details.

### Contextual Keywords

The keywords for top-level declarations (`blueprint`, `backend`, `scheme`, etc.) are *contextual* -
they are only reserved at positions where a top-level declaration can appear. Inside declarations,
these can be used as field names, parameter names, or other identifiers.
See [Scope](30_scope.md#contextual-keywords) for details.

## Order Independence

Top-level declarations can reference each other regardless of declaration order:

```frel
module myapp

// Can reference ButtonBackend before it's declared
blueprint MyButton {
    with ButtonBackend()
    // ...
}

// Declared after MyButton
backend ButtonBackend {
    count : u32 = 0
}
```

This works because the compiler processes all top-level declarations in a module before checking
references.

## Multiple Files, Same Module

Multiple files can contribute top-level declarations to the same module:

```frel
// File: buttons/primary.frel
module myapp.ui

blueprint PrimaryButton { ... }
```

```frel
// File: buttons/secondary.frel
module myapp.ui

blueprint SecondaryButton { ... }
```

Both `PrimaryButton` and `SecondaryButton` belong to the `myapp.ui` module and can be imported:

```frel
import myapp.ui.PrimaryButton
import myapp.ui.SecondaryButton
```

See [Modules](20_modules.md) for details on the module system.
