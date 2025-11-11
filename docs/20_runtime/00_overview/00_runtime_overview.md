# Runtime Overview

## Blueprint Closure

Each blueprint defines one or more closures:

```frel
blueprint Chat(                             ─┐ blueprint closure
    message : String,                        │ 
    names : List<String>                     │
) {                                          │ 
    count = names.length                     │
                                             │
    if message.is_not_empty {                │
        column {                             │  ─┐ blueprint + column closure
            repeat on names { name ->        │   │   ─┐ bloopint + column + loop closure
                text { "$message $name!" }   │   │    │
            }                                │   │   ─┘
        }                                    │  ─┘
    } else {                                 │
        row {                                │  ─┐ blueprint + row closure
            text { "nothing to say" }        │   │
        }                                    │  ─┘ 
    }                                        │
                                             │
    text { "$count recipent(s)" }            │
}                                           ─┘
```

When a blueprint has one or more parameters with the `Blueprint` type, the use site of that
blueprint generates anonymous closures, one for each `Blueprint` parameter.

For example, if we use the `Article` blueprint below, it will generate a closure for each
slot.

```frel
blueprint Article {
    header : Blueprint,
    content : Blueprint,
    footer : Blueprint
}    

blueprint Page {
    Article {
        at header : text { "Header" }
        at content: {
            text { "Content line 1" }
            text { "Content line 2" }
        }
        at footer : text { "Footer" }
    }
}
```

Each blueprint closure defines **bindings**, each binding has a name and a type.

## Fragment Closure

When fragments are created from the blueprint, a fragment closure is created for each fragment.
The bindings in the fragment closure bind a **reactive identity** to the names defined by 
the blueprint.

**Binding name**: the unique name of the binding in the closure, shadowing is not allowed.
The binding name is one of (actual names from the `Chat` example above):

- parameter name: `message`, `names`
- blueprint value name: `count`
- `anonymous#123`, where 123 is a unique, system-assigned number, used for expressions passed as
  parameters: `$count recipent(s)`

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
