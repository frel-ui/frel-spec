# Backend and Contract Calls

Frel expressions can call backend methods, backend commands, and contracts.

## Backend Methods

Backend methods are pure functions that compute values from backend state. They can be called from expressions.

### Syntax

```frel
method_name()
method_name(arg1, arg2)
```

### Where Methods Can Be Called

Methods can be called from any context:
- Field declarations
- Fragment bodies
- Instruction values
- Any expression context
- Event handlers

### Calling Methods in Expressions

```frel
backend Analytics {
    rawData : List<Metric> = []
    threshold : f64 = 0.8

    method calculate_score(weight: f64) : f64
    method aggregate_totals() : Summary
    method filter_by_threshold() : List<Metric>
}

blueprint Dashboard {
    with Analytics

    // In field declarations
    score : f64 = calculate_score(0.8)
    summary : Summary = aggregate_totals()
    filtered : List<Metric> = filter_by_threshold()

    // In fragment bodies
    column {
        text { "Score: ${calculate_score(1.0)}" }
        text { "Total: ${summary.total}" }
    }

    // In instruction values
    box {
        text { "Results" }
    }
    .. height { compute_dynamic_height() }
}
```

**Key properties:**
- Methods are pure - no side effects
- Methods return typed values
- Arguments are Frel expressions
- Can be called from any context (expressions and event handlers)
- Results participate in reactive system

## Backend Commands

Backend commands are side-effecting operations that can be called from event handlers.

### Syntax

```frel
command_name()
command_name(arg1, arg2)
```

**Restrictions:**
- Commands can ONLY be called from event handlers
- Commands cannot be called from field declarations or other expressions
- Arguments are Frel expressions

### Calling Commands in Event Handlers

```frel
backend MessageBackend {
    message : String = ""

    command send_message()
    command delete_message(id: u32)
    command clear_all()
}

blueprint MessageInput {
    with MessageBackend

    textfield { message }

    button { "Send" }
        .. on_click {
            send_message()
            message = ""  // Reset input
        }

    button { "Delete First" }
        .. on_click {
            delete_message(1)
        }

    button { "Clear" }
        .. on_click {
            clear_all()
        }
}
```

**Key properties:**
- Commands have side effects
- Commands do not return values
- Can only be called from event handlers
- Arguments are Frel expressions

## Contracts

Contracts define external API operations that return data.

### Declaration

Contracts are declared at the module level:

```frel
contract UserAPI {
    get_user(id: u32) : User
    get_all_users() : List<User>
    search_users(query: String) : List<User>
}
```

### Contract Call Syntax

```frel
ContractName.operation_name(arguments)
```

### Contract Calls in Expressions

Contract calls can be used in field declarations:

```frel
blueprint UserProfile(userId: u32) {
    // Contract call in field declaration
    user : User = UserAPI.get_user(userId)

    // Use the loaded data
    name : String = user.name

    text { name }
}
```

Contract calls in backend fields:

```frel
backend UserListBackend {
    searchQuery : String = ""

    // Contract call with reactive parameter
    searchResults : List<User> = UserAPI.search_users(searchQuery)
}

blueprint UserSearch {
    with UserListBackend

    text_editor { searchQuery }

    resultCount : i32 = searchResults.length
    text { "${resultCount} results" }

    repeat on searchResults { user ->
        text { user.name }
    }
}
```

**Key behaviors:**
- Contract calls are reactive - they re-execute when parameters change
- Results have availability states (see [Reactivity Model](../20_data_model/10_reactivity_model.md))

## Examples

### Methods in Expressions

```frel
backend ShoppingCart {
    items : List<CartItem> = []
    taxRate : f64 = 0.08

    method calculate_subtotal() : f64
    method calculate_tax() : f64
    method calculate_total() : f64
    method item_count() : i32
}

blueprint CartSummary {
    with ShoppingCart

    // Call methods in field declarations
    subtotal : f64 = calculate_subtotal()
    tax : f64 = calculate_tax()
    total : f64 = calculate_total()
    count : i32 = item_count()

    column {
        text { "${count} items" }
        text { "Subtotal: $${subtotal}" }
        text { "Tax: $${tax}" }
        text { "Total: $${total}" }
    }
}
```

