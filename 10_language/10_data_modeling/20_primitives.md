# Primitive Types

Frel provides a comprehensive set of primitive types for common data needs.

## Integers

Signed and unsigned integers of various sizes:

```frel
scheme Example {
    // Signed integers
    count .. i32       // 32-bit signed
    offset .. i64      // 64-bit signed
    small .. i8        // 8-bit signed
    medium .. i16      // 16-bit signed

    // Unsigned integers
    id .. u64          // 64-bit unsigned
    index .. u32       // 32-bit unsigned
    byte .. u8         // 8-bit unsigned
    port .. u16        // 16-bit unsigned
}
```

## Floating Point Numbers

Single and double precision floating point numbers:

```frel
scheme Measurements {
    price .. f64       // Double precision (64-bit)
    ratio .. f32       // Single precision (32-bit)
    percentage .. f32
}
```

## Boolean

Binary true/false values:

```frel
scheme Flags {
    active .. bool
    enabled .. bool
    is_visible .. bool
}
```

## String

Unicode text strings:

```frel
scheme Text {
    name .. String
    description .. String
    email .. String
}
```

### String Operations

```frel
let text = "Hello"

// Length and checks
text.len()              // Character count
text.is_empty()         // -> false
text.contains("ell")    // -> true

// Case conversion
text.to_lowercase()     // "hello"
text.to_uppercase()     // "HELLO"

// Trimming
text.trim()            // Remove leading/trailing whitespace
text.trim_start()
text.trim_end()

// Substring operations
text.starts_with("He")  // -> true
text.ends_with("lo")    // -> true
```

### Validation

```frel
scheme ValidatedText {
    username .. String
        .. min_len { 3 }
        .. max_len { 20 }
        .. pattern { "^[a-zA-Z0-9_]+$" }
        .. blank { false }
}
```

## Secret

A string type for sensitive data like passwords, API keys, and tokens:

```frel
scheme Credentials {
    password .. Secret
        .. min_len { 8 }
        .. blank { false }

    api_key .. Secret
        .. blank { false }

    auth_token .. Secret
        .. optional
}
```

### Properties

- **Secure handling**: Host implementations should avoid logging or displaying secrets
- **Validation**: Supports same validation rules as String (length, pattern, etc.)
- **UI rendering**: Automatically renders as masked input fields
- **Serialization**: May be excluded or masked in serialization by default

### Use Cases

**Authentication:**
```frel
scheme LoginForm {
    username .. String
        .. blank { false }

    password .. Secret
        .. min_len { 8 }
        .. blank { false }
}
```

**API credentials:**
```frel
scheme ApiConfig {
    endpoint .. Url
    api_key .. Secret
        .. blank { false }

    api_secret .. Secret
        .. optional
}
```

## Decimal

Arbitrary-precision decimal numbers for financial calculations and cases requiring exact decimal representation:

```frel
scheme Financial {
    account_balance .. Decimal
        .. default { Decimal::zero() }

    transaction_amount .. Decimal
        .. min { Decimal::zero() }

    price .. Decimal
        .. precision { 2 }  // 2 decimal places
}
```

### Operations

```frel
let amount = Decimal::from_str("123.45")

// Arithmetic
amount + Decimal::from_str("10.00")
amount - Decimal::from_str("5.50")
amount * Decimal::from_i64(2)
amount / Decimal::from_i64(3)

// Rounding
amount.round(2)        // Round to 2 decimal places
amount.floor()
amount.ceil()

// Comparison
if amount > Decimal::zero() {
    // Positive amount
}

// Conversion
amount.to_string()     // "123.45"
amount.to_f64()        // Lossy conversion to float
```

### Validation

```frel
scheme Product {
    price .. Decimal
        .. min { Decimal::zero() }
        .. max { Decimal::from_str("999999.99") }
        .. precision { 2 }

    discount_rate .. Decimal
        .. range_decimal { "0.00" "1.00" }  // 0% to 100%
}
```

### Use Cases

**Financial calculations:**
```frel
scheme Invoice {
    subtotal .. Decimal
    tax_rate .. Decimal
        .. default { Decimal::from_str("0.0825") }  // 8.25%

    total .. Decimal
}
```

