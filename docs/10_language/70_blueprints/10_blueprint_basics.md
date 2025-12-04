# Blueprint Basics

## Syntax

Blueprints are declared using the `blueprint` keyword and use the DSL syntax specified below.

A blueprint is composed of:

- a name
- zero or more parameters
- body that contains zero or more statements

```text
<blueprint> ::= "blueprint" <name> [ "(" <param-list> ")" ] "{" <body> "}"
<param-list> ::= <param> { "," <param> }
<param> ::= <param-name> ":" <param-type> [ "=" <default-expr> ]
<body> ::= { <statement> }
```

### Name

**Definition**

`<name>` is the identifier of the blueprint; it is the symbol used to refer
to (instantiate) the blueprint from other blueprints. It exists at compile time and does not create
a runtime fragment instance by itself.

**Rules**

- `<name>` must be a valid host language identifier (case-sensitive, not a host language keyword).

### Parameters

**Definition**

Each `<param>` declares a reactive value that the fragment receives from its parent.
Parameters enable reactive data flow and between fragments.

**Syntax Examples**

```frel
blueprint UserDisplay(
    name: String,
    count: i32,
    user: ref User,
    total: f64,
    label: String = "Default"
)
```

**Blueprint Parameters**

Parameters can also accept blueprints using the `Blueprint<P1,...Pn>` type. This enables
higher-order blueprints that accept other blueprints as children:

```frel
blueprint Container(
    content: Blueprint             // Blueprint with no parameters
)

blueprint TextWrapper(
    content: Blueprint<String>     // Blueprint expecting one String parameter
)

blueprint Editor(
    header: Blueprint<String, bool>,      // Multiple parameters
    renderer: Blueprint<String>
)
```

During runtime `Blueprint` parameters automatically capture their closure environment, allowing
nested fragments to access parent closure.

**Rules**

- `<param-name>` must be a valid identifier
- `<param-type>` must be a valid Frel type (including `Blueprint<...>`)
- Optional parameters use `?` suffix: `name: String?`
- Default values must be pure Frel expressions

### Body

**Definition**

`<body>` is a sequence of DSL statements that build the fragment's structure and behavior.

**Rules**

- The body may be empty.
- When present, statements must conform to the DSL’s statement categories and their respective
  semantics (reactive/pure expression constraints, etc.).

**Statements**

- [**Backend binding**](#backend-binding) - Connect fragment to backend state
- **Local declarations**
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

1. **Instantiates or references a backend**: Either creates a new backend instance or references a
   backend parameter
2. **Imports backend namespace**: Makes all backend state fields and commands directly accessible
   without qualification

**Rules:**

- Exactly one `with` statement is allowed per blueprint
- The `with` statement must appear before any usage of backend state or commands
- All backend state fields become accessible as unqualified identifiers
- All backend commands become callable as unqualified functions

### Using Backend Parameters

When a backend is passed as a parameter, `with` imports its namespace:

```frel
blueprint UserEditor(editor: Editor) {
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
blueprint UserProfile(user_id: u32) {
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

blueprint UserProfile(user_id: u32) {
    with EditorWithValidation(user_id)

    // Access state from both included backends
    text_editor { username }  // from Editor

    when has_errors {  // from ValidationBackend
        text { error_message }
    }
}
```

This pattern maintains the "one backend per blueprint" rule while allowing composition at the
backend definition level.

## Example

```frel
blueprint Counter(label: String) {
    count = 0

    column {
        .. padding { 16 } .. border { color: #FF0000 width: 1 }

        button {
            .. on_click { count = count + 1 }
            text { "Click me" }
        }

        text { "${label}: ${count}" } .. text_small
    }
}
```
