# Schemes

Schemes define structured data types with built-in validation, metadata, and automatic UI
integration. They serve as the foundation for forms, structured state, and data persistence in
Frel applications.

## Syntax

```
scheme <Name> {
    <field_name> .. <type> [.. <instruction>]*
    ...
}
```

## Semantics

- **Fields**: Named typed values with optional validation and metadata
- **Types**: Primitives, Frel collections (List, Set, Map, Tree), DateTime types, enums, or nested
  schemes
- **Instructions**: Validation rules, defaults, constraints attached via `..`
- **Validation**: Automatic validation based on field instructions
- **UI Integration**: Automatic form generation and two-way data binding

## Basic Example

```frel
scheme User {
    username .. String
        .. min_len { 3 }
        .. max_len { 20 }
        .. blank { false }

    email .. String
        .. pattern { r"^[\w\.-]+@[\w\.-]+\.\w+$" }
        .. blank { false }

    age .. u8
        .. range { 13 150 }
        .. optional

    newsletter .. bool
        .. default { false }
}
```

## Field Types

Schemes support all Frel types. For details on primitive types, collections, and DateTime types,
see [Data Basics](10_data_basics.md).

### DateTime Types

Frel provides standard temporal types:

```frel
scheme Event {
    // Instant - point in time (UTC)
    created_at .. Instant
        .. default { Instant::now() }

    // LocalDate - date without time (2024-03-15)
    event_date .. LocalDate

    // LocalTime - time without date (14:30:00)
    alarm_time .. LocalTime

    // LocalDateTime - date and time, no timezone
    scheduled .. LocalDateTime

    // Timezone - IANA timezone identifier
    timezone .. Timezone
        .. optional

    // Duration - time span
    duration .. Duration
        .. default { Duration::hours(1) }
}
```

### Enums

Both top-level and inline enums are supported:

```frel
enum Status { draft published archived }

scheme Article {
    // Top-level enum
    status .. Status
        .. default { Status::draft }

    // Inline enum
    priority enum { low medium high }
        .. default { priority::medium }
}
```

See [Enums](50_enums.md) for more details.

### Nested Schemes

Schemes can reference other schemes:

```frel
scheme Address {
    street .. String .. blank { false }
    city .. String .. blank { false }
    zip .. String .. pattern { r"\d{5}" }
}

scheme User {
    name .. String
    address .. Address
        .. optional

    // Collection of nested schemes
    previous_addresses .. List<Address>
        .. max_items { 5 }
}
```

## Field Instructions

### Common Validation

#### String Validation

```frel
scheme Text {
    name .. String
        .. min_len { 3 }          // Minimum length
        .. max_len { 50 }         // Maximum length
        .. blank { false }        // Disallow empty strings
        .. pattern { "^[A-Z]" }   // Regex pattern
        .. multiline { true }     // UI hint for textarea
}
```

#### Numeric Validation

```frel
scheme Numbers {
    age .. u8
        .. range { 18 65 }        // Min and max inclusive
        .. default { 18 }

    price .. Decimal
        .. min { Decimal::zero() }           // Minimum value
        .. max { Decimal::from_str("1000000.00") }     // Maximum value
        .. precision { 2 }
}
```

#### DateTime Validation

```frel
scheme Scheduling {
    birth_date .. LocalDate
        .. before { LocalDate::today() }  // Must be in the past

    appointment .. Instant
        .. after { Instant::now() }       // Must be in the future

    duration .. Duration
        .. min { Duration::minutes(15) }
        .. max { Duration::hours(8) }
}
```

#### Collection Validation

```frel
scheme Lists {
    tags .. List<String>
        .. min_items { 1 }
        .. max_items { 10 }
        .. each .. min_len { 2 }    // Validate each item
        .. each .. max_len { 20 }
        .. default { List::new() }

    categories .. Set<String>
        .. max_items { 5 }

    metadata .. Map<String, String>
        .. max_items { 20 }
        .. key_pattern { "^[a-z_]+$" }  // Validate keys
}
```

