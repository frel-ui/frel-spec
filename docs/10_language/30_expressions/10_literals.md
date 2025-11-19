# Literals

Literals are constant values written directly in Frel expressions.

**Note:** Type annotations are mandatory for all field declarations in Frel.

## Numbers

### Integer Literals

```frel
zero : i32 = 0
positive : i32 = 42
negative : i32 = -17
large : i32 = 1_000_000  // Underscores for readability

// Different bases
hex : i32 = 0xFF        // Hexadecimal
binary : i32 = 0b1010   // Binary
octal : i32 = 0o755     // Octal
```

**Properties:**
- Literal must fit in the declared type
- Supports different integer types: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`

### Floating Point Literals

```frel
pi : f64 = 3.14159
small : f64 = 0.001
scientific : f64 = 1.5e-10
negative : f64 = -2.5
```

**Properties:**
- Supports `f32` and `f64` floating point types

## Strings

### String Literals

```frel
name : String = "Alice"
empty : String = ""
```

**Escape sequences:**
```frel
escaped : String = "Hello \"World\"\n\tIndented"
price : String = "Total: \$99"
// Supported: \n \r \t \\ \" \' \$
```

**Note:** Multi-line strings are not supported. Use escape sequences like `\n` for line breaks.

### String Templates

String templates allow embedded expressions:

```frel
name : String = "Alice"
age : i32 = 30
greeting : String = "Hello, ${name}! You are ${age} years old."

// Nested expressions
message : String = "Total: ${price * quantity}"
status : String = "Status: ${isActive ? 'Active' : 'Inactive'}"

// Field access in templates
info : String = "User: ${user.name}, Email: ${user.email}"
```

**Rules:**
- Use `${expression}` for interpolation
- Expression can be any valid Frel expression
- Result is always a `String`

## Booleans

```frel
yes : bool = true
no : bool = false
```

## Null

```frel
nothing : i32? = null
```

**Usage:**
- Represents absence of value
- Only valid for optional types (`T?`)
- `null` can be assigned to any optional type

```frel
maybeNumber: i32? = null
maybeName: String? = null
value: i32? = null
```

## Collections

Collection literals create ordered sequences:

```frel
// Simple collections
numbers : List<i32> = [1, 2, 3, 4, 5]
names : List<String> = ["Alice", "Bob", "Charlie"]

// Empty collection
empty: List<i32> = []

// Nested collections
matrix : List<List<i32>> = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
]

// With trailing comma
items : List<String> = [
    "first",
    "second",
    "third",
]
```

**Properties:**
- All elements must be compatible with the declared element type

**Note:** To access elements, use `repeat` for iteration. Direct index access is not supported.

## Objects

Object literals create structured data:

```frel
// Simple object
point : Point = {
    x: 10,
    y: 20
}

// Nested objects
user : User = {
    name: "Alice",
    age: 30,
    address: {
        street: "123 Main St",
        city: "Springfield"
    }
}

// With computed values
bounds : Bounds = {
    left: x,
    top: y,
    right: x + width,
    bottom: y + height
}

// Trailing comma allowed
config : Config = {
    enabled: true,
    timeout: 5000,
}
```

**Usage:**
- Typically used for scheme construction
- Field names must be valid identifiers
- Field values can be any expression
- Type annotation specifies the scheme type


## Literal Type Rules

### Numeric Literal Adaptation

Numeric literals adapt to their target type context:

```frel
// Integer literal in i32 context
count : i32 = 42

// Same literal in f64 context evaluates as f64
price : f64 = 42         // Evaluates as 42.0

// Literal in expression with f64
total : f64 = 3.14 * 2   // 2 evaluates as 2.0
```

For type conversions between variables, use host language backend functions.

### String Templates

Use string templates for building strings:

```frel
// String templates for any type
message : String = "Count: ${42}"
label : String = "Active: ${true}"
info : String = "User: ${user.name}"

// Templates are the only way to build strings
greeting : String = "Hello, ${firstName} ${lastName}!"
```

**Note:** String concatenation with `+` operator is not supported.

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

currentStatus : Status = Status.Loading
```

### Scheme Construction

```frel
scheme Point {
    x : i32
    y : i32
}

// Using object literal syntax
origin : Point = Point { x: 0, y: 0 }
```

## Examples

### Combining Literals

```frel
blueprint Example() {
    // Numbers
    count : i32 = 42
    price : f64 = 19.99
    total : f64 = count * price

    // Strings
    firstName : String = "Alice"
    lastName : String = "Smith"
    fullName : String = "${firstName} ${lastName}"
    greeting : String = "Hello, ${fullName}!"

    // Booleans
    isActive : bool = true
    isExpensive : bool = price > 50.0
    shouldShow : bool = isActive && !isExpensive

    // Collections
    numbers : List<i32> = [1, 2, 3, 4, 5]

    // Objects
    user : User = {
        name: fullName,
        active: isActive,
        balance: total
    }

    // Conditionals with literals
    status : String = isActive ? "Online" : "Offline"
    badge : String = count > 99 ? "99+" : "${count}"
}
```
