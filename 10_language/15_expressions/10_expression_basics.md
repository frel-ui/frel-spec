# Expression Basics

Frel has its own expression language for defining reactive computations, store initializers, and
conditional logic. This expression language is pure (side-effect free) by design and independent of
any host language.

## Overview

**Frel expressions** are used throughout the DSL for:

- Store initialization values
- Derived store computations
- Conditional expressions in control flow
- Fragment parameter defaults
- Attribute values

All Frel expressions are **pure** - they cannot perform I/O, mutations, or other side effects. This
purity is guaranteed by the language design itself, not by validation.

## Design Principles

1. **Pure by design**: No assignments, I/O, or side effects possible
2. **Host-independent**: Same expression language works with any backend implementation language
3. **Type-safe**: Full type checking at compile time
4. **Readable**: Clear, familiar syntax inspired by modern languages
5. **Limited scope**: Complex logic belongs in backends, not expressions

## Expression vs Statement Contexts

Frel distinguishes between two contexts:

### Expression Context (Pure)

Used in:

- Store initializers: `decl doubled = count * 2`
- Derived stores: `decl total = price * quantity`
- Conditionals: `when age >= 18 { ... }`
- Fragment parameters: `fragment User(name: String = "Guest")`
- Attribute values: `.. width { containerWidth * 0.8 }`

**Only Frel expressions allowed** - no side effects possible. Backend commands cannot be called from expressions.

### Statement Context (Effectful)

Used in:

- Event handlers: `.. on_click { count = count + 1 }`

**Side effects allowed** - this is the only place where:
- Writable stores can be mutated: `count = count + 1`
- Backend commands can be called: `save()`, `load_user()`

Event handlers contain a sequence of statements, where each statement is either:
1. **Store assignment**: `<store> = <frel-expr>` - assigns a pure Frel expression to a writable store
2. **Command call**: `<command-name>(<args>)` - calls a backend command with Frel expression arguments

**No host language control flow** - use Frel's control flow constructs (`when`, `select`) outside the handler, or implement complex logic in backend commands.

Note: Backend lifecycle hooks and command implementations are written entirely in the host language, not in Frel DSL.

## What's Included

The Frel expression language includes:

- **Literals**: Numbers, strings, booleans, null, arrays, objects
- **Identifiers**: Variable references, parameters, store names
- **Operators**: Arithmetic, comparison, logical
- **Field access**: `user.profile.name`
- **Optional chaining**: `user?.profile?.name`
- **Array indexing**: `items[0]`
- **Method calls**: `text.toUpperCase()` (from whitelist)
- **Lambdas**: `items.map(x => x * 2)` (for functional operations)
- **Conditionals**: `age >= 18 ? "adult" : "minor"`
- **Pattern matching**: `match status { ... }`
- **String templates**: `"Hello, ${name}!"`

## What's NOT Included

These are not allowed in Frel expressions:

- **Assignments**: `x = x + 1` ❌ (use event handlers)
- **Backend commands**: `save()`, `load()` ❌ (call from event handlers only)
- **I/O operations**: `console.log()`, `fetch()` ❌ (use sources or commands)
- **Async/await**: `await getData()` ❌ (use sources)
- **Loops**: `for`, `while` ❌ (use functional methods like `map`, `filter`, `reduce`)
- **Multiple statements**: `{ stmt1; stmt2; stmt3 }` ❌
- **Side effects**: Anything that modifies external state ❌

## Examples

### Simple Expressions

```frel
fragment Calculator(a: i32, b: i32) {
    // Arithmetic
    decl sum = a + b
    decl difference = a - b
    decl product = a * b
    decl quotient = a / b

    // Comparisons
    decl isEqual = a == b
    decl isGreater = a > b

    // Logical
    decl bothPositive = a > 0 && b > 0
    decl eitherNegative = a < 0 || b < 0

    // Conditional
    decl sign = sum > 0 ? "positive" : sum < 0 ? "negative" : "zero"
}
```

### Field Access

```frel
fragment UserProfile(user: User) {
    decl fullName = user.firstName + " " + user.lastName
    decl city = user.address.city
    decl firstTag = user.tags[0]

    // Optional chaining
    decl avatarUrl = user?.profile?.avatar?.url
}
```

### Functional Operations

```frel
fragment ItemList(items: List<Item>) {
    // Map
    decl prices = items.map(item => item.price)

    // Filter
    decl activeItems = items.filter(item => item.active)

    // Reduce
    decl total = prices.reduce((sum, price) => sum + price, 0)

    // Chaining
    decl expensiveActiveItems = items
        .filter(item => item.active)
        .filter(item => item.price > 100)
        .map(item => item.name)
}
```

### Pattern Matching

```frel
fragment StatusDisplay(status: Status) {
    decl message = match status {
        Status::Loading => "Please wait..."
        Status::Ready => "Done!"
        Status::Error(err) => "Error: ${err}"
    }

    decl color = match status {
        Status::Loading => Color::Blue
        Status::Ready => Color::Green
        Status::Error(_) => Color::Red
    }
}
```

### String Templates

```frel
fragment Greeting(user: User, count: i32) {
    decl greeting = "Hello, ${user.name}!"
    decl itemLabel = "You have ${count} ${count == 1 ? 'item' : 'items'}"
    decl fullMessage = "${greeting} ${itemLabel}"
}
```

## Complex Logic Goes in Backends

When logic is too complex for expressions, implement it in the backend as a command or derived store:

```frel
// ❌ Don't do this - too complex for expressions
decl validated = {
    let errors = [];
    if (name.length < 3) errors.push("Name too short");
    if (!email.includes("@")) errors.push("Invalid email");
    return errors.length === 0;
}

// ✅ Option 1: Break down into simple expressions
backend UserEditor {
    writable name: String = ""
    writable email: String = ""

    // Multiple simple derived stores
    decl hasName = name.length >= 3
    decl hasEmail = email.includes("@")
    decl isValid = hasName && hasEmail
}

// ✅ Option 2: Implement complex logic as a command (in host language)
backend UserEditor {
    writable name: String = ""
    writable email: String = ""

    writable validationErrors: List<String> = []

    // Command declaration (implementation in host language)
    command validate()
}

// Backend implementation (in Rust/TypeScript/etc)
impl UserEditor {
    async fn validate(&mut self) {
        let mut errors = vec![];
        if self.name.len() < 3 {
            errors.push("Name too short".to_string());
        }
        if !self.email.contains('@') {
            errors.push("Invalid email".to_string());
        }
        self.validationErrors.set(errors);
    }
}

// Usage in fragment
fragment UserForm() {
    with UserEditor

    button { "Validate" }
        .. on_click { validate() }  // Call command from event handler

    when validationErrors.length > 0 {
        column {
            repeat on validationErrors as error {
                text { error } .. color { Red }
            }
        }
    }
}
```

## Further Documentation

- [**Literals**](20_literals.md) - Numbers, strings, booleans, arrays, objects
- [**Operators**](30_operators.md) - Arithmetic, comparison, logical operators
- [**Field Access and Calls**](40_field_access.md) - Member access, method calls, indexing
- [**Lambdas**](50_lambdas.md) - Anonymous functions for functional operations
- [**Pattern Matching**](60_pattern_matching.md) - Match expressions and patterns
- [**Type System**](70_type_system.md) - Expression type checking and inference
- [**Standard Library**](80_standard_library.md) - Built-in functions and methods
