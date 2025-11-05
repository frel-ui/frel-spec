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

## Expressions and Statements

**[Frel Expressions](15_expressions/10_expression_basics.md):** Pure expressions that form Frel's
own expression language. These are used throughout the DSL for store initializers, derived stores,
conditionals, and attribute values. Frel expressions are pure by design (no side effects) and
host-language independent.

**Statement Context:** Event handlers are the only place in the Frel DSL where side effects are allowed.
Event handlers contain store assignments and backend command calls. Backend lifecycle hooks and command
implementations are written entirely in the host language (not in Frel DSL).

## Hosts

> [!NOTE]
>
> Frel has its own expression language that is independent of any host language. This allows
> the same Frel code to work with different backend implementations (Rust, TypeScript, Python, etc.)
> The general syntax and control statements are intentionally different from host language syntax
> to avoid confusion and maintain clear boundaries.
>
> Similarly, the main target platform (for now) is the web browser. However, the DSL is
> designed to be portable to other platforms.
>

**Host Language:** The programming language used to implement backends, commands, and complex business
logic. Each host language needs a compiler backend that generates appropriate code from the Frel DSL.

**Host Platform:** The UI platform that the application runs on. This can be "browser",
Android, iOS, GTK, skia, etc. Each host platform needs a runtime adapter that provides
the necessary integrations.

## Additional information

- [**Expressions**](15_expressions/10_expression_basics.md)
- [**Box Model**](70_fragment/20_box_model.md)
- [**Standard Fragments**](70_fragment/30_standard_fragments.md)
- [**Resources**](50_resources/10_resource_basics.md)
- [**Themes**](60_themes/10_theme_basics.md)
