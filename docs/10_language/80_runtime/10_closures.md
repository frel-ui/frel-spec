# Runtime Closures

Runtime closures are created dynamically as fragments are instantiated from blueprints.
Unlike [scopes](../10_organization/30_scope.md) which exist at compile time, runtime closures exist
during program execution and bind names to actual reactive data.

## Overview

When a blueprint is instantiated, the Frel runtime creates a **fragment** with its own **fragment
closure**. The fragment closure:

- Binds each name to a **reactive identity** (see [Reactivity Model](../20_data_model/10_reactivity_model.md))
- Tracks dependencies between reactive computations
- Enables automatic updates when data changes
- Lives as long as the fragment exists

## Bindings

Each binding in a fragment closure has:

- **Name**: The identifier used in source code
- **Type**: The Frel type of the bound data
- **Reactive Identity**: A unique identifier for the data instance

Binding names come from:

- **Parameter names**: From blueprint parameters
- **Local value names**: From declarations like `count = names.length`
- **Anonymous bindings**: System-generated names for expressions (e.g., `anonymous#123`)

## Example

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

## Nested Fragment Closures

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
the [Reactivity Model](../20_data_model/10_reactivity_model.md) documentation.
