// Frel Runtime Testing Framework
//
// Provides utilities for testing Frel applications using trace-based verification.

import { Runtime, type RuntimeOptions } from './runtime.js';
import {
    Tracer,
    MemorySink,
    ALL_CATEGORIES,
    type TraceEvent,
    type TraceCategory,
} from './tracer.js';
import type { RuntimeSnapshot, BlueprintMetadata } from './types.js';

// ========================================================================
// Test Result Types
// ========================================================================

/** Result of running a test */
export interface TestResult {
    /** Trace events captured during execution */
    events: TraceEvent[];
    /** Final runtime state snapshot */
    snapshot: RuntimeSnapshot;
    /** Root closure ID */
    rootId: number;
}

/** Options for running a test */
export interface RunTestOptions {
    /** Which trace categories to capture (default: all) */
    categories?: Set<TraceCategory>;
    /** Entry blueprint name (default: finds ".Main") */
    entryBlueprint?: string;
}

/** Options for comparing traces */
export interface CompareOptions {
    /** Ignore timestamp differences (default: true) */
    ignoreTimestamps?: boolean;
    /** Categories to compare (default: all) */
    categories?: Set<TraceCategory>;
}

/** Difference between two traces */
export interface TraceDiff {
    type: 'missing' | 'extra' | 'mismatch';
    index: number;
    expected?: TraceEvent;
    actual?: TraceEvent;
    message: string;
}

// ========================================================================
// Test Runner
// ========================================================================

/**
 * Register function type that modules export.
 * Generated modules call this to register their metadata.
 */
export type RegisterFunction = (runtime: Runtime) => void;

/**
 * Run a test with a set of blueprint metadata.
 *
 * @param registerFn - Function that registers blueprint metadata
 * @param options - Test options
 * @returns Test result with events and snapshot
 */
export function runTest(
    registerFn: RegisterFunction,
    options: RunTestOptions = {}
): TestResult {
    const categories = options.categories ?? ALL_CATEGORIES;
    const sink = new MemorySink();
    const tracer = new Tracer({
        enabled: true,
        categories,
        sink,
    });

    const runtime = new Runtime({ tracer });

    // Register metadata from the module
    registerFn(runtime);

    // Run the application
    const rootId = runtime.run(options.entryBlueprint);

    return {
        events: sink.events,
        snapshot: runtime.snapshot(),
        rootId,
    };
}

/**
 * Run a test with pre-registered metadata.
 * Useful when you have already created a Runtime with metadata.
 */
export function runTestWithRuntime(
    runtime: Runtime,
    tracer: Tracer,
    entryBlueprint?: string
): TestResult {
    const rootId = runtime.run(entryBlueprint);
    const sink = tracer.getSink() as MemorySink;

    return {
        events: sink.events,
        snapshot: runtime.snapshot(),
        rootId,
    };
}

// ========================================================================
// NDJSON Baseline I/O
// ========================================================================

/**
 * Serialize trace events to NDJSON format.
 * Each event is on its own line for git-friendly diffs.
 */
export function serializeToNdjson(events: TraceEvent[]): string {
    return events.map(e => JSON.stringify(e)).join('\n') + '\n';
}

/**
 * Parse NDJSON format to trace events.
 */
export function parseNdjson(ndjson: string): TraceEvent[] {
    const trimmed = ndjson.trim();
    if (!trimmed) return [];

    return trimmed.split('\n').map((line, i) => {
        try {
            return JSON.parse(line) as TraceEvent;
        } catch (e) {
            throw new Error(`Failed to parse NDJSON at line ${i + 1}: ${line}`);
        }
    });
}

// ========================================================================
// Trace Comparison
// ========================================================================

/**
 * Compare two traces and return differences.
 *
 * @param actual - Actual trace events from test run
 * @param expected - Expected trace events from baseline
 * @param options - Comparison options
 * @returns Array of differences, empty if traces match
 */
