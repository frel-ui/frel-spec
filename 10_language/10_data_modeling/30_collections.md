# Collections

Frel provides platform-independent collection types (`List`, `Set`, `Map`, `Tree`) that are implemented using
Frel-provided host-language-dependent wrappers (`FrelList`, `FrelSet`, `FrelMap`, `FrelTree`). These wrappers are
small abstractions around native host language types with built-in reactivity support.

## List - Ordered, Indexed Collection

Ordered sequence of values with integer indexing:

```frel
scheme ListExamples {
    tags .. List<String>
    scores .. List<f64>
    items .. List<Item>
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
    categories .. Set<String>
    ids .. Set<u64>
    visited_pages .. Set<String>
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

Key-value mapping with hashable key types:

```frel
scheme MapExamples {
    metadata .. Map<String, String>
    counters .. Map<String, i32>
    prices .. Map<u64, f64>
    settings .. Map<Uuid, String>
    endpoints .. Map<Url, String>
}
```

**Key Type Restrictions:** Map keys must be hashable types:
- **Numeric types**: All integer types (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`) and floating-point types (`f32`, `f64`)
- **Decimal**: Arbitrary-precision decimal numbers
- **String**: UTF-8 text strings
- **Url**: URL/URI values
- **Uuid**: Universally unique identifiers
- **Enum variants**: Any enum type

