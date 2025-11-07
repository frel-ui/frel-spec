# Sources

Sources are special stores that produce values asynchronously from external systems - like 
timers, network requests, or event streams. They're managed by the runtime and represent 
effectful operations that happen outside the fragment's control.

## Syntax

`source <id> [:<type>]? = <producer>(…options…) [.. on_value { val: T -> <handler> }]`

## Semantics

- **Kind**: Producing store managed by the runtime (effectful). Not writable from fragments.
- **Status**: All sources have a status accessible via `<id>.status() → FrelStatus`
- **Value access**: Source values are accessed directly (e.g., `user.name`), not via `.latest()`
- **Status states**: `Loading` (fetching), `Ready` (data available), `Error(FrelError)` (failed)
- **Event handler**: `on_value` runs each time the source produces a new value. The handler receives the unwrapped value (not `Option<T>`)
- **As a dependency**: Sources can be read by other stores, triggering reactive updates. Status propagates automatically.
- **Typical producers**: `interval(ms)`, `fetch(|| …)`, `sse(url, event)`

## Lifecycle

- Sources are created eagerly when the fragment is created
- Sources are dropped when the fragment is dropped
- Sources may start async operations in the background, but that is outside the scope of the DSL
- Retry, cancellation, and error handling are all up to the source implementation
- The DSL only cares about:
  - Having a status (`Loading`, `Ready`, `Error`)
  - Having an optional value (`None` when status is `Loading` or `Error`)

> [!NOTE]
>
> Source status and value at initialization are **not** specified as it depends on the actual
> implementation. Some sources, especially shared ones such as tick or counters, may have a
> ready-to-use value at initialization, these start with `Ready` (or `Error`). Some others
> may need time to get the value, these start with `Loading`.

## Status Type

```frel
enum FrelStatus {
    Loading            // Source is fetching data
    Ready              // Data is available
    Error(FrelError)   // An error occurred
}
```

