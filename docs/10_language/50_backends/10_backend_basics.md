# Backend Basics

Backends in Frel provide stateful data management and host language integration. They serve as the bridge between Frel's declarative UI layer and imperative computation in your host language (Rust, TypeScript, etc.).

## Backend Declaration

Backends are declared at the module level:

```frel
backend MessageBackend {
    message : String = ""
    charCount : i32 = message.length

    method validate() : bool
    command send_message()
}
```

## Backend Components

A backend consists of three kinds of members:

### 1. Fields

Fields hold state and reactive computations:

```frel
backend Counter {
    count : i32 = 0
    doubled : i32 = count * 2
    isEven : bool = count % 2 == 0
}
```

**Field properties:**
- Must have explicit type annotation
- Can have default values (literals or expressions)
- Participate in reactive system
- Can reference other fields in the same backend
- Can call contracts for data loading

**Field initialization:**

```frel
backend UserBackend {
    // Literal initialization
    userId : u32 = 0

    // Expression initialization
    timestamp : i64 = get_current_time()

    // Contract call initialization
    user : User = UserAPI.get_user(userId)

    // Reactive computation
    displayName : String = user.name
}
```

### 2. Methods

Methods are **pure functions** that compute values from backend state:

```frel
backend Analytics {
    rawData : List<Metric> = []
    threshold : f64 = 0.8

    method calculate_score(weight: f64) : f64
    method aggregate_totals() : Summary
    method filter_by_threshold() : List<Metric>
}
```

**Method properties:**
- Declared with `method` keyword
- Must specify return type
- Can take parameters with explicit types
- Are **pure** - no side effects
- Can be called from **any context** (expressions and event handlers)
- Implemented in host language
- Results participate in reactive system

**Method declaration syntax:**

```frel
method method_name() : ReturnType
method method_name(param1: Type1, param2: Type2) : ReturnType
```

### 3. Commands

Commands are **side-effecting operations** triggered by user actions:

```frel
backend TodoBackend {
    input : String = ""
    todos : List<Todo> = []

    command add_todo(text: String)
    command remove_todo(index: i32)
    command clear_all()
    command save_to_storage()
}
```

**Command properties:**
- Declared with `command` keyword
- Can take parameters with explicit types
- Do **not** have return types (implicitly void)
- Have **side effects** (modify state, call APIs, etc.)
- Can **only** be called from **event handlers**
- Cannot be called from expressions
- Implemented in host language

**Command declaration syntax:**

```frel
command command_name()
command command_name(param1: Type1, param2: Type2)
```

## Method vs Command

The key distinction between methods and commands:

| Aspect        | Method                         | Command                    |
|---------------|--------------------------------|----------------------------|
| Keyword       | `method`                       | `command`                  |
| Purity        | Pure function                  | Side effects               |
| Returns       | Value (explicit type)          | Nothing (void)             |
| Callable from | Expressions and event handlers | Event handlers only        |
| Purpose       | Compute values                 | Perform actions            |
| Examples      | Calculate, filter, aggregate   | Save, delete, send, update |

### When to Use Methods

Use methods for:
- Computing derived values from state
- Filtering or transforming data
- Complex calculations that need host language
- Pure queries that don't modify state
- Any computation that returns a value

**Examples:**

```frel
backend ShoppingCart {
    items : List<CartItem> = []
    taxRate : f64 = 0.08

    // Methods - pure computations
    method calculate_subtotal() : f64
    method calculate_tax() : f64
    method calculate_total() : f64
    method find_item(id: u32) : CartItem?
    method filter_by_category(category: String) : List<CartItem>
}
```

### When to Use Commands

Use commands for:
- User-triggered actions
- Modifying backend state
- Calling external APIs
- Saving data
- Any operation with side effects

**Examples:**

```frel
backend ShoppingCart {
    items : List<CartItem> = []

    // Commands - side effects
    command add_item(item: CartItem)
    command remove_item(id: u32)
    command clear_cart()
    command checkout()
    command save_for_later()
}
```

## Using Backends in Blueprints

Blueprints connect to backends using the `with` clause:

```frel
blueprint CounterUI {
    with Counter

    // Access backend fields
    text { "${count}" }

    // Call methods in expressions
    evenLabel : String = is_even() ? "Even" : "Odd"
    text { evenLabel }

    // Call commands in event handlers
    button { "+" }
        .. on_click { increment() }

    button { "Reset" }
        .. on_click { reset() }
}
```

### Calling Methods

Methods are called in expression contexts:

```frel
blueprint Dashboard {
    with Analytics

    // In field declarations
    score : f64 = calculate_score(0.8)
    summary : Summary = aggregate_totals()
    filtered : List<Metric> = filter_by_threshold()

    // In fragment bodies
    text { "Score: ${calculate_score(1.0)}" }

    // In instruction values
    column {\n        text { "Result" }
    }
    .. height { compute_height() }
}
```

### Calling Methods from Event Handlers

Methods can also be called from event handlers since they're pure:

```frel
blueprint FormSubmit {
    with Analytics

    button { "Calculate and Save" }
        .. on_click {
            // Call method to compute value
            score : f64 = calculate_score(0.8)

            // Use the result in a command
            save_score(score)
        }
}
```

### Calling Commands

Commands are called only from event handlers:

```frel
blueprint TodoInput {
    with TodoBackend

    textfield { input }

    button { "Add" }
        .. on_click {
            add_todo(input)
            input = ""
        }

    button { "Clear All" }
        .. on_click {
            clear_all()
        }
}
```