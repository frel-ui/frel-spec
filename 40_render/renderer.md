# Fragment Renderer

## Definitions

**Actual UI**: The platform-dependent UI which is actually shown to the user.

**Adapter**: A platform-dependent adapter that is able to create/remove/update nodes in the actual UI.

**Abstract Render Tree (ART)**: Runtime representation of a scene. A tree of nodes that contains all information needed to render the actual UI.

**Renderable Fragment**: A fragment that emits nodes to the ART.

**Node**: A part of the ART that can be added/removed/updated.

[**Container**](../10_language/70_blueprint/30_standard_blueprints.md#containers) : A node that can contain other nodes.

**Positional Container** : A container in which the children are placed at pre-defined positions.

**Algorithmic Container** : A container in which the children are placed by a layout algorithm.

[**Primitive**](primitives.md): Wrapper around an actual UI element instance (DOM node, View instance, UIView instance).

**DIP**: Device Independent Pixel

## Render flow

```text
[Event Handler]
   ├─ Primary store mutations (direct changes from event handlers)
   ↓
[Store runtime] 
   ├─ Drain notifications (cascading store mutations), add nodes to render queue
   ↓
[Instruction Applier]
   ├─ Apply instructions → update supplied render data
   ├─ Determine invalidation type (layout / paint)
   ├─ Generate paint patches
   ↓
[Layout Engine]
   ├─ Measure intrinsic sizes
   ├─ Compute layout → update derived render data (positions, bounds, line boxes, etc.)
   ├─ Generate position patches
   ↓
[Adapter]
   ├─ Apply position & paint patches in batch (DIP → px)
   └─ Commit to actual UI
   ↓
Output (UI / PDF / PNG / etc.)
```

## Render data

Each node has a render data structure which consists of two major parts:

- supplied render data:
    - styling, alignment, layout algorithm
- derived render data:
    - actual positions

### Supplied render data

Each renderable fragment has an external store called `instructions`. The
 **Fragment Compiler** initializes this store from the instructions present in
the DSL.

Renderable fragments subscribe to this store, and whenever the value of the 
store changes, they update the supplied render data of their node according
to the instructions.

Instructions which handle dimensional data **always** store the value in DIP
(Device Independent Pixel).

```rust
struct DIP(f32); // device independent pixel

struct Surrounding {
    top : DIP,
    right : DIP,
    left : DIP,
    bottom: DIP
}

struct Color {
    red : u8,
    green : u8,
    blue : u8,
    opacity : u8
}

enum Instruction {
    Padding(Surrounding),
    Background(Color)
}

struct RenderData {
    dirty_mask : u64, // 1 when the field has been changed since the last render
    padding: Surrounding,
    background_color : Color
}
```