### Optional Fields

Fields are required by default. Use the `.. optional` instruction to make a field nullable:

```frel
scheme Profile {
    // Required by default
    username .. String

    // Explicitly optional
    bio .. String
        .. optional

    age .. u8
        .. optional
}
```

**Optional field semantics:**
- Maps to `Option<T>` in Rust, `T | null` in TypeScript, `Optional[T]` in Python
- Can be absent (`null`/`None`) or present with a value
- Validation rules (if any) only apply when value is present
- Optional fields can have defaults, but default is not required

### Default Values

Default values are evaluated at instance creation:

```frel
scheme Settings {
    theme .. String
        .. default { "dark" }

    volume .. f32
        .. default { 0.5 }

    created_at .. Instant
        .. default { Instant::now() }  // Evaluated at creation

    tags .. List<String>
        .. default { List::new() }
}
```

### Read-Only Fields

```frel
scheme Entity {
    id .. u64
        .. readonly { true }

    created_at .. Instant
        .. default { Instant::now() }
        .. readonly { true }
}
```

### Metadata Instructions

For code generation, database mapping, etc:

```frel
scheme User {
    id .. u64
        .. db_primary_key { true }
        .. readonly { true }

    email .. String
        .. db_unique { true }
        .. db_indexed { true }

    created_at .. Instant
        .. auto_now_add { true }
        .. readonly { true }

    updated_at .. Instant
        .. auto_now { true }
}
```

### Localization Instructions

```frel
scheme User {
    username .. String
        .. label { "user.username" }           // I18n key for label
        .. help_text { "user.username.help" }  // I18n key for tooltip
        .. placeholder { "user.username.placeholder" }
}
```

## Usage in Fragments

### Basic Form Binding

```frel
scheme Login {
    username .. String
        .. min_len { 3 }
        .. blank { false }

    password .. Secret
        .. min_len { 8 }
        .. blank { false }

    remember .. Boolean
        .. default { false }
}

fragment LoginForm() {
    writable credentials = Login()

    column {
        gap { 16 }

        // Automatic two-way binding with validation
        text_input { credentials.username }
        secret_input { credentials.password }

        row {
            checkbox { credentials.remember }
            text { "Remember me" }
        }

        button { "Log in" }
            .. enabled { credentials.is_valid() }
            .. on_click {
                api::login(credentials.clone())
            }
    }
}
```

### Auto-Generated UI Elements

When bound to widgets, schemes automatically provide:

1. **Placeholders**: Generated from field names or `placeholder` instruction
2. **Labels**: Generated from field names or `label` instruction
3. **Error messages**: Generated from validation rules
4. **Input constraints**: Applied as widget attributes (maxlength, pattern, etc.)
5. **Validation**: Real-time field-level validation

**Example:**

```frel
text_input { user.email }
// Automatically:
// - placeholder: "Email"
// - pattern: email regex from scheme
// - error: "Invalid email format" on validation failure
// - maxlength: if max_len specified
```

### Custom Error Handling

Override auto-generated errors when needed:

```frel
fragment CustomForm() {
    writable user = User()

    text_input { user.email }
        .. error {
            if user.email.has_error() {
                "Please enter a valid email address"
            } else {
                ""
            }
        }
}
```

## Validation API

Schemes generate validation methods automatically:

```frel
writable user = User()

// Check if all fields are valid
user.is_valid() // -> bool

// Get all validation errors
user.errors()  // -> Vec<FieldError>

// Check specific field
user.email.is_valid()  // -> bool
user.email.error()     // -> Option<String>

// Validate on demand
user.validate()  // Runs all validations
```

### FieldError Structure

```rust
struct FieldError {
    field: String,      // Field name
    message: String,    // Error message
    code: String,       // Error code (e.g., "min_len", "pattern")
}
```

## Reactivity and Stores

Scheme instances support fine-grained reactivity:

