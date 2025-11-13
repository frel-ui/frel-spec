# Field Access

Field access expressions allow you to read data from schemes, backends, and other structured types.

## Basic Field Access

Access fields using dot notation:

```frel
blueprint UserProfile(user: User) {
    name : String = user.name
    email : String = user.email
    city : String = user.address.city
}
```

### Nested Access

Fields can be chained to access nested data:

```frel
deepValue : String = root.level1.level2.level3.value
```

## Optional Chaining

Use `?.` to safely access fields that might be null:

```frel
blueprint UserAvatar(user: User) {
    // If user.profile is null, avatarUrl becomes null
    avatarUrl : String? = user?.profile?.avatar?.url

    // Can mix regular and optional chaining
    city : String? = user?.address.city
}
```

**Behavior:**
- If any part of the chain is `null`, the entire expression evaluates to `null`
- The result type becomes optional (`T?`)
- Stops evaluation at the first `null` encountered

**Examples:**

```frel
scheme Profile {
    avatar : Avatar?
}

scheme Avatar {
    url : String
    size : i32
}

scheme User {
    profile : Profile?
}

// Various chaining scenarios
url1 : String? = user?.profile?.avatar?.url  // All optional
url2 : String? = user.profile?.avatar?.url   // First required, rest optional
size : i32? = user?.profile?.avatar?.size    // Chain with different field type
```

## Collection Query Methods

Collections provide read-only query methods:

### Length

Get the number of elements:

```frel
items : List<Item> = get_items()
count : i32 = items.length
```

**Available for:**
- `List<T>`
- `Set<T>`
- `Map<K,V>`

### Examples

```frel
blueprint ShoppingCart(items: List<CartItem>) {
    itemCount : i32 = items.length
    hasItems : bool = items.length > 0

    // Use in conditionals
    checkoutEnabled : bool = hasItems && items.length <= maxItems

    // Use in display
    countText : String = "${items.length} items"
}
```

## Iteration with Repeat

To access individual collection elements, use the `repeat` fragment instruction:

```frel
blueprint TodoList(items: List<Todo>) {
    itemCount : i32 = items.length

    column {
        text { "You have ${itemCount} tasks" }

        repeat on items { todo ->
            row {
                checkbox { todo.done }
                text { todo.text }
            }
        }
    }
}
```

**Why no index access?**

Direct index access (`items[0]`) is not supported because:
- Index-based UI patterns are fragile and error-prone
- Iteration with `repeat` is more declarative and robust
- Eliminates index-out-of-bounds errors
- Encourages better UI patterns

## Field Access and Availability

Field access propagates availability states:

```frel
blueprint UserDisplay(userId: u32) {
    // Contract call - starts as Loading
    user : User = UserAPI.get_user(userId)

    // Field access inherits availability
    name : String = user.name
    // When user is Loading: name is Loading
    // When user is Ready: name is Ready
    // When user is Error: name is Error
}
```

See [Reactivity Model](../20_data_model/10_reactivity_model.md#availability-propagation) for details.

## Reference Fields

Reference fields access data from arenas:

```frel
scheme Thermometer {
    id : UUID .. identity
    location : ref Location
    name : String
}

blueprint ThermometerCard(thermo: Thermometer) {
    // Access reference field - triggers arena lookup
    locationName : String = thermo.location.name

    text { locationName }
}
```

**Key properties:**
- Reference lookup happens automatically
- Availability tracks arena resolution
- Field access chains through references transparently

See [Reactivity Model - Reference Types](../20_data_model/10_reactivity_model.md#reference-types) for details.

## Examples

### Simple Display

```frel
blueprint UserBadge(user: User) {
    fullName : String = "${user.firstName} ${user.lastName}"

    text { fullName }
}
```

### Nested Data

```frel
blueprint AddressDisplay(user: User) {
    street : String = user.address.street
    city : String = user.address.city
    state : String = user.address.state
    zip : String = user.address.zip

    fullAddress : String = "${street}, ${city}, ${state} ${zip}"

    text { fullAddress }
}
```

### Optional Fields

```frel
blueprint ContactInfo(user: User) {
    // Optional phone
    hasPhone : bool = user.phone != null
    phoneDisplay : String = user.phone != null ?
                            user.phone :
                            "No phone"

    // Optional chaining for nested optional
    website : String? = user?.profile?.website
    websiteDisplay : String = website != null ?
                              website :
                              "No website"
}
```

### Collection Queries

```frel
blueprint TodoSummary(todos: List<Todo>) {
    totalCount : i32 = todos.length
    isEmpty : bool = todos.length == 0

    // Computed in host language backend
    completedCount : i32 = backend.count_completed()

    summaryText : String = isEmpty ?
                           "No tasks" :
                           "${completedCount} of ${totalCount} complete"

    text { summaryText }
}
```

## When to Use Host Language

For complex queries, filtering, or transformations, use the host language:

```frel
// ❌ Don't try to filter/transform in Frel
// This is not supported:
// activeUsers = users.filter(u => u.isActive)  // NOT AVAILABLE

// ✅ Do filtering in host language backend
backend UserList {
    allUsers : List<User> = UserAPI.get_all()

    // Filtered in Rust/TypeScript/etc
    activeUsers : List<User> = filter_active_users()
    recentUsers : List<User> = filter_recent_users()
}

blueprint UserListDisplay {
    with UserList

    // Simple query in Frel
    activeCount : i32 = activeUsers.length

    // Display in Frel
    text { "${activeCount} active users" }

    repeat on activeUsers { user ->
        text { user.name }
    }
}
```
