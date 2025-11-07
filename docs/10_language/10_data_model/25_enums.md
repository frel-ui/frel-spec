# Enums

Enums define a fixed set of named variants, providing type-safe alternatives to string constants
or numeric codes. They are commonly used for state machines, status values, and categorical data.

## Syntax

```
enum <Name> { <variant1> <variant2> <variant3> ... }
```

## Semantics

- **Variants**: Space-separated identifiers representing the possible values
- **Scope**: Top-level declarations, available throughout the module
- **Ordering**: Variants maintain declaration order
- **Usage**: Can be used as types in schemes, blueprints, and function signatures