```frel
scheme Settings {
    theme .. String
    font_size .. u8
    auto_save .. bool
}

fragment SettingsPanel() {
    writable settings = Settings()

    column {
        // Component A: subscribes only to settings.theme
        text_input { settings.theme }

        // Component B: subscribes only to settings.font_size
        number_input { settings.font_size }

        // When theme changes, only Component A recomputes
        // When font_size changes, only Component B recomputes
    }
}
```

**Field-level subscriptions:** The runtime tracks which fields are accessed and only notifies
dependents when those specific fields change.

## Complex Examples

### User Profile with Nested Data

```frel
enum AccountType { free premium enterprise }

scheme SocialLinks {
    twitter .. String .. optional .. pattern { "^@\\w+" }
    github .. String .. optional
    linkedin .. String .. optional
}

scheme UserProfile {
    // Identity
    id .. u64 .. readonly { true }
    username .. String
        .. min_len { 3 }
        .. max_len { 20 }
        .. pattern { "^[a-zA-Z0-9_]+$" }
        .. blank { false }

    email .. String
        .. blank { false }
        .. pattern { r"^[\w\.-]+@[\w\.-]+\.\w+$" }

    // Profile details
    display_name .. String
        .. max_len { 50 }
        .. blank { true }

    bio .. String
        .. max_len { 500 }
        .. optional
        .. multiline { true }

    avatar_url .. String
        .. optional

    // Account info
    account_type .. AccountType
        .. default { AccountType::free }

    verified .. Boolean
        .. default { false }
        .. readonly { true }

    // Nested
    social_links .. SocialLinks
        .. optional

    // Collections
    interests .. Set<String>
        .. max_items { 10 }
        .. each .. max_len { 30 }

    // Temporal
    joined_at .. Instant
        .. default { Instant::now() }
        .. readonly { true }

    last_seen .. Instant
        .. optional
}
```

### Form with Complex Validation

```frel
scheme Registration {
    username .. String
        .. min_len { 3 }
        .. max_len { 20 }
        .. pattern { "^[a-zA-Z0-9_]+$" }
        .. blank { false }

    email .. String
        .. blank { false }
        .. pattern { r"^[\w\.-]+@[\w\.-]+\.\w+$" }

    password .. Secret
        .. min_len { 8 }
        .. blank { false }

    confirm_password .. Secret
        .. blank { false }
        .. validate { |value, data|
            value == data.password
        }
        .. error_message { "Passwords must match" }

    age .. u8
        .. range { 13 150 }

    terms_accepted .. Boolean
        .. validate { |value, _| value == true }
        .. error_message { "You must accept the terms" }

    newsletter .. Boolean
        .. default { false }
}

fragment RegistrationForm() {
    writable registration = Registration()

    column {
        gap { 16 }

        text_input { registration.username }
        text_input { registration.email }
        secret_input { registration.password }
        secret_input { registration.confirm_password }
        number_input { registration.age }

        row {
            checkbox { registration.terms_accepted }
            text { "I accept the terms and conditions" }
        }

        row {
            checkbox { registration.newsletter }
            text { "Subscribe to newsletter" }
        }

        // Show all errors
        when !registration.is_valid() {
            column {
                repeat on registration.errors() as error {
                    text { error.message }
                        .. font { color: Red size: 12 }
                }
            }
        }

        button { "Register" }
            .. enabled { registration.is_valid() }
            .. on_click {
                api::register(registration.clone())
            }
    }
}
```

### Application Configuration with New Primitives

```frel
enum Environment { development staging production }

scheme ApiConfig {
    // URL type for endpoints
    base_url .. Url
        .. blank { false }
        .. scheme_allowed { ["https"] }

    // Secret type for credentials
    api_key .. Secret
        .. blank { false }

    api_secret .. Secret
        .. blank { false }

    // Decimal for rate limits
    rate_limit .. Decimal
        .. default { Decimal::from_str("1000.0") }
        .. min { Decimal::zero() }

    timeout_seconds .. u32
        .. default { 30 }
}

scheme UserSettings {
    // UUID for user identification
    user_id .. Uuid
        .. default { Uuid::new() }
        .. readonly { true }

    // Color preferences
    theme_primary .. Color
        .. default { Color::hex("#007bff") }

    theme_accent .. Color
        .. default { Color::hex("#28a745") }

    // URL for profile
    profile_url .. Url
        .. optional

    // File storage
    avatar .. Blob
        .. optional
        .. max_size { 2097152 }  // 2 MB
        .. mime_types { ["image/jpeg" "image/png"] }

    // Decimal for financial
    subscription_price .. Decimal
        .. default { Decimal::from_str("9.99") }
        .. precision { 2 }

    environment .. Environment
        .. default { Environment::production }

    created_at .. Instant
        .. default { Instant::now() }
        .. readonly { true }
}
```