**Not allowed as keys:** Collections (List, Set, Map, Tree), Schemes, DateTime types, Secret, Color, Blob.

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
    data_points .. List<List<f64>>

    // Map with list values
    user_tags .. Map<u64, List<String>>

    // Set of numeric IDs
    active_users .. Set<u64>

    // Map with set values
    user_permissions .. Map<Uuid, Set<String>>

    // Enum as map key
    status_counts .. Map<TaskStatus, u32>

    // URL as map key
    api_cache .. Map<Url, String>
}
```

## Collection Validation

Collections support validation instructions:

```frel
scheme ValidatedCollections {
    // Size constraints
    tags .. List<String>
        .. min_items { 1 }
        .. max_items { 10 }

    // Item validation
    emails .. List<String>
        .. each .. pattern { r"^[\w\.-]+@[\w\.-]+\.\w+$" }

    // Default values
    categories .. Set<String>
        .. default { Set::new() }

    // Key validation for maps
    metadata .. Map<String, String>
        .. max_items { 20 }
        .. key_pattern { "^[a-z_]+$" }
}
```

## Reactivity

Collections support fine-grained reactivity:

```frel
blueprint TodoList() {
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

## Tree - Hierarchical Structure

First-class hierarchical collection with automatic node ID management, efficient updates, and fine-grained
reactivity. Trees are stored in a flat structure internally for optimal performance and scalability.

### Overview

**Use Tree when:**
- Data has parent-child relationships (file systems, org charts, categories, navigation menus)
- You need hierarchical navigation
- Structure changes frequently (add/remove/move nodes)
- UI needs to render nested/recursive views

**Storage model:** Flat hash-map based storage with ID references for optimal performance and reactivity.

### Syntax

```frel
scheme Example {
    hierarchy .. Tree<NodeType>
    navigation .. Tree<MenuItem>
    file_system .. Tree<FileNode>
}
```

### Semantics

- **Type parameter**: `T` must be a valid Frel or host language type
- **Node IDs**: Automatically generated and managed internally
- **Structure**: Multi-child - each node can have any number of children
- **Homogeneous**: All nodes contain the same type `T`
- **Flat storage**: Nodes stored in HashMap, hierarchy tracked separately
- **Cycle prevention**: Tree operations prevent cycles automatically

### Basic Example

```frel
scheme FileNode {
    name .. String .. blank { false }
    size .. u64 .. optional
    is_folder .. bool
}

scheme FileSystem {
    files .. Tree<FileNode>
}

blueprint FileManager() {
    writable fs = FileSystem {
        files: Tree::new(FileNode {
            name: "root",
            size: None,
            is_folder: true
        })
    }

    // Tree operations here
}
```

### Core Operations

#### Creation

```frel
// Create tree with root node
writable tree = Tree::new(root_value)

// Get root node ID
decl root_id = tree.root()
```

#### Access Operations

```frel
// Get node value by ID
decl node = tree.get(node_id)  // -> Option<&T>

// Get node value mutably (for in-place updates)
tree.get_mut(node_id)  // -> Option<&mut T>

// Get parent of node
decl parent_id = tree.parent(node_id)  // -> Option<NodeId>

// Get children of node
decl children = tree.children(node_id)  // -> List<NodeId>

// Check if node exists
decl exists = tree.contains(node_id)  // -> bool

// Get path from root to node
decl path = tree.path(node_id)  // -> List<NodeId>

// Get depth of node
decl depth = tree.depth(node_id)  // -> u32
```

#### Mutation Operations

```frel
// Add child node (returns new node ID)
let child_id = tree.insert(parent_id, child_value)

// Add child at specific position
let child_id = tree.insert_at(parent_id, index, child_value)

// Remove node and all descendants
tree.remove(node_id)

// Update node value
tree.update(node_id, new_value)

// Move node to new parent (reparent)
tree.move_node(node_id, new_parent_id)

// Move node to specific position under parent
tree.move_to(node_id, new_parent_id, index)

// Swap two sibling nodes
tree.swap(node_id_1, node_id_2)
```

#### Query Operations

```frel
// Find node by predicate
decl found = tree.find(|node| node.name == "config.json")  // -> Option<NodeId>

// Find all matching nodes
decl matches = tree.find_all(|node| node.is_folder)  // -> List<NodeId>

// Count total nodes
decl count = tree.node_count()  // -> usize

// Count descendants of node
decl desc_count = tree.descendant_count(node_id)  // -> usize

// Check if node is ancestor of another
decl is_anc = tree.is_ancestor(ancestor_id, descendant_id)  // -> bool

// Get siblings of node
decl siblings = tree.siblings(node_id)  // -> List<NodeId>
```

#### Traversal

```frel
// Pre-order traversal (parent before children)
decl nodes = tree.traverse_preorder(node_id)  // -> List<NodeId>

// Post-order traversal (children before parent)
decl nodes = tree.traverse_postorder(node_id)  // -> List<NodeId>

// Breadth-first traversal (level by level)
decl nodes = tree.traverse_breadth_first(node_id)  // -> List<NodeId>

// Get all leaf nodes in subtree
decl leaves = tree.leaves(node_id)  // -> List<NodeId>
```

### Tree Reactivity

Trees support fine-grained reactivity at multiple granularities:

**Node-level subscriptions:**
```frel
writable tree = Tree::new(root)

// Subscribe to specific node value
decl node = tree.get(node_id)
// Recomputes only when this node's value changes

// Subscribe to node's children list
decl children = tree.children(node_id)
// Recomputes when children added/removed/reordered
```

**Subtree subscriptions:**
```frel
// Subscribe to any change in subtree
decl subtree_data = tree.subtree(root_id)
// Recomputes when any node in subtree is added/removed/updated

// Count nodes in subtree (structural subscription)
decl count = tree.descendant_count(node_id)
// Recomputes when nodes added/removed in subtree
```

**Structural subscriptions:**
```frel
// Subscribe to tree structure
decl total_nodes = tree.node_count()
// Recomputes when any node added/removed anywhere

// Subscribe to specific path
decl path_exists = tree.contains(node_id)
// Recomputes if node is removed or path changes
```

### UI Integration

#### Recursive Rendering

```frel
blueprint TreeView(tree: Tree<FileNode>, node_id: NodeId) {
    decl node = tree.get(node_id).unwrap()
    decl children = tree.children(node_id)

    column {
        // Node content
        row {
            icon { if node.is_folder { "folder" } else { "file" } }
            text { node.name }
        }

        // Recursive children
        when node.is_folder && !children.is_empty() {
            column {
                indent { 20 }
                repeat on children as child_id {
                    TreeView(tree, child_id)
                }
            }
        }
    }
}
```

#### Interactive Tree with Expand/Collapse

```frel
blueprint FileExplorer() {
    writable tree = Tree::new(FileNode {
        name: "root",
        size: None,
        is_folder: true
    })

    writable selected: Option<NodeId> = None
    writable expanded: Set<NodeId> = Set::new()

    column {
        FileTreeNode(tree, tree.root(), selected, expanded)
    }
}

blueprint FileTreeNode(
    tree: Tree<FileNode>,
    node_id: NodeId,
    selected: Option<NodeId>,
    expanded: Set<NodeId>
) {
    decl node = tree.get(node_id).unwrap()
    decl children = tree.children(node_id)
    decl is_selected = selected == Some(node_id)
    decl is_expanded = expanded.contains(node_id)

    column {
        // Node row
        row {
            // Expand/collapse button
            when node.is_folder && !children.is_empty() {
                button { if is_expanded { "▼" } else { "▶" } }
                    .. on_click {
                        if is_expanded {
                            expanded.remove(node_id)
                        } else {
                            expanded.insert(node_id)
                        }
                    }
            }

            // Node content
            row {
                icon { if node.is_folder { "folder" } else { "file" } }
                text { node.name }
            }
            .. background { if is_selected { LightBlue } else { Transparent } }
            .. on_click { selected = Some(node_id) }
        }

        // Children (if expanded)
        when is_expanded {
            column {
                indent { 20 }
                repeat on children as child_id {
                    FileTreeNode(tree, child_id, selected, expanded)
                }
            }
        }
    }
}
```

### Tree Use Cases

**Navigation menus:**
```frel
scheme MenuItem {
    label .. String
    icon .. String .. optional
    route .. String .. optional
}

scheme AppState {
    navigation .. Tree<MenuItem>
}
```

**File browsers:**
```frel
scheme FileNode {
    name .. String
    size .. u64 .. optional
    is_folder .. bool
    created .. Instant
}
```

**Organizational charts:**
```frel
scheme Employee {
    name .. String
    title .. String
    email .. String
    avatar_url .. String .. optional
}
```

**Category hierarchies:**
```frel
scheme Category {
    name .. String
    description .. String .. optional
    product_count .. u32
}
```

### Performance Considerations

The flat HashMap-based storage provides several advantages:

1. **Constant-time access**: O(1) lookup by NodeId
2. **Efficient updates**: Updating one node doesn't require traversing tree
3. **Fine-grained reactivity**: Only affected nodes trigger notifications
4. **Backend alignment**: Easy to sync with database queries by ID

**Optimization tips:**
```frel
// Good - traverse once and cache
decl all_nodes = tree.traverse_preorder(root)
repeat on all_nodes as node_id {
    render_node(tree.get(node_id).unwrap())
}

// Avoid - repeated traversals
repeat on tree.children(parent) as child {
    if tree.children(child).is_empty() {  // Recomputes each time
        // ...
    }
}

// Better - cache structure queries
decl is_leaf = tree.children(node_id).is_empty()
```

### Tree Best Practices

**Use NodeId consistently:**
```frel
// Good - pass NodeIds
blueprint TreeNode(tree: Tree<T>, node_id: NodeId) { }

// Avoid - extracting and passing values
blueprint TreeNode(tree: Tree<T>, value: T) { }  // Loses tree context
```

**Batch operations:**
```frel
// Good - batch related operations
button { "Move All" } .. on_click {
    for node_id in nodes_to_move {
        tree.move_node(node_id, new_parent)
    }
    // Single reactivity update after all moves
}
```

**Cache expensive queries:**
```frel
// Good - cache traversal results
decl all_leaves = tree.leaves(root_id)

repeat on all_leaves as leaf_id {
    // Use cached result
}

// Avoid - repeated expensive queries
repeat on some_list as item {
    let leaves = tree.leaves(root_id)  // Recomputes every iteration
}
```

**Handle errors gracefully:**
```frel
// Good - check node exists
when tree.contains(node_id) {
    decl node = tree.get(node_id).unwrap()
    // Use node
}

// Avoid - unwrap without checking
decl node = tree.get(node_id).unwrap()  // May panic if node removed
```

## Type Mapping

| Frel Type    | Rust           | TypeScript     | Python         |
|--------------|----------------|----------------|----------------|
| `List<T>`    | `FrelList<T>`  | `FrelList<T>`  | `FrelList[T]`  |
| `Set<T>`     | `FrelSet<T>`   | `FrelSet<T>`   | `FrelSet[T]`   |
| `Map<K,V>`   | `FrelMap<K,V>` | `FrelMap<K,V>` | `FrelMap[K,V]` |
| `Tree<T>`    | `FrelTree<T>`  | `FrelTree<T>`  | `FrelTree[T]`  |
| `NodeId`     | `usize`/`u64`  | `number`       | `int`          |

**Note:** `FrelList`, `FrelSet`, `FrelMap`, and `FrelTree` are Frel-provided wrappers around host language-specific
implementations (e.g., `Vec`, `HashSet`, `HashMap` in Rust) with built-in reactivity support.

## Best Practices

### Choose the Right Collection

```frel
// Use List when order matters
scheme Playlist {
    songs .. List<Song>  // Order is important
}

// Use Set for uniqueness
scheme User {
    roles .. Set<String>  // No duplicate roles
}

// Use Map for lookups
scheme Cache {
    entries .. Map<String, String>  // Fast key lookup
}
```

### Initialize with Defaults

```frel
scheme Config {
    tags .. List<String>
        .. default { List::new() }

    categories .. Set<String>
        .. default { Set::new() }

    metadata .. Map<String, String>
        .. default { Map::new() }
}
```

### Validate Collection Contents

```frel
scheme Article {
    tags .. List<String>
        .. min_items { 1 }          // At least one tag
        .. max_items { 10 }         // Maximum 10 tags
        .. each .. min_len { 2 }    // Each tag at least 2 chars
        .. each .. max_len { 20 }   // Each tag max 20 chars
}
```
