# Blueprint compilation

During compilation each blueprint is turned into:

1. a closure scheme
2. a function table
3. an internal binding function
4. call sites
5. anonymous blueprints

## Closure Scheme

Each parameter and each declared field in the blueprint is added to the closure scheme.

```frel
blueprint A(p1 : u32, p2 : u32) {
    sum : u32 = p1 + p2
    B(sum)
}
```

The closure scheme is:

```frel
scheme A$Closure {
    p1 : u32
    p2 : u32
    sum : u32
}
```

## Function Table

The function table contains:

- the internal binding function
- call site binding functions
- subscription callback functions

The internal binding function and the call site binding functions bind the entries of the table to
subscriptions.

```javascript
// updates `A.sum` when `A.p1` or `A.p2` changes
function A$sum$subscription_callback(runtime, subscription) {
    runtime.set(subscription, "sum", runtime.get(subscription, "p1") + runtime.get(subscription, "p2"))
}

// binds `A.sum` to `A.p1` and `A.p2` through the subscription callback
function A$internal_binding(runtime, closure) {
    runtime.subscribe(closure, closure, OneOf("p1","p2"), A$sum$subscription_callback)
}

// sets `B.p1` from `A.sum` at call site `123`
function A$123$p1$subscription_callback(runtime, subscription) {
    runtime.set(subscription, "p1", runtime.get(subscription, "sum"))
}

// binds `B.p1` to `A.sum` through the subscription callback at call site `123`
function A$123$call_site_binding(runtime, source_closure, target_closure) {
    runtime.subscribe(source_closure, target_closure, Key("sum"), A$123$p1$subscription_callback)
}
```

## Internal Binding Function


The internal binding function takes the closure and creates the subscriptions
which are internal to the closure. This function is called during blueprint instantiation.

```frel
blueprint A(p1 : u32, p2 : u32) {
    sum : u32 = p1 + p2
}
```

In this case:

1. we have a field named `sum` in the closure
2. we have a subscription on the closure with the selector: `OneOf("p1","p2")`
3. we have a callback function that updates the field `sum` on notification

## Call sites

A call site consists of:

- the blueprint to build
- the call site binding function
- call site subscription callback functions

The call site binding function creates the subscriptions between the closure of the
declaring fragment and the closure of the declared fragment. It is called
during blueprint instantiation.

