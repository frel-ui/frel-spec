# Runtime Overview

## Expressions

Expressions in blueprints are small reactive calculations.

```frel
blueprint Example(
    a : u32,
    b : u32
) {
    result : u32 = a * b
}
```

When the compiler encounters such a calculation, it creates a new **computation**. The computation:

- is a **composite type**,
- is bound to a given fragment closure,
- has a unique **reactive identity**, which is stored in the fragment closure,
- subscribes to the data it uses,
- recomputes when the subscribed data changes,
- contains a value
- notifies its subscribers when the value changes
