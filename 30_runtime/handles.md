# Fragment Handles

A fragment handle is the runtime owner of a fragment lifecycle and resources.

A fragment handle contains bookkeeping information about a fragment:

- internal stores
- subscriptions
- children

The handles do not contain any application data or any specific logic, the
information is used to free resources when the fragment is dropped.

```rust
pub type FragmentKey = Index;

trait FragmentRuntime {
    // Methods for allocating/freeing handles and coordinating cleanup
}

struct FragmentRuntimeImpl {
    handles: Arena<FragmentHandle>
}

pub struct FragmentHandle {
    pub id: u32,
    pub desc: &'static FragmentDesc,
    pub internal_stores: smallvec::SmallVec<[StoreKey; 8]>, // used for cleanup when the handle is dropped
    pub subscriptions: smallvec::SmallVec<[SubscriptionKey; 8]>, // used for cleanup when the handle is dropped
    pub children: smallvec::SmallVec<[FragmentHandle; 8]> // used for cleanup when the handle is dropped
}
```
