# Tree

The `Tree<T>` type provides a first-class hierarchical collection with automatic node ID 
management, efficient updates, and fine-grained reactivity. Trees are stored in flat 
structure internally for optimal performance and scalability.

## Overview

**Use Tree when:**
- Data has parent-child relationships (file systems, org charts, categories)
- You need hierarchical navigation
- Structure changes frequently (add/remove/move nodes)
- UI needs to render nested/recursive views

**Storage model:** Flat hash-map based storage with ID references for optimal performance 
and reactivity.

## Syntax

```frel
scheme Example {
    hierarchy { Tree<NodeType> }
}
```

## Semantics

- **Type parameter**: `T` must be a valid Frel or host language type
- **Node IDs**: Automatically generated and managed internally
- **Structure**: Multi-child - each node can have any number of children
- **Homogeneous**: All nodes contain the same type `T`
- **Flat storage**: Nodes stored in HashMap, hierarchy tracked separately
- **Cycle prevention**: Tree operations prevent cycles automatically

## Basic Example

```frel
scheme FileNode {
    name { String } .. blank { false }
    size { u64 } .. optional { true }
    is_folder { bool }
}

scheme FileSystem {
    files { Tree<FileNode> }
}

fragment FileManager() {
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

## Core Operations

### Creation

```frel
// Create tree with root node
writable tree = Tree::new(root_value)

// Get root node ID
decl root_id = tree.root()
```

### Access Operations

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

### Mutation Operations

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

### Query Operations

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

### Traversal

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

## Reactivity

Trees support fine-grained reactivity at multiple granularities:

### Node-level Subscriptions

```frel
writable tree = Tree::new(root)

// Subscribe to specific node value
decl node = tree.get(node_id)
// Recomputes only when this node's value changes

// Subscribe to node's children list
decl children = tree.children(node_id)
// Recomputes when children added/removed/reordered
```

### Subtree Subscriptions

```frel
// Subscribe to any change in subtree
decl subtree_data = tree.subtree(root_id)
// Recomputes when any node in subtree is added/removed/updated

// Count nodes in subtree (structural subscription)
decl count = tree.descendant_count(node_id)
// Recomputes when nodes added/removed in subtree
```

### Structural Subscriptions

```frel
// Subscribe to tree structure
decl total_nodes = tree.node_count()
// Recomputes when any node added/removed anywhere

