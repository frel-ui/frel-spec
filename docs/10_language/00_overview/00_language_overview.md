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