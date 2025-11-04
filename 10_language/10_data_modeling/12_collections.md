# Collections

Frel provides platform-independent collection types (`List`, `Set`, `Map`) that are implemented using
Frel-provided host-language-dependent wrappers (`FrelList`, `FrelSet`, `FrelMap`). These wrappers are
small abstractions around native host language types with built-in reactivity support.

## List - Ordered, Indexed Collection

Ordered sequence of values with integer indexing:

```frel
scheme ListExamples {
    tags { List<String> }
    scores { List<f64> }
    items { List<Item> }
}
```

### Operations

```frel
let tags = List::new()

// Adding elements
tags.push("rust")           // Add to end
tags.push("web")
tags.insert(0, "frel")      // Insert at index

// Accessing elements
tags.len()                  // -> 3
tags.get(0)                 // -> Some("frel")
tags.first()                // -> Some("frel")
tags.last()                 // -> Some("web")
tags.is_empty()             // -> false

// Removing elements
tags.remove(1)              // Remove at index
tags.pop()                  // Remove and return last

// Iteration
tags.iter()
tags.contains("rust")       // Check membership
```

## Set - Unique Values, Unordered

Unordered collection of unique values:

```frel
scheme SetExamples {
    categories { Set<String> }
    ids { Set<u64> }
    visited_pages { Set<String> }
}
```

### Operations

```frel
let categories = Set::new()

// Adding elements
categories.insert("electronics")
categories.insert("electronics")  // Duplicate ignored

// Checking membership
categories.contains("electronics") // -> true
categories.len()                   // -> 1
categories.is_empty()              // -> false

// Removing elements
categories.remove("electronics")

// Set operations
set1.union(set2)           // All elements from both
set1.intersection(set2)    // Common elements
set1.difference(set2)      // Elements in set1 but not set2
set1.is_subset(set2)       // Check if all elements in set2
```

## Map - Key-Value Pairs

Key-value mapping with primitive type keys:

```frel
scheme MapExamples {
    metadata { Map<String, String> }
    counters { Map<String, i32> }
    prices { Map<u64, f64> }
    settings { Map<Uuid, String> }
}
```

**Important:** Map keys must be primitive types (String, integers, floats, bool, Uuid).

### Operations

```frel
let metadata = Map::new()

// Adding/updating entries
metadata.insert("version", "1.0")
metadata.insert("author", "Alice")

// Accessing values
metadata.get("version")        // -> Some("1.0")
metadata.get("missing")        // -> None
metadata.contains_key("author") // -> true

// Removing entries
metadata.remove("version")

// Querying
metadata.len()                 // -> 1
metadata.is_empty()            // -> false
metadata.keys()                // Iterator over keys
metadata.values()              // Iterator over values
```

## Nested Collections

Collections can be nested to model complex data structures:

```frel
scheme Analytics {
    // List of lists
    data_points { List<List<f64>> }

    // Map with list values
    user_tags { Map<u64, List<String>> }

    // Set of numeric IDs
    active_users { Set<u64> }

    // Map with set values
    user_permissions { Map<Uuid, Set<String>> }
}
```

## Collection Validation

Collections support validation instructions:

```frel
scheme ValidatedCollections {
    // Size constraints
    tags { List<String> }
        .. min_items { 1 }
        .. max_items { 10 }

    // Item validation
    emails { List<String> }
        .. each .. pattern { r"^[\w\.-]+@[\w\.-]+\.\w+$" }

    // Default values
    categories { Set<String> }
        .. default { Set::new() }

    // Key validation for maps
    metadata { Map<String, String> }
        .. max_items { 20 }
        .. key_pattern { "^[a-z_]+$" }
}
```

## Reactivity

Collections support fine-grained reactivity:

```frel
fragment TodoList() {
    writable todos = List::new()

    column {
        // Subscribes to list changes
        repeat on todos as todo {
            text { todo }
        }

        button { "Add" }
            .. on_click {
                todos.push("New item")
                // UI automatically updates
            }
    }
}
```

**Reactivity granularity:**
- **List**: Tracks insertions, removals, and reordering
- **Set**: Tracks additions and removals
- **Map**: Tracks key insertions, updates, and removals

## Type Mapping

| Frel Type    | Rust          | TypeScript    | Python        |
|--------------|---------------|---------------|---------------|
| `List<T>`    | `FrelList<T>` | `FrelList<T>` | `FrelList[T]` |
| `Set<T>`     | `FrelSet<T>`  | `FrelSet<T>`  | `FrelSet[T]`  |
| `Map<K,V>`   | `FrelMap<K,V>`| `FrelMap<K,V>`| `FrelMap[K,V]`|

**Note:** `FrelList`, `FrelSet`, and `FrelMap` are Frel-provided wrappers around host language-specific
implementations (e.g., `Vec`, `HashSet`, `HashMap` in Rust) with built-in reactivity support.

## Best Practices

### Choose the Right Collection

```frel
// Use List when order matters
scheme Playlist {
    songs { List<Song> }  // Order is important
}

// Use Set for uniqueness
scheme User {
    roles { Set<String> }  // No duplicate roles
}

// Use Map for lookups
scheme Cache {
    entries { Map<String, String> }  // Fast key lookup
}
```

### Initialize with Defaults

```frel
scheme Config {
    tags { List<String> }
        .. default { List::new() }

    categories { Set<String> }
        .. default { Set::new() }

    metadata { Map<String, String> }
        .. default { Map::new() }
}
```

### Validate Collection Contents

```frel
scheme Article {
    tags { List<String> }
        .. min_items { 1 }          // At least one tag
        .. max_items { 10 }         // Maximum 10 tags
        .. each .. min_len { 2 }    // Each tag at least 2 chars
        .. each .. max_len { 20 }   // Each tag max 20 chars
}
```
