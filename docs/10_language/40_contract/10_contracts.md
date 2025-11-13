# Contracts

Contracts define the interface to external services and remote APIs. They are pure declarations of
what operations are available, without any implementation details such as URLs, authentication, or
transport mechanisms.

## Syntax

```text
<contract> ::= "contract" <name> "{" <method>* "}"
<method>   ::= <method-name> "(" [ <param-list> ] ")" [ ":" <return-type> ]
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
    get_user(id: u32) : User
    list_users(page: u32, limit: u32) : List<User>
    update_user(id: u32, data: UserUpdate) : User
    delete_user(id: u32)
}
```

**Rules:**

- Parameters must have explicit types
- Return types are optional
- All methods are async (no explicit `async` keyword needed)

## Usage in Backends

Contracts are declared as dependencies in backends using the `uses` keyword:

```frel
backend UserManagement(user_id: u32) {
    uses UserAPI

    user_data: User

    command load_user()
    command save_user()
}
```

## Usage in Fragments

Contracts can be called directly from fragment scope to create reactive sources:

```frel
blueprint UserProfile(user_id: u32) {
    // Contract call returns a User with Loading availability first
    // When the data arrives it sets the User availability to Ready and reactivity updates everything
    user : User = UserAPI.get_user(user_id)

    text { user.name }
}
```