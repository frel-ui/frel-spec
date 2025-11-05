# Fragment Creation

Fragment creation statements instantiate child fragments within the body of a fragment.
A **block** following a fragment name is an **inline fragment**, conceptually a small anonymous fragment definition.
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
* The block `{ ... }` is always an **inline fragment definition**, not an instance.
* In a multi-slot block, each `at <slot>:` binds a fragment (inline or named) to a slot parameter.
* A plain block binds to the callee's **default slot** (usually named `content`).

## Syntax (Informal)

```text
<creation>     ::= <name> [ "(" <arg-list> ")" ] <block-or-slots>? <postfix>*
<arg-list>     ::= <arg> { "," <arg> }
<arg>          ::= <expr> | <param-name> "=" <expr>

<block-or-slots> ::= <default-block> | <slot-block>
<default-block>  ::= "{" <body> "}"
<slot-block> ::= "{" <slot-binding> { <separator> <slot-binding> } "}"
<slot-binding>   ::= "at" <slot-name> ":" <fragment-value>
<separator>  ::= "," | newline
<fragment-value> ::= <inline-fragment> | <fragment-ref>
<inline-fragment> ::= [ <param-list> "->" ] "{" <body> "}"
<param-list>     ::= <param-name> { "," <param-name> }
<fragment-ref>    ::= <name>

<postfix>        ::= ".." <instruction>

# Fragment parameter types
<fragment-type> ::= "Fragment" [ "<" <fragment-params> ">" ]
<fragment-params> ::= <fragment-param> { "," <fragment-param> }
<fragment-param> ::= [<store-kind>] <type>
<store-kind> ::= "writable" | "source" | "decl" | "fanin"
```

## Semantics

### 1. Value Parameters

* Supplied only inside `(...)`.
* Parentheses are **optional** when there are no parameters (e.g., `Counter { }` is equivalent to `Counter() { }`).
* May be passed **positionally** or by **name**:
  - Positional: `SomeFragment(12, "text")`
  - Named: `SomeFragment(width = 12, label = "text")`
  - Mixed: `SomeFragment(12, label = "text")` — positional arguments must come before named ones
* Once a named argument is used, all subsequent arguments must also be named.
* Missing required parameters or extra parameters are compile-time errors.
* Named arguments may appear in any order (after positional ones).
* `<expr>` must be a pure host language expression (no side effects).
* Reactive dependencies from stores are tracked automatically.

### 2. Fragment Parameters (Default Slot & Named Slots)

* A plain block `{ ... }` provides a fragment for the **default slot**.
* A slot block `{ at slot1: ..., at slot2: ... }` provides fragments for **named slots**.
* Fragments may be given inline (`{ ... }`) or by reference (`FragmentName`).
* If a fragment has only a default slot, the short form `{ ... }` is preferred.
* For explicitness, named passing is allowed:
  `higherOrder(12) { at content: FragmentName }`

**Fragment Parameter Types**

Fragment parameters use the `Fragment<P1,...Pn>` type to declare what parameters they expect:

```frel
fragment Container(content: Fragment) {
    // content is a fragment with no parameters
    content()
}

fragment TextWrapper(content: Fragment<String>) {
    decl hw = "Hello World!"
    content(hw)  // Must pass a String
}

fragment ItemRenderer(
    header: Fragment<String, bool>,
    item: Fragment<User>,
    footer: Fragment
) {
    header("Title", true)
    item(current_user)
    footer()
}
```

When invoking a fragment parameter, you must pass arguments matching its type signature:

```frel
fragment Parent() {
    TextWrapper {
        at content: { s ->  // Anonymous fragment receives String 's'
            text { s }
        }
    }

    // Or pass by reference
    TextWrapper { at content: MyTextFragment }
}

fragment MyTextFragment(text: String) {
    text { text }
}
```

### 3. Desugaring and Type Inference

Each inline block is desugared into a compiler-synthesized **anonymous fragment definition** and passed by name.

**Basic Example:**

```frel
column { text { "A" } }
```

becomes conceptually:

```frel
fragment __anon_1() { text { "A" } }
column { at content: __anon_1 }
```

