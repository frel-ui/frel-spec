# Fragment Basics

## Syntax

Fragment definitions are declared using the `fragment` keyword and use the DSL syntax specified below.

A fragment definition is composed of:

- a name
- zero or more parameters
- body that contains zero or more statements

```text
<fragment> ::= "fragment" <name> [ "(" <param-list> ")" ] "{" <body> "}"
<param-list> ::= <param> { "," <param> }
<param> ::= [<store-kind>] <param-name> ":" <param-type> [ "=" <default-expr> ]
<store-kind> ::= "writable" | "source" | "decl" | "fanin"
<body> ::= { <statement> }
```

### Name

**Definition**

`<name>` is the identifier of the fragment definition; it is the symbol used to refer
to (instantiate) the fragment from other fragments. It exists at compile time and does not create
a runtime instance by itself.

**Rules**

- `<name>` must be a valid host language identifier (case-sensitive, not a host language keyword).

### Parameters

**Definition**

Each `<param>` declares a reactive store that the fragment receives from its parent.
Parameters enable reactive data flow and store sharing between fragments.

**Store Kind**

Parameters can specify what kind of store they accept. For detailed information
see [Store Basics](../20_reactive_state/10_store_basics.md).

When no store kind is specified, `decl` is assumed (read-only reactive).

**Syntax Examples**

```frel
fragment UserDisplay(
    name: String,              // Same as: decl name: String
    writable count: i32,       // Writable store parameter
    source user: User,         // Source store parameter
    decl total: f64,           // Explicit read-only store
    label: String = "Default"  // Default value
)
```

**Fragment Parameters**

Parameters can also accept fragments using the `Fragment<P1,...Pn>` type. This enables higher-order fragments that accept other fragments as children:

```frel
fragment Container(
    content: Fragment             // Fragment with no parameters
)

fragment TextWrapper(
    content: Fragment<String>     // Fragment expecting one String parameter
)

fragment Editor(
    header: Fragment<String, bool>,      // Multiple parameters
    renderer: Fragment<writable String>  // With explicit store kind
)
```

Fragment parameters automatically capture their closure environment, allowing nested fragments to access parent stores. See [Fragment Creation](40_fragment_creation.md) for details on how fragment parameters work and how type inference determines the `Fragment<...>` type for anonymous fragments.

**Rules**

- `<param-name>` must be a valid identifier
- `<param-type>` must be a valid Frel type (including `Fragment<...>`)
- Optional parameters use `?` suffix: `name: String?`
- Default values must be pure Frel expressions
- Store kind prefix is optional (defaults to `decl`)
- For `Fragment<...>` parameters, store kinds in the type signature default to `decl` when omitted

### Body

**Definition**

`<body>` is a sequence of DSL statements that build the fragment's structure and behavior.

**Rules**

- The body may be empty.
- When present, statements must conform to the DSL’s statement categories and their respective semantics (reactive/pure expression constraints, etc.).

**Statements**

- [**Backend binding**](#backend-binding) - Connect fragment to backend state
- [**Store declarations**](20_reactive_state/10_store_basics.md)
- [**Data modeling**](25_data_modeling/20_schemes.md) - Schemes and enums
- [**Fragment creation**](40_fragment_creation.md)
- [**Control statements**](50_control_statements.md)
- [**Instructions**](60_instructions.md)
- [**Event handlers**](70_event_handlers.md)
- [**Detached UI**](80_detached_ui.md)

## Backend Binding

### Syntax

```text
<with-statement> ::= "with" <backend-expr>
<backend-expr> ::= <backend-constructor> | <backend-param>
<backend-constructor> ::= <backend-name> [ "(" <arg-list> ")" ]
<backend-param> ::= <param-name>
```

### Semantics

The `with` keyword declares the backend state for a fragment. It serves two purposes:

1. **Instantiates or references a backend**: Either creates a new backend instance or references a backend parameter
2. **Imports backend namespace**: Makes all backend state fields and commands directly accessible without qualification

**Rules:**

- Exactly one `with` statement is allowed per fragment
- The `with` statement must appear before any usage of backend state or commands
- All backend state fields become accessible as unqualified identifiers
- All backend commands become callable as unqualified functions

### Using Backend Parameters

When a backend is passed as a parameter, `with` imports its namespace:

```frel
fragment UserEditor(editor: Editor) {
    with editor

    // Access backend state directly
    text_editor { username }
    text_editor { email }

    // Call backend commands directly
    button { "Save" } .. on_click { save() }

    // Backend state in conditionals
    when save_state.pending {
        spinner { }
    }
}
```

### Creating Backend Instances

When a backend needs to be created locally, `with` instantiates and imports it:

```frel
fragment UserProfile(user_id: u32) {
    with Editor(user_id)

    column {
        text_editor { username }
        text_editor { email }

        button { "Save" }
            .. enabled { !save_state.pending }
            .. on_click { save() }
    }
}
```

### Multiple Backends via Composition

If a fragment needs multiple backends, compose them using `include`:

```frel
backend EditorWithValidation(user_id: u32) {
    include Editor(user_id)
    include ValidationBackend
}

fragment UserProfile(user_id: u32) {
    with EditorWithValidation(user_id)

    // Access state from both included backends
    text_editor { username }  // from Editor

    when has_errors {  // from ValidationBackend
        text { error_message }
    }
}
```

This pattern maintains the "one backend per fragment" rule while allowing composition at the backend definition level.

## Example

```frel
fragment Counter(label: String) {
    decl count = 0

    column {
        padding { 16 } .. border { Red, 1 }

        button {
            on_click { count = count + 1 }
            text { "Click me" }
        }

        text { "${label}: ${count}" } .. text_small
    }
}
```