**Cryptocurrency:**
```frel
scheme CryptoWallet {
    balance .. Decimal
        .. precision { 18 }  // Wei precision for Ethereum
}
```

**Important:** Use `Decimal` instead of `f32`/`f64` when exact decimal representation is required (money, percentages, rates).

## Uuid

Universally unique identifiers (UUIDs) following RFC 4122:

```frel
scheme Entity {
    id .. Uuid
        .. default { Uuid::new() }

    external_id .. Uuid
        .. optional

    correlation_id .. Uuid
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
    id .. Uuid
        .. default { Uuid::new() }
        .. readonly { true }

    username .. String
}
```

**External identifiers:**
```frel
scheme Order {
    id .. u64
    external_id .. Uuid  // For external API integration
        .. default { Uuid::new() }
}
```

**Correlation tracking:**
```frel
scheme ApiRequest {
    correlation_id .. Uuid
        .. default { Uuid::new() }

    trace_id .. Uuid
        .. optional
}
```

### Properties

- **Uniqueness**: Virtually guaranteed to be unique across space and time
- **Format**: 128-bit value, typically displayed as 36-character hex string
- **Version**: Frel uses UUID v4 (random) by default
- **Ordering**: UUIDs are comparable and can be used as Map keys

## Url

URL/URI type with validation for web addresses and resource identifiers:

```frel
scheme WebContent {
    homepage .. Url
        .. blank { false }

    avatar_url .. Url
        .. optional

    api_endpoint .. Url
        .. scheme_allowed { ["https"] }  // Only HTTPS URLs
}
```

### URL Operations

```frel
// Parse from string
let url = Url::parse("https://example.com/path?query=value")

// Access components
url.scheme()       // "https"
url.host()         // "example.com"
url.path()         // "/path"
url.query()        // "query=value"

// Manipulation
url.join("subpath")        // Resolve relative path
url.set_query("new=value")

// Validation
url.is_valid()     // Check if URL is well-formed
```

### Validation

```frel
scheme SocialProfile {
    website .. Url
        .. blank { false }
        .. scheme_allowed { ["http" "https"] }

    github_url .. Url
        .. optional
        .. domain_allowed { ["github.com"] }

    avatar .. Url
        .. optional
        .. scheme_allowed { ["https"] }
}
```

### Use Cases

**Links and navigation:**
```frel
scheme MenuItem {
    label .. String
    url .. Url
        .. optional
}
```

**API configuration:**
```frel
scheme ServiceConfig {
    base_url .. Url
        .. blank { false }

    webhook_url .. Url
        .. optional
        .. scheme_allowed { ["https"] }
}
```

**Media references:**
```frel
scheme Media {
    image_url .. Url
    thumbnail_url .. Url
        .. optional
}
```

## Color

Color values for styling and theming:

```frel
scheme Theme {
    primary_color .. Color
        .. default { Color::hex("#007bff") }

    background_color .. Color
        .. default { Color::rgb(255, 255, 255) }

    accent_color .. Color
        .. optional
}
```

### Color Constructors

```frel
// Hexadecimal
Color::hex("#FF5733")
Color::hex("#F57")          // Short form

// RGB
Color::rgb(255, 87, 51)

// RGBA (with alpha channel)
Color::rgba(255, 87, 51, 0.8)

// HSL
Color::hsl(9, 100, 60)

// HSLA
Color::hsla(9, 100, 60, 0.8)

// Named colors
Color::named("red")
Color::named("transparent")
```

### Color Operations

```frel
let color = Color::hex("#FF5733")

// Conversions
color.to_hex()       // "#FF5733"
color.to_rgb()       // (255, 87, 51)
color.to_rgba()      // (255, 87, 51, 1.0)
color.to_hsl()       // (9, 100, 60)

// Manipulation
color.lighten(0.2)   // Lighten by 20%
color.darken(0.2)    // Darken by 20%
color.saturate(0.1)  // Increase saturation
color.desaturate(0.1)
color.opacity(0.5)   // Set alpha to 0.5

// Properties
color.luminance()    // 0.0 to 1.0
color.is_light()     // -> bool
color.is_dark()      // -> bool
```

### Validation

```frel
scheme BrandColors {
    primary .. Color
        .. blank { false }

    secondary .. Color
        .. optional

    // Ensure sufficient contrast
    text_color .. Color
        .. contrast_with { primary_color, 4.5 }  // WCAG AA standard
}
```

### Use Cases

**Theming:**
```frel
scheme AppTheme {
    primary .. Color .. default { Color::hex("#007bff") }
    secondary .. Color .. default { Color::hex("#6c757d") }
    success .. Color .. default { Color::hex("#28a745") }
    danger .. Color .. default { Color::hex("#dc3545") }
    warning .. Color .. default { Color::hex("#ffc107") }
    info .. Color .. default { Color::hex("#17a2b8") }

    background .. Color .. default { Color::rgb(255, 255, 255) }
    text .. Color .. default { Color::rgb(33, 37, 41) }
}
```

**User customization:**
```frel
scheme UserPreferences {
    theme_color .. Color
        .. default { Color::hex("#007bff") }

    highlight_color .. Color
        .. optional
}
```

**Data visualization:**
```frel
scheme ChartSeries {
    name .. String
    color .. Color
    data .. List<f64>
}
```

## Blob

Binary data type for handling files, images, and other binary content:

```frel
scheme MediaFile {
    file_data .. Blob
        .. max_size { 10485760 }  // 10 MB in bytes

    thumbnail .. Blob
        .. optional
        .. max_size { 1048576 }   // 1 MB

    mime_type .. String
        .. blank { false }
}
```

### Blob Operations

```frel
// Create from bytes
let blob = Blob::from_bytes(bytes)

// Read file
let blob = Blob::from_file("/path/to/file")

// Properties
blob.size()          // Size in bytes
blob.mime_type()     // MIME type (if available)
blob.is_empty()      // -> bool

// Conversion
blob.to_bytes()      // Raw byte array
blob.to_base64()     // Base64-encoded string
blob.to_data_url()   // Data URL for embedding

// Slicing
blob.slice(0, 1024)  // Get first 1024 bytes
```

### Validation

```frel
scheme ImageUpload {
    image .. Blob
        .. blank { false }
        .. max_size { 5242880 }  // 5 MB
        .. mime_types { ["image/jpeg" "image/png" "image/webp"] }

    alt_text .. String
        .. max_len { 200 }
}
```

### Use Cases

**File uploads:**
```frel
scheme FileUpload {
    file .. Blob
        .. blank { false }
        .. max_size { 104857600 }  // 100 MB

    filename .. String
        .. blank { false }

    content_type .. String
}
```

**Avatar images:**
```frel
scheme UserProfile {
    username .. String
    avatar .. Blob
        .. optional
        .. max_size { 2097152 }  // 2 MB
        .. mime_types { ["image/jpeg" "image/png"] }
}
```

**Document storage:**
```frel
scheme Document {
    title .. String
    content .. Blob
        .. blank { false }
        .. mime_types { ["application/pdf" "application/msword"] }

    thumbnail .. Blob
        .. optional
}
```

**Binary API responses:**
```frel
scheme DownloadResult {
    data .. Blob
    filename .. String
    content_type .. String
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

## Type Mapping

Frel primitive types map to native types in different host languages:

| Frel Type  | Rust           | TypeScript      | Python          |
|------------|----------------|-----------------|-----------------|
| `String`   | `String`       | `string`        | `str`           |
| `i8`..`i64`, `u8`..`u64` | `i8`..`i64`, `u8`..`u64` | `number` | `int` |
| `f32`, `f64` | `f32`, `f64` | `number`      | `float`         |
| `bool`     | `bool`         | `boolean`       | `bool`          |
| `Secret`   | `Secret`       | `Secret`        | `Secret`        |
| `Decimal`  | `rust_decimal::Decimal` | `Decimal` (custom) | `decimal.Decimal` |
| `Uuid`     | `uuid::Uuid`   | `string` (UUID) | `uuid.UUID`     |
| `Url`      | `url::Url`     | `URL`           | `urllib.parse.ParseResult` |
| `Color`    | `Color`        | `Color`         | `Color`         |
| `Blob`     | `Vec<u8>`      | `Blob`/`Uint8Array` | `bytes`     |

**Note:** Some types like `Secret`, `Decimal`, `Color` require Frel-provided wrappers or custom implementations in each host language to ensure consistent behavior and security properties.
