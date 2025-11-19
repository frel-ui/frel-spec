# Control statements

The DSL provides three core control statements that allow conditional rendering,
iteration, and branching logic within blueprint definitions. These statements integrate
with the reactive system, ensuring that the UI automatically reacts to state changes.

| Statement  | Purpose                             | Typical Use                            |
|------------|-------------------------------------|----------------------------------------|
| **when**   | Conditional single-branch rendering | Show/hide content based on a condition |
| **repeat** | Iterative rendering over lists      | Render dynamic collections or tables   |
| **select** | Multi-branch conditional rendering  | Choose one layout among several states |

## `when` Statement

**Syntax:**

```frel
when <bool-expr> <statement>
[else <statement>]
```

### Semantics

* Evaluates a **pure**, side-effect-free `<bool-expr>` (reactively).
* Switches between branches (`when` / `else`) as the condition changes.
* Cleans up the inactive branch automatically.

## `repeat` Statement

**Syntax:**

```frel
repeat on <iterable> [as <item>] [by <key-expr>] <statement>
```

### Semantics

* Iterates over `<iterable>` which must resolve to something implementing `IntoIterator<Item = T>`.
* Each iteration produces a child reactive scope.
* Incremental updates are performed via **keyed diffing**:

    * Insert new keys → create new item views
    * Remove missing keys → dispose old item views
    * Moved keys → reorder without re-creation
    * Changed items → update via `PartialEq` or `changed(&old, &new)`

### Keys and Identity

* Use `by <key-expr>` to specify item identity.
* `<key-expr>` must yield a `Key: Eq + Hash + Clone`.
* If omitted:

    * Use `item.key()` if the item implements the `Keyed` trait.
    * Otherwise, index-based diffing is used (reorders become remove+insert).

**Trait support:**

```rust
pub trait Keyed {
    type Key: Eq + std::hash::Hash + Clone;
    fn key(&self) -> Self::Key;
}
```

### Iteration Locals

Each iteration provides read-only locals for convenience:

* `_index` – current index (0-based)
* `_first` – true if first iteration
* `_last` – true if last iteration

### Example

```frel
repeat on users by user.id as user {
  row {
    text { "#${_index + 1} ${user.name}" }
  }
}
```

### `<iterable>` Definition

* Any reactive expression returning a container implementing `IntoIterator`.
* Common examples: `Vec<T>`, `&[T]`, `Arc<[T]>`, `BTreeMap<K, V>`.
* For streaming data, connect to a `fanin` maintaining a reactive `Vec<T>`.

## `select` Statement

**Syntax (boolean guards):**

```frel
select {
  <bool-expr> => <statement>
  [<bool-expr> => <statement>]*
  [else => <statement>]
}
```

**Syntax (enum-based sugar):**

```frel
select on <enum-expr> {
  VariantA(x) => { ... }
  VariantB { id, .. } => { ... }
  else => { ... }
}
```