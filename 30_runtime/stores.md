# Store subsystem

1. The application state is represented by a (possibly high) number of stores.
2. `Store` = the sole source of truth.
    1. `Readable`: `get_any() : &dyn Any`, `subscribe(id)`, `unsubscribe(id)`
    2. `Writable`: adds `set_any(&dyn Any)`
3. Change propagation is push: when a Writable changes, it notifies subscribers.
4. Value access is pull: subscribers decide when to call `get()` (might be immediate, but could be deferred to the render/compute phase).

Core tenets:

- **Single abstraction**: `Store`.
- **One promise**: “You’ll be notified when the truth might have changed.”
- **One action**: “Call `get()` when you care.”

Notes: 

- One single thread manages the state of the application, no multi-thread synchronization is needed.
- Stores use `dyn Any` to avoid type constraints, the macro should ensure type safety and generate 
  downcast when necessary
- The store subsystem is mainly used from generated code. While it has to be consistent and 
  error-free, the code generation will offer guarantees. This means that fail paths do not have to 
  be developer-friendly; they are supposed not to be reached anyway.

Each fragment has two store sets:

- **External stores**: passed to the fragment
- **Internal stores**: declared and owned by the fragment

## Store types

| Kind       | Emits | Can write? | Who owns lifetime? | Typical use                    |
|------------|-------|------------|--------------------|--------------------------------|
| `Readable` | Yes   | No         | Declaring owner    | One-way data flow to child     |
| `Writable` | Yes   | Yes        | Declaring owner    | Two-way models, internal state |
| `Const`    | No    | No         | Creator            | Literals, config flags         |
| `Derived`  | Yes   | No         | Creator            | Computed from other stores     |

Note: Both `Readable` and `Writable` stores are implemented by `EmittingStore`. The difference
is semantic at the store subsystem level. The macro keeps track of store types and refuses code
that would write into a readable store.

## Store runtime

The store runtime:

- owns stores, provides `StoreKey` for referencing stores
- owns subscriptions, provides `SubscriptionKey` for referencing subscriptions
- provides functions to add/remove stores, subscribe/unsubscribe stores
- provides `set_value` function to change values in the stores with delayed notification
    - `set_value` queues the subscriptions to be notified
    - `drain_notifications` call the callbacks to deliver the notifications

## Store value change

In general, store value changes result in notifications. This is not desired when
the `set_any` is called with the same value as the current one.

As stores are type erased, comparison of the current and the new value is not trivial.

Emitting stores may support this optimization by having a comparison function `eq_fn`. This
function may be passed to the store during creation, so it can call it whenever `set_any`
is called and skip the change if the values are the same.

Code generation can use the `mk_eq_fn` function to create the `eq_fn` function for
any type that supports `PartialEq`.

## Notification mechanism

Application state is independent of rendering; reactivity is handled by stores calling the
callback function provided at subscription time.

- Callback calls **DO NOT** preserve the order of subscriptions.
- Callbacks are processed in batches called *generations*.
- `drain_notifications` works in a loop where each cycle performs the following steps.
    1. Sanity check, the loop should finish in less than or equal `GEN_LIMIT` cycles (1000 by default).
    2. Take all pending notifications from the pending queue: the **processed generation**.
    3. Empty the pending queue and increase the generation counter,
    4. Call callback functions of the **processed generation**.
        1. May change store values, those changes belong to the **next generation**.
        2. May queue a re-render (which is processed outside the store subsystem).
    5. Stop the cycle if the **next generation** has no notifications.

This ensures that all pending work is finished and all cascading changes are applied to
the appropriate stores.

Note: the `drain_notifications` function is not reentrant, it panics if called from within
a callback, this is by design.

`set_value` function of stores save the current generation of the runtime (`last_set_gen` 
field of the store when the store is writable).

### Subscribe during drain

Subscription during drain (which is almost all subscriptions in practice) has the
following behaviour.

If `last_set_gen` of the subscribed store is the current generation of the runtime, the
runtime adds the subscription to the pending queue. This ensures that the subscriber
will be notified of store changes as well.

If `last_set_gen` is not the current generation of the runtime, the subscription is **NOT**
added to the pending queue. If some callbacks change the store value later during the drain,
the subscription will be automatically added to the pending queue by the normal mechanism.

### Unsubscribe during drain

Unsubscribe during drain (also almost all cases in practice) simply removes the subscription
from the subscriptions managed by the runtime.

If the subscription key is reached later during the drain, the function won't be able to
fetch the `StoreSubscription` as it has been already removed. In this case it simply skips
the key and goes on.

### Derived store subscription

Derived stores subscribe to other stores. The linker performs this subscription when
fragments are linked.

When a derived store is removed, the runtime automatically removes all subscriptions to
other stores.

Derived store logic is implemented in the callback function provided to the stores the
derived store subscribes to. In most cases the macro generates this function.

## Store types and traits

```rust
use thunderdome::{Index,Arena};

pub type StoreKey = Index;
pub type SubscriptionKey = Index;
pub type StoreCallback = Rc<dyn Fn(StoreKey, SubscriptionKey, &mut StoreEffects)>;

pub struct StoreSubscription {
    store: StoreKey,
    callback: StoreCallback
}

pub trait StoreRuntime {
    /// Allocate a new store; returns a stable handle.
    fn alloc_store(&mut self, s: Box<dyn Store>) -> StoreKey;

    /// Remove a store.
    fn free_store(&mut self, key: StoreKey);

    /// Subscribe to a store.
    fn subscribe(&mut self, key: StoreKey, cb: StoreCallback) -> SubscriptionKey;

    /// Unsubscribe from a store.
    fn unsubscribe(&mut self, key: SubscriptionKey) -> bool;

    /// Erased write. Store appends subs into the sink; we enqueue those notifications
    /// into `pending`.
    fn set_value(&mut self, store_key: StoreKey, value: Box<dyn Any>);

    /// Drain the pending notifications and invoke callbacks.
    fn drain_notifications(&mut self);
}

pub trait Store {
    /// Untyped read; macro-generated code downcasts at use-sites.
    fn get_any(&self) -> &dyn Any;

    /// Untyped write; macro-generated code performs checked downcast.
    fn set_any(&mut self, value: Box<dyn Any>, sink: &mut SubSink);

    /// Register a listener. When returns with true, the store has been changed
    /// in this generation; therefore, the key should be added to the pending queue.
    fn subscribe(&mut self, key: SubscriptionKey, generation : StoreGeneration) -> bool;

    /// Remove a listener by id.
    fn unsubscribe(&mut self, key: SubscriptionKey);
    
    /// Subscriptions of this store. Used for cleanup, it is a life slice, not a snapshot.
    /// (Single-thread design guarantees that it cannot change during cleanup).
    fn subscriptions(&self) -> Option<&[SubscriptionKey]>;

    /// Dependencies of this store. Used for cleanup, it is a life slice, not a snapshot.
    /// (Single-thread design guarantees that it cannot change during cleanup).
    fn dependencies(&self) -> Option<&[SubscriptionKey]>;

    /// (Optional) stable debug/type name for tooling/metrics.
    fn debug_name(&self) -> &'static str { std::any::type_name::<Self>() }
}
```

## Data sources

Data sources are stores that produce values from outside sources such as timers, network,
etc.

## Planned improvements

- Support for partial store subscriptions.
