# Primitive Types

Frel supports all standard host language primitive types.

## Integers

Signed and unsigned integers of various sizes:

```frel
scheme Example {
    // Signed integers
    count { i32 }      // 32-bit signed
    offset { i64 }     // 64-bit signed
    small { i8 }       // 8-bit signed
    medium { i16 }     // 16-bit signed

    // Unsigned integers
    id { u64 }         // 64-bit unsigned
    index { u32 }      // 32-bit unsigned
    byte { u8 }        // 8-bit unsigned
    port { u16 }       // 16-bit unsigned
}
```

## Floating Point Numbers

Single and double precision floating point numbers:

```frel
scheme Measurements {
    price { f64 }      // Double precision (64-bit)
    ratio { f32 }      // Single precision (32-bit)
    percentage { f32 }
}
```

## Boolean

Binary true/false values:

```frel
scheme Flags {
    active { bool }
    enabled { bool }
    is_visible { bool }
}
```

## String

Unicode text strings:

```frel
scheme Text {
    name { String }
    description { String }
    email { String }
}
```

## Type Ranges and Limits

| Type  | Min Value                  | Max Value                  | Bytes |
|-------|----------------------------|----------------------------|-------|
| `i8`  | -128                       | 127                        | 1     |
| `i16` | -32,768                    | 32,767                     | 2     |
| `i32` | -2,147,483,648             | 2,147,483,647              | 4     |
| `i64` | -9,223,372,036,854,775,808 | 9,223,372,036,854,775,807  | 8     |
| `u8`  | 0                          | 255                        | 1     |
| `u16` | 0                          | 65,535                     | 2     |
| `u32` | 0                          | 4,294,967,295              | 4     |
| `u64` | 0                          | 18,446,744,073,709,551,615 | 8     |
| `f32` | ~1.18e-38                  | ~3.40e+38                  | 4     |
| `f64` | ~2.23e-308                 | ~1.80e+308                 | 8     |
