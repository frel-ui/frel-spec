// Frel JavaScript Runtime
//
// Platform-independent reactive core for Frel applications.

export { Runtime, Everything, Structural, Carried, Key, OneOf } from './runtime.js';
export type { RuntimeOptions } from './runtime.js';

export type {
    DatumIdentity,
    ClosureIdentity,
    Identity,
    SubscriptionIdentity,
    FunctionIdentity,
    Availability,
    Selector,
    Callback,
    DatumData,
    ClosureData,
    SubscriptionData,
    BlueprintMetadata,
    CallSiteMetadata,
    RuntimeSnapshot,
    DatumSnapshotData,
    ClosureSnapshotData,
} from './types.js';

// Tracer module
export {
    Tracer,
    MemorySink,
    ConsoleSink,
    CompositeSink,
    ALL_CATEGORIES,
    createTestTracer,
    createDebugTracer,
    selectorToString,
} from './tracer.js';
export type {
    TraceCategory,
    TraceEvent,
    TraceSink,
    TracerConfig,
} from './tracer.js';

// Testing module
export {
    runTest,
    runTestWithRuntime,
    serializeToNdjson,
    parseNdjson,
    compareTraces,
    formatDiffs,
    assertTracesMatch,
    assertContainsEvent,
    countEvents,
    createTestRuntime,
    filterByCategory,
    getEvents,
} from './testing.js';
export type {
    TestResult,
    RunTestOptions,
    CompareOptions,
    TraceDiff,
    RegisterFunction,
} from './testing.js';
