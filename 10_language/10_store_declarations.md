# Store declarations

Store declarations define named reactive variables that participate in dependency tracking and
notification propagation. Each store kind specifies ownership, mutability, and reactivity behavior.

## Read-only

`decl <id> [:<type>]? = <expr>`
 
- **Kind:** subscribes to all stores used in `<expr>`, const if no other stores are used.
- **Initializer**: `<expr>` must be a PHLE.
- **Writes:** not assignable.
- **Updates:** recomputed when any dep changes (glitch-free; one recompute per drain).
- **Guards:** graphs must be acyclic; cycles are a runtime error (drain notifications cycle limit).

> [!NOTE]
>
> From the DSL perspective, read-only stores with and without dependencies are the same, the later
> is just a specific case where the dependency set is empty. I think this is a clear mental model,
> the important thing about these stores is that they are read-only.
> 
> From an implementation perspective, stores with no dependencies can be optimized while stores with
> dependencies need some bookkeeping, subscriptions, and notification propagation. However, that is
> purely an implementation detail, not a DSL concern.
> 
> I think adding a second keyword to explicitly differentiate is more confusing than helpful.
>

## Writable stores

`[<lifetime>] writable <id> [: <key_expr>]? [:<type>]? = <expr>`

- **Kind:** Writable stores hold mutable state with no automatic subscriptions to other stores.
- **Initializer:** `<expr>` evaluated once at store creation (even if it mentions stores, there's no subscription), must be a PHLE.
- **Writes:** `<id> = <expr2>` allowed in event handlers at any time, `<expr2>` must be a PHLE.
- **Updates:** only by direct assignment (no automatic recomputation).
- **Reactivity:** dependent stores are notified when the value changes.
- **Persistence:** persistence is implemented by the adapter of the host platform.

`<lifetime>` is one of:
- (omitted) — fragment-scoped (default)
- `session` — survives fragment destruction, cleared on app restart
- `persistent` — survives app restart

`<key_expr>` (Key Expression) is:
- Required for `session` and `persistent` stores
- String expression (literal or interpolated) that uniquely identifies the store
- Must be a PHLE, evaluated once at store creation time

**Shorthands:**

| Shorthand                             | Full form                                    |
|---------------------------------------|----------------------------------------------|
| `writable <id> = <expr>`              | `writable <id> = <expr>` (fragment-scoped)   |
| `session <id>: <key> = <expr>`        | `session writable <id>: <key> = <expr>`      |
| `persistent <id>: <key> = <expr>`     | `persistent writable <id>: <key> = <expr>`   |

### Lifetime scopes

| Lifetime     | Survives                   | Storage           | Key required |
|--------------|----------------------------|-------------------|--------------|
| (default)    | Fragment instance only     | Memory            | No           |
| `session`    | Fragment destroy           | Session registry  | Yes          |
| `persistent` | App restart                | Platform storage  | Yes          |

### Key expressions

**Requirements:**
- Must be unique across all stores with the same lifetime in the app
- For `persistent` stores, keys must be stable across app restarts
- Use stable identifiers (user IDs, document IDs) not transient values (indices, positions)

**Examples:**
```frel
session writable split_pos: "app.split" = 300
session writable filter: "UserTable.${table_id}.filter" = ""
persistent writable theme: "app.theme" = "dark"
persistent writable settings: "user.${user_id}.settings" = default_settings()
```

### Type constraints

- Default and `session` lifetime: any type
- `persistent` lifetime: type must be serializable (implement appropriate traits for the host platform)

### Reusable fragments pattern

When the same fragment is instantiated multiple times, pass a scope parameter:

```frel
fragment UserTable(users: Vec<User>, scope: String) {
    session writable filter: "${scope}.filter" = ""
    session writable sort_column: "${scope}.sort" = "name"
    // ...
}

fragment DocumentEditor(doc: Document, scope: String) {
    persistent writable font_size: "${scope}.fontSize" = 14
    persistent writable zoom: "${scope}.zoom" = 1.0
    // ...
}

fragment App() {
    column {
        UserTable(active_users, scope: "active")
        UserTable(archived_users, scope: "archived")
    }
}
```

## Multi-input

`fanin <id> [:<type>]? = <calc_expr> [with <reducer>]`

- **Kind:** writable state subscribed to all stores read by `<calc_expr>`.
- **Calculation:** `<calc_expr>` is re-evaluated when deps change to produce an input value, must be a PHLE.
- **Reducer:** combines current state and inputs into the next state.
- **Default reducer:** replace(state, input) = input (i.e., mirror deps).
- **Custom reducer:** user supplies a closure: |state, input| → state.
- **Writes:** `<id> = <expr2>` is allowed and simply changes the current state; future dep changes keep applying the reducer on top of that; must be a PHLE.
- **Order/consistency:** per drain cycle, `<calc_expr>` is evaluated once after dependencies settle; reducer is applied once (no per-dep glitches).
- **Side effects:** Reducers should be pure; side effects belong to event handlers or sources.
- **Built-in reducers:**
  - `replace` (default): `(_, input) -> input`
  - `append` : `(vec, item) -> { vec.push(item); vec }`
  - `union` : `(set, items) -> set ∪ items`
  - `max_by`, `min_by`
  - `coalesce` : `(state, input_opt) -> input_opt.unwrap_or(state)`

## Data source

`source <id> [:<type>]? = <producer>(…options…)`

- **Kind:** producing store managed by the runtime (effectful). Not writable from fragments.
- **Views for expressions:**
  - `<id>.latest() → Option<T>`: most recent item (if any).
  - `<id>.status() → Status<E> = { Loading | Ready | Error(E) }`
- **As a dependency:** may feed fanin directly (events are the inputs).
- **Typical producers:** interval(ms = 1000), fetch(|| …), sse(url, event = "…").

Lifecycle:

- sources are created eagerly when the fragment is created
- sources are dropped when the fragment is dropped
- sources may start async operations in the background, but that is outside the scope of the DSL
- retry, cancellation, and error handling are all up to source, the DSL only cares about
  - having a status
  - having an optional value, `None` when status is `Loading` or `Error`

>
> [!NOTE]
>
> Source status and value at initialization are **not** specified as it depends on the actual
> implementation. Some sources, especially shared ones such as tick or counters, may have a
> ready-to-use value at initialization, these start with `Ready` (or `Error`). Some others
> may need time to get the value, these start with `Loading`.
> 

## Syntax examples

```frel
// decl — const and derived
decl theme = "light"                          // const
decl total = items.map(|i| i.price).sum()     // derived (reads `items`)

// writable — manual state (fragment-scoped)
writable page = 0
on_click { page = page + 1 }

// session — survives fragment recreation (session-scoped)
session split_pos: "app.split" = 300          // shorthand
session sidebar_width: "app.sidebar" = 250     // shorthand
on_drag { split_pos = new_position }

// persistent — survives app restart (storage-backed)
persistent user_theme: "app.theme" = "dark"    // shorthand
persistent recent_files: "app.recent" = vec![] // shorthand
on_select { user_theme = "light" }

// fanin — mirror (default reducer = replace)
fanin selection = external.selection           // mirrors external.selection
// manual override is OK:
on_click { selection = Some(id) }              // next external change will replace again

// fanin — accumulate with a reducer
fanin log = events.latest() with |state: Vec<Event>, input: Event| {
  let mut s = state;
  s.push(input);
  s
}

// fanin — “sticky until next search” list
fanin filtered = items.filter(|i| i.matches(query)) with |state, input| {
  // e.g., dedupe or union policies go here
  input
}

// source — effectful event producers
source tick   = interval(ms = 1000)                 // () every second
source user   = fetch(|| api.user(user_id))         // one-shot (may retry per options)
source updates = sse("/events", event = "update")   // stream of Update

// using source views directly in decls (pure)
decl last_user   = user.latest().unwrap_or_default()
decl load_status = user.status()

// piping sources into state with fanin
fanin current_user = user                 // replace on each emission
fanin timeline     = updates with append  // accumulate all updates
fanin beats        = tick with |n, _| n + 1

// scoped usage inside a loop
repeat on ids as id {
  source row_user = fetch(|| api.user(id))
  decl name = row_user.latest().map(|u| u.name).unwrap_or("…".into())
  text { name }
}
```

## Type inference

Store statements may contain a type. If not specified, the type is inferred from the expression.

As the DSL is compiled into Rust, the type inference is done by the compiler. It will fail when
Rust type inference fails, and the compiler message will provide additional information.
