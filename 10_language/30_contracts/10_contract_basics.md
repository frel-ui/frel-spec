# Contracts

Contracts define the interface to external services and remote APIs. They are pure declarations of
what operations are available, without any implementation details such as URLs, authentication, or
transport mechanisms.

## Syntax

```text
<contract> ::= "contract" <name> "{" <method>* "}"
<method>   ::= <method-name> "(" [ <param-list> ] ")" [ "->" <return-type> ]
```

## Semantics

- **Pure interface**: No implementation, URLs, authentication, or transport details
- **Runtime binding**: Actual service implementations are registered at runtime by the host
  application
- **Type safe**: Parameters and return types are validated at compile time
- **Async by default**: All contract methods are implicitly asynchronous

## Declaration

```frel
contract UserAPI {
    get_user(id: u32) -> User
    list_users(page: u32, limit: u32) -> List<User>
    update_user(id: u32, data: UserUpdate) -> User
    delete_user(id: u32)
}
```

**Rules:**

- Contract names use PascalCase
- Method names use snake_case
- Parameters must have explicit types
- Return types are optional (void/unit if omitted)
- All methods are async (no explicit `async` keyword needed)

## Usage in Backends

Contracts are declared as dependencies in backends using the `uses` keyword:

```frel
backend UserManagement(user_id: u32) {
    uses UserAPI

    writable user_data: User

    command load_user()
    command save_user()
}
```

The backend implementation can then call contract methods:

```rust
async fn load_user(&self) {
    let api = get_contract::<UserAPI>();
    let user = api.get_user(self.user_id).await?;
    self.user_data.set(user);
}
```

## Usage in Fragments (as Sources)

Contracts can be called directly from fragment scope to create reactive sources:

```frel
fragment UserProfile(user_id: u32) {
    // Contract call returns a source
    source user = UserAPI.get_user(user_id)

    text { user.latest().map(|u| u.name).unwrap_or("Loading...") }
}
```

## Runtime Registration

Contract implementations are registered at application startup:

```rust
fn main() {
    let transport = HttpTransport::new("https://api.example.com")
        .with_auth(BearerToken::from_env("API_TOKEN"));

    register_contract::<UserAPI>(transport);

    Application::run();
}
```

## Error Handling

Contract methods return `Result<T, ServiceError>` implicitly. Error handling is performed in the
backend or event handler:

```rust
match api.get_user(id).await {
Ok(user) => { /* handle success */ }
Err(e) => { /* handle error */ }
}
```

## Generated Code

The contract declaration generates a trait that must be implemented:

```rust
#[async_trait]
pub trait UserAPI: Service {
    async fn get_user(&self, id: u32) -> Result<User, ServiceError>;
    async fn list_users(&self, page: u32, limit: u32) -> Result<Vec<User>, ServiceError>;
    async fn update_user(&self, id: u32, data: UserUpdate) -> Result<User, ServiceError>;
    async fn delete_user(&self, id: u32) -> Result<(), ServiceError>;
}
```

## Example

```frel
contract PointAPI {
    get_point(id: u32) -> PointConfig
    list_points(network_id: u32) -> List<PointConfig>
    create_point(data: PointConfig) -> PointConfig
    update_point(id: u32, data: PointConfig) -> PointConfig
    delete_point(id: u32)
    validate_point(data: PointConfig) -> ValidationResult
}
```
