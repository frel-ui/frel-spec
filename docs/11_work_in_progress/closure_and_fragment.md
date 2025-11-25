
**IMPORTANT**: Do not use this file, it contains obsolated information.

## 4. Fragment Structure

Fragments form a parent-child hierarchy, each representing an instantiated blueprint.

```json
{
    "fragment_id": 100,
    "blueprint_name": "UserCard",
    "parent_fragment_id": 50,
    "child_fragment_ids": [101, 102, 103],
    "closure_id": 1000
}
```

## 5. Closure Storage

Each fragment has a closure that binds names to values (intrinsic) or identityIds (composite).

```json
{
    "closure_id": 1000,
    "fragment_id": 100,
    "blueprint_name": "UserCard",
    "closure": {
        "user_id": 100,
        "display_name": "Alice",
        "user": 2001,
        "$anon_0": 42,
        "count": 0,
        "doubled": 0
    },
    "parent_closure": null
}
```