**Type Inference for Anonymous Fragments:**

The compiler infers the `Fragment<...>` type for anonymous fragments based on the expected parameter type:

```frel
fragment TextWrapper(content: Fragment<String>) {
    decl hw = "Hello World!"
    content(hw)
}

// Usage with anonymous fragment
TextWrapper {
    // Compiler infers this anonymous fragment has type Fragment<String>
    text { "The text is: ${$0}" }  // $0 is the first parameter (String)
}
```

The desugared form:

```frel
fragment __anon_1(p0: String) {  // Parameter inferred from Fragment<String>
    text { "The text is: ${p0}" }
}
TextWrapper { at content: __anon_1 }
```

**Anonymous Fragment Parameter Syntax:**

When an anonymous fragment has parameters, use parameter names or positional references:

```frel
// Named parameter (explicit)
TextWrapper { s ->
    text { "Text: ${s}" }
}

// Positional reference (implicit)
TextWrapper {
    text { "Text: ${$0}" }
}

// Multiple parameters
ItemRenderer {
    at header: { title, isActive ->
        text { "${title} - ${isActive}" }
    }
}

// Or with positional references
ItemRenderer {
    at header: {
        text { "${$0} - ${$1}" }
    }
}
```

**Closure Capture:**

* Anonymous fragments close over their lexical scope (capture visible stores reactively).
* Each inline block produces a unique synthesized fragment definition.
* Captured stores remain reactive - changes propagate to the anonymous fragment automatically.

**Example with Closure:**

```frel
fragment A() {
    decl i = 1

    column {
        row {
            b(i)  // Anonymous fragment { b(i) } captures 'i'
        }
    }
}
```

Desugars to:

```frel
fragment A() {
    decl i = 1

    fragment __anon_row() {
        b(i)  // 'i' captured from parent scope
    }

    fragment __anon_column() {
        row { at content: __anon_row }
    }

    column { at content: __anon_column }
}
```

The anonymous fragments capture `i` reactively. When `i` changes, `b(i)` automatically receives the updated value. The intermediate fragments (`column` and `row`) don't need to know about `i` - it's automatically carried in the closure.

### 4. Instructions: Inner vs Postfix Syntax

Instructions (layout, styling, event handlers) can be written in two ways:

1. **Inner Syntax** - Inside the fragment's content block: `box { width { 300 } }`
2. **Postfix Syntax** - After the fragment using `..`: `box { } .. width { 300 }`

Both forms are semantically identical and apply to the fragment's root node.

**Precedence:**

- All instructions are applied in the order they appear in the source code.
- Postfix instructions are applied after inner instructions.
- When the same instruction is applied multiple times, the last one wins.
- For multi-parameter instructions parameters are **added**. For example:
  - `padding { top : 16 } .. padding { bottom : 16 }` is equivalent to `padding { top : 16, bottom : 16 }`
  - `padding { top : 16, bottom : 16 } .. padding { bottom : 32 }` is equivalent to `padding { top : 16, bottom : 32 }`

**Instruction Chaining:**

Multiple instructions can be chained using `..` regardless of syntax:

```frel
// Postfix chaining
box { } .. width { 300 } .. height { 200 } .. padding { 16 }

// Inner chaining
box {
    width { 300 } .. height { 200 }
    padding { 16 }
    
    text { "Content" }
}
```

**Style Guidelines:**

Use **inner syntax**:
- When the fragment has a content block with child fragments
- **Convention:** Place instructions before child fragments for better readability

```frel
box {
    width { 300 }
    height { 200 }
    padding { 16 }
    
    text { "Hello" }
}

// Or with chaining:
box {
    width { 300 } .. height { 200 } .. padding { 16 }
    
    text { "Hello" }
}
```

Use **postfix syntax**:
- For single-line declarations to keep code compact
- When the fragment has no content block (empty `{ }`)
- When adding instructions to a fragment reference (required)

```frel
text { "Title" } .. font { size: 30 }
box { } .. width { 300 } .. height { 50 }
CustomComponent() .. padding { 16 }
```

**General principles:**
- Prefer inner syntax by default, use postfix for special cases and one-liners
- Place instructions before children when using inner syntax (not mandatory, but improves readability)
- Choose the style that maximizes readability for the specific context

### 5. Lifetime and Reactivity

* Child fragments subscribe to stores used in value parameters or captured by inline fragments.
* All subscriptions are cleaned up automatically when the parent is destroyed.

## Error Conditions

| Condition                                     | Kind         | Description                                          |
|-----------------------------------------------|--------------|------------------------------------------------------|
| Unknown fragment name                         | Compile-time | Not found in scope.                                  |
| Unknown or duplicate parameter                | Compile-time | Invalid or repeated name.                            |
| Type mismatch in parameter                    | Compile-time | Incompatible type.                                   |
| Impure expression                             | Compile-time | Expression has side effects.                         |
| Named argument before positional              | Compile-time | Positional args must come first.                     |
| Too many positional arguments                 | Compile-time | More positional args than parameters.                |
| Missing required parameter                    | Compile-time | Required parameter not supplied.                     |
| Block supplied but callee has no default slot | Compile-time | Use `at <slot>:` explicitly.                         |
| Unknown slot name                             | Compile-time | Slot not declared by callee.                         |
| Non-fragment value in slot                    | Compile-time | Slot expects a fragment.                             |
| Fragment parameter arity mismatch             | Compile-time | Anonymous fragment parameters don't match signature. |
| Fragment parameter type mismatch              | Compile-time | Anonymous fragment parameter types incompatible.     |
| Fragment store kind mismatch                  | Compile-time | Fragment parameter store kind less restrictive.      |

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
text { "Click" } .. padding { 8 } .. border { Red, 1 }

// 8. Inner styling (for fragment with content)
button {
    padding { 8 }
    border { Red, 1 }

    text { "Click" }
}
```

### Fragment Parameters and Closures

```frel
// Higher-order fragment with Fragment parameter
fragment Container(content: Fragment) {
    column {
        padding { 16 }
        border { Gray, 1 }
        content()
    }
}

// Usage - anonymous fragment with no parameters
Container {
    text { "Hello World" }
}

// Fragment expecting a String parameter
fragment TextWrapper(content: Fragment<String>) {
    decl hw = "Hello World!"
    content(hw)
}

// Usage - anonymous fragment with explicit parameter
TextWrapper { s ->
    text { "The text is: ${s}" }
}

// Usage - anonymous fragment with positional reference
TextWrapper {
    text { "The text is: ${$0}" }
}

// Closure capture example
fragment Parent() {
    decl count = 0

    Container {
        // Anonymous fragment captures 'count' from parent scope
        text { "Count: ${count}" }
        button { "Increment" } .. on_click { count = count + 1 }
    }
}

// Multiple fragment parameters
fragment Layout(
    header: Fragment<String>,
    content: Fragment,
    footer: Fragment<i32, bool>
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
        text { title } .. font { size: 24, weight: Bold }
    }
    at content: {
        text { "Main content here" }
    }
    at footer: { count, isActive ->
        text { "Items: ${count}, Active: ${isActive}" }
    }
}

// Fragment with writable store parameter
fragment Editor(renderer: Fragment<writable String>) {
    writable text = "Initial"
    renderer(text)
}

// Usage - pass writable store to fragment
Editor { writable_text ->
    text_editor { writable_text }
}

// Complex closure example
fragment ListManager() {
    decl items = ["A", "B", "C"]
    writable selected = 0

    column {
        // Nested anonymous fragments all capture parent stores
        repeat on items by $0 as item {
            button {
                // This anonymous fragment captures both 'item' and 'selected'
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

All fragments support an optional `tooltip` slot for contextual help:

```frel
button {
    "Save"
    at tooltip: {
        padding { 6 }
        background { color: Black }
        text { "Ctrl+S to save" } .. font { color: White }
    }
}
```

The tooltip automatically:
- Renders in the `tooltip` channel
- Shows on hover with 500ms delay
- Hides when pointer leaves
- Positions relative to parent (smart repositioning if needed)

See [Detached UI - Tooltip](80_detached_ui.md#tooltip) for full details.