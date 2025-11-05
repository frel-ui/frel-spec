# Frel

**Frel:** a language for building reactive, declarative, and composable user interfaces. It is
accompanied by a compiler and a runtime. Frel is a DSL (domain-specific language) that is
compiled to a host language.

## Glossary

**Host Language:** The programming language used to implement backends, commands, and complex business
logic. Examples: Rust, TypeScript, Kotlin. Each host language needs a compiler plugin that generates
appropriate code from the Frel DSL.

**Host Platform:** The UI platform that the application runs on. Examples: web browser, iOS, Android,
GTK, desktop (via Skia or similar). Each host platform needs a runtime adapter that provides the
necessary integrations.

[**Module**](05_modules/10_module_basics.md): A logical namespace for organizing Frel declarations.
Each `.frel` file must declare its module (e.g., `module frel.ui.components`), and declarations from
other modules are imported with `use` statements. Modules provide structure and prevent naming conflicts.

[**Store**](20_reactive_state/10_store_basics.md): A named reactive variable that forms the foundation of
Frel's state management system. Stores automatically track dependencies and propagate changes, enabling
declarative reactive UIs. There are four kinds: read-only (`decl`), writable, fan-in, and source stores.

[**Source**](20_reactive_state/50_sources.md): A special store that produces values asynchronously from
external systems - like timers, network requests, or event streams. Sources represent effectful operations
that happen outside the fragment's control, with native status tracking (Loading, Ready, Error).

[**Type System**](10_data_modeling/10_data_basics.md): Frel has a static type system with type inference.
Types include primitives (i32, f64, bool, String), collections (Set<T>, List<T>, Map<K,V>, Tree<T>),
optional types (T?), enums, schemes. The type system ensures safety while remaining host-language independent.

[**Enum**](10_data_modeling/50_enums.md): Declaration of a fixed set of named variants for type-safe
categorical data. Used for state machines, status values, and configuration options. Enums provide
compile-time safety and automatic string conversion.

[**Scheme**](10_data_modeling/60_schemes.md): Declaration of a structured data type with built-in
validation, constraints, and metadata. Schemes define the shape of data with typed fields and
validation rules, supporting automatic form generation and data binding.

[**Contract**](30_contracts/10_contract_basics.md): Declaration of an interface to external services and remote APIs.
Contracts define available operations without implementation details (URLs, authentication,
transport), which are bound at runtime.

[**Backend**](40_backends/10_backend_basics.md): Declaration of a reactive state container with behavior.
Backends encapsulate related stores (decl, writable, fanin, source) along with commands and lifecycle hooks
that operate on those stores. They separate business logic from UI declarations and compose with other backends.

[**Command**](40_backends/10_backend_basics.md#commands): An async method declared in a backend that can be
called from event handlers. Commands are implemented in the host language and are the primary way to trigger
complex business logic with side effects.

[**Theme**](60_themes/10_theme_basics.md): A reusable styling configuration to be used in a fragment template.

[**Resource**](50_resources/10_resource_basics.md): A reusable UI asset to be used in a fragment template.

[**Fragment**](70_fragment/10_fragment_basics.md): Declaration of a reusable UI component. A fragment
has a name, parameters, and a body containing stores, UI elements, and event handlers. Fragments are
instantiated at runtime and compose to build the complete user interface.

[**Event Handler**](70_fragment/70_event_handlers.md): A callback that executes in response to user
interactions or system events. Event handlers are the only place in Frel where side effects are allowed -
they can mutate stores and call backend commands.

**[Frel Expressions](15_expressions/10_expression_basics.md):** Pure expressions that form Frel's
own expression language. These are used throughout the DSL for store initializers, derived stores,
conditionals, and attribute values. Frel expressions are pure by design (no side effects) and
host-language independent.

**[Error Handling](90_error_handling/10_error_basics.md):** Frel has no error handling
constructs at the language level. Errors are simply application state - sources have built-in error
states, and commands set store values to indicate failures. The UI renders error states like any
other data, keeping the language purely declarative.

## How Concepts Fit Together

Frel applications are built from these interconnected pieces:

1. **Data Model**: Define your domain with **Enums** and **Schemes**
2. **Services**: Declare external APIs with **Contracts**
3. **Business Logic**: Create **Backends** that contain:
   - **Stores** (reactive state including **Sources**)
   - **Commands** (methods with side effects, implemented in the host language)
   - Dependencies on **Contracts**
4. **UI**: Build **Fragments** that:
   - Reference **Backends** (via `with`)
   - Bind to **Stores** (reactive data)
   - Respond to user input via **Event Handlers**
   - Call **Commands** from event handlers
5. **Styling**: Apply **Themes** and use **Resources**

All expressions throughout Frel use **Frel Expressions** - a pure, host-independent expression language.

## Design Philosophy

Frel has its own expression language that is independent of any host language. This allows
the same Frel code to work with different backend implementations (Rust, TypeScript, Kotlin, etc.)

The general syntax of control statements is intentionally different from host language syntax
to avoid confusion and maintain clear boundaries.

The main target platform (for now) is the web browser. However, the DSL is designed to be portable
to other platforms.