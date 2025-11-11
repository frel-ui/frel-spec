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

[**Type System**](../10_data_model/10_reactivity_model.md): Frel has a static type system with type inference.
Types include primitives (i32, f64, bool, String), collections (Set<T>, List<T>, Map<K,V>, Tree<T>),
optional types (T?), enums, schemes. The type system ensures safety while remaining host-language independent.

[**Enum**](../10_data_model/20_intrinsic_types.md#enums): Declaration of a fixed set of named variants for type-safe
categorical data. Used for state machines, status values, and configuration options. Enums provide
compile-time safety and automatic string conversion.

[**Scheme**](../10_data_model/35_schemes.md): Declaration of a structured data type with built-in
validation, constraints, and metadata. Schemes define the shape of data with typed fields and
validation rules, supporting automatic form generation and data binding.

**Contract**: Declaration of an interface to external services and remote APIs.
Contracts define available operations without implementation details (URLs, authentication,
transport), which are bound at runtime.

**Backend**: Declaration of a reactive state container with behavior.
Backends encapsulate related stores (decl, writable, fanin, source) along with commands and lifecycle hooks
that operate on those stores. They separate business logic from UI declarations and compose with other backends.

**Command**: An async method declared in a backend that can be
called from event handlers. Commands are implemented in the host language and are the primary way to trigger
complex business logic with side effects.

**Theme**: A reusable styling configuration to be used in a blueprint.

**Resource**: A reusable UI asset to be used in a blueprint.

**Blueprint**: Declaration of a reusable UI component template. A blueprint
has a name, parameters, and a body containing stores, UI elements, and event handlers. Blueprints are
written in the Frel language and define how to build fragments.

**Fragment**: The runtime instance of a blueprint that contains stores,
child fragments, and rendered UI elements. Fragments are created from blueprints and compose to build the
complete user interface.

**Event Handler**: A callback that executes in response to user
interactions or system events. Event handlers are the only place in Frel where side effects are allowed -
they can mutate stores and call backend commands.

**Frel Expressions**: Pure expressions that form Frel's
own expression language. These are used throughout the DSL for store initializers, derived stores,
conditionals, and attribute values. Frel expressions are pure by design (no side effects) and
host-language independent.

**Error Handling:** Frel has no error handling
constructs at the language level. Errors are simply application state - sources have built-in error
states, and commands set store values to indicate failures. The UI renders error states like any
other data, keeping the language purely declarative.

### Example

```frel
contract MessageAPI {
    call send(message : String)
    receive() : Message
}

scheme User {
    id   : UUID .. identity
    name : String
}

scheme Message {
    id      : UUID .. identity
    sender  : User
    content : String .. blank { false } 
    sentAt  : Instant
}

arena UserArena {
    for User
}

arena MessageArena {
    for Message with MessageAPI // automatically manages background task with `receive`
}

backend MessageBackend {
    use MessageAPI
    
    theme : MessageTheme
    new_message : String
    
    command send_message()
}

resource Colors {
    self_background
    received_background
}

resource Strings {
    new_message
}

resource Graphics {
    send
}

theme MessageTheme {

    corner_radius = 10

    group message_container {
        corner_radius { corner_radius }
        width { container }
    }

    group self_container {
        background { color: self_background }
        align_self_left
    }
    
    group received_container { 
        background { color: received_background }
        align_self_right
    }
    
}

blueprint MessageList {
    with MessageBackend
    
    column {
        column {
            repeat on MessageArena { message ->
                text { message.content }
                 .. if message.user.is_self { theme.self_container } else { theme.received_container }               
            }
        }
        row {
            text_editor { new_message }
            icon { Graphics.send } .. on_click { send_message }
        }
    }
}
```

```kotlin
fun MessageBackend.send_message() {
    MessageAPI.send(new_message)
    new_message = ""
}

suspend fun MessageArena.run() {
    while (isActive) {
        this.process(MessageAPI.receive())
    }
}
```