export function compareTraces(
    actual: TraceEvent[],
    expected: TraceEvent[],
    options: CompareOptions = {}
): TraceDiff[] {
    const ignoreTimestamps = options.ignoreTimestamps ?? true;
    const categories = options.categories;

    // Filter by categories if specified
    const filterFn = categories
        ? (e: TraceEvent) => categories.has(e.category)
        : () => true;

    const filteredActual = actual.filter(filterFn);
    const filteredExpected = expected.filter(filterFn);

    const diffs: TraceDiff[] = [];

    const maxLen = Math.max(filteredActual.length, filteredExpected.length);

    for (let i = 0; i < maxLen; i++) {
        const actualEvent = filteredActual[i];
        const expectedEvent = filteredExpected[i];

        if (!actualEvent && expectedEvent) {
            diffs.push({
                type: 'missing',
                index: i,
                expected: expectedEvent,
                message: `Missing event at index ${i}: ${formatEvent(expectedEvent)}`,
            });
        } else if (actualEvent && !expectedEvent) {
            diffs.push({
                type: 'extra',
                index: i,
                actual: actualEvent,
                message: `Extra event at index ${i}: ${formatEvent(actualEvent)}`,
            });
        } else if (actualEvent && expectedEvent) {
            if (!eventsEqual(actualEvent, expectedEvent, ignoreTimestamps)) {
                diffs.push({
                    type: 'mismatch',
                    index: i,
                    actual: actualEvent,
                    expected: expectedEvent,
                    message: `Mismatch at index ${i}:\n  Expected: ${formatEvent(expectedEvent)}\n  Actual:   ${formatEvent(actualEvent)}`,
                });
            }
        }
    }

    return diffs;
}

/**
 * Check if two events are equal.
 */
function eventsEqual(a: TraceEvent, b: TraceEvent, ignoreTimestamps: boolean): boolean {
    if (a.category !== b.category) return false;
    if (a.event !== b.event) return false;
    if (!ignoreTimestamps && a.timestamp !== b.timestamp) return false;

    // Deep compare data
    return JSON.stringify(a.data) === JSON.stringify(b.data);
}

/**
 * Format an event for display.
 */
function formatEvent(e: TraceEvent): string {
    const dataStr = Object.entries(e.data)
        .map(([k, v]) => `${k}=${JSON.stringify(v)}`)
        .join(' ');
    return `[${e.category}] ${e.event} ${dataStr}`;
}

/**
 * Format trace differences for display in test output.
 */
export function formatDiffs(diffs: TraceDiff[]): string {
    if (diffs.length === 0) return '';

    const lines = [`Trace mismatch: ${diffs.length} difference(s)`];
    for (const diff of diffs) {
        lines.push(`  ${diff.message}`);
    }
    return lines.join('\n');
}

// ========================================================================
// Assertions
// ========================================================================

/**
 * Assert that actual events match expected baseline.
 * Throws an error with a formatted diff if they don't match.
 */
export function assertTracesMatch(
    actual: TraceEvent[],
    expected: TraceEvent[],
    options: CompareOptions = {}
): void {
    const diffs = compareTraces(actual, expected, options);
    if (diffs.length > 0) {
        throw new Error(formatDiffs(diffs));
    }
}

/**
 * Assert that the trace contains a specific event.
 */
export function assertContainsEvent(
    events: TraceEvent[],
    category: TraceCategory,
    eventName: string,
    dataMatcher?: Partial<Record<string, unknown>>
): void {
    const found = events.find(e => {
        if (e.category !== category || e.event !== eventName) return false;
        if (!dataMatcher) return true;

        for (const [key, value] of Object.entries(dataMatcher)) {
            if (e.data[key] !== value) return false;
        }
        return true;
    });

    if (!found) {
        const matcherStr = dataMatcher
            ? ` with data ${JSON.stringify(dataMatcher)}`
            : '';
        throw new Error(`Expected event [${category}] ${eventName}${matcherStr} not found`);
    }
}

/**
 * Count events matching the given criteria.
 */
export function countEvents(
    events: TraceEvent[],
    category: TraceCategory,
    eventName?: string
): number {
    return events.filter(e => {
        if (e.category !== category) return false;
        if (eventName && e.event !== eventName) return false;
        return true;
    }).length;
}

// ========================================================================
// Test Helpers
// ========================================================================

/**
 * Create a simple test runtime with tracing enabled.
 */
export function createTestRuntime(): { runtime: Runtime; tracer: Tracer; sink: MemorySink } {
    const sink = new MemorySink();
    const tracer = new Tracer({
        enabled: true,
        categories: ALL_CATEGORIES,
        sink,
    });
    const runtime = new Runtime({ tracer });

    return { runtime, tracer, sink };
}

/**
 * Filter events by category.
 */
export function filterByCategory(
    events: TraceEvent[],
    ...categories: TraceCategory[]
): TraceEvent[] {
    const categorySet = new Set(categories);
    return events.filter(e => categorySet.has(e.category));
}

/**
 * Get all events of a specific type.
 */
export function getEvents(
    events: TraceEvent[],
    category: TraceCategory,
    eventName: string
): TraceEvent[] {
    return events.filter(e => e.category === category && e.event === eventName);
}
