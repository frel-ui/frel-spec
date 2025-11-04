# DateTime Types

Frel provides standard temporal types for working with dates, times, and durations.

## Instant - Point in Time

Represents a specific moment in time (UTC):

```frel
scheme Event {
    created_at { Instant }
        .. default { Instant::now() }

    published_at { Instant }
        .. optional { true }

    updated_at { Instant }
}
```

### Operations

```frel
// Current time
let now = Instant::now()

// Comparison
if event.created_at < Instant::now() {
    // Event is in the past
}

// Arithmetic with Duration
let future = now + Duration::hours(2)
let past = now - Duration::days(1)

// Time difference
let elapsed = Instant::now() - event.created_at  // Returns Duration
```

### Use Cases

**Timestamps:**
```frel
scheme Post {
    created_at { Instant }
        .. default { Instant::now() }
        .. readonly { true }

    updated_at { Instant }
        .. auto_now { true }
}
```

**Scheduling:**
```frel
scheme Meeting {
    start_time { Instant }
    end_time { Instant }
}
```

## LocalDate - Date Without Time

Represents a calendar date without time information (e.g., 2024-03-15):

```frel
scheme Schedule {
    birth_date { LocalDate }

    event_date { LocalDate }
        .. default { LocalDate::today() }

    deadline { LocalDate }
}
```

### Operations

```frel
// Current date
let today = LocalDate::today()

// Construct from components
let date = LocalDate::from_ymd(2024, 3, 15)

// Access components
date.year()    // -> 2024
date.month()   // -> 3
date.day()     // -> 15

// Comparison
if deadline < LocalDate::today() {
    // Deadline has passed
}

// Arithmetic
let tomorrow = today + Duration::days(1)
let last_week = today - Duration::days(7)
```

### Validation

```frel
scheme Event {
    birth_date { LocalDate }
        .. before { LocalDate::today() }  // Must be in the past

    event_date { LocalDate }
        .. after { LocalDate::today() }   // Must be in the future
}
```

## LocalTime - Time Without Date

Represents a time of day without date information (e.g., 14:30:00):

```frel
scheme Alarm {
    alarm_time { LocalTime }

    reminder_time { LocalTime }
        .. default { LocalTime::from_hms(9, 0, 0) }
}
```

### Operations

```frel
// Construct from components
let time = LocalTime::from_hms(14, 30, 0)      // 14:30:00
let precise = LocalTime::from_hms_nano(14, 30, 0, 500000000)

// Access components
time.hour()        // -> 14
time.minute()      // -> 30
time.second()      // -> 0

// Comparison
if current_time > alarm_time {
    trigger_alarm()
}
```

### Use Cases

**Daily schedules:**
```frel
scheme DailySchedule {
    wake_up { LocalTime }
    work_start { LocalTime }
    lunch { LocalTime }
    work_end { LocalTime }
}
```

**Opening hours:**
```frel
scheme BusinessHours {
    open_time { LocalTime }
    close_time { LocalTime }
}
```

## LocalDateTime - Date and Time Without Timezone

Represents a date and time without timezone information:

```frel
scheme Appointment {
    scheduled { LocalDateTime }

    completed { LocalDateTime }
        .. optional { true }
}
```

### Operations

```frel
// Current local datetime
let now = LocalDateTime::now()

// Construct from components
let dt = LocalDateTime::from_ymd_hms(2024, 3, 15, 14, 30, 0)

// Access components
dt.year()      // -> 2024
dt.month()     // -> 3
dt.day()       // -> 15
dt.hour()      // -> 14
dt.minute()    // -> 30
dt.second()    // -> 0

// Extract date/time
dt.date()      // -> LocalDate
dt.time()      // -> LocalTime

// Comparison and arithmetic
let later = dt + Duration::hours(2)
```

### Use Cases

**Event scheduling (without timezone):**
```frel
scheme Event {
    scheduled_at { LocalDateTime }
    notes { String }
}
```

## Timezone - IANA Timezone Identifier

Represents a timezone (e.g., "America/New_York", "Europe/London"):

```frel
scheme UserSettings {
    timezone { Timezone }
        .. default { Timezone::system() }
}
```

### Operations

```frel
// System timezone
let tz = Timezone::system()

// Parse timezone identifier
let ny = Timezone::parse("America/New_York")
let london = Timezone::parse("Europe/London")

// Convert to string
tz.to_string()  // "America/New_York"

// Use with Instant
let instant = Instant::now()
let local_time = instant.in_timezone(tz)
```

### Use Cases

**User preferences:**
```frel
scheme UserProfile {
    timezone { Timezone }
        .. default { Timezone::system() }
}
```

**Multi-timezone scheduling:**
```frel
scheme Meeting {
    start_time { Instant }
    timezone { Timezone }
}
```

**Timezone conversion:**
```frel
fragment TimeDisplay(instant: Instant, user_tz: Timezone) {
    decl local_time = instant.in_timezone(user_tz)
    text { local_time.format("%Y-%m-%d %H:%M:%S %Z") }
}
```

