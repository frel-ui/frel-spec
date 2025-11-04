# Frel

Frel is a user interface DSL for building reactive, declarative, and composable user interfaces.

This document is a specification of Frel.

**Frel**: A declarative language for describing user interfaces.

## Glossary

[**Enum**](10_data_modeling/50_enums.md): Declaration of a fixed set of named variants for type-safe
categorical data. Used for state machines, status values, and configuration options. Enums provide
compile-time safety and automatic string conversion.

[**Scheme**](10_data_modeling/60_schemes.md): Declaration of a structured data type with built-in
validation, constraints, and metadata. Schemes define the shape of data with typed fields and 
validation rules, supporting automatic form generation and data binding.

[**Contract**](30_contracts/10_contract_basics.md): Declaration of an interface to external services and remote APIs.
Contracts define available operations without implementation details (URLs, authentication,
transport), which are bound at runtime.

[**Backend definition**](40_backends/10_backend_basics.md): Declaration of a reactive state container with behavior.
Backends encapsulate related stores (decl, writable, fanin, source) along with commands and lifecycle hooks
that operate on those stores. They separate business logic from UI declarations and compose with other backends.
At runtime, backend definitions are instantiated into backends.

[**Theme**](60_themes/10_theme_basics.md): A reusable styling configuration to be used in a fragment template.

[**Resource**](50_resources/10_resource_basics.md): A reusable UI asset to be used in a
fragment template.

[**Fragment definition**](70_fragment/10_fragment_basics.md): Declaration of a reusable UI component
that is instantiated at runtime as a fragment. Its surface breaks down into a name, parameters,
and a body of DSL statements that construct layout, state, and logic.

## Hosts

> [!NOTE]
>
> While the main language of the library is Rust, the DSL is quite independent of the
> main language. Only expressions are written in Rust, the general syntax and the control
> statements are intentionally different from the Rust syntax. The reason behind this is
> to avoid confusion and unambiguity.
>
> Similarly, the main target platform (for now) is the web browser. However, the DSL is
> designed to be portable to other platforms.
>

**Host Language:** The programming language that is used for expressions and statements in the DSL,
also the target language for the generated Fragment IR. Each host language needs a compile-time
plugin that translates the DSL into Fragment IR.

**Host Language Expression (HLE):** An expression that is written in the host language. These
are used in the DSL to construct the fragment's logic. Expressions evaluate to a value.

**Pure HLE (PHLE):** An expression that is written in the host language and does not have any
side effects. Pure HLE expressions are allowed in the DSL body for store initializers, derived
stores, fragment parameters, and control flow conditions.

**Host Language Statement (HLS):** A statement that is written in the host language. Statements
may have side effects (assignments, I/O, control flow, etc.). HLS are only allowed inside event
handler bodies.

**Host Platform:** The UI platform that the host language runs on. This can be "browser",
Android, iOS, GTK, skia etc. Each host platform needs a runtime adapter that provides
the necessary integrations.

## Additional information

- [**Box Model**](70_fragment/20_box_model.md)
- [**Standard Fragments**](70_fragment/30_standard_fragments.md)
- [**Resources**](50_resources/10_resource_basics.md)
- [**Themes**](60_themes/10_theme_basics.md)
