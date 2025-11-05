# Data Basics

Frel provides a type system for modeling application data. The data model consists of:

- primitive types (integers, floats, booleans, strings, and specialized types)
- collections (lists, sets, maps, trees)
- temporal types (dates, times, durations)
- user-defined structured types (enums and schemes)

## Type System Overview

The Frel type system is designed to be:

- **Platform-independent**: Types map naturally to host languages (Rust, TypeScript, Python)
- **Validatable**: Built-in validation and constraints at the type level
- **Reactive**: Types are either immutable or support fine-grained reactivity out-of-the-box.
- **Composable**: Types can be nested and combined to model complex domains

## Type Categories

### [Primitive Types](20_primitives.md)

Standard scalar types and specialized primitives:

- **Integers**: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`
- **Floating point**: `f32`, `f64`
- **Boolean**: `bool`
- **String**: `String` (Unicode text)
- **Secret**: Sensitive string data (passwords, API keys)
- **Decimal**: Arbitrary-precision decimal numbers (financial calculations)
- **Uuid**: Universally unique identifiers
- **Url**: URL/URI with validation
- **Color**: Color values for styling (RGB, HSL, hex)
- **Blob**: Binary data (files, images)

All primitive types are immutable.

### Fragment Type

The `Fragment<P1,...Pn>` type represents a fragment definition with its closure environment:

- **Fragment**: A fragment with no parameters
- **Fragment<T>**: A fragment with one parameter of type `T`
- **Fragment<T1, T2, ...>**: A fragment with multiple typed parameters

Fragment parameters can specify store kinds:

```frel
fragment Container(content: Fragment<String>) {
    // content expects a String parameter (defaults to 'decl')
}

fragment Editor(renderer: Fragment<writable String, bool>) {
    // renderer expects a writable String and a bool (decl)
}
```

When a fragment is passed as a parameter, it automatically captures its closure - all stores visible in its lexical scope. This means nested fragments have access to parent stores without explicit passing:

```frel
fragment A() {
    decl i = 1
    column {
        row {
            b(i)  // Anonymous fragment { b(i) } captures 'i' in closure
        }
    }
}
```

Store kind defaults follow regular parameter rules: when omitted, `decl` is assumed.

### [Collections](30_collections.md)

Platform-independent collection types with built-in reactivity:

- **List**: Ordered, indexed collection
- **Set**: Unique values, unordered
- **Map**: Key-value pairs (numeric, String, Url, Uuid, enum keys)
- **Tree**: Hierarchical structure with automatic node management

Collections are implemented using Frel-provided wrappers (`FrelList`, `FrelSet`, `FrelMap`, `FrelTree`) around
native host language types.

### [DateTime Types](40_datetime.md)

Standard temporal types for working with dates, times, and durations:

- **Instant**: Point in time (UTC)
- **LocalDate**: Date without time
- **LocalTime**: Time without date
- **LocalDateTime**: Date and time without timezone
- **Timezone**: IANA timezone identifier
- **Duration**: Time span

All temporal types are immutable.

### User-Defined Types

- **[Enums](50_enums.md)**: Fixed sets of named variants
- **[Schemes](60_schemes.md)**: Structured data types with validation

## Type Usage Examples

### Basic Usage

```frel
scheme User {
    // Primitives
    id .. Uuid
        .. default { Uuid::new() }
    username .. String
    age .. u8
    verified .. bool

    // Specialized primitives
    password .. Secret
    avatar_url .. Url
        .. optional
    theme_color .. Color
        .. default { Color::hex("#007bff") }

    // Collections
    tags .. List<String>
    roles .. Set<String>
    preferences .. Map<String, String>

    // DateTime
    created_at .. Instant
    birth_date .. LocalDate
    timezone .. Timezone
}
```

### Nested Collections

```frel
scheme Analytics {
    // List of lists
    data_points .. List<List<f64>>

    // Map with list values
    user_tags .. Map<u64, List<String>>

    // Set of numeric IDs
    active_users .. Set<u64>
}
```

### Optional Types

Any type can be made optional:

```frel
scheme Profile {
    name .. String                     // Required
    bio .. String                      // Required but can be empty
        .. blank { true }
    avatar_url .. String               // Optional
        .. optional
    age .. u8                         // Optional
        .. optional
}
```

## Type Mapping

Frel types map to native types in different host languages:

| Frel Type       | Rust                    | TypeScript          | Python                     |
|-----------------|-------------------------|---------------------|----------------------------|
| `String`        | `String`                | `string`            | `str`                      |
| `i32`, `u64`... | `i32`, `u64`...         | `number`            | `int`                      |
| `f32`, `f64`    | `f32`, `f64`            | `number`            | `float`                    |
| `bool`          | `bool`                  | `boolean`           | `bool`                     |
| `Secret`        | `Secret`                | `Secret`            | `Secret`                   |
| `Decimal`       | `rust_decimal::Decimal` | `Decimal`           | `decimal.Decimal`          |
| `Uuid`          | `uuid::Uuid`            | `string` (UUID)     | `uuid.UUID`                |
| `Url`           | `url::Url`              | `URL`               | `urllib.parse.ParseResult` |
| `Color`         | `Color`                 | `Color`             | `Color`                    |
| `Blob`          | `Vec<u8>`               | `Blob`/`Uint8Array` | `bytes`                    |
| `List<T>`       | `FrelList<T>`           | `FrelList<T>`       | `FrelList[T]`              |
| `Set<T>`        | `FrelSet<T>`            | `FrelSet<T>`        | `FrelSet[T]`               |
| `Map<K,V>`      | `FrelMap<K,V>`          | `FrelMap<K,V>`      | `FrelMap[K,V]`             |
| `Tree<T>`       | `FrelTree<T>`           | `FrelTree<T>`       | `FrelTree[T]`              |
| `Instant`       | `DateTime<Utc>`         | `Date`              | `datetime` (UTC)           |
| `LocalDate`     | `NaiveDate`             | Custom              | `date`                     |
| `LocalTime`     | `NaiveTime`             | Custom              | `time`                     |
| `LocalDateTime` | `NaiveDateTime`         | Custom              | `datetime`                 |
| `Timezone`      | `Tz`                    | `string`            | `ZoneInfo`                 |
| `Duration`      | `Duration`              | Custom              | `timedelta`                |

**Note:** `FrelList`, `FrelSet`, `FrelMap`, and `FrelTree` are Frel-provided wrappers around host
language-specific implementations (e.g., `Vec`, `HashSet`, `HashMap` in Rust) with built-in reactivity support.

## Next Steps

- **[Primitive Types](20_primitives.md)**: All basic and specialized primitive types
- **[Collections](30_collections.md)**: List, Set, Map, and Tree types with built-in reactivity
- **[DateTime Types](40_datetime.md)**: Working with dates, times, and durations
- **[Miscellaneous Types](50_misc.md)**: Reserved for future specialized types
- **[Enums](50_enums.md)**: Define fixed sets of named variants for categorical data
- **[Schemes](60_schemes.md)**: Create structured types with validation and metadata
