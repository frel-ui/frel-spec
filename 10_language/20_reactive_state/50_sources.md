# Sources

Sources are special stores that produce values asynchronously from external systems - like 
timers, network requests, or event streams. They're managed by the runtime and represent 
effectful operations that happen outside the fragment's control.

## Syntax

`source <id> [:<type>]? = <producer>(…options…) [.. on_value |val: T| { <handler> }]`

## Semantics

- **Kind**: Producing store managed by the runtime (effectful). Not writable from fragments.
- **Views for expressions**:
  - `<id>.latest() → Option<T>`: Most recent value produced (if any).
  - `<id>.status() → Status<E>`: Current status - `Loading`, `Ready`, or `Error(E)`.
- **Event handler**: `on_value` runs each time the source produces a new value. The handler receives the unwrapped value (not `Option<T>`).
- **As a dependency**: Sources can be read by other stores, triggering reactive updates.
- **Typical producers**: `interval(ms)`, `fetch(|| …)`, `sse(url, event)`.

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

```rust
enum Status<E> {
    Loading,           // Source is fetching data
    Ready,             // Data is available
    Error(E),          // An error occurred
}
```

## Examples

### Using on_value Handler

The `on_value` handler is the recommended way to handle source events:

```frel
fragment NotificationBadge() {
    source notifications = sse("/notifications")
    writable unread_count = 0

    // Handler runs each time a notification arrives
    notifications .. on_value |notif: Notification| {
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
fragment Clock() {
    source tick = interval(ms: 1000)

    decl current_time = tick.latest()
        .map(|_| chrono::Local::now().format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "...".to_string())

    text { current_time }
        .. font { size: 32 weight: 700 }
}
```

### Single Fetch

One-time data fetch:

```frel
fragment UserProfile(user_id: u32) {
    source user = fetch(|| api::get_user(user_id))

    decl status = user.status()

    select on status {
        Status::Loading => column {
            spinner { }
            text { "Loading user..." }
        }

        Status::Ready => {
            decl user_data = user.latest().unwrap()

            column {
                text { user_data.name } .. font { size: 20 weight: 700 }
                text { user_data.email }
                text { "Joined: ${user_data.created_at}" }
            }
        }

        Status::Error(e) => column {
            text { "Error: ${e}" } .. font { color: Red }
            button { "Retry" }
                .. on_click { /* trigger refetch */ }
        }
    }
}
```

### Server-Sent Events (SSE)

Streaming data with accumulation:

```frel
fragment LiveFeed() {
    source updates = sse(url: "/api/feed", event: "update")
    writable messages: Vec<Message> = vec![]

    updates .. on_value |msg: Message| {
        messages.push(msg)
    }

    column {
        text { "Live Feed (${messages.len()} messages)" }

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
            .. on_click { messages = vec![] }
    }
}
```

### Multiple Sources

Combining different data sources:

```frel
fragment Dashboard() {
    source stats = fetch(|| api::get_stats())
    source notifications = sse(url: "/notifications")
    source health = interval_fetch(|| api::health_check(), interval_ms: 5000)

    writable notif_list: Vec<Notification> = vec![]

    notifications .. on_value |notif: Notification| {
        notif_list.push(notif)
    }

    column {
        gap { 16 }

        // Stats section
        when stats.status() == Status::Ready {
            decl data = stats.latest().unwrap()
            StatsCard(data)
        }

        // Health indicator
        row {
            decl is_healthy = health.latest()
                .map(|h| h.is_ok())
                .unwrap_or(false)

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
fragment PostViewer() {
    writable selected_post_id: Option<u32> = None

    // Source that re-fetches when selected_post_id changes
    source post = fetch_when(
        selected_post_id,
        |id| api::get_post(id)
    )

    column {
        // Post list
        repeat on available_posts as p {
            button { p.title }
                .. on_click { selected_post_id = Some(p.id) }
        }

        // Post detail
        when selected_post_id.is_some() {
            select on post.status() {
                Status::Loading => spinner { }

                Status::Ready => {
                    decl post_data = post.latest().unwrap()

                    column {
                        text { post_data.title } .. font { size: 24 weight: 700 }
                        text { post_data.content }
                    }
                }

                Status::Error(e) => text { "Error: ${e}" } .. font { color: Red }
            }
        }
    }
}
```

### Pagination with Sources

```frel
fragment PaginatedList() {
    writable page = 0
    writable page_size = 25

    source items = fetch_paginated(
        || api::get_items(page, page_size)
    )

    decl status = items.status()
    decl current_items = items.latest().unwrap_or_default()

    column {
        when status == Status::Loading {
            spinner { }
        }

        repeat on current_items as item {
            ItemCard(item)
        }

        row {
            button { "Previous" }
                .. enabled { page > 0 && status == Status::Ready }
                .. on_click { page = page - 1 }

            text { "Page ${page + 1}" }

            button { "Next" }
                .. enabled { status == Status::Ready }
                .. on_click { page = page + 1 }
        }

        when status == Status::Error(e) {
            text { "Failed to load: ${e}" } .. font { color: Red }
        }
    }
}
```

