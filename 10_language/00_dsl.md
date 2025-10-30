# Fragment DSL

This document is not a user guide but a specification of the Fragment DSL. Definitions
introduced in the other part of the documentation are simply referenced here.

The **Fragment DSL** is a declarative language for describing **fragment templates**.

A fragment template declares a reusable component that compiles to Fragment IR and 
is instantiated at runtime as a fragment. Its surface breaks down into a name, parameters, 
and a body of DSL statements that construct layout, state, and logic.

> [!NOTE] 
>
> While the main language of the library is Rust, the DSL is quite independent of the
> main language. Only expressions are written in Rust, the general syntax and the 
> control statements are intentionally different from the Rust syntax. The reason
> behind this is to avoid confusion and unambiguity.
>

## Hosts

**Host Language:** The programming language that is used for expressions in the DSL, also the
target language for the generated Fragment IR. Each host language needs a compile-time plugin
that translates the DSL into Fragment IR.

**Host Platform:** The UI platform that the host language runs on. This can be "browser",
Android, iOS, GTK, skia etc. Each host platform needs a runtime adapter that provides
the necessary integrations.

## Syntax

The templates are defined using the `fragment!` macro and use the DSL syntax specified below.

A fragment template is composed of:

- a name
- zero or more parameters
- body that contains zero or more statements

```text
<fragment> ::= "fragment!" "{" <name> "(" [ <param-list> ] ")" "{" <body> "}" "}"
<param-list> ::= <param> { "," <param> } 
<param> ::= <param-name> ":" <param-type>
<body> ::= { <statement> }
```

### Name

**Definition** 

`<name>` is the identifier of the fragment template; it is the symbol used to refer
to (instantiate) the template from other templates. It exists at compile time and does not create
a runtime instance by itself.

**Rules**

- `<name>` must be a valid Rust identifier (case-sensitive, not a Rust keyword).

### Parameters

**Definition** 

Each `<param>` declares an external store the fragment reads from (data it does not own).
Parameters supply inputs and wiring for reactivity from parents into the fragment.

**Rules**

- `<param-name>` must be a valid Rust identifier.
- `<param-type>` must be a valid Rust type (path/type expression).
- Lifetimes are not supported in parameter types (e.g., `&'a str`, `Foo<'a>` are rejected), as 
  they do not translate cleanly to the reactive app-state model.
- Generics: Generic types are allowed without lifetimes (e.g., Vec<String>, Option<u32>).

### Body

**Definition**

`<body>` is a sequence of DSL statements that build the fragment's structure and behavior. 

**Rules**

- The body may be empty.
- When present, statements must conform to the DSL’s statement categories and their respective semantics (reactive/pure expression constraints, etc.).

**Statements**

- [**Store declarations**](10_store_declarations.md)
- [**Fragment creation**](20_fragment_creation.md)
- [**Control statements**](30_control_statements.md)
- [**Instructions**](40_instructions.md)
- [**Detached UI**](50_detached_ui.md)

## Example

```frel
fragment Counter(label : String) {
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

## Additional information

- [**Box Model**](box_model.md)
- [**Standard Templates**](standard_templates.md)
- [**Resources**](60_resources.md)
- [**Themes**](70_themes.md)
