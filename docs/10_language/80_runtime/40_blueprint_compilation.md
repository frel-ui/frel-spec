# Blueprint compilation

During compilation each blueprint is turned into:

1. a closure scheme
2. a function table
3. an internal binding function
4. call sites
5. anonymous blueprints

## Internal Biding Function

The internal binding function creates the subscriptions that update the field declared in
the closure.

```frel
blueprint A(p1 : u32, p2 : u32) {
    sum : u32 = p1 + p2
}
```

In this case:

1. we have a field named `sum` in the closure
2. we have a subscription on the closure with the selector: `OneOf("p1","p2")`
3. the callback function of the closure (added to the function table):

```javascript
function_table.push( 
    function(subscription) {
        subscription.target_closure.set(
            "sum",
            subscription.source_closure.get("p1") + subscription.source_closure.get("p2")
        )
    }
)
```

The role of the internal binding function is to create the subscription for a
specific closure during blueprint instantiation.