### Polling

Repeatedly fetch data at intervals:

```frel
fragment SystemMonitor() {
    source metrics = poll(
        producer: || api::get_metrics(),
        interval_ms: 2000
    )

    decl latest_metrics = metrics.latest()
    decl last_update = metrics.latest()
        .map(|_| chrono::Local::now())
        .unwrap_or_default()

    column {
        text { "System Metrics" } .. font { size: 20 weight: 700 }
        text { "Last update: ${last_update}" } .. font { color: Gray }

        when metrics.status() == Status::Ready && latest_metrics.is_some() {
            decl m = latest_metrics.unwrap()

            column {
                gap { 8 }

                row {
                    text { "CPU:" }
                    text { "${m.cpu_percent}%" }
                        .. font { color: if m.cpu_percent > 80 { Red } else { Green } }
                }

                row {
                    text { "Memory:" }
                    text { "${m.memory_mb} MB" }
                }

                row {
                    text { "Active Connections:" }
                    text { "${m.connections}" }
                }
            }
        }
    }
}
```

### WebSocket

Bidirectional communication:

```frel
fragment ChatRoom(room_id: String) {
    source messages = websocket(url: "/chat/${room_id}")
    writable draft = ""
    writable chat_history: Vec<Message> = vec![]

    messages .. on_value |msg: Message| {
        chat_history.push(msg)
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
        decl status = messages.status()
        text {
            match status {
                Status::Loading => "Connecting...",
                Status::Ready => "Connected",
                Status::Error(e) => "Disconnected: ${e}",
            }
        } .. font {
            color: match status {
                Status::Loading => Orange,
                Status::Ready => Green,
                Status::Error(_) => Red,
            }
        }
    }
}
```

### Cached Fetch

Fetch with caching behavior:

```frel
fragment CachedData(key: String) {
    source data = fetch_cached(
        key: key.clone(),
        producer: || api::get_data(key.clone()),
        ttl_seconds: 300  // 5 minute cache
    )

    decl is_cached = data.is_cached()

    column {
        when is_cached {
            text { "📦 From cache" } .. font { color: Gray size: 12 }
        }

        select on data.status() {
            Status::Loading => spinner { }

            Status::Ready => {
                decl value = data.latest().unwrap()
                DataView(value)
            }

            Status::Error(e) => text { "Error: ${e}" } .. font { color: Red }
        }
    }
}
```

### Dependent Sources

Source that depends on another source:

```frel
fragment UserPosts() {
    source user = fetch(|| api::get_current_user())

    // This source depends on user being ready
    source posts = user.latest()
        .map(|u| fetch(|| api::get_posts(u.id)))
        .unwrap_or_else(|| empty_source())

    column {
        when user.status() == Status::Ready {
            decl user_data = user.latest().unwrap()
            text { "${user_data.name}'s Posts" }
        }

        select on posts.status() {
            Status::Loading => spinner { }

            Status::Ready => {
                decl post_list = posts.latest().unwrap_or_default()

                repeat on post_list as post {
                    PostCard(post)
                }
            }

            Status::Error(e) => text { "Failed to load posts: ${e}" }
        }
    }
}
```

## Best Practices

### Handle All Status States

Always handle Loading, Ready, and Error states for one-time fetches:

```frel
select on source.status() {
    Status::Loading => spinner { }
    Status::Ready => { /* show data */ }
    Status::Error(e) => { /* show error */ }
}
```

### Use on_value for Accumulation

Use `on_value` handlers to accumulate source events:

```frel
// Good - explicit accumulation
source items = sse("/items")
writable item_list: Vec<Item> = vec![]

items .. on_value |item: Item| {
    item_list.push(item)
}

// Also good - use .latest() for simple derived values
decl latest_item = items.latest()
```

### Extract Status Checks

For cleaner code, extract status to a derived store:

```frel
source data = fetch(|| api::get_data())

decl is_loading = data.status() == Status::Loading
decl is_error = matches!(data.status(), Status::Error(_))
decl is_ready = data.status() == Status::Ready

when is_loading {
    spinner { }
}
```

### Null Safety with Latest

Always handle the Option from `.latest()`:

```frel
// Good - handle None case
decl user_name = user.latest()
    .map(|u| u.name.clone())
    .unwrap_or_else(|| "Unknown".to_string())

// Risky - will panic if None
decl user_name = user.latest().unwrap().name
```