### Commands in Event Handlers

```frel
backend Counter {
    count : i32 = 0

    method is_even() : bool
    method abs_value() : i32

    command increment()
    command decrement()
    command reset()
}

blueprint CounterUI {
    with Counter

    // Call method in expression
    evenLabel : String = is_even() ? "Even" : "Odd"

    column {
        text { "${count}" }
        text { evenLabel }
        text { "Absolute: ${abs_value()}" }

        row {
            button { "+" }
                .. on_click { increment() }

            button { "-" }
                .. on_click { decrement() }

            button { "Reset" }
                .. on_click { reset() }
        }
    }
}
```

### Contract Calls for Data Loading

```frel
blueprint TodoApp(userId: u32) {
    // Multiple contract calls
    user : User = UserAPI.get_user(userId)
    todos : List<Todo> = TodoAPI.get_todos_for_user(userId)

    // Use loaded data
    userName : String = user.name
    todoCount : i32 = todos.length

    column {
        text { "Todos for ${userName}" }
        text { "${todoCount} tasks" }

        repeat on todos { todo ->
            text { todo.text }
        }
    }
}
```

### Reactive Contract Calls

```frel
backend SearchBackend {
    query : String = ""

    // Re-executes when query changes
    results : List<Item> = SearchAPI.search(query)
}

blueprint SearchUI {
    with SearchBackend

    textfield { query }

    resultCount : i32 = results.length
    hasResults : bool = results.length > 0

    when hasResults {
        text { "${resultCount} results" }
        repeat on results { item ->
            text { item.title }
        }
    }

    when !hasResults {
        text { "No results" }
    }
}
```

### Methods and Commands Together

```frel
backend TodoBackend {
    input : String = ""
    todos : List<Todo> = []

    method active_count() : i32
    method completed_count() : i32
    method is_valid_input() : bool

    command add_todo(text: String)
    command remove_todo(index: i32)
    command clear_completed()
}

blueprint TodoInput {
    with TodoBackend

    // Call methods in expressions
    activeCount : i32 = active_count()
    completedCount : i32 = completed_count()
    canAdd : bool = is_valid_input()

    column {
        text { "Active: ${activeCount}, Completed: ${completedCount}" }

        textfield { input }

        button { "Add" }
            .. enabled { canAdd }
            .. on_click {
                add_todo(input)
                input = ""
            }

        button { "Clear Completed" }
            .. on_click { clear_completed() }
    }
}
```

### Methods Called from Event Handlers

Since methods are pure, they can also be called within event handlers:

```frel
backend Calculator {
    a : f64 = 0.0
    b : f64 = 0.0
    result : f64 = 0.0

    method compute_sum() : f64
    method compute_product() : f64

    command save_result(value: f64)
}

blueprint CalculatorUI {
    with Calculator

    button { "Calculate Sum" }
        .. on_click {
            // Call method in event handler
            save_result(compute_sum())
        }

    button { "Calculate Product" }
        .. on_click {
            // Call method and use result
            result = compute_product()
        }
}
```

## Guidelines

### When to Use Methods

Use methods for:
- Computing derived values from backend state
- Filtering or transforming data
- Complex calculations that need host language
- Pure queries that don't modify state
- Any computation that returns a value

### When to Use Commands

Use commands for:
- User-triggered actions
- Modifying backend state
- Write operations
- Side effects
- Operations called from event handlers only

### When to Use Contracts

Use contracts for:
- Loading data from external sources
- Reactive data queries
- Read operations from external APIs

## Method vs Command Summary

| Aspect        | Method                         | Command             |
|---------------|--------------------------------|---------------------|
| Purity        | Pure function                  | Side effects        |
| Returns       | Value (typed)                  | Nothing (void)      |
| Callable from | Expressions and event handlers | Event handlers only |
| Purpose       | Compute values                 | Perform actions     |

See [Backend Basics](../50_backends/10_backend_basics.md) for complete backend documentation.
