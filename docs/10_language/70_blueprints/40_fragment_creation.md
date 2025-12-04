# Fragment Creation

Fragment creation statements instantiate child fragments during instantiation.
A **block** following a blueprint name is an **inline blueprint**, conceptually a small anonymous
blueprint.
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
  at slot1: { text { "stuff-1" } }   // inline blueprint
  at slot2: MyTextBlueprint          // blueprint reference
}
```

Notes:

* A fragment body uses **either** default content **or** `at` slot bindings—never both mixed together.
* A plain block `{ ... }` provides content for the **default slot** (usually named `content`).
* A slot block `{ at slot1: ..., at slot2: ... }` provides content for **named slots**.
* If you use any `at slot:` binding, all content must use `at` syntax (including the default slot).
* Slot bindings accept two forms:
  - `at slot: { ... }` - inline blueprint with content (braces required)
  - `at slot: BlueprintName` - blueprint reference (bare identifier, no body)
* The implicit `tooltip` slot only requires `at` syntax if you actually use it.

## Syntax (Informal)

```text
<creation>     ::= <name> [ "(" <arg-list> ")" ] <block-or-slots>? <postfix>*
<arg-list>     ::= <arg> { "," <arg> }
<arg>          ::= <expr> | <param-name> "=" <expr>

<block-or-slots> ::= <default-block> | <slot-block>
<default-block>  ::= "{" <body> "}"
<slot-block> ::= "{" <slot-binding> { <separator> <slot-binding> } "}"
<slot-binding>   ::= "at" <slot-name> ":" <blueprint-value>
<separator>  ::= "," | newline
<blueprint-value> ::= <inline-blueprint> | <blueprint-ref>
<inline-blueprint> ::= [ <param-list> "->" ] "{" <body> "}"
<param-list>     ::= <param-name> { "," <param-name> }
<blueprint-ref>    ::= <name>

<postfix>        ::= ".." <instruction>

# Blueprint parameter types
<blueprint-type> ::= "Blueprint" [ "<" <blueprint-params> ">" ]
<blueprint-params> ::= <blueprint-param> { "," <blueprint-param> }
<blueprint-param> ::= <type>
```

## Semantics

### 1. Normal Parameters

* Are representing reactive values.
* Supplied only inside `(...)`.
* Parentheses are **optional** when there are no parameters (e.g., `Counter { }` is equivalent to
  `Counter() { }`).
* May be passed **positionally** or by **name**:
    - Positional: `SomeFragment(12, "text")`
    - Named: `SomeFragment(width = 12, label = "text")`
    - Mixed: `SomeFragment(12, label = "text")` — positional arguments must come before named ones
* Once a named argument is used, all subsequent arguments must also be named.
* Missing required parameters or extra parameters are compile-time errors.
* Named arguments may appear in any order (after positional ones).
* `<expr>` must be a pure host language expression (no side effects).
* Reactive dependencies are tracked automatically.

### 2. Blueprint Parameters (Default Slot & Named Slots)

* A plain block `{ ... }` provides a blueprint for the **default slot**.
* A slot block `{ at slot1: ..., at slot2: ... }` provides blueprints for **named slots**.
* Blueprints may be given inline (`{ ... }`) or by reference (`BlueprintName`).
* If a blueprint has only a default slot, the short form `{ ... }` is preferred.
* For explicitness, named passing is allowed:
  `higherOrder(12) { at content: BlueprintName }`

**Blueprint Parameter Types**

Blueprint parameters use the `Blueprint<P1,...Pn>` type to declare what parameters they expect:

```frel
blueprint Container(content: Blueprint) {
    // content is a blueprint with no parameters
    content()
}

blueprint TextWrapper(content: Blueprint<String>) {
    hw = "Hello World!"
    content(hw)  // Must pass a String
}

blueprint ItemRenderer(
    header: Blueprint<String, bool>,
    item: Blueprint<User>,
    footer: Blueprint
) {
    header("Title", true)
    item(current_user)
    footer()
}
```

When invoking a blueprint parameter, you must pass arguments matching its type signature:

```frel
blueprint Parent() {
    TextWrapper {
        at content: { s ->  // Anonymous blueprint receives String 's'
            text { s }
        }
    }

    // Or pass by reference
    TextWrapper { at content: MyTextBlueprint }
}

blueprint MyTextBlueprint(text: String) {
    text { text }
}
```

### 3. Desugaring and Type Inference

Each inline block is desugared into a compiler-synthesized **anonymous blueprint** and passed by
name.

**Basic Example:**

```frel
column { text { "A" } }
```

becomes conceptually:

```frel
blueprint __anon_1() { text { "A" } }
column { at content: __anon_1 }
```

**Type Inference for Anonymous Blueprints:**

The compiler infers the `Blueprint<...>` type for anonymous blueprints based on the expected
parameter type:

```frel
blueprint TextWrapper(content: Blueprint<String>) {
    hw = "Hello World!"
    content(hw)
}

// Usage with anonymous blueprint
TextWrapper {
    // Compiler infers this anonymous blueprint has type Blueprint<String>
    text { "The text is: ${$0}" }  // $0 is the first parameter (String)
}
```

The desugared form:

```frel
blueprint __anon_1(p0: String) {  // Parameter inferred from Blueprint<String>
    text { "The text is: ${p0}" }
}
TextWrapper { at content: __anon_1 }
```

**Anonymous Blueprint Parameter Syntax:**

When an anonymous blueprint has parameters, you can use:

- **Named parameters** with the arrow syntax `{ param1, param2 -> ... }`
- **Default `it` parameter** for single-parameter blueprints (omit parameter list)
- **Positional references** `$0`, `$1`, etc. (less common, named parameters preferred)

```frel
// Named parameter (explicit)
TextWrapper { s ->
    text { "Text: ${s}" }
}

// Default 'it' for single parameter (no arrow needed)
TextWrapper {
    text { "Text: ${it}" }
}

// Positional reference (implicit) - less idiomatic
TextWrapper {
    text { "Text: ${$0}" }
}

// Multiple parameters - must use named parameters
ItemRenderer {
    at header: { title, isActive ->
        text { "${title} - ${isActive}" }
    }
}

// Multiple parameters with positional references - not recommended
ItemRenderer {
    at header: {
        text { "${$0} - ${$1}" }
    }
}
```

**Style Guidelines:**

- Prefer `it` for single parameters when the meaning is clear from context
- Use explicit names when `it` would be unclear or when working with multiple nested scopes
- Prefer named parameters over positional references (`$0`, `$1`) for better readability

**Unified Parameter Syntax:**

This parameter syntax (`{ param1, param2 -> ... }` with default `it`) is **unified across Frel**:

- Anonymous blueprint parameters use it (as shown above)
- Event handler parameters use the same syntax (see [Event Handlers](70_event_handlers.md))
- This consistency makes the language more predictable and easier to learn

**Closure Capture:**

* Anonymous blueprints close over their lexical scope (capture visible values reactively).
* Each inline block produces a unique synthesized blueprint.
* Captured values remain reactive - changes propagate to fragments created from the anonymous
  blueprint automatically.

**Example with Closure:**

```frel
blueprint A() {
    i = 1

    column {
        row {
            b(i)  // Anonymous blueprint { b(i) } captures 'i'
        }
    }
}
```

Desugars to:

```frel
blueprint A() {
    i = 1

    blueprint __anon_row() {
        b(i)  // 'i' captured from parent scope
    }

    blueprint __anon_column() {
        row { at content: __anon_row }
    }

    column { at content: __anon_column }
}
```

The anonymous blueprints capture `i` reactively. When `i` changes in the runtime fragment, the child
fragments created from `b(i)` automatically receive the updated value. The intermediate blueprints (
`column` and `row`) don't need to know about `i` - it's automatically carried in the closure.

### 4. Instructions: Inner vs Postfix Syntax

Instructions (layout, styling, event handlers) are always prefixed with `..` and can appear in two positions:

1. **Inner Syntax** - Inside the blueprint's content block: `box { .. width { 300 } }`
2. **Postfix Syntax** - After the blueprint: `box { } .. width { 300 }`

Both forms are semantically identical and apply to the created fragment's root node.

**Syntax:**

The `..` prefix is required for all instructions, distinguishing them from fragment creation:

```frel
box {
    .. width { 300 }      // instruction (has .. prefix)
    .. height { 200 }     // instruction (has .. prefix)
    text { "Hello" }      // fragment creation (no prefix)
}
```

**Precedence:**

- All instructions are applied in the order they appear in the source code.
- Postfix instructions are applied after inner instructions.
- When the same instruction is applied multiple times, the last one wins.
- For multi-parameter instructions parameters are **added**. For example:
    - `.. padding { top : 16 } .. padding { bottom : 16 }` is equivalent to
      `.. padding { top : 16, bottom : 16 }`
    - `.. padding { top : 16, bottom : 16 } .. padding { bottom : 32 }` is equivalent to
      `.. padding { top : 16, bottom : 32 }`

**Instruction Chaining:**

Multiple instructions can be chained using `..`:

```frel
// Postfix chaining
box { } .. width { 300 } .. height { 200 } .. padding { 16 }

// Inner with chaining
box {
    .. width { 300 } .. height { 200 } .. padding { 16 }

    text { "Content" }
}
```

**Style Guidelines:**

Use **inner syntax**:

- When the blueprint has a content block with child elements
- **Convention:** Place instructions before child elements for better readability

```frel
box {
    .. width { 300 }
    .. height { 200 }
    .. padding { 16 }

    text { "Hello" }
}

// Or with chaining:
box {
    .. width { 300 } .. height { 200 } .. padding { 16 }

    text { "Hello" }
}
```

Use **postfix syntax**:

- For single-line declarations to keep code compact
- When the blueprint has no content block (empty `{ }`)
- When adding instructions to a blueprint reference (required)

```frel
text { "Title" } .. font { size: 30 }
box { } .. width { 300 } .. height { 50 }
CustomComponent() .. padding { 16 }
```

**General principles:**

- Prefer inner syntax by default, use postfix for special cases and one-liners
- Place instructions before children when using inner syntax (not mandatory, but improves
  readability)
- Choose the style that maximizes readability for the specific context

### 5. Lifetime and Reactivity

* Child fragments subscribe to values used in parameters or captured by inline blueprints.
* All subscriptions are cleaned up automatically when the parent fragment is destroyed.

## Error Conditions

| Condition                                     | Kind         | Description                                           |
|-----------------------------------------------|--------------|-------------------------------------------------------|
| Unknown blueprint name                        | Compile-time | Not found in scope.                                   |
| Unknown or duplicate parameter                | Compile-time | Invalid or repeated name.                             |
| Type mismatch in parameter                    | Compile-time | Incompatible type.                                    |
| Impure expression                             | Compile-time | Expression has side effects.                          |
| Named argument before positional              | Compile-time | Positional args must come first.                      |
| Too many positional arguments                 | Compile-time | More positional args than parameters.                 |
| Missing required parameter                    | Compile-time | Required parameter not supplied.                      |
| Block supplied but callee has no default slot | Compile-time | Use `at <slot>:` explicitly.                          |
| Unknown slot name                             | Compile-time | Slot not declared by callee.                          |
| Non-blueprint value in slot                   | Compile-time | Slot expects a blueprint.                             |
| Blueprint parameter arity mismatch            | Compile-time | Anonymous blueprint parameters don't match signature. |
| Blueprint parameter type mismatch             | Compile-time | Anonymous blueprint parameter types incompatible.     |

## Examples

### Basic Fragment Creation

```frel
// 1. Default slot only
text { "Hello" }

// 2. Value parameters (positional) + default slot
higherOrder(12) { text { "Body" } }

// 3. Value parameters (named)
higherOrder(width = 12, height = 24) { text { "Body" } }

// 4. Value parameters (mixed: positional then named)
higherOrder(12, height = 24, label = "Title") { text { "Body" } }

// 5. Multi-slot with inline and named fragments
multiSlot {
  at header: { row { text { "Header" } } }
  at item:   ItemRowFragment
  at footer: { row { text { "Footer" } } }
}

// 6. Passing default content by name
column { at content: TwoLabels }

// 7. Postfix styling (for fragment with no content)
text { "Click" } .. padding { 8 } .. border { color: #FF0000 width: 1 }

// 8. Inner styling (for fragment with content)
button {
    .. padding { 8 }
    .. border { color: #FF0000 width: 1 }

    text { "Click" }
}
```

### Blueprint Parameters and Closures

```frel
// Higher-order blueprint with Blueprint parameter
    blueprint Container(content: Blueprint) {
        column {
            .. padding { 16 }
            .. border { color: #D0D5DD width: 1 }
            content()
        }
    }

// Usage - anonymous blueprint with no parameters
Container {
    text { "Hello World" }
}

// Blueprint expecting a String parameter
blueprint TextWrapper(content: Blueprint<String>) {
    hw : String = "Hello World!"
    content(hw)
}

// Usage - anonymous blueprint with explicit parameter
TextWrapper { s ->
    text { "The text is: ${s}" }
}

// Usage - anonymous blueprint with positional reference
TextWrapper {
    text { "The text is: ${$0}" }
}

// Closure capture example
blueprint Parent() {
    count : u32 = 0

    Container {
        // Anonymous blueprint captures 'count' from parent scope
        text { "Count: ${count}" }
        button { "Increment" } .. on_click { count = count + 1 }
    }
}

// Multiple blueprint parameters
blueprint Layout(
    header: Blueprint<String>,
    content: Blueprint,
    footer: Blueprint<i32, bool>
) {
    column {
        header("My App")
        content()
        footer(42, true)
    }
}

// Usage with multiple slots
Layout {
    at header: { title ->
        text { title } .. font { size: 24 weight: Bold }
    }
    at content: {
        text { "Main content here" }
    }
    at footer: { count, isActive ->
        text { "Items: ${count}, Active: ${isActive}" }
    }
}

// Complex closure example
blueprint ListManager() {
    items : List<String> = ["A", "B", "C"]
    selected = 0

    column {
        // Nested anonymous blueprints all capture parent values
        repeat on items by $0 as item {
            button {
                // This anonymous blueprint captures both 'item' and 'selected'
                text { item }
                when selected == _index {
                    text { " ✓" }
                }
            } .. on_click {
                selected = _index
            }
        }
    }
}
```

## Built-in Slots

### Tooltip Slot

All blueprints support an optional `tooltip` slot for contextual help.
When using the tooltip slot, all content must use `at` syntax:

```frel
button {
    at content: { text { "Save" } }
    at tooltip: {
        .. padding { 6 }
        .. background { color: #000000 }
        text { "Ctrl+S to save" } .. font { color: #FFFFFF }
    }
}
```

The tooltip automatically:

- Renders in the `tooltip` channel
- Shows on hover with 500ms delay
- Hides when pointer leaves
- Positions relative to parent (smart repositioning if needed)

See [Detached UI - Tooltip](80_detached_ui.md#tooltip) for full details.
