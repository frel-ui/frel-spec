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

>> TODO When does validation trigger automatically? (on field change? on form submit? manual only?)
>> TODO What's the reactivity behavior of is_valid() and errors()? Do they subscribe to field changes?
>> TODO How do cross-field validations work? (e.g., "end_date must be after start_date")
>> TODO What's the syntax for custom validation functions?

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

### Default Values

Default values are evaluated at instance creation. Use the `.. default { value }` instruction to
specify default values for fields.

### Read-Only Fields

Use `.. readonly { true }` to mark fields that should not be modified after creation. Useful for
timestamps, IDs, and other immutable data.

## Validation API

Schemes generate validation methods automatically:

- **`is_valid()`**: Check if all fields are valid (returns bool)
- **`errors()`**: Get all validation errors (returns a list of FieldError)
- **`field.is_valid()`**: Check specific field validity
- **`field.error()`**: Get error for specific field (returns FieldError?)
- **`validate()`**: Manually trigger validation

### FieldError Structure

Contains:

- **`field`**: Field name
- **`message`**: Error message
- **`code`**: Error code (e.g., "min_len", "pattern")

## Virtual Fields

Virtual fields are computed properties that are derived from other fields and evaluated reactively.
They are not stored as part of the scheme's data but are calculated on-demand based on their
dependencies.

### Syntax

```frel
scheme <Name> {
    <field_name> : <type> [.. <instruction>]*
    virtual <virtual_field_name> : <type> = <expression>
}
```

### Example

```frel
scheme User {
    id : UUID .. identity
    firstName : String
    lastName : String

    virtual fullName : String = "${firstName} ${lastName}"
}
```

### Behavior

- **Not stored**: Virtual fields are computed, not persisted
- **Reactive**: Virtual fields automatically recompute when their dependencies change
- **Dependencies**: The compiler tracks which fields a virtual field depends on and subscribes to
  those fields
- **Availability**: Virtual fields propagate availability from their dependencies according to the
  rule: `Error > Loading > Ready`
- **Composition**: Virtual fields can reference other virtual fields in the same scheme

### Cross-Arena References

Virtual fields can resolve references across arenas:

```frel
scheme Thermometer {
    id : UUID .. identity
    location : ref Location

    virtual locationName : String = location.name
}
```

When accessing `location.name`, the reference is resolved through the arena, and availability is
propagated if the referenced entity is loading or missing.

For detailed semantics on identity, reactivity, and availability propagation, see
the [Reactivity Model](10_reactivity_model.md) documentation.