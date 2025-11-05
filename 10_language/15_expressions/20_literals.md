# Literals

Literals are constant values written directly in Frel expressions.

## Numbers

### Integer Literals

```frel
decl zero = 0
decl positive = 42
decl negative = -17
decl large = 1_000_000  // Underscores for readability

// Different bases
decl hex = 0xFF        // Hexadecimal
decl binary = 0b1010   // Binary
decl octal = 0o755     // Octal
```

**Type inference:**
- Literals without decimal point infer to `i32` by default
- Can be explicitly typed: `42u32`, `42i64`
- Must fit in the target type

### Floating Point Literals

```frel
decl pi = 3.14159
decl small = 0.001
decl scientific = 1.5e-10
decl negative = -2.5
```

**Type inference:**
- Literals with decimal point infer to `f64` by default
- Can be explicitly typed: `3.14f32`

## Strings

### String Literals

```frel
decl name = "Alice"
decl empty = ""
decl multiline = "This is a
multi-line string"
```

**Escape sequences:**
```frel
decl escaped = "Hello \"World\"\n\tIndented"
// Supported: \n \r \t \\ \" \'
```

### String Templates

String templates allow embedded expressions:

```frel
decl name = "Alice"
decl age = 30
decl greeting = "Hello, ${name}! You are ${age} years old."

// Nested expressions
decl message = "Total: ${price * quantity}"
decl status = "Status: ${isActive ? 'Active' : 'Inactive'}"

// Field access in templates
decl info = "User: ${user.name}, Email: ${user.email}"
```

**Rules:**
- Use `${expression}` for interpolation
- Expression can be any valid Frel expression
- Result is always a `String`

## Booleans

```frel
decl yes = true
decl no = false
```

## Null

```frel
decl nothing = null
```

**Usage:**
- Represents absence of value
- Only valid for optional types (`T?`)
- `null` can be assigned to any optional type

```frel
decl maybeNumber: i32? = null
decl maybeName: String? = null

// Type inference with null requires annotation
decl value: i32? = null  // Must specify type
```

## Arrays

Array literals create ordered, indexed collections:

```frel
// Simple arrays
decl numbers = [1, 2, 3, 4, 5]
decl names = ["Alice", "Bob", "Charlie"]
decl mixed = [1, "two", true]  // Type: List<Any>

// Empty array (needs type annotation)
decl empty: List<i32> = []

// Nested arrays
decl matrix = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
]

// With trailing comma
decl items = [
    "first",
    "second",
    "third",
]
```

**Type inference:**
- Array type inferred from elements: `[1, 2, 3]` → `List<i32>`
- All elements must have compatible types
- Mixed types infer to common supertype or `Any`

## Objects

Object literals create structured data:

```frel
// Simple object
decl point = {
    x: 10,
    y: 20
}

// Nested objects
decl user = {
    name: "Alice",
    age: 30,
    address: {
        street: "123 Main St",
        city: "Springfield"
    }
}

// With computed values
decl bounds = {
    left: x,
    top: y,
    right: x + width,
    bottom: y + height
}

// Trailing comma allowed
decl config = {
    enabled: true,
    timeout: 5000,
}
```

**Usage:**
- Typically used for scheme construction
- Field names must be valid identifiers
- Field values can be any expression
- Type inferred from structure

## Type Annotations

Literals can have explicit type annotations:

```frel
decl x: i32 = 42
decl y: f64 = 3.14
decl items: List<String> = ["a", "b", "c"]
decl maybe: i32? = null
```

## Literal Type Rules

### Number Coercion

```frel
// Integer to float - implicit
decl result: f64 = 42  // OK: 42 → 42.0

// Float to integer - explicit only
decl rounded: i32 = Math.floor(3.14)  // OK
decl bad: i32 = 3.14  // Error: Cannot convert f64 to i32
```

### String Concatenation

```frel
// String + any type
decl message = "Count: " + 42           // "Count: 42"
decl label = "Active: " + true          // "Active: true"
decl info = "User: " + user.name        // "User: Alice"

// Or use templates
decl message = "Count: ${42}"
```

### Boolean Context

Only `bool` type is valid in boolean contexts:

```frel
// OK
when isActive { ... }

// Error - no truthy/falsy coercion
when count { ... }  // Error: expected bool, found i32
when name { ... }   // Error: expected bool, found String

// Must be explicit
when count > 0 { ... }      // OK
when name.length > 0 { ... } // OK
```

## Special Literals

### Enum Variants

```frel
enum Status {
    Loading
    Ready
    Error
}

decl currentStatus = Status::Loading
```

### Scheme Construction

```frel
scheme Point {
    x .. i32
    y .. i32
}

// Using object literal syntax
decl origin = Point { x: 0, y: 0 }
```

## Examples

### Combining Literals

```frel
fragment Example() {
    // Numbers
    decl count = 42
    decl price = 19.99
    decl total = count * price

    // Strings
    decl firstName = "Alice"
    decl lastName = "Smith"
    decl fullName = firstName + " " + lastName
    decl greeting = "Hello, ${fullName}!"

    // Booleans
    decl isActive = true
    decl isExpensive = price > 50.0
    decl shouldShow = isActive && !isExpensive

    // Arrays
    decl numbers = [1, 2, 3, 4, 5]
    decl doubled = numbers.map(x => x * 2)

    // Objects
    decl user = {
        name: fullName,
        active: isActive,
        balance: total
    }

    // Conditionals with literals
    decl status = isActive ? "Online" : "Offline"
    decl badge = count > 99 ? "99+" : "${count}"
}
```
