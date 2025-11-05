# Rust Interop

This document specifies how Frel language constructs map to Rust code.

## 1. Scheme Representation

Schemes are represented as **traits** in Rust. The actual storage mechanism is an internal implementation detail handled by the Frel runtime.

### Design Rationale

- **Trait-based**: Schemes generate traits, not structs. This keeps storage implementation flexible.
- **No direct instantiation**: Users don't create scheme instances directly; the Frel runtime manages storage.
- **Field access via methods**: Each field becomes a pair of accessor methods (getter and optional setter).
- **Validation API**: Schemes provide validation methods that operate on the internal state.

### Generated Trait Structure

For a Frel scheme definition:

```frel
scheme User {
    username .. String
        .. min_len { 3 }
        .. blank { false }

    email .. String
        .. pattern { r"^[\w\.-]+@[\w\.-]+\.\w+$" }
        .. blank { false }

    age .. u8
        .. range { 13 150 }
        .. optional
}
```

The Frel compiler generates:

```rust
/// Generated trait for the User scheme
pub trait User: Scheme {
    // Field accessors - read only
    fn username(&self) -> &str;
    fn email(&self) -> &str;
    fn age(&self) -> Option<u8>;

    // Field mutators (for writable scheme instances)
    fn set_username(&mut self, value: String);
    fn set_email(&mut self, value: String);
    fn set_age(&mut self, value: Option<u8>);

    // Validation API
    fn validate(&self) -> Result<(), Vec<FieldError>>;
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
    fn errors(&self) -> Vec<FieldError> {
        self.validate().err().unwrap_or_default()
    }

    // Field-specific validation
    fn validate_username(&self) -> Result<(), FieldError>;
    fn validate_email(&self) -> Result<(), FieldError>;
    fn validate_age(&self) -> Result<(), FieldError>;
}
```

### Base Scheme Trait

All generated scheme traits extend a base `Scheme` trait:

```rust
/// Base trait for all Frel schemes
pub trait Scheme {
    /// Returns the scheme name (for debugging/logging)
    fn scheme_name(&self) -> &'static str;

    /// Validates all fields
    fn validate_all(&self) -> Result<(), Vec<FieldError>>;

    /// Serialization support
    fn to_json(&self) -> Result<String, SerializationError>;
    fn from_json(json: &str) -> Result<Box<dyn Self>, SerializationError>
    where
        Self: Sized;
}
```

### Field Accessor Patterns

#### Read-Only Fields

For fields marked with `.. readonly { true }`:

```rust
pub trait Entity: Scheme {
    fn id(&self) -> u64;
    // No set_id() method generated
}
```

#### Optional Fields

Optional fields return `Option<T>`:

```rust
pub trait Profile: Scheme {
    fn bio(&self) -> Option<&str>;
    fn set_bio(&mut self, value: Option<String>);
}
```

#### Nested Schemes

Nested schemes return references to the nested scheme trait:

```frel
scheme Address {
    street .. String
    city .. String
}

scheme User {
    name .. String
    address .. Address
}
```

Generates:

```rust
pub trait User: Scheme {
    fn name(&self) -> &str;
    fn set_name(&mut self, value: String);

    fn address(&self) -> &dyn Address;
    fn set_address(&mut self, value: Box<dyn Address>);
}
```

#### Collection Fields

Collections return references to Frel collection wrappers:

```rust
pub trait User: Scheme {
    fn tags(&self) -> &FrelList<String>;
    fn tags_mut(&mut self) -> &mut FrelList<String>;

    fn roles(&self) -> &FrelSet<String>;
    fn roles_mut(&mut self) -> &mut FrelSet<String>;

    fn metadata(&self) -> &FrelMap<String, String>;
    fn metadata_mut(&mut self) -> &mut FrelMap<String, String>;
}
```

### Validation API

#### FieldError Structure

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct FieldError {
    /// Field name (e.g., "username")
    pub field: String,

    /// Human-readable error message
    pub message: String,

    /// Error code for programmatic handling (e.g., "min_len", "pattern")
    pub code: String,

    /// Optional metadata (e.g., {"min": 3, "actual": 2})
    pub metadata: Option<HashMap<String, String>>,
}
```

#### Validation Method Behavior

```rust
// validate() returns all errors
let errors = user.validate();
match errors {
    Ok(()) => println!("User is valid"),
    Err(errs) => {
        for err in errs {
            println!("{}: {}", err.field, err.message);
        }
    }
}

// is_valid() is a convenience method
if user.is_valid() {
    save_user(&user);
}

// errors() returns empty vec if valid
let all_errors = user.errors();
```

#### Field-Specific Validation

```rust
// Validate a single field
match user.validate_username() {
    Ok(()) => println!("Username is valid"),
    Err(err) => println!("Username error: {}", err.message),
}
```

### Instantiation

Schemes must be instantiable from Rust code for several use cases:

1. **Sources**: When receiving data from external systems (APIs, websockets, etc.)
2. **Event handlers**: When constructing data to send or store
3. **Testing**: Creating instances for unit tests
4. **Default initialization**: When the DSL specifies `writable user = User()`

#### Factory Functions

For each scheme, the Frel compiler generates factory functions:

```rust
/// Create a new User instance with default values from the scheme definition
pub fn create_user() -> Box<dyn User> {
    // Applies defaults from scheme:
    // - username defaults to ""
    // - email defaults to ""
    // - age defaults to None
}

