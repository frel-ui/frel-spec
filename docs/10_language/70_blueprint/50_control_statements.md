# Control statements

The DSL provides three core control statements that allow conditional rendering,
iteration, and branching logic within blueprint definitions. These statements integrate
with the reactive store system, ensuring that the UI automatically reacts to state changes.

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

* Evaluates a **pure**, side-effect-free `<bool-expr>`.
* Compiles to a **derived store** that re-evaluates reactively.
* Switches between branches (`when` / `else`) as the condition changes.
* Cleans up the inactive branch automatically.

### Notes

* `<bool-expr>` must be a Rust-subset expression returning `bool`.
* Re-evaluation occurs once per drain cycle for consistency.

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

### Semantics

* Subscribes to all stores in the expressions.
* Evaluates branches in order; **first match wins**.
* When no `else` is present and none of the conditions match: emits an anchor node.

## `<bool-expr>` Definition

**Goal:** Seamless Rust integration while preserving reactivity and purity.

**Allowed constructs:**

* Literals, identifiers (stores/locals), field access.
* Method calls on immutable data.
* Boolean operators: `! && || == != < <= > >=`
* Option helpers: `.is_some()`, `.is_none()`, `.map(...)`.
* Collection helpers: `.is_empty()`, `.len()`, `.contains(&x)`.
* No side effects (no mutation or impure calls).

**Reactive behavior:**

* Compiles to a derived store tracking dependencies automatically.
* Re-evaluated during each runtime drain cycle.

**Example:**

```frel
when user.is_some() && !notifications.is_empty() {
  show_notifications()
}
```