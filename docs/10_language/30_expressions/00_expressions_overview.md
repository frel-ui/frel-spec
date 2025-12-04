# Expression Overview

Frel expressions are the declarative building blocks for defining "what to show" in your UI. They
are designed to be minimal, readable, and focused on display logic rather than computation.

## Design Philosophy

### Frel vs Host Language

Frel makes a clear distinction between two kinds of logic:

- **Frel Expressions (DSL)**: Declarative display logic - "what to show"
- **Host Language**: Imperative computation - "what to calculate/fetch/process"

This separation keeps Frel simple and maintainable while allowing complex logic to be implemented in
the full power of your host language (Rust, TypeScript, etc.).

### Example: Display vs Computation

```frel
// ✅ Good: Simple display formatting in Frel
blueprint UserCard(user: User) {
    greeting : String = "Hello, ${user.firstName} ${user.lastName}!"
    statusColor : Color = user.isActive ? #16A34A : #9CA3AF

    text { greeting } .. color { statusColor }
}

// ✅ Good: Complex computation in host language
backend UserCardBackend {
    user : User = UserAPI.get_user(id)

    // Computed in Rust/TypeScript/etc
    activityScore : f64 = calculate_activity_score()
    recommendations : List<User> = find_similar_users()
}
```

### When to Use Each

**Use Frel expressions for:**

- String formatting and templates
- Simple conditional display (`condition ? a : b`)
- Field access for display (`user.name`)
- Basic arithmetic for formatting (`price * quantity`, `currentYear - birthYear`)
- Calling backend commands and contracts

**Use host language for:**

- Complex validation logic
- Data transformation and filtering
- Business rules and calculations
- API calls and async operations
- Anything requiring loops or complex control flow

## Where Expressions Are Used

Frel expressions appear in several contexts:

### Field Declarations

Fields can be initialized with expressions:

```frel
backend MessageBackend {
    message : String = ""
    charCount : i32 = message.length
    isValid : bool = charCount > 0 && charCount <= 280
}
```

### Blueprint Parameters

Blueprints can have default parameter values:

```frel
blueprint UserBadge(
    name: String = "Guest",
    size: i32 = 24
) {
    // ...
}
```

### Blueprint Bodies

Expressions are used in fragment building:

```frel
blueprint UserProfile(user: User) {
    fullName : String = "${user.firstName} ${user.lastName}"

    text { fullName }

    when user.isPremium {
        icon { "crown" } .. color { #F2C94C }
    }
}
```

### Instruction Values

Styling and layout instructions use expressions:

```frel
box {
    text { "Hello" }
}
.. width { containerWidth * 0.8 }
.. color { isActive ? #16A34A : #9CA3AF }
```

### Event Handlers

Event handlers contain assignments and command calls:

```frel
button { "Send" }
    .. on_click {
        backend.send_message()
        message = ""
    }
```

## Expression Capabilities

### Minimal by Design

Frel expressions are intentionally kept minimal to:

- Keep the DSL simple and easy to learn
- Reduce compiler complexity
- Encourage moving complex logic to host language
- Maintain readability

### What's Included

**Tier 1 - Essential:**

- Literals: `42`, `"text"`, `true`, `null`, `[1, 2, 3]`, `{x: 10, y: 20}`
- Field access: `user.name`, `user.address.city`
- Optional chaining: `user?.profile?.avatar`
- String templates: `"Hello ${name}!"`
- Ternary operator: `condition ? a : b`
- Null coalescing (Elvis): `optionalValue ?: default`

**Tier 2 - Display Logic:**

- Comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Logical operators: `&&`, `||`, `!`
- Simple arithmetic: `+`, `-`, `*`, `/`, `%`, `**`

**Backend Integration:**

- Backend command calls: `backend.save()`
- Contract calls: `UserAPI.get_user(id)`

**Collection Query APIs (Read-only):**

- Length: `.length`

### What's NOT Included

These features are intentionally excluded to keep Frel simple:

- **Collection transformations**: `.filter()`, `.map()`, `.reduce()` - compute in host language
- **Pattern matching**: `match` expressions - postponed, use ternary or host language
- **Index access**: `items[0]` - use iteration with `repeat` instead
- **Complex string methods**: `.split()`, `.replace()` - do in host language
- **Loops**: `for`, `while` - use `repeat` for UI iteration, host language for computation
- **Control flow in handlers**: `if`, `when`, `select` - these are fragment instructions, not
  handler constructs

## Reactive Evaluation

All Frel expressions participate in Frel's reactive system:

### Availability Propagation

Every expression has an availability state:

- **Loading**: Data is being fetched or computed
- **Ready**: Data is available
- **Error**: An error occurred

See [Reactivity Model](../20_data_model/02_type_system.md#availability-propagation) for details.

### Example

```frel
blueprint UserProfile(userId: u32) {
    // Contract call - starts as Loading
    user : User = UserAPI.get_user(userId)

    // Expression inherits user's availability
    greeting : String = "Hello, ${user.name}!"

    // When user is Loading: greeting is Loading
    // When user is Ready: greeting is Ready
    // When user is Error: greeting is Error
}
```

## Further Documentation

- [**Literals**](10_literals.md) - Numbers, strings, booleans, collections, objects
- [**Operators**](20_operators.md) - Arithmetic, comparison, logical operators
- [**Field Access**](30_field_access.md) - Accessing fields, optional chaining, collection queries
- [**Backend and Contract Calls**](40_calls.md) - Calling commands and contracts
- [**Event Handlers**](50_event_handlers.md) - Mutations and side effects
