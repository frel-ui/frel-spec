# Runtime

The runtime orchestrates the execution of all dynamic behavior in the system.

It owns the store subsystem, manages fragments (handles, stores, subscriptions), and coordinates rendering 
through discrete runtime frames.

Each frame represents a complete, deterministic processing step that transforms events 
and state changes into visible output deltas.

## Runtime Frame

The runtime frame is a transactional execution invoked by the adapter 
when events are available for processing.

It runs event handlers, drains store notifications, and generates render patches.

The adapter later applies to produce the final UI, but that is not part of the frame.
The frame begins with an array of events to process and ends with an array of UI patches.

Each frame runs to completion before another begins.
If new events arrive during a frame, they are queued and processed in the next one.

```text
[Adapter]
    ├─ Add events to the event queue  
    ↓
    ┌─── frame start ──
    │     ↓
    │  [Event Handler]
    │     ├─ Primary store mutations (direct changes from event handlers)
    │     ↓
    │  [Store subsystem] 
    │     ├─ Drain notifications (cascading store mutations), add nodes to render queue
    │     ↓
    │  [Fragment Renderer]
    │     ├─ Process the render queue
    │     ├─ Generate UI patches
    │     ↓
    └─── frame end ──
    ↓
[Adapter]
    ├─ Apply UI patches
    └─ Commit to actual UI
    ↓
Output (UI / PDF / PNG / etc.)
```
