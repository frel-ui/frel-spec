# Operators

Frel supports standard arithmetic, comparison, and logical operators.

## Arithmetic Operators

### Basic Arithmetic

```frel
decl sum = a + b           // Addition
decl difference = a - b    // Subtraction
decl product = a * b       // Multiplication
decl quotient = a / b      // Division
decl remainder = a % b     // Modulo/remainder
decl power = a ** b        // Exponentiation
```

**Type rules:**
- Both operands must be numeric types
- Result type is the wider of the two operand types
- Integer division truncates (use `f64` for floating division)

### Unary Operators

```frel
decl negated = -x          // Negation
decl positive = +x         // Positive (no-op, for symmetry)
```

### Precedence

From highest to lowest:

1. Unary: `-`, `+`
2. Exponentiation: `**`
3. Multiplicative: `*`, `/`, `%`
4. Additive: `+`, `-`

```frel
decl result = -2 ** 3        // -8 (negation has higher precedence)
decl result2 = 2 + 3 * 4     // 14 (multiplication first)
decl result3 = (2 + 3) * 4   // 20 (parentheses override)
```

## Comparison Operators

```frel
decl equal = a == b          // Equal
decl notEqual = a != b       // Not equal
decl lessThan = a < b        // Less than
decl lessOrEqual = a <= b    // Less than or equal
decl greaterThan = a > b     // Greater than
decl greaterOrEqual = a >= b // Greater than or equal
```

**Type rules:**
- Operands must have compatible types
- Result is always `bool`
- Numeric types can be compared with each other
- Strings compare lexicographically
- Enums compare by variant order

**Examples:**

```frel
// Numbers
decl check1 = 42 == 42.0     // true (numeric equality)
decl check2 = 10 < 20        // true

// Strings
decl check3 = "abc" < "def"  // true (lexicographic)
decl check4 = "hello" == "hello"  // true

// Booleans
decl check5 = true == false  // false

// Enums
enum Priority { Low, Medium, High }
decl check6 = Priority::Low < Priority::High  // true
```

## Logical Operators

### Binary Logical

```frel
decl and = a && b            // Logical AND
decl or = a || b             // Logical OR
```

**Type rules:**
- Both operands must be `bool`
- Result is `bool`
- Short-circuit evaluation

**Short-circuit behavior:**

```frel
// AND: Right side not evaluated if left is false
decl safe = count > 0 && total / count > 10

// OR: Right side not evaluated if left is true
decl hasValue = cache.get(key) || fetchRemote(key)
```

### Unary Logical

```frel
decl not = !condition        // Logical NOT
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
decl result = !a || b && c   // (!a) || (b && c)
decl result2 = a || b && c   // a || (b && c)
```

## Ternary Conditional

```frel
decl result = condition ? valueIfTrue : valueIfFalse
```

**Type rules:**
- Condition must be `bool`
- Both branches must have compatible types
- Result type is the common type of both branches

**Examples:**

```frel
// Simple
decl status = isActive ? "Active" : "Inactive"

// Nested
decl label = count == 0 ? "empty" :
             count == 1 ? "single" :
             "multiple"

// With expressions
decl price = isPremium ? basePrice * 1.5 : basePrice

// Type compatibility
decl value: i32? = hasValue ? someNumber : null
```

## String Operators

### Concatenation

```frel
decl fullName = firstName + " " + lastName
decl greeting = "Hello, " + name + "!"
```

**Type rules:**
- At least one operand must be `String`
- Other types are automatically converted to string
- Result is always `String`

**Examples:**

```frel
decl message = "Count: " + 42              // "Count: 42"
decl label = "Active: " + true             // "Active: true"
decl info = "Price: $" + (19.99 * 1.1)     // "Price: $21.989"
```

### String Templates (Preferred)

```frel
// Instead of concatenation, use templates
decl fullName = "${firstName} ${lastName}"
decl greeting = "Hello, ${name}!"
decl message = "Count: ${count}"
```

## Operator Chaining

Operators can be chained naturally:

```frel
// Arithmetic
decl result = a + b * c - d / e

// Comparisons (chaining creates AND)
decl inRange = min <= value && value <= max

// Logical
decl valid = isActive && !isExpired && hasPermission

// Mixed
decl shouldShow = count > 0 && (isAdmin || isOwner)
```

## Operator Precedence Table

Complete precedence from highest to lowest:

| Level | Operators           | Description              | Associativity |
|-------|---------------------|--------------------------|---------------|
| 1     | `()` `[]` `.`       | Grouping, access         | Left-to-right |
| 2     | `!` `-` `+`         | Unary                    | Right-to-left |
| 3     | `**`                | Exponentiation           | Right-to-left |
| 4     | `*` `/` `%`         | Multiplicative           | Left-to-right |
| 5     | `+` `-`             | Additive                 | Left-to-right |
| 6     | `<` `<=` `>` `>=`   | Relational               | Left-to-right |
| 7     | `==` `!=`           | Equality                 | Left-to-right |
| 8     | `&&`                | Logical AND              | Left-to-right |
| 9     | `||`                | Logical OR               | Left-to-right |
| 10    | `? :`               | Ternary conditional      | Right-to-left |

## Type Coercion

Frel has minimal automatic type coercion:

### Allowed Coercions

```frel
// Integer to float
decl result: f64 = 42        // 42 → 42.0

// Any type to string (with + operator or templates)
decl message = "Value: " + 42     // OK
decl template = "Value: ${42}"    // OK
```

### NOT Allowed

```frel
// Float to integer
decl bad: i32 = 3.14        // Error: use Math.floor/ceil/round

// Number to boolean
decl bad = if count { ... } // Error: use count > 0

// String to number
decl bad = "42" + 10        // Error: use parseInt("42")

// Null to non-optional
decl bad: i32 = null        // Error: use i32? instead
```

## Examples

### Complex Expressions

```frel
blueprint Calculator(a: i32, b: i32, c: i32) {
    // Arithmetic
    decl result1 = (a + b) * c
    decl result2 = a ** 2 + b ** 2
    decl average = (a + b + c) / 3

    // Comparisons
    decl allPositive = a > 0 && b > 0 && c > 0
    decl inOrder = a <= b && b <= c
    decl hasMax = a >= b && a >= c

    // Logical
    decl valid = allPositive && inOrder
    decl needsCheck = !valid || hasMax

    // Ternary
    decl status = valid ? "OK" : "Invalid"
    decl label = allPositive ?
                 inOrder ? "Ascending" : "Unordered" :
                 "Has negatives"

    // Mixed
    decl score = (a + b + c) / 3
    decl grade = score >= 90 ? "A" :
                 score >= 80 ? "B" :
                 score >= 70 ? "C" :
                 score >= 60 ? "D" : "F"
}
```

### Practical Usage

```frel
blueprint UserCard(user: User) {
    // Field access with operators
    decl fullName = user.firstName + " " + user.lastName
    decl age = user.birthYear != null ? 2024 - user.birthYear : null

    // Validation
    decl hasEmail = user.email.length > 0
    decl hasPhone = user.phone != null && user.phone.length > 0
    decl canContact = hasEmail || hasPhone

    // Conditional display
    decl statusColor = user.isActive ?
                       user.isPremium ? Color::Gold : Color::Green :
                       Color::Gray

    decl statusText = user.isActive ? "Active" : "Inactive"
    decl badge = user.isPremium ? "👑 Premium" : ""

    // Formatting
    decl displayName = "${fullName}${badge}"
    decl subtitle = canContact ?
                    "Contact available" :
                    "No contact info"
}
```
