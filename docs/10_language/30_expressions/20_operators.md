# Operators

Frel supports standard arithmetic, comparison, and logical operators for display logic.

## Arithmetic Operators

### Basic Arithmetic

```frel
sum : i32 = a + b           // Addition
difference : i32 = a - b    // Subtraction
product : i32 = a * b       // Multiplication
quotient : i32 = a / b      // Division
remainder : i32 = a % b     // Modulo/remainder
power : i32 = a ** b        // Exponentiation
```

**Type rules:**
- Both operands must be the same numeric type
- No automatic widening between types (e.g., cannot mix `i32` and `f64`)
- Integer division truncates (use `f64` for floating point division)

**Common uses:**
```frel
// Display formatting
total : f64 = price * quantity
age : i32 = currentYear - birthYear
percentage : f64 = (value / max) * 100.0
```

### Unary Operators

```frel
negated : i32 = -x          // Negation
positive : i32 = +x         // Positive (no-op, for symmetry)
```

### Precedence

From highest to lowest:

1. Unary: `-`, `+`
2. Exponentiation: `**`
3. Multiplicative: `*`, `/`, `%`
4. Additive: `+`, `-`

```frel
result : i32 = -2 ** 3        // -8 (negation has higher precedence)
result2 : i32 = 2 + 3 * 4     // 14 (multiplication first)
result3 : i32 = (2 + 3) * 4   // 20 (parentheses override)
```

## Comparison Operators

```frel
equal : bool = a == b          // Equal
notEqual : bool = a != b       // Not equal
lessThan : bool = a < b        // Less than
lessOrEqual : bool = a <= b    // Less than or equal
greaterThan : bool = a > b     // Greater than
greaterOrEqual : bool = a >= b // Greater than or equal
```

**Type rules:**
- Operands must have compatible types
- Result is always `bool`
- Numeric types can be compared with each other
- Strings compare lexicographically
- Enums compare by variant order

**Examples:**

```frel
// Numbers (same type)
check1 : bool = 42 == 42       // true
check2 : bool = 10 < 20        // true
check3 : bool = 3.14 > 2.71    // true

// Strings
check4 : bool = "abc" < "def"  // true (lexicographic)
check5 : bool = "hello" == "hello"  // true

// Booleans
check6 : bool = true == false  // false

// Enums
enum Priority { Low, Medium, High }
check7 : bool = Priority.Low < Priority.High  // true
```

## Logical Operators

### Binary Logical

```frel
and : bool = a && b            // Logical AND
or : bool = a || b             // Logical OR
```

**Type rules:**
- Both operands must be `bool`
- Result is `bool`
- Short-circuit evaluation

**Short-circuit behavior:**

```frel
// AND: Right side not evaluated if left is false
safe : bool = count > 0 && total / count > 10

// OR: Right side not evaluated if left is true
hasData : bool = cache.has(key) || remote.has(key)
```

### Unary Logical

```frel
not : bool = !condition        // Logical NOT
```

**Type rules:**
- Operand must be `bool`
- Result is `bool`

### Precedence

From highest to lowest:

1. Unary: `!`
2. AND: `&&`
3. OR: `||`

```frel
result : bool = !a || b && c   // (!a) || (b && c)
result2 : bool = a || b && c   // a || (b && c)
```

## Ternary Conditional

```frel
result : Type = condition ? valueIfTrue : valueIfFalse
```

**Type rules:**
- Condition must be `bool`
- Both branches must have compatible types
- Result type is the common type of both branches

**Examples:**

```frel
// Simple conditional display
status : String = isActive ? "Active" : "Inactive"

// Nested ternary
label : String = count == 0 ? "empty" :
                 count == 1 ? "single" :
                 "multiple"

// With expressions
price : f64 = isPremium ? basePrice * 1.5 : basePrice

// Type compatibility
value: i32? = hasValue ? someNumber : null
```

## Null Coalescing (Elvis Operator)

```frel
result : T = optionalValue ?: defaultValue
```

The Elvis operator `?:` provides a concise way to handle null values with defaults.

**Type rules:**
- Left side must be optional type `T?`
- Right side must be type `T` (same type, can be optional if result is optional)
- Result type is `T` (non-optional) if right side is non-optional
- Result type is `T?` (optional) if right side is optional
- Only checks for `null` (not empty strings, zero, false, etc.)

**Examples:**

```frel
// Simple null coalescing (result is non-optional)
displayName : String = user.name ?: "Anonymous"
avatarUrl : String = user.avatarUrl ?: "/default.png"
count : i32 = stats.totalCount ?: 0

// With optional chaining
profileImage : String = user?.profile?.avatar?.url ?: "/default-avatar.png"

// Chaining multiple defaults (right-associative)
primaryEmail : String = user.primaryEmail ?: user.backupEmail ?: "no-email@example.com"

// Result can be optional if right side is optional
maybeName : String? = user.name ?: user.nickname  // Both are String?
fallbackId : i32? = primary.id ?: backup.id       // Both are i32?

// More readable than ternary for null checks
// Instead of:
userName : String = user.name != null ? user.name : "Guest"
// Use:
userName : String = user.name ?: "Guest"
```

**Precedence:**
- Lower than equality operators
- Higher than ternary conditional `? :`
- Right-associative (like ternary)

