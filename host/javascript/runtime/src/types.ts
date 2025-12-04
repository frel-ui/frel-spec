// Frel Runtime Type Definitions

/** Identity for datums (even numbers, low bit = 0) */
export type DatumIdentity = number;

/** Identity for closures (odd numbers, low bit = 1) */
export type ClosureIdentity = number;

/** Combined identity type - can be datum or closure */
export type Identity = DatumIdentity | ClosureIdentity;

/** Subscription identity (separate namespace) */
export type SubscriptionIdentity = number;

/** Function identity (separate namespace) */
export type FunctionIdentity = number;

/** Availability state */
export type Availability = 'Loading' | 'Ready' | 'Error';

/** Selector for subscriptions */
export type Selector =
    | { type: 'Everything' }
    | { type: 'Structural' }
    | { type: 'Carried' }
    | { type: 'Key'; key: string }
    | { type: 'OneOf'; keys: string[] };

/** Callback function signature */
export type Callback = (runtime: Runtime, subscription: SubscriptionData) => void;

/** Datum structure */
export interface DatumData {
    identity_id: DatumIdentity;
    type: string;
    structural_rev: number;
    carried_rev: number;
    set_generation: number;
    availability: Availability;
    error: string | null;
    owner: ClosureIdentity | null;
    fields: Record<string, unknown> | null;
    items: unknown[] | null;
    subscriptions: Set<SubscriptionIdentity>;
}

/** Closure structure */
export interface ClosureData {
    closure_id: ClosureIdentity;
    blueprint: string;

    // Structural (unsubscribable, holds closure identities)
    parent_closure_id: ClosureIdentity | null;
    child_closure_ids: ClosureIdentity[];

    // Cleanup (unsubscribable)
    subscriptions_to_this: Set<SubscriptionIdentity>;
    subscriptions_by_this: Set<SubscriptionIdentity>;
    owned_datum: Set<DatumIdentity>;

    // Fields (subscribable)
    fields: Record<string, unknown>;
    set_generation: number;
}

/** Subscription structure */
export interface SubscriptionData {
    subscription_id: SubscriptionIdentity;
    source_id: Identity;
    target_id: Identity;
    selector: Selector;
    callback_id: FunctionIdentity;
}

/** Metadata for a blueprint */
export interface BlueprintMetadata {
    internal_binding: (runtime: Runtime, closure_id: ClosureIdentity) => void;
    call_sites: Record<string, CallSiteMetadata>;
}

/** Call site metadata */
export interface CallSiteMetadata {
    blueprint: string;
    binding: (runtime: Runtime, parent_id: ClosureIdentity, child_id: ClosureIdentity) => void;
}

// Forward declaration for Runtime (actual class in runtime.ts)
import type { Runtime } from './runtime.js';