// Subscribe to specific path
decl path_exists = tree.contains(node_id)
// Recomputes if node is removed or path changes
```

## UI Integration

### Recursive Rendering

```frel
fragment TreeView(tree: Tree<FileNode>, node_id: NodeId) {
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

### Interactive Tree

```frel
fragment FileExplorer() {
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

fragment FileTreeNode(
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

### Drag and Drop

```frel
fragment DraggableTree(tree: Tree<FileNode>) {
    writable dragging: Option<NodeId> = None

    column {
        repeat on tree.children(tree.root()) as node_id {
            DraggableNode(tree, node_id, dragging)
        }
    }
}

fragment DraggableNode(
    tree: Tree<FileNode>,
    node_id: NodeId,
    dragging: Option<NodeId>
) {
    decl node = tree.get(node_id).unwrap()

    row {
        text { node.name }
    }
    .. draggable { true }
    .. on_drag_start { dragging = Some(node_id) }
    .. on_drag_end { dragging = None }
    .. on_drop |dropped_id: NodeId| {
        if let Some(drag_id) = dragging {
            tree.move_node(drag_id, dropped_id)
        }
    }
}
```

## Complex Examples

### File System with Operations

```frel
scheme FileNode {
    name { String } .. blank { false }
    size { u64 } .. optional { true }
    is_folder { bool }
    created { Instant } .. default { Instant::now() }
}

fragment FileManager() {
    writable fs = Tree::new(FileNode {
        name: "root",
        size: None,
        is_folder: true,
        created: Instant::now()
    })

    writable selected: Option<NodeId> = None
    writable clipboard: Option<NodeId> = None

    column {
        // Toolbar
        row {
            button { "New Folder" }
                .. enabled { selected.is_some() }
                .. on_click {
                    if let Some(parent_id) = selected {
                        let parent = fs.get(parent_id).unwrap()
                        if parent.is_folder {
                            fs.insert(parent_id, FileNode {
                                name: "New Folder",
                                size: None,
                                is_folder: true,
                                created: Instant::now()
                            })
                        }
                    }
                }

            button { "Delete" }
                .. enabled { selected.is_some() && selected != Some(fs.root()) }
                .. on_click {
                    if let Some(node_id) = selected {
                        fs.remove(node_id)
                        selected = None
                    }
                }

            button { "Cut" }
                .. enabled { selected.is_some() }
                .. on_click { clipboard = selected }

            button { "Paste" }
                .. enabled { clipboard.is_some() && selected.is_some() }
                .. on_click {
                    if let (Some(cut_id), Some(dest_id)) = (clipboard, selected) {
                        if fs.get(dest_id).unwrap().is_folder {
                            fs.move_node(cut_id, dest_id)
                            clipboard = None
                        }
                    }
                }
        }

        // Tree view
        FileTreeView(fs, fs.root(), selected)

        // Details panel
        when selected.is_some() {
            decl node = fs.get(selected.unwrap()).unwrap()
            decl path = fs.path(selected.unwrap())
                .iter()
                .map(|id| fs.get(*id).unwrap().name.clone())
                .collect::<Vec<_>>()
                .join("/")

            column {
                text { "Name: ${node.name}" }
                text { "Path: ${path}" }
                text { "Created: ${node.created}" }
                when !node.is_folder && node.size.is_some() {
                    text { "Size: ${node.size.unwrap()} bytes" }
                }
            }
        }
    }
}
```

### Organizational Chart

```frel
scheme Employee {
    name { String } .. blank { false }
    title { String }
    email { String }
    avatar_url { String } .. optional { true }
}

fragment OrgChart() {
    writable org = Tree::new(Employee {
        name: "CEO",
        title: "Chief Executive Officer",
        email: "ceo@company.com",
        avatar_url: None
    })

    // Add some employees
    let ceo_id = org.root()
    let cto_id = org.insert(ceo_id, Employee {
        name: "CTO",
        title: "Chief Technology Officer",
        email: "cto@company.com",
        avatar_url: None
    })

    column {
        EmployeeCard(org, org.root())
    }
}

fragment EmployeeCard(tree: Tree<Employee>, emp_id: NodeId) {
    decl emp = tree.get(emp_id).unwrap()
    decl reports = tree.children(emp_id)
    decl report_count = reports.len()

    column {
        // Employee card
        row {
            when emp.avatar_url.is_some() {
                image { emp.avatar_url.unwrap() }
                    .. width { 50 } .. height { 50 }
                    .. corner_radius { 25 }
            }

            column {
                text { emp.name } .. font { weight: 700 size: 16 }
                text { emp.title } .. font { size: 14 }
                text { emp.email } .. font { size: 12 color: Gray }
            }
        }
        .. padding { 12 }
        .. border { color: LightGray width: 1 }
        .. corner_radius { 8 }

        // Direct reports
        when report_count > 0 {
            column {
                gap { 8 }
                indent { 40 }

                text { "${report_count} direct report${if report_count > 1 { 's' } else { '' }}" }
                    .. font { size: 12 color: Gray }

                repeat on reports as report_id {
                    EmployeeCard(tree, report_id)
                }
            }
        }
    }
}
```

### Category Taxonomy

```frel
scheme Category {
    name { String } .. blank { false }
    description { String } .. optional { true }
    product_count { u32 } .. default { 0 }
}

fragment CategoryBrowser() {
    writable categories = Tree::new(Category {
        name: "All Products",
        description: Some("Root category"),
        product_count: 0
    })

    writable current_path: List<NodeId> = List::new()

    decl current_node = if current_path.is_empty() {
        categories.root()
    } else {
        current_path.last().unwrap()
    }

    decl category = categories.get(current_node).unwrap()
    decl children = categories.children(current_node)

    column {
        // Breadcrumb
        row {
            gap { 8 }

            button { "Home" }
                .. on_click { current_path = List::new() }

            repeat on current_path as node_id {
                decl cat = categories.get(node_id).unwrap()
                text { " / " }
                button { cat.name }
                    .. on_click {
                        // Navigate to this category
                        let index = current_path.iter().position(|id| id == node_id).unwrap()
                        current_path = current_path[..=index].to_vec()
                    }
            }
        }

        // Current category
        column {
            text { category.name } .. font { size: 24 weight: 700 }
            when category.description.is_some() {
                text { category.description.unwrap() } .. font { size: 14 color: Gray }
            }
            text { "${category.product_count} products" } .. font { size: 12 }
        }

        // Subcategories
        column {
            gap { 12 }

            repeat on children as child_id {
                decl child = categories.get(child_id).unwrap()

                row {
                    column {
                        text { child.name } .. font { size: 16 weight: 600 }
                        text { "${child.product_count} products" } .. font { size: 12 }
                    }
                    .. on_click {
                        current_path.push(child_id)
                    }
                }
                .. padding { 12 }
                .. border { color: LightGray width: 1 }
                .. corner_radius { 4 }
            }
        }
    }
}
```

## Performance Considerations

### Flat Storage Benefits

The flat HashMap-based storage provides several advantages:

1. **Constant-time access**: O(1) lookup by NodeId
2. **Efficient updates**: Updating one node doesn't require traversing tree
3. **Fine-grained reactivity**: Only affected nodes trigger notifications
4. **Backend alignment**: Easy to sync with database queries by ID

### Optimization Tips

```frel
// Good - traverse once and cache
decl all_nodes = tree.traverse_preorder(root)
repeat on all_nodes as node_id {
    render_node(tree.get(node_id).unwrap())
}

// Avoid - repeated traversals
repeat on tree.children(parent) as child {
    // This traverses children list each time
    if tree.children(child).is_empty() {
        // ...
    }
}

// Better - cache structure queries
decl is_leaf = tree.children(node_id).is_empty()
```

## Best Practices

### 1. Use NodeId consistently

```frel
// Good - pass NodeIds
fragment TreeNode(tree: Tree<T>, node_id: NodeId) { }

// Avoid - extracting and passing values
fragment TreeNode(tree: Tree<T>, value: T) { }  // Loses tree context
```

### 2. Batch operations

```frel
// Good - batch related operations
button { "Move All" } .. on_click {
    for node_id in nodes_to_move {
        tree.move_node(node_id, new_parent)
    }
    // Single reactivity update after all moves
}
```

### 3. Cache expensive queries

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

### 4. Handle errors gracefully

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

| Frel Type | Rust | TypeScript | Python |
|-----------|------|------------|--------|
| `Tree<T>` | Custom `Tree<T>` | Custom `Tree<T>` | Custom `Tree[T]` |
| `NodeId` | `usize` or `u64` | `number` | `int` |

Implementation details vary by platform but maintain same API surface.