## Duration - Time Span

Represents a length of time:

```frel
scheme Meeting {
    duration { Duration }
        .. default { Duration::hours(1) }
        .. min { Duration::minutes(15) }
        .. max { Duration::hours(8) }
}
```

### Constructors

```frel
Duration::nanoseconds(500)
Duration::microseconds(1000)
Duration::milliseconds(500)
Duration::seconds(30)
Duration::minutes(15)
Duration::hours(2)
Duration::days(7)
Duration::weeks(4)
```

### Operations

```frel
let d1 = Duration::hours(2)
let d2 = Duration::minutes(30)

// Arithmetic
let total = d1 + d2              // 2.5 hours
let difference = d1 - d2         // 1.5 hours
let doubled = d1 * 2             // 4 hours

// Conversion
d1.as_seconds()                  // -> 7200
d1.as_minutes()                  // -> 120
d1.as_hours()                    // -> 2

// Comparison
if task_duration > Duration::hours(1) {
    // Long task
}
```

### Validation

```frel
scheme Task {
    estimated_duration { Duration }
        .. min { Duration::minutes(5) }
        .. max { Duration::hours(8) }

    actual_duration { Duration }
        .. optional { true }
}
```

### Use Cases

**Timeouts:**
```frel
scheme Config {
    request_timeout { Duration }
        .. default { Duration::seconds(30) }

    session_timeout { Duration }
        .. default { Duration::hours(24) }
}
```

**Time tracking:**
```frel
scheme TimeEntry {
    duration { Duration }
    task { String }
    date { LocalDate }
}
```

## Combined Examples

### Event with Timezone Support

```frel
scheme Event {
    title { String }

    // Store as Instant (UTC) for consistency
    start_time { Instant }

    // Store user's timezone for display
    timezone { Timezone }

    duration { Duration }
        .. default { Duration::hours(1) }
}

fragment EventDisplay(event: Event, user_tz: Timezone) {
    // Convert to user's timezone for display
    decl local_start = event.start_time.in_timezone(user_tz)
    decl local_end = (event.start_time + event.duration).in_timezone(user_tz)

    column {
        text { event.title }
        text { local_start.format("%Y-%m-%d %H:%M") }
        text { "Duration: ${event.duration.as_hours()}h" }
    }
}
```

### Scheduling System

```frel
scheme Appointment {
    id { Uuid }
        .. default { Uuid::new() }

    title { String }

    // Use Instant for absolute time
    scheduled_at { Instant }

    // User's timezone for display
    timezone { Timezone }

    duration { Duration }
        .. default { Duration::minutes(30) }
        .. min { Duration::minutes(15) }
        .. max { Duration::hours(4) }

    created_at { Instant }
        .. default { Instant::now() }
        .. readonly { true }
}
```

### Business Hours

```frel
scheme BusinessHours {
    open_time { LocalTime }
        .. default { LocalTime::from_hms(9, 0, 0) }

    close_time { LocalTime }
        .. default { LocalTime::from_hms(17, 0, 0) }

    timezone { Timezone }

    closed_dates { Set<LocalDate> }
}
```

## Type Mapping

| Frel Type       | Rust            | TypeScript | Python           |
|-----------------|-----------------|------------|------------------|
| `Instant`       | `DateTime<Utc>` | `Date`     | `datetime` (UTC) |
| `LocalDate`     | `NaiveDate`     | Custom     | `date`           |
| `LocalTime`     | `NaiveTime`     | Custom     | `time`           |
| `LocalDateTime` | `NaiveDateTime` | Custom     | `datetime`       |
| `Timezone`      | `Tz`            | `string`   | `ZoneInfo`       |
| `Duration`      | `Duration`      | Custom     | `timedelta`      |

## Best Practices

### Use Instant for Absolute Time

```frel
// Good - absolute time, timezone-independent
scheme Event {
    start_time { Instant }
    timezone { Timezone }  // Store for display
}

// Avoid - ambiguous without timezone
scheme Event {
    start_time { LocalDateTime }  // Which timezone?
}
```

### Store UTC, Display in Local

```frel
fragment EventCard(event: Event, user_tz: Timezone) {
    // Store as Instant (UTC) in backend
    // Display in user's timezone in UI
    decl local_time = event.start_time.in_timezone(user_tz)
    text { local_time.format("%Y-%m-%d %H:%M %Z") }
}
```

### Use LocalDate for Calendar Dates

```frel
// Good - date without time
scheme Holiday {
    date { LocalDate }
    name { String }
}

// Avoid - unnecessary time information
scheme Holiday {
    date { Instant }  // Overkill for just a date
}
```

### Validate Date Ranges

```frel
scheme Event {
    start_date { LocalDate }
    end_date { LocalDate }
        .. validate { |end, data| end >= data.start_date }
        .. error_message { "End date must be after start date" }
}
```