## String Templates

For building strings, use string templates:

```frel
fullName : String = "${firstName} ${lastName}"
greeting : String = "Hello, ${name}!"
message : String = "Count: ${count}"
```

**Type rules:**
- Use `${expression}` syntax for interpolation
- Any expression can be embedded
- Result is always `String`

**Examples:**

```frel
// Simple interpolation
label : String = "Active: ${isActive}"
info : String = "Price: $${price}"

// With expressions
total : String = "Total: ${price * quantity}"
status : String = "Status: ${isActive ? 'Active' : 'Inactive'}"

// Field access
userInfo : String = "User: ${user.name}, Email: ${user.email}"
```

**Note:** String concatenation with `+` operator is not supported. Use templates instead.

## Operator Chaining

Operators can be chained naturally:

```frel
// Arithmetic
result : i32 = a + b * c - d / e

// Comparisons
inRange : bool = min <= value && value <= max

// Logical
valid : bool = isActive && !isExpired && hasPermission

// Mixed
shouldShow : bool = count > 0 && (isAdmin || isOwner)
```

## Operator Precedence Table

Complete precedence from highest to lowest:

| Level | Operators         | Description              | Associativity |
|-------|-------------------|--------------------------|---------------|
| 1     | `()`              | Grouping                 | Left-to-right |
| 2     | `!` `-` `+`       | Unary                    | Right-to-left |
| 3     | `**`              | Exponentiation           | Right-to-left |
| 4     | `*` `/` `%`       | Multiplicative           | Left-to-right |
| 5     | `+` `-`           | Additive                 | Left-to-right |
| 6     | `<` `<=` `>` `>=` | Relational               | Left-to-right |
| 7     | `==` `!=`         | Equality                 | Left-to-right |
| 8     | `&&`              | Logical AND              | Left-to-right |
| 9     | `\|\|`            | Logical OR               | Left-to-right |
| 10    | `?:`              | Null coalescing (Elvis)  | Right-to-left |
| 11    | `? :`             | Ternary conditional      | Right-to-left |

## Type Compatibility

Frel uses strict type checking with minimal automatic conversions:

### Literal Adaptation

Numeric literals adapt to their target type context:

```frel
// Integer literal used in i32 context
count : i32 = 42

// Same literal used in f64 context evaluates as f64
price : f64 = 42         // Evaluates as 42.0

// Literal adapts in expressions
total : f64 = 3.14 * 2   // 2 evaluates as 2.0 in f64 context
```

### No Mixed-Type Arithmetic

```frel
a : i32 = 10
b : f64 = 3.14

// ❌ Error: cannot mix i32 and f64
result : f64 = a * b

// ✅ OK: literals adapt to context
result : f64 = 10 * b    // 10 evaluates as 10.0
```

### String Templates Only

```frel
// ✅ OK: Use string templates
message : String = "Value: ${42}"
label : String = "Count: ${count}"

// ❌ Error: No string concatenation with +
// message : String = "Value: " + 42
```

### No Truthy/Falsy Coercion

```frel
// ✅ OK: Explicit boolean condition
when count > 0 { ... }
when name.length > 0 { ... }

// ❌ Error: No implicit boolean conversion
when count { ... }        // Error: expected bool, found i32
when name { ... }         // Error: expected bool, found String
```

### Type Conversions in Host Language

For conversions between types, use host language backend functions:

```frel
backend Calculator {
    floatValue : f64 = 3.14

    // Conversion happens in host language
    intValue : i32 = floor_value()
}
```

```rust
// Rust implementation
impl Calculator {
    fn floor_value(&self) -> i32 {
        self.floatValue.floor() as i32
    }
}
```

## Examples

### Display Logic

```frel
blueprint UserCard(user: User) {
    // Field access with string templates
    fullName : String = "${user.firstName} ${user.lastName}"
    age : i32? = user.birthYear != null ? 2024 - user.birthYear : null

    // Validation for display
    hasEmail : bool = user.email.length > 0
    hasPhone : bool = user.phone != null && user.phone.length > 0
    canContact : bool = hasEmail || hasPhone

    // Conditional display
    statusColor : Color = user.isActive ?
                          user.isPremium ? Gold : Green :
                          Gray

    statusText : String = user.isActive ? "Active" : "Inactive"
    badge : String = user.isPremium ? "Premium" : ""

    // Formatting with templates
    displayName : String = "${fullName} ${badge}"
    subtitle : String = canContact ?
                        "Contact available" :
                        "No contact info"
}
```

### When to Use Host Language

For complex calculations, use the host language:

```frel
// ❌ Don't do complex calculations in Frel
score : f64 = (
    metric1 * weight1 +
    metric2 * weight2 +
    metric3 * weight3 +
    adjustment_factor * baseline
) / normalization_constant

// ✅ Do calculations in host language
backend Analytics {
    // Computed in Rust/TypeScript/etc
    score : f64 = calculate_weighted_score()
}

blueprint ScoreDisplay {
    with Analytics

    // Simple display formatting in Frel
    scoreText : String = "Score: ${score}"
    rating : String = score > 0.8 ? "Excellent" :
                      score > 0.6 ? "Good" :
                      score > 0.4 ? "Fair" : "Poor"
}
```
