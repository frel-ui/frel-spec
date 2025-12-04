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

* Iterates over `<iterable>` which must be a collection or an arena.
* Each iteration produces a child reactive scope.
* Incremental updates are performed via **keyed diffing**:

    * Insert new keys → create new item views
    * Remove missing keys → dispose old item views
    * Moved keys → reorder without re-creation
    * Changed items → no action (reactivity will take care)

### Keys and Identity

* Use `by <key-expr>` to specify item identity.
* If omitted:
  * If the type to iterate over is a scheme with identity, that identity field is used.
  * Othewise index-based diffing is used (reorders become remove+insert).

### Example

```frel
repeat on users by user.id as user {
  row {
    text { user.display_label }
  }
}
```

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