See [Data Basics](../10_data_modeling/10_data_basics.md#status-and-error-types) for details on `FrelError` and `FrelStatus`.

## Examples

### Using on_value Handler

The `on_value` handler is the recommended way to handle source events:

```frel
blueprint NotificationBadge() {
    source notifications = sse("/notifications")
    writable unread_count = 0

    // Handler runs each time a notification arrives
    notifications .. on_value { notif: Notification ->
        unread_count = unread_count + 1
        show_toast(notif.message)
    }

    button { "Notifications (${unread_count})" }
        .. on_click { unread_count = 0 }
}
```

### Timer/Interval

Periodic events:

```frel
blueprint Clock() {
    source tick: Instant = interval(ms: 1000)

    decl current_time = tick.format("%H:%M:%S")

    select on tick.status() {
        FrelStatus::Loading => text { "..." }
        FrelStatus::Ready => text { current_time } .. font { size: 32 weight: 700 }
        FrelStatus::Error(_) => text { "Error" }
    }
}
```

### Single Fetch

One-time data fetch:

```frel
blueprint UserProfile(user_id: u32) {
    source user: User = fetch(|| api::get_user(user_id))

    select on user.status() {
        FrelStatus::Loading => column {
            spinner { }
            text { "Loading user..." }
        }

        FrelStatus::Ready => column {
            text { user.name } .. font { size: 20 weight: 700 }
            text { user.email }
            text { "Joined: " + user.created_at }
        }

        FrelStatus::Error(e) => column {
            text { "Error: " + e.message } .. font { color: Red }
            button { "Retry" }
                .. on_click { /* trigger refetch */ }
        }
    }
}
```

### Server-Sent Events (SSE)

Streaming data with accumulation:

```frel
blueprint LiveFeed() {
    source updates = sse(url: "/api/feed", event: "update")
    writable messages: List<Message> = []

    updates .. on_value { msg: Message ->
        // TODO: List append operation not yet specified
        add_message(msg)
    }

    column {
        // TODO: List .len() operation not yet specified
        text { "Live Feed" }

        scroll {
            vertical
            height { 400 }

            column {
                gap { 8 }

                repeat on messages as msg {
                    row {
                        text { "${msg.timestamp}" } .. font { color: Gray }
                        text { msg.content }
                    }
                }
            }
        }

        button { "Clear" }
            .. on_click { messages = [] }
    }
}
```

### Multiple Sources

Combining different data sources:

```frel
blueprint Dashboard() {
    source stats: Stats = fetch(|| api::get_stats())
    source notifications: Notification = sse(url: "/notifications")
    source health: HealthCheck = interval_fetch(|| api::health_check(), interval_ms: 5000)

    writable notif_list: List<Notification> = []

    notifications .. on_value { notif: Notification ->
        // TODO: List append operation not yet specified
        add_notification(notif)
    }

    column {
        gap { 16 }

        // Stats section
        when stats.status() == FrelStatus::Ready {
            StatsCard(stats)
        }

        // Health indicator
        row {
            decl is_healthy = health.is_ok

            icon { if is_healthy { "check_circle" } else { "error" } }
                .. tint { if is_healthy { Green } else { Red } }

            text { if is_healthy { "System OK" } else { "System Error" } }
        }

        // Live notifications
        column {
            text { "Notifications (${notif_list.len()})" }

            repeat on notif_list as notif {
                NotificationCard(notif)
            }
        }
    }
}
```

### Dependent Fetch

Fetch data based on user selection:

```frel
blueprint PostViewer() {
    writable selected_post_id: u32? = null

    // Source that re-fetches when selected_post_id changes
    source post: Post = fetch_when(
        selected_post_id,
        |id| api::get_post(id)
    )

    column {
        // Post list
        repeat on available_posts as p {
            button { p.title }
                .. on_click { selected_post_id = p.id }
        }

        // Post detail
        when selected_post_id != null {
            select on post.status() {
                FrelStatus::Loading => spinner { }

                FrelStatus::Ready => column {
                    text { post.title } .. font { size: 24 weight: 700 }
                    text { post.content }
                }

                FrelStatus::Error(e) => text { "Error: " + e.message } .. font { color: Red }
            }
        }
    }
}
```

### Pagination with Sources

```frel
blueprint PaginatedList() {
    writable page = 0
    writable page_size = 25

    source items: List<Item> = fetch_paginated(
        || api::get_items(page, page_size)
    )

    column {
        when items.status() == FrelStatus::Loading {
            spinner { }
        }

        when items.status() == FrelStatus::Ready {
            repeat on items as item {
                ItemCard(item)
            }
        }

        row {
            button { "Previous" }
                .. enabled { page > 0 && items.status() == FrelStatus::Ready }
                .. on_click { page = page - 1 }

            text { "Page " + (page + 1) }

            button { "Next" }
                .. enabled { items.status() == FrelStatus::Ready }
                .. on_click { page = page + 1 }
        }

        when items.status() == FrelStatus::Error(e) {
            text { "Failed to load: " + e.message } .. font { color: Red }
        }
    }
}
```

### Polling

Repeatedly fetch data at intervals:

```frel
blueprint SystemMonitor() {
    source metrics: Metrics = poll(
        producer: || api::get_metrics(),
        interval_ms: 2000
    )

    column {
        text { "System Metrics" } .. font { size: 20 weight: 700 }

        when metrics.status() == FrelStatus::Ready {
            column {
                gap { 8 }

                row {
                    text { "CPU:" }
                    text { metrics.cpu_percent + "%" }
                        .. font { color: if metrics.cpu_percent > 80 { Red } else { Green } }
                }

                row {
                    text { "Memory:" }
                    text { metrics.memory_mb + " MB" }
                }

                row {
                    text { "Active Connections:" }
                    text { metrics.connections }
                }
            }
        }

        when metrics.status() == FrelStatus::Loading {
            spinner { }
        }
    }
}
```

### WebSocket

Bidirectional communication:

```frel
blueprint ChatRoom(room_id: String) {
    source messages = websocket(url: "/chat/${room_id}")
    writable draft = ""
    writable chat_history: List<Message> = []

    messages .. on_value { msg: Message ->
        // TODO: List append operation not yet specified
        add_chat_message(msg)
    }

    column {
        // Message history
        scroll {
            vertical
            height { 400 }

            column {
                repeat on chat_history as msg {
                    row {
                        text { msg.author } .. font { weight: 700 }
                        text { msg.content }
                    }
                }
            }
        }

        // Input area
        row {
            text_input { draft }
                .. placeholder { "Type a message..." }

            button { "Send" }
                .. enabled { !draft.is_empty() }
                .. on_click {
                    send_message(room_id.clone(), draft.clone())
                    draft = ""
                }
        }

        // Connection status
        text {
            match messages.status() {
                FrelStatus::Loading => "Connecting...",
                FrelStatus::Ready => "Connected",
                FrelStatus::Error(e) => "Disconnected: " + e.message,
            }
        } .. font {
            color: match messages.status() {
                FrelStatus::Loading => Orange,
                FrelStatus::Ready => Green,
                FrelStatus::Error(_) => Red,
            }
        }
    }
}
```

### Cached Fetch

Fetch with caching behavior:

```frel
blueprint CachedData(key: String) {
    source data: Data = fetch_cached(
        key: key,
        producer: || api::get_data(key),
        ttl_seconds: 300  // 5 minute cache
    )

    decl is_cached = data.is_cached()

    column {
        when is_cached {
            text { "📦 From cache" } .. font { color: Gray size: 12 }
        }

        select on data.status() {
            FrelStatus::Loading => spinner { }

            FrelStatus::Ready => DataView(data)

            FrelStatus::Error(e) => text { "Error: " + e.message } .. font { color: Red }
        }
    }
}
```

### Dependent Sources

Source that depends on another source:

```frel
blueprint UserPosts() {
    source user: User = fetch(|| api::get_current_user())

    // This source depends on user being ready
    source posts: List<Post> = fetch_when(
        user,
        |u| api::get_posts(u.id)
    )

    column {
        when user.status() == FrelStatus::Ready {
            text { user.name + "'s Posts" }
        }

        select on posts.status() {
            FrelStatus::Loading => spinner { }

            FrelStatus::Ready => {
                repeat on posts as post {
                    PostCard(post)
                }
            }

            FrelStatus::Error(e) => text { "Failed to load posts: " + e.message }
        }
    }
}
```

## Best Practices

### Handle All Status States

Always handle Loading, Ready, and Error states for one-time fetches:

```frel
select on source.status() {
    FrelStatus::Loading => spinner { }
    FrelStatus::Ready => { /* show data */ }
    FrelStatus::Error(e) => { /* show error */ }
}
```

### Use on_value for Accumulation

Use `on_value` handlers to accumulate source events:

```frel
// Good - explicit accumulation
source items: Item = sse("/items")
writable item_list: List<Item> = []

items .. on_value { item: Item ->
    // TODO: List append operation not yet specified
    add_item(item)
}
```

### Extract Status Checks

For cleaner code, extract status to a derived store:

```frel
source data: Data = fetch(|| api::get_data())

decl is_loading = data.status() == FrelStatus::Loading
decl is_error = matches!(data.status(), FrelStatus::Error(_))
decl is_ready = data.status() == FrelStatus::Ready

when is_loading {
    spinner { }
}
```

### Direct Value Access

Source values are accessed directly when status is Ready. Status propagates automatically through derived stores:

```frel
source user: User = fetch("/api/user")

// Direct access - status propagates automatically
decl username = user.name
decl greeting = "Hello, " + username

// UI handles status automatically
text { greeting }  // Shows "" when Loading/Error, greeting when Ready

// Explicit status handling when needed
select on user.status() {
    FrelStatus::Loading => spinner { }
    FrelStatus::Ready => text { greeting }
    FrelStatus::Error(e) => text { "Error: " + e.message }
}
```