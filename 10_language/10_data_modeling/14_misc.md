# Miscellaneous Types

Additional specialized types provided by Frel.

## Uuid

Universally unique identifiers (UUIDs) following RFC 4122:

```frel
scheme Entity {
    id { Uuid }
        .. default { Uuid::new() }

    external_id { Uuid }
        .. optional { true }

    correlation_id { Uuid }
}
```

### UUID Operations

```frel
// Generate new random UUID (v4)
let id = Uuid::new()

// Parse from string
let parsed = Uuid::parse("550e8400-e29b-41d4-a716-446655440000")

// Convert to string representation
id.to_string()  // "550e8400-e29b-41d4-a716-446655440000"

// Comparison
if id1 == id2 {
    // UUIDs are equal
}
```

### Use Cases

**Primary keys:**
```frel
scheme User {
    id { Uuid }
        .. default { Uuid::new() }
        .. readonly { true }

    username { String }
}
```

**External identifiers:**
```frel
scheme Order {
    id { u64 }
    external_id { Uuid }  // For external API integration
        .. default { Uuid::new() }
}
```

**Correlation tracking:**
```frel
scheme ApiRequest {
    correlation_id { Uuid }
        .. default { Uuid::new() }

    trace_id { Uuid }
        .. optional { true }
}
```

### Properties

- **Uniqueness**: Virtually guaranteed to be unique across space and time
- **Format**: 128-bit value, typically displayed as 36-character hex string
- **Version**: Frel uses UUID v4 (random) by default
- **Ordering**: UUIDs are comparable and can be used as Map keys

### Type Mapping

| Frel Type | Rust   | TypeScript      | Python |
|-----------|--------|-----------------|--------|
| `Uuid`    | `Uuid` | `string` (UUID) | `UUID` |

**Note:** In TypeScript, UUIDs are represented as strings with UUID format validation.
