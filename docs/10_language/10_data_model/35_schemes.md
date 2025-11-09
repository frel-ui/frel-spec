# Schemes

Schemes define structured data types with built-in validation, metadata, and automatic UI
integration. They serve as the foundation for forms, structured state, and data persistence in
Frel applications.

## Syntax

```
scheme <Name> {
    <field_name> : <type> [.. <instruction>]*
    // other fields ...
}
```

## Semantics

- **Fields**: Named typed values with optional validation and metadata
- **Types**: Any Frel type. Can be nested, cannot be recursive.
- **Instructions**: Validation rules, defaults, constraints attached via `..`
- **Validation**: Automatic validation based on field instructions

## Field Instructions

### Common Validation

#### String Validation

- **`min_len`**: Minimum length
- **`max_len`**: Maximum length
- **`blank`**: Allow or disallow empty strings
- **`pattern`**: Regex pattern for validation
- **`multiline`**: UI hint for textarea rendering

#### Numeric Validation

- **`range`**: Min and max inclusive
- **`min`**: Minimum value
- **`max`**: Maximum value
- **`precision`**: Decimal precision (for Decimal type)

#### DateTime Validation

- **`before`**: Must be before specified time/date
- **`after`**: Must be after specified time/date
- **`min`**: Minimum duration
- **`max`**: Maximum duration

#### Collection Validation

- **`min_items`**: Minimum number of elements
- **`max_items`**: Maximum number of elements
- **`each`**: Apply validation to each element
- **`key_pattern`**: Regex for Map keys (string keys)

### Optional Fields

Fields are required by default. Use the `.. optional` instruction to make a field nullable.

**Optional field semantics:**

- Maps to `Option<T>` in Rust, `T | null` in TypeScript, `Optional[T]` in Python
- Can be absent (`null`/`None`) or present with a value
- Validation rules only apply when value is present
- Optional fields can have defaults, but default is not required

### Default Values

Default values are evaluated at instance creation. Use the `.. default { value }` instruction to
specify default values for fields.

### Read-Only Fields

Use `.. readonly { true }` to mark fields that should not be modified after creation. Useful for
timestamps, IDs, and other immutable data.

## Validation API

Schemes generate validation methods automatically:

- **`is_valid()`**: Check if all fields are valid (returns bool)
- **`errors()`**: Get all validation errors (returns list of FieldError)
- **`field.is_valid()`**: Check specific field validity
- **`field.error()`**: Get error for specific field (returns Option)
- **`validate()`**: Manually trigger validation

### FieldError Structure

Contains:

- **`field`**: Field name
- **`message`**: Error message
- **`code`**: Error code (e.g., "min_len", "pattern")