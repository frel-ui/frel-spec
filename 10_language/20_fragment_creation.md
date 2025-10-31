# Fragment Creation

Fragment creation statements instantiate child fragments within the body of a fragment.
A **block** following a fragment name is an **inline template literal**, conceptually a small fragment template.
Blocks may bind to the **default content slot** or to **named slots**.

## Surface Forms

```frel
// Basic leaf fragment with default-content block
text { "stuff" }

// Higher-order fragment with a default content block
column { text { "stuff" } }

// Higher-order fragment with value parameters and content block
higherOrder(12) { text { "stuff" } }

// Higher-order fragment with multiple content slots
multiSlot {
  at slot1: { text { "stuff-1" } }
  at slot2: text { "stuff-2" }
}
```

Notes:

* Parentheses `()` are **only** for value parameters.
* The block `{ ... }` is always a **template literal**, not an instance.
* In a multi-slot block, each `at <slot>:` binds a template (inline or named) to a slot parameter.
* A plain block binds to the callee’s **default slot** (usually named `content`).

## Syntax (Informal)

```text
<creation>     ::= <name> [ "(" [ <arg-list> ] ")" ] <block-or-slots>? <postfix>*
<arg-list>     ::= <arg> { "," <arg> }
<arg>          ::= <expr> | <param-name> "=" <expr>

<block-or-slots> ::= <default-block> | <slot-block>
<default-block>  ::= "{" <body> "}"
<slot-block> ::= "{" <slot-binding> { <separator> <slot-binding> } "}"
<slot-binding>   ::= "at" <slot-name> ":" <template-value>
<separator>  ::= "," | newline
<template-value> ::= <inline-template> | <template-ref>
<inline-template> ::= "{" <body> "}"
<template-ref>    ::= <name>

<postfix>        ::= ".." <instruction>
```

## Semantics

### 1. Value Parameters

* Supplied only inside `(...)`.
* May be passed **positionally** or by **name**:
  - Positional: `SomeFragment(12, "text")`
  - Named: `SomeFragment(width = 12, label = "text")`
  - Mixed: `SomeFragment(12, label = "text")` — positional arguments must come before named ones
* Once a named argument is used, all subsequent arguments must also be named.
* Missing required parameters or extra parameters are compile-time errors.
* Named arguments may appear in any order (after positional ones).
* `<expr>` must be a pure host language expression (no side effects).
* Reactive dependencies from stores are tracked automatically.

### 2. Template Parameters (Default Slot & Named Slots)

* A plain block `{ ... }` provides a template for the **default slot**.
* A slot block `{ at slot1: ..., at slot2: ... }` provides templates for **named slots**.
* Templates may be given inline (`{ ... }`) or by reference (`TemplateName`).
* If a fragment has only a default slot, the short form `{ ... }` is preferred.
* For explicitness, named passing is allowed:
  `higherOrder(12) { at content: TemplateName }`

### 3. Desugaring

Each inline block is desugared into a compiler-synthesized **anonymous template** and passed by name.

```frel
column { text { "A" } }
```

becomes conceptually:

```frel
fragment __anon_1() { text { "A" } }
column { at content: __anon_1 }
```

* Anonymous templates close over their lexical scope (capture visible stores reactively).
* Each inline block produces a unique synthesized template.

### 4. Postfix Instructions

Postfix instructions (`.. padding { 8 }`) apply layout or styling to the fragment’s root node.
Illegal combinations produce compile-time errors.

### 5. Lifetime and Reactivity

* Child fragments subscribe to stores used in value parameters or captured by inline templates.
* All subscriptions are cleaned up automatically when the parent is destroyed.

## Error Conditions

| Condition                                     | Kind         | Description                           |
|-----------------------------------------------|--------------|---------------------------------------|
| Unknown fragment/template name                | Compile-time | Not found in scope.                   |
| Unknown or duplicate parameter                | Compile-time | Invalid or repeated name.             |
| Type mismatch in parameter                    | Compile-time | Incompatible Rust type.               |
| Impure expression                             | Compile-time | Expression has side effects.          |
| Named argument before positional              | Compile-time | Positional args must come first.      |
| Too many positional arguments                 | Compile-time | More positional args than parameters. |
| Missing required parameter                    | Compile-time | Required parameter not supplied.      |
| Block supplied but callee has no default slot | Compile-time | Use `at <slot>:` explicitly.          |
| Unknown slot name                             | Compile-time | Slot not declared by callee.          |
| Non-template value in slot                    | Compile-time | Slot expects a template.              |

## Examples

```frel
// 1. Default slot only
text { "Hello" }

// 2. Value parameters (positional) + default slot
higherOrder(12) { text { "Body" } }

// 3. Value parameters (named)
higherOrder(width = 12, height = 24) { text { "Body" } }

// 4. Value parameters (mixed: positional then named)
higherOrder(12, height = 24, label = "Title") { text { "Body" } }

// 5. Multi-slot with inline and named templates
multiSlot {
  at header: { row { text { "Header" } } }
  at item:   ItemRowTemplate
  at footer: { row { text { "Footer" } } }
}

// 6. Passing default content by name
column { at content: TwoLabels }

// 7. Postfix styling
button { text { "Click" } } .. padding { 8 } .. border { Red, 1 }
```

## Built-in Slots

### Tooltip Slot

All fragments support an optional `tooltip` slot for contextual help:

```frel
button {
    "Save"
    at tooltip: {
        text { "Ctrl+S to save" }
        .. padding { 6 }
        .. background { color: Black }
        .. font { color: White }
    }
}
```

The tooltip automatically:
- Renders in the `tooltip` channel
- Shows on hover with 500ms delay
- Hides when pointer leaves
- Positions relative to parent (smart repositioning if needed)

See [Detached UI - Tooltip](50_detached_ui.md#tooltip) for full details.