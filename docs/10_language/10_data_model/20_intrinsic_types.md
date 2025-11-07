# Intrinsic Types

Frel provides a comprehensive set of intrinsic types for modeling application data.

## Numeric Types

### Integers

Signed and unsigned integers of various sizes:

- **`i8`**: 8-bit signed integer (-128 to 127)
- **`i16`**: 16-bit signed integer (-32,768 to 32,767)
- **`i32`**: 32-bit signed integer (-2,147,483,648 to 2,147,483,647)
- **`i64`**: 64-bit signed integer (-9,223,372,036,854,775,808 to 9,223,372,036,854,775,807)
- **`u8`**: 8-bit unsigned integer (0 to 255)
- **`u16`**: 16-bit unsigned integer (0 to 65,535)
- **`u32`**: 32-bit unsigned integer (0 to 4,294,967,295)
- **`u64`**: 64-bit unsigned integer (0 to 18,446,744,073,709,551,615)

### Floating Point Numbers

- **`f32`**: Single precision 32-bit floating point number (~1.18e-38 to ~3.40e+38)
- **`f64`**: Double precision 64-bit floating point number (~2.23e-308 to ~1.80e+308)

### Decimal

- **`Decimal`**: Arbitrary-precision decimal numbers for financial calculations and cases requiring
  exact decimal representation. Avoids floating-point rounding errors.

## Text Types

### String

- **`String`**: Unicode text strings.

### Secret

- **`Secret`**: A string type for sensitive data like passwords, API keys, and tokens. Host
  implementations should avoid logging or displaying secrets. Automatically renders as masked input
  fields in UI. May be excluded or masked in serialization by default.

## Boolean

- **`bool`**: Binary true/false values.

## Identifier Types

### Uuid

- **`Uuid`**: Universally unique identifiers (UUIDs).

### Url

- **`Url`**: URL/URI type with validation for web addresses and resource identifiers. Supports
  parsing, component access (scheme, host, path, query), and manipulation.

## Visual Types

### Color

- **`Color`**: Color values for styling and theming.

## Binary Data

### Blob

- **`Blob`**: Binary data type for handling files, images, and other binary content.

## Temporal Types

### Instant

- **`Instant`**: Represents a specific moment in time (UTC). Used for absolute timestamps that are
  timezone-independent.

### LocalDate

- **`LocalDate`**: Represents a calendar date without time information (e.g., 2024-03-15). Used for
  birth dates, event dates, deadlines, and other date-only values.

### LocalTime

- **`LocalTime`**: Represents a time of day without date information (e.g., 14:30:00). Used for
  daily schedules, alarm times, opening hours, and other time-only values.

### LocalDateTime

- **`LocalDateTime`**: Represents a date and time without timezone information. Combines date and
  time components.

### Timezone

- **`Timezone`**: IANA timezone identifier (e.g., "America/New_York", "Europe/London"). Used for
  timezone-aware operations and converting Instant values to local time.

### Duration

- **`Duration`**: Represents a length of time or time span. Supports constructors for various
  units (nanoseconds, microseconds, milliseconds, seconds, minutes, hours, days, weeks) and
  arithmetic operations.

### Enums

>> TODO
