# Data Basics

Frel provides a type system for modeling application data. The data model consists of primitive
types, collections, temporal types, and user-defined structured types (enums, schemes, and trees).

## Type System Overview

The Frel type system is designed to be:

- **Platform-independent**: Types map naturally to host languages (Rust, TypeScript, Python)
- **Validatable**: Built-in validation and constraints at the type level
- **Composable**: Types can be nested and combined to model complex domains

## Type Categories

### [Primitive Types](11_primitives.md)

Standard scalar types supported by all host languages:

- **Integers**: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`
- **Floating point**: `f32`, `f64`
- **Boolean**: `bool`
- **String**: `String`

### [Miscellaneous Types](14_misc.md)

Additional specialized types:

- **Uuid**: Universally unique identifiers

### [Collections](12_collections.md)

Platform-independent collection types with built-in reactivity:

- **List**: Ordered, indexed collection
- **Set**: Unique values, unordered
- **Map**: Key-value pairs (primitive keys only)

Collections are implemented using Frel-provided wrappers (`FrelList`, `FrelSet`, `FrelMap`) around
native host language types.

### [DateTime Types](13_datetime.md)

Standard temporal types for working with dates, times, and durations:

- **Instant**: Point in time (UTC)
- **LocalDate**: Date without time
- **LocalTime**: Time without date
- **LocalDateTime**: Date and time without timezone
- **Timezone**: IANA timezone identifier
- **Duration**: Time span

### User-Defined Types

- **[Enums](20_enums.md)**: Fixed sets of named variants
- **[Schemes](30_schemes.md)**: Structured data types with validation
- **[Trees](40_trees.md)**: Hierarchical data structures

## Type Usage Examples

### Basic Usage

```frel
scheme User {
    // Primitives
    id { u64 }
    username { String }
    age { u8 }
    verified { bool }

    // Miscellaneous
    uuid { Uuid }

    // Collections
    tags { List<String> }
    roles { Set<String> }
    preferences { Map<String, String> }

    // DateTime
    created_at { Instant }
    birth_date { LocalDate }
    timezone { Timezone }
}
```

### Nested Collections

```frel
scheme Analytics {
    // List of lists
    data_points { List<List<f64>> }

    // Map with list values
    user_tags { Map<u64, List<String>> }

    // Set of numeric IDs
    active_users { Set<u64> }
}
```

### Optional Types

Any type can be made optional:

```frel
scheme Profile {
    name { String }                    // Required
    bio { String }                     // Required but can be empty
        .. blank { true }
    avatar_url { String }              // Optional
        .. optional { true }
    age { u8 }                        // Optional
        .. optional { true }
}
```

## Type Mapping

Frel types map to native types in different host languages:

| Frel Type       | Rust            | TypeScript      | Python           |
|-----------------|-----------------|-----------------|------------------|
| `String`        | `String`        | `string`        | `str`            |
| `i32`, `u64`... | `i32`, `u64`... | `number`        | `int`            |
| `f32`, `f64`    | `f32`, `f64`    | `number`        | `float`          |
| `bool`          | `bool`          | `boolean`       | `bool`           |
| `Uuid`          | `Uuid`          | `string` (UUID) | `UUID`           |
| `List<T>`       | `FrelList<T>`   | `FrelList<T>`   | `FrelList[T]`    |
| `Set<T>`        | `FrelSet<T>`    | `FrelSet<T>`    | `FrelSet[T]`     |
| `Map<K,V>`      | `FrelMap<K,V>`  | `FrelMap<K,V>`  | `FrelMap[K,V]`   |
| `Tree<T>`       | `FrelTree<T>`   | `FrelTree<T>`   | `FrelTree[T]`    |
| `Instant`       | `DateTime<Utc>` | `Date`          | `datetime` (UTC) |
| `LocalDate`     | `NaiveDate`     | Custom          | `date`           |
| `LocalTime`     | `NaiveTime`     | Custom          | `time`           |
| `LocalDateTime` | `NaiveDateTime` | Custom          | `datetime`       |
| `Timezone`      | `Tz`            | `string`        | `ZoneInfo`       |
| `Duration`      | `Duration`      | Custom          | `timedelta`      |

**Note:** `FrelList`, `FrelSet`, `FrelMap`, and `FrelTree` are Frel-provided wrappers around host
language-specific implementations (e.g., `Vec`, `HashSet`, `HashMap` in Rust) with built-in reactivity support.

## Next Steps

- **[Primitive Types](11_primitives.md)**: Integers, floats, booleans, and strings
- **[Miscellaneous Types](14_misc.md)**: Uuid and other specialized types
- **[Collections](12_collections.md)**: List, Set, and Map types
- **[DateTime Types](13_datetime.md)**: Working with dates, times, and durations
- **[Enums](20_enums.md)**: Define fixed sets of named variants for categorical data
- **[Schemes](30_schemes.md)**: Create structured types with validation and metadata
- **[Trees](40_trees.md)**: Work with hierarchical data structures