/// Create a User instance with specific values (validates on construction)
pub fn create_user_with(
    username: String,
    email: String,
    age: Option<u8>,
) -> Result<Box<dyn User>, Vec<FieldError>> {
    let user = create_user();
    user.set_username(username);
    user.set_email(email);
    user.set_age(age);

    // Validate before returning
    user.validate()?;
    Ok(user)
}

/// Create a User from unvalidated data (e.g., from JSON/API responses)
/// This skips validation during construction but the instance can be validated later
pub fn create_user_unchecked(
    username: String,
    email: String,
    age: Option<u8>,
) -> Box<dyn User> {
    let user = create_user();
    user.set_username(username);
    user.set_email(email);
    user.set_age(age);
    user
}
```

#### Builder Pattern

For schemes with many fields, a builder pattern is also generated:

```rust
pub struct UserBuilder {
    username: Option<String>,
    email: Option<String>,
    age: Option<u8>,
}

impl UserBuilder {
    pub fn new() -> Self {
        Self {
            username: None,
            email: None,
            age: None,
        }
    }

    pub fn username(mut self, value: String) -> Self {
        self.username = Some(value);
        self
    }

    pub fn email(mut self, value: String) -> Self {
        self.email = Some(value);
        self
    }

    pub fn age(mut self, value: u8) -> Self {
        self.age = Some(value);
        self
    }

    /// Build with validation
    pub fn build(self) -> Result<Box<dyn User>, Vec<FieldError>> {
        create_user_with(
            self.username.unwrap_or_default(),
            self.email.unwrap_or_default(),
            self.age,
        )
    }

    /// Build without validation
    pub fn build_unchecked(self) -> Box<dyn User> {
        create_user_unchecked(
            self.username.unwrap_or_default(),
            self.email.unwrap_or_default(),
            self.age,
        )
    }
}
```

#### Usage in Sources

Sources commonly create scheme instances from external data:

```rust
// In a source implementation for fetching user data
async fn fetch_user(id: u32) -> Result<Box<dyn User>, Error> {
    let response = api_client.get_user(id).await?;

    // Create User from API response
    let user = UserBuilder::new()
        .username(response.username)
        .email(response.email)
        .age(response.age)
        .build()?; // Validates

    Ok(user)
}
```

#### Usage in Event Handlers

Event handlers can construct schemes:

```frel
fragment UserCreator() {
    writable username = ""
    writable email = ""

    button { "Create User" }
        .. on_click {
            // Construct a User in the event handler
            let new_user = create_user_with(
                username.clone(),
                email.clone(),
                None
            );

            match new_user {
                Ok(user) => save_user(user),
                Err(errors) => show_errors(errors),
            }
        }
}
```

### Serialization

Schemes automatically support JSON serialization:

```rust
// Serialize to JSON
let json = user.to_json()?;
// {"username": "alice", "email": "alice@example.com", "age": 25}

// Deserialize from JSON
let user: Box<dyn User> = User::from_json(json)?;
```

For other formats, schemes implement standard Rust traits:

```rust
// Generated implementations (via serde)
impl Serialize for UserImpl { /* generated */ }
impl Deserialize for UserImpl { /* generated */ }
```

Where `UserImpl` is the internal storage struct (not exposed to users).

### Usage Examples

#### Creating and Validating

```rust
let mut user = create_user();

user.set_username("alice".to_string());
user.set_email("alice@example.com".to_string());
user.set_age(Some(25));

if !user.is_valid() {
    for error in user.errors() {
        eprintln!("Validation error in {}: {}", error.field, error.message);
    }
}
```

#### Using in Fragments (via DSL)

```frel
fragment UserEditor() {
    writable user = User()

    column {
        text_input { user.username }
        text_input { user.email }
        number_input { user.age }

        when !user.is_valid() {
            column {
                repeat on user.errors() as error {
                    text { error.message }
                        .. font { color: Red size: 12 }
                }
            }
        }

        button { "Save" }
            .. enabled { user.is_valid() }
            .. on_click {
                save_user(user.clone())
            }
    }
}
```

#### Accessing Nested Fields

```rust
let user: Box<dyn User> = create_user();
let address = user.address();
let city = address.city(); // &str
```

### Design Decisions

#### Why Traits Instead of Structs?

1. **Flexibility**: Storage implementation can be optimized without changing the API
2. **Reactivity**: The runtime can wrap fields in reactive cells transparently
3. **Testing**: Easy to mock schemes with custom implementations
4. **Future-proofing**: Can add features (like undo/redo, time-travel) without API changes

#### Performance Considerations

- **Trait objects**: `Box<dyn Scheme>` has virtual dispatch overhead, but:
  - Most operations happen through generated code (monomorphized)
  - Only cross-fragment boundaries use trait objects
  - Modern CPUs handle indirect calls efficiently
- **Field access**: Getters are `#[inline]` and typically optimized away
- **Validation**: Only runs on-demand or on mutation (configurable)

### Future Extensions

Potential additions to the scheme trait system:

1. **Derive macros**: Allow custom structs to implement scheme traits
2. **Builder pattern**: Fluent API for construction
3. **Lenses**: Functional updates for nested structures
4. **Partial validation**: Validate only changed fields
5. **Async validation**: Support for validation requiring I/O

---

**Status**: Scheme representation is specified. Other sections (enums, backends, stores, etc.) to be added.
