# Runtime Closures

Runtime closures are created dynamically as fragments are instantiated from blueprints.
Unlike [scopes](../10_organization/30_scope.md) which exist at compile time, runtime closures exist
during program execution and bind names to actual reactive data.

## Overview

When a blueprint is instantiated, the Frel runtime creates a **fragment** with its own **fragment
closure**. 

Blueprint:

```frel
blueprint Outer {
    Middle("Noname", UserProfile {  })
}

blueprint Middle(
    p1 : String,
    p2 : UserProfile
) {
    a : u32 = 42
    b : Avatar = Avatar {  }
    
    Inner(p1, p2.account_name.is_not_empty)
}

blueprint Inner(
    p1 : String,
    p2 : bool
) {
    text { "${p1} : ${p2}" }
}
```

Closure:

```javascript
datum[5001] = {
    type : "UserProfile",
    identity_id: 5001,
    // composite datum fields would be here (omitted for clarity)
}

closures[1001] = {
    blueprint : "Outer",
    closure_id: 1001,
    parent_closure_id: null,
    child_closure_ids : [ 1002 ],
    fields: {
        "<c123p1>" : "Noname", // synthetic field for descendants
        "<c123p2>" : 5001 // synthetic field for descendants
    },
    subscriptions_to_this : [ 4001, 4002 ],
    subscriptions_by_this : []
}

subscriptions[4001] = {
    subscription_id: 4001,
    source_id: 1001,
    target_id: 1002,
    callback_id: 3001,
    selector: Key("<c123p1>")
}

callbacks[3001] = function (subscription) {
    closures[subscription.target_id].set("p1", closures[subscription.source_id].get("<c123p1>"))
}

subscriptions[4002] = {
    subscription_id: 4002,
    source_id: 1001,
    target_id: 1002,
    callback_id: 3002,
    selector: Key("<c123p2>")
}

callbacks[3002] = function (subscription) {
    closures[subscription.target_id].set("p2", closures[subscription.source_id].get("<c123p2>"))
}

datum[5002] = {
    type : "Avatar",
    identity_id: 5002,
    // composite datum fields would be here (omitted for clarity)
}

closures[1002] = {
    blueprint: "Middle",
    closure_id: 1002,
    parent_closure_id: 1001,
    child_closure_ids : [ 1003 ],
    fields : {
        p1 : "Noname",
        p2 : 5001,
        a: 42,
        b: 5002
    },
    subscriptions_to_this : [ 4003, 4004 ],
    subscriptions_by_this : [ 4001, 4002 ] 
}

subscriptions[4003] = {
    subscription_id: 4003,
    source_id: 1002,
    target_id: 1003,
    callback_id: 3003,
    selector: Key("p1")
}

callbacks[3003] = function (subscription) {
    closures[subscription.target_id].set("p1", closures[subscription.source_id].get("p1"))
}

subscriptions[4004] = {
    subscription_id: 4004,
    source_id: 1002,
    target_id: 1003,
    callback_id: 3004,
    selector: Key("p2")
}

callbacks[3004] = function (subscription) {
    closures[subscription.target_id].set("p2", closures[subscription.source_id].get("p2").account_name.is_not_empty)
}

closures[1003] = {
    blueprint: "Inner",
    closure_id: 1003,
    parent_closure_id: 1002,
    child_closure_ids : [ ],
    fields : {
        p1 : "Noname",
        p2 : true
    },
    subscriptions_to_this : [ 4005 ], // `text` subscribes
    subscriptions_by_this: [ 4003, 4004 ]
}

subscriptions[4005] = {
    subscription_id: 4005,
    source_id: 1003,
    target_id: 1004,
    callback_id: 3005,
    selector: OneOf("p1", "p2")
}

callbacks[3005] = function (subscription) {
    closures[subscription.target_id].set("text", closures[subscription.source_id].get("p1") + " : " + closures[subscription.source_id].get("p2"))
}
```