### E-commerce Product Scheme

```frel
enum ProductStatus { draft active out_of_stock discontinued }
enum ProductCategory { electronics clothing home_garden books }

scheme Money {
    amount .. Decimal
        .. min { Decimal::zero() }
        .. precision { 2 }
    currency .. String .. default { "USD" }
}

scheme Dimensions {
    length .. f64 .. min { 0.0 }
    width .. f64 .. min { 0.0 }
    height .. f64 .. min { 0.0 }
    unit { enum { cm inch } } .. default { unit::cm }
}

scheme Product {
    // Identity
    id .. u64 .. readonly { true }
    sku .. String
        .. min_len { 3 }
        .. max_len { 50 }
        .. blank { false }

    // Basic info
    name .. String
        .. min_len { 3 }
        .. max_len { 200 }
        .. blank { false }

    description .. String
        .. max_len { 2000 }
        .. multiline { true }

    category .. ProductCategory

    status .. ProductStatus
        .. default { ProductStatus::draft }

    // Pricing
    price .. Money

    sale_price .. Money
        .. optional

    // Inventory
    quantity .. u32
        .. default { 0 }

    // Physical attributes
    dimensions .. Dimensions
        .. optional

    weight .. f64
        .. min { 0.0 }
        .. optional

    // Media
    images .. List<String>
        .. max_items { 10 }

    // Metadata
    tags .. Set<String>
        .. max_items { 20 }

    specifications .. Map<String, String> }
        .. max_items { 50 }

    // Timestamps
    created_at .. Instant
        .. default { Instant::now() }
        .. readonly { true }

    updated_at .. Instant
        .. auto_now { true }
}
```

## Best Practices

### Naming Conventions

```frel
// Scheme names: PascalCase
scheme UserProfile { }
scheme ShoppingCart { }

// Field names: snake_case
scheme Example {
    user_name .. String
    email_address .. String
    is_active .. Boolean
}
```

### Validation Placement

Put validation in schemes, not in UI code:

```frel
// Good - validation in scheme
scheme Email {
    address .. String
        .. pattern { r"^[\w\.-]+@[\w\.-]+\.\w+$" }
        .. blank { false }
}

// Avoid - validation in fragment
fragment EmailForm() {
    writable email = ""

    // Don't do this - validation belongs in scheme
    button { "Submit" }
        .. enabled { email.contains("@") }
}
```

### Optional vs Blank

```frel
scheme User {
    // Optional: field may be absent
    middle_name .. String
        .. optional

    // Blank false: field must be present and non-empty
    first_name .. String
        .. blank { false }

    // Both: field may be absent, but if present must not be empty
    bio .. String
        .. optional
        .. blank { false }
}
```

### Scheme Granularity

**Small focused schemes:**

```frel
// Good - focused, reusable
scheme Address { street city zip }
scheme ContactInfo { email phone }
scheme User {
    name .. String
    address .. Address
    contact .. ContactInfo
}

// Avoid - monolithic
scheme User {
    name street city zip email phone  // Too flat
}
```

## Serialization

Schemes automatically support serialization:

```frel
scheme User {
    id .. u64
    name .. String
}

let user = User { id: 1, name: "Alice" }

// JSON serialization
user.to_json()  // {"id": 1, "name": "Alice"}

// Deserialization
User::from_json(json_string)  // Result<User, ParseError>
