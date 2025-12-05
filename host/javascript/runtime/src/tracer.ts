// Frel Runtime Tracing

/**
 * Trace event categories that can be enabled/disabled independently.
 */
export type TraceCategory =
    | 'datum'        // Datum lifecycle: create, destroy
    | 'closure'      // Closure/fragment lifecycle: create, destroy, instantiate
    | 'field'        // Field changes: set (no get - too high volume)
    | 'subscription' // Reactive flow: subscribe, unsubscribe, notify
    | 'notification' // Drain loop: drain_start, drain_end, callback, generation
    | 'rendering';   // Future: render operations

/**
 * All available trace categories.
 */
export const ALL_CATEGORIES: Set<TraceCategory> = new Set([
    'datum',
    'closure',
    'field',
    'subscription',
    'notification',
    'rendering',
]);

/**
 * A single trace event.
 */
export interface TraceEvent {
    /** High-resolution timestamp (from performance.now() or Date.now()) */
    timestamp: number;
    /** Event category */
    category: TraceCategory;
    /** Event name within category (e.g., 'create', 'set', 'notify') */
    event: string;
    /** Event-specific data */
    data: Record<string, unknown>;
}

/**
 * Sink interface for receiving trace events.
 */
export interface TraceSink {
    /** Write a single trace event */
    write(event: TraceEvent): void;
    /** Flush any buffered events (for file sinks) */
    flush(): void;
    /** Clear all stored events (for memory sinks) */
    clear(): void;
}

/**
 * In-memory sink that stores events in an array.
 * Useful for testing and analysis.
 */
export class MemorySink implements TraceSink {
    public events: TraceEvent[] = [];

    write(event: TraceEvent): void {
        this.events.push(event);
    }

    flush(): void {
        // No-op for memory sink
    }

    clear(): void {
        this.events = [];
    }
}

/**
 * Console sink that pretty-prints events for debugging.
 */
export class ConsoleSink implements TraceSink {
    private readonly colors: Record<TraceCategory, string> = {
        datum: '\x1b[36m',        // Cyan
        closure: '\x1b[35m',      // Magenta
        field: '\x1b[33m',        // Yellow
        subscription: '\x1b[32m', // Green
        notification: '\x1b[34m', // Blue
        rendering: '\x1b[31m',    // Red
    };
    private readonly reset = '\x1b[0m';

    write(event: TraceEvent): void {
        const color = this.colors[event.category] || '';
        const dataStr = Object.entries(event.data)
            .map(([k, v]) => `${k}=${JSON.stringify(v)}`)
            .join(' ');
        console.log(
            `${color}[${event.category}]${this.reset} ${event.event} ${dataStr}`
        );
    }

    flush(): void {
        // No-op for console sink
    }

    clear(): void {
        // No-op for console sink
    }
}

/**
 * Composite sink that writes to multiple sinks.
 */
export class CompositeSink implements TraceSink {
    constructor(private readonly sinks: TraceSink[]) {}

    write(event: TraceEvent): void {
        for (const sink of this.sinks) {
            sink.write(event);
        }
    }

    flush(): void {
        for (const sink of this.sinks) {
            sink.flush();
        }
    }

    clear(): void {
        for (const sink of this.sinks) {
            sink.clear();
        }
    }
}

/**
 * Configuration for the tracer.
 */
export interface TracerConfig {
    /** Whether tracing is enabled */
    enabled: boolean;
    /** Which categories to trace */
    categories: Set<TraceCategory>;
    /** Where to send trace events */
    sink: TraceSink;
}

/**
 * Main tracer class that filters and dispatches trace events.
 */
export class Tracer {
    public readonly enabled: boolean;
    private readonly categories: Set<TraceCategory>;
    private readonly sink: TraceSink;
    private readonly getTimestamp: () => number;

    constructor(config: TracerConfig) {
        this.enabled = config.enabled;
        this.categories = config.categories;
        this.sink = config.sink;

        // Use performance.now() if available, otherwise Date.now()
        this.getTimestamp =
            typeof performance !== 'undefined'
                ? () => performance.now()
                : () => Date.now();
    }

    /**
     * Record a trace event if tracing is enabled and category is active.
     */
    trace(
        category: TraceCategory,
        event: string,
        data: Record<string, unknown>
    ): void {
        if (!this.enabled || !this.categories.has(category)) {
            return;
        }

        this.sink.write({
            timestamp: this.getTimestamp(),
            category,
            event,
            data,
        });
    }

    /**
     * Check if a specific category is being traced.
     */
    isTracing(category: TraceCategory): boolean {
        return this.enabled && this.categories.has(category);
    }

    /**
     * Flush the sink (for file-based sinks).
     */
    flush(): void {
        this.sink.flush();
    }

    /**
     * Clear all events from the sink.
     */
    clear(): void {
        this.sink.clear();
    }

    /**
     * Get the underlying sink (useful for accessing MemorySink.events).
     */
    getSink(): TraceSink {
        return this.sink;
    }
}

/**
 * Create a tracer with all categories enabled and a memory sink.
 * Convenient for testing.
 */
export function createTestTracer(): Tracer {
    return new Tracer({
        enabled: true,
        categories: ALL_CATEGORIES,
        sink: new MemorySink(),
    });
}

/**
 * Create a tracer that logs to console.
 * Convenient for debugging.
 */
export function createDebugTracer(
    categories: Set<TraceCategory> = ALL_CATEGORIES
): Tracer {
    return new Tracer({
        enabled: true,
        categories,
        sink: new ConsoleSink(),
    });
}

/**
 * Selector type to string for tracing purposes.
 */
export function selectorToString(selector: {
    type: string;
    key?: string;
    keys?: string[];
}): string {
    switch (selector.type) {
        case 'Everything':
            return 'Everything';
        case 'Structural':
            return 'Structural';
        case 'Carried':
            return 'Carried';
        case 'Key':
            return `Key(${selector.key})`;
        case 'OneOf':
            return `OneOf(${selector.keys?.join(', ')})`;
        default:
            return selector.type;
    }
}
