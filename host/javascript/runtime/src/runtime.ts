// Frel Runtime
//
// The runtime manages reactive data, subscriptions, and the notification system.

import type {
    DatumIdentity,
    ClosureIdentity,
    Identity,
    SubscriptionIdentity,
    FunctionIdentity,
    DatumData,
    ClosureData,
    SubscriptionData,
    Selector,
    Callback,
    Availability,
    BlueprintMetadata,
    RuntimeSnapshot,
    DatumSnapshotData,
    ClosureSnapshotData,
} from './types.js';
import { Tracer, selectorToString } from './tracer.js';

const GEN_LIMIT = 1000;

/** Options for Runtime constructor */
export interface RuntimeOptions {
    /** Optional tracer for debugging and testing */
    tracer?: Tracer;
}

export class Runtime {
    // Maps
    private datum: Map<DatumIdentity, DatumData> = new Map();
    private closures: Map<ClosureIdentity, ClosureData> = new Map();
    private subscriptions: Map<SubscriptionIdentity, SubscriptionData> = new Map();
    private functions: Map<FunctionIdentity, Callback> = new Map();

    // Static lookup (keyed by qualified name)
    private metadata: Map<string, BlueprintMetadata> = new Map();

    // Identity counters
    // datum: even (low bit = 0), closures: odd (low bit = 1)
    private next_datum_id: DatumIdentity = 0;
    private next_closure_id: ClosureIdentity = 1;
    private next_subscription_id: SubscriptionIdentity = 0;
    private next_function_id: FunctionIdentity = 0;

    // Event/notification queues
    private event_queue: unknown[] = [];
    private notification_queue: Set<SubscriptionIdentity> = new Set();
    private current_generation: number = 0;

    // Tracing
    private tracer?: Tracer;

    constructor(options?: RuntimeOptions) {
        this.tracer = options?.tracer;
    }

    // ========================================================================
    // Identity Allocation
    // ========================================================================

    alloc_datum_id(): DatumIdentity {
        const id = this.next_datum_id;
        this.next_datum_id += 2; // Stay even
        return id;
    }

    alloc_closure_id(): ClosureIdentity {
        const id = this.next_closure_id;
        this.next_closure_id += 2; // Stay odd
        return id;
    }

    alloc_subscription_id(): SubscriptionIdentity {
        return this.next_subscription_id++;
    }

    alloc_function_id(): FunctionIdentity {
        return this.next_function_id++;
    }

    // ========================================================================
    // Identity Helpers
    // ========================================================================

    is_datum(id: Identity): id is DatumIdentity {
        return (id & 1) === 0;
    }

    is_closure(id: Identity): id is ClosureIdentity {
        return (id & 1) === 1;
    }

    // ========================================================================
    // Datum Operations
    // ========================================================================

    create_datum(type: string, fields: Record<string, unknown> = {}, owner: ClosureIdentity | null = null): DatumIdentity {
        const id = this.alloc_datum_id();
        const datum: DatumData = {
            identity_id: id,
            type,
            structural_rev: 1,
            carried_rev: 1,
            set_generation: this.current_generation,
            availability: 'Ready',
            error: null,
            owner,
            fields,
            items: null,
            subscriptions: new Set(),
        };
        this.datum.set(id, datum);

        this.tracer?.trace('datum', 'create', { id, type, fields, owner });

        return id;
    }

    create_collection(type: string, items: unknown[] = [], owner: ClosureIdentity | null = null): DatumIdentity {
        const id = this.alloc_datum_id();
        const datum: DatumData = {
            identity_id: id,
            type,
            structural_rev: 1,
            carried_rev: 1,
            set_generation: this.current_generation,
            availability: 'Ready',
            error: null,
            owner,
            fields: null,
            items,
            subscriptions: new Set(),
        };
        this.datum.set(id, datum);

        this.tracer?.trace('datum', 'create', { id, type, items, owner });

        return id;
    }

    get_datum(id: DatumIdentity): DatumData | undefined {
        return this.datum.get(id);
    }

    destroy_datum(id: DatumIdentity): void {
        const datum = this.datum.get(id);
        if (!datum) return;

        // Unsubscribe all subscriptions to this datum
        for (const sub_id of datum.subscriptions) {
            this.subscriptions.delete(sub_id);
        }

        this.datum.delete(id);

        this.tracer?.trace('datum', 'destroy', { id });
    }

    // ========================================================================
    // Closure Operations
    // ========================================================================

    create_closure(blueprint_name: string, parent_closure_id: ClosureIdentity | null): ClosureIdentity {
        const id = this.alloc_closure_id();
        const closure: ClosureData = {
            closure_id: id,
            blueprint: blueprint_name,
            parent_closure_id,
            child_closure_ids: [],
            subscriptions_to_this: new Set(),
            subscriptions_by_this: new Set(),
            owned_datum: new Set(),
            fields: {},
            set_generation: this.current_generation,
        };
        this.closures.set(id, closure);

        // Add to parent's children
        if (parent_closure_id !== null) {
            const parent = this.closures.get(parent_closure_id);
            if (parent) {
                parent.child_closure_ids.push(id);
            }
        }

        this.tracer?.trace('closure', 'create', { id, blueprint: blueprint_name, parent: parent_closure_id });

        return id;
    }

    get_closure(id: ClosureIdentity): ClosureData | undefined {
        return this.closures.get(id);
    }

    destroy_closure(id: ClosureIdentity): void {
        const closure = this.closures.get(id);
        if (!closure) return;

        // 1. Unsubscribe all subscriptions_to_this
        for (const sub_id of closure.subscriptions_to_this) {
            this.subscriptions.delete(sub_id);
        }

        // 2. Unsubscribe all subscriptions_by_this
        for (const sub_id of closure.subscriptions_by_this) {
            const sub = this.subscriptions.get(sub_id);
            if (sub) {
                // Remove from source's subscription list
                if (this.is_datum(sub.source_id)) {
                    const source = this.datum.get(sub.source_id);
                    source?.subscriptions.delete(sub_id);
                } else {
                    const source = this.closures.get(sub.source_id);
                    source?.subscriptions_to_this.delete(sub_id);
                }
            }
            this.subscriptions.delete(sub_id);
        }

        // 3. Destroy all owned_datum
        for (const datum_id of closure.owned_datum) {
            this.destroy_datum(datum_id);
        }

        // 4. Recursively destroy all children
        for (const child_id of [...closure.child_closure_ids]) {
            this.destroy_closure(child_id);
        }

        // 5. Remove from parent's child list
        if (closure.parent_closure_id !== null) {
            const parent = this.closures.get(closure.parent_closure_id);
            if (parent) {
                const idx = parent.child_closure_ids.indexOf(id);
                if (idx !== -1) {
                    parent.child_closure_ids.splice(idx, 1);
                }
            }
        }

        // 6. Remove the closure
        this.closures.delete(id);

        this.tracer?.trace('closure', 'destroy', { id });
    }

    // ========================================================================
    // Field Access (works for both datum and closure)
    // ========================================================================

    get(id: Identity, field: string): unknown {
        if (this.is_datum(id)) {
            const datum = this.datum.get(id);
            return datum?.fields?.[field];
        } else {
            const closure = this.closures.get(id);
            return closure?.fields[field];
        }
    }

    set(id: Identity, field: string, value: unknown): void {
        if (this.is_datum(id)) {
            this.set_datum_field(id, field, value);
        } else {
            this.set_closure_field(id, field, value);
        }
    }

    private set_datum_field(id: DatumIdentity, field: string, value: unknown): void {
        const datum = this.datum.get(id);
        if (!datum || !datum.fields) return;

        const old_value = datum.fields[field];
        if (old_value === value) return; // No change

        datum.fields[field] = value;
        datum.structural_rev++;
        datum.carried_rev++;
        datum.set_generation = this.current_generation;

        this.tracer?.trace('field', 'set', {
            id,
            field,
            old: old_value,
            new: value,
            generation: this.current_generation,
        });

        // Notify subscribers
        this.notify_subscribers(id, datum.subscriptions, 'structural', field);

        // Propagate carried change up ownership chain
        if (datum.owner !== null) {
            this.propagate_carried(datum.owner);
        }
    }

    private set_closure_field(id: ClosureIdentity, field: string, value: unknown): void {
        const closure = this.closures.get(id);
        if (!closure) return;

        const old_value = closure.fields[field];
        if (old_value === value) return; // No change

        closure.fields[field] = value;
        closure.set_generation = this.current_generation;

        this.tracer?.trace('field', 'set', {
            id,
            field,
            old: old_value,
            new: value,
            generation: this.current_generation,
        });

        // Notify subscribers
        this.notify_subscribers(id, closure.subscriptions_to_this, 'structural', field);
    }

    private propagate_carried(owner_id: ClosureIdentity): void {
        const closure = this.closures.get(owner_id);
        if (!closure) return;

        // Notify carried-only subscribers
        for (const sub_id of closure.subscriptions_to_this) {
            const sub = this.subscriptions.get(sub_id);
            if (sub && (sub.selector.type === 'Everything' || sub.selector.type === 'Carried')) {
                this.notification_queue.add(sub_id);
            }
        }
    }

    private notify_subscribers(
        id: Identity,
        subscriptions: Set<SubscriptionIdentity>,
        change_type: 'structural' | 'carried',
        field?: string
    ): void {
        for (const sub_id of subscriptions) {
            const sub = this.subscriptions.get(sub_id);
            if (!sub) continue;

            const matches = this.selector_matches(sub.selector, change_type, field);
            if (matches) {
                this.notification_queue.add(sub_id);
                this.tracer?.trace('subscription', 'notify', {
                    sub_id,
                    source: id,
                    selector: selectorToString(sub.selector),
                });
            }
        }
    }

    private selector_matches(selector: Selector, change_type: 'structural' | 'carried', field?: string): boolean {
        switch (selector.type) {
            case 'Everything':
                return true;
            case 'Structural':
                return change_type === 'structural';
            case 'Carried':
                return change_type === 'carried';
            case 'Key':
                return field === selector.key;
            case 'OneOf':
                return field !== undefined && selector.keys.includes(field);
        }
    }

    // ========================================================================
    // Collection Operations
    // ========================================================================

    get_items(id: DatumIdentity): unknown[] | null {
        const datum = this.datum.get(id);
        return datum?.items ?? null;
    }

    set_items(id: DatumIdentity, items: unknown[]): void {
        const datum = this.datum.get(id);
        if (!datum) return;

        datum.items = items;
        datum.structural_rev++;
        datum.carried_rev++;
        datum.set_generation = this.current_generation;

        this.notify_subscribers(id, datum.subscriptions, 'structural');

        if (datum.owner !== null) {
            this.propagate_carried(datum.owner);
        }
    }

    // ========================================================================
    // Availability
    // ========================================================================

    get_availability(id: DatumIdentity): Availability {
        const datum = this.datum.get(id);
        return datum?.availability ?? 'Error';
    }

    set_availability(id: DatumIdentity, availability: Availability): void {
        const datum = this.datum.get(id);
        if (!datum) return;

        if (datum.availability === availability) return;

        datum.availability = availability;
        datum.structural_rev++;
        datum.set_generation = this.current_generation;

        this.notify_subscribers(id, datum.subscriptions, 'structural');
    }

    // ========================================================================
    // Subscriptions
    // ========================================================================

    subscribe(
        source_id: Identity,
        target_id: Identity,
        selector: Selector,
        callback: Callback
    ): SubscriptionIdentity {
        const sub_id = this.alloc_subscription_id();
        const callback_id = this.register_function(callback);

        const sub: SubscriptionData = {
            subscription_id: sub_id,
            source_id,
            target_id,
            selector,
            callback_id,
        };

        this.subscriptions.set(sub_id, sub);

        // Add to source's subscription list
        if (this.is_datum(source_id)) {
            const source = this.datum.get(source_id);
            source?.subscriptions.add(sub_id);
        } else {
            const source = this.closures.get(source_id);
            source?.subscriptions_to_this.add(sub_id);
        }

        // Add to target's subscriptions_by_this (if closure)
        if (this.is_closure(target_id)) {
            const target = this.closures.get(target_id);
            target?.subscriptions_by_this.add(sub_id);
        }

        this.tracer?.trace('subscription', 'subscribe', {
            id: sub_id,
            source: source_id,
            target: target_id,
            selector: selectorToString(selector),
        });

        // Subscribe during drain: check if we need immediate notification
        const set_gen = this.is_datum(source_id)
            ? this.datum.get(source_id)?.set_generation
            : this.closures.get(source_id)?.set_generation;

        if (set_gen === this.current_generation) {
            this.notification_queue.add(sub_id);
        }

        return sub_id;
    }

    unsubscribe(sub_id: SubscriptionIdentity): void {
        const sub = this.subscriptions.get(sub_id);
        if (!sub) return;

        // Remove from source
        if (this.is_datum(sub.source_id)) {
            const source = this.datum.get(sub.source_id);
            source?.subscriptions.delete(sub_id);
        } else {
            const source = this.closures.get(sub.source_id);
            source?.subscriptions_to_this.delete(sub_id);
        }

        // Remove from target
        if (this.is_closure(sub.target_id)) {
            const target = this.closures.get(sub.target_id);
            target?.subscriptions_by_this.delete(sub_id);
        }

        this.subscriptions.delete(sub_id);

        this.tracer?.trace('subscription', 'unsubscribe', { id: sub_id });
    }

    // ========================================================================
    // Functions
    // ========================================================================

    register_function(fn: Callback): FunctionIdentity {
        const id = this.alloc_function_id();
        this.functions.set(id, fn);
        return id;
    }

    get_function(id: FunctionIdentity): Callback | undefined {
        return this.functions.get(id);
    }

    // ========================================================================
    // Metadata
    // ========================================================================

    register_metadata(qualified_name: string, meta: BlueprintMetadata): void {
        this.metadata.set(qualified_name, meta);
    }

    get_metadata(qualified_name: string): BlueprintMetadata | undefined {
        return this.metadata.get(qualified_name);
    }

    // ========================================================================
    // Instantiation
    // ========================================================================

    instantiate(
        blueprint_name: string,
        parent_closure_id: ClosureIdentity | null,
        params: Record<string, unknown>
    ): ClosureIdentity {
        const closure_id = this.create_closure(blueprint_name, parent_closure_id);
        const closure = this.closures.get(closure_id)!;

        // Set parameters
        for (const [key, value] of Object.entries(params)) {
            closure.fields[key] = value;
        }

        this.tracer?.trace('closure', 'instantiate', { id: closure_id, blueprint: blueprint_name, params });

        const meta = this.metadata.get(blueprint_name);
        if (meta) {
            // Call internal binding (if present)
            if (meta.internal_binding) {
                meta.internal_binding(this, closure_id);
            }

            // Instantiate top-level children
            for (const idx of meta.top_children) {
                const call_site = meta.call_sites[idx];
                if (call_site) {
                    const child_id = this.instantiate(call_site.blueprint, closure_id, {});
                    call_site.binding(this, closure_id, child_id);
                }
            }
        }

        return closure_id;
    }

    // ========================================================================
    // Events
    // ========================================================================

    put_event(event: unknown): void {
        this.event_queue.push(event);
    }

    drain_events(): void {
        while (this.event_queue.length > 0) {
            const event = this.event_queue.shift();
            this.process_event(event);
        }
        this.drain_notifications();
    }

    private process_event(_event: unknown): void {
        // Events are processed by platform-specific code
        // This is a hook for the runtime to process internal events
    }

    // ========================================================================
    // Notifications
    // ========================================================================

    private drain_notifications(): void {
        if (this.notification_queue.size === 0) return;

        this.tracer?.trace('notification', 'drain_start', { queue_size: this.notification_queue.size });

        let generation_count = 0;

        while (this.notification_queue.size > 0) {
            // Sanity check
            generation_count++;
            if (generation_count > GEN_LIMIT) {
                console.error(`Notification drain exceeded ${GEN_LIMIT} generations, aborting`);
                this.notification_queue.clear();
                return;
            }

            // Take all pending notifications
            const processed = [...this.notification_queue];
            this.notification_queue.clear();
            this.current_generation++;

            this.tracer?.trace('notification', 'generation', { generation: this.current_generation, callbacks: processed.length });

            // Execute callbacks
            for (const sub_id of processed) {
                const sub = this.subscriptions.get(sub_id);
                if (!sub) continue; // May have been unsubscribed

                const callback = this.functions.get(sub.callback_id);
                if (callback) {
                    this.tracer?.trace('notification', 'callback', { sub_id, callback_id: sub.callback_id });
                    callback(this, sub);
                }
            }
        }

        this.tracer?.trace('notification', 'drain_end', { generations: generation_count });
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    get_current_generation(): number {
        return this.current_generation;
    }

    // ========================================================================
    // Entry Point
    // ========================================================================

    /**
     * Run the application by instantiating the entry blueprint.
     *
     * If no entryBlueprint is provided, finds a blueprint ending with ".Main"
     * in the registered metadata.
     *
     * @param entryBlueprint - Optional fully qualified blueprint name (e.g., "mymodule.Main")
     * @returns The root closure ID
     */
    run(entryBlueprint?: string): ClosureIdentity {
        let blueprintName = entryBlueprint;

        if (!blueprintName) {
            // Find a blueprint ending with ".Main"
            for (const name of this.metadata.keys()) {
                if (name.endsWith('.Main')) {
                    blueprintName = name;
                    break;
                }
            }
        }

        if (!blueprintName) {
            throw new Error(
                'No entry blueprint found. Either pass a blueprint name to run() or ' +
                'register a blueprint ending with ".Main"'
            );
        }

        const rootId = this.instantiate(blueprintName, null, {});
        this.drain_notifications();
        return rootId;
    }

    // ========================================================================
    // Snapshot
    // ========================================================================

    /**
     * Capture a snapshot of the current runtime state.
     * Useful for testing and debugging.
     */
    snapshot(): RuntimeSnapshot {
        const datums: Map<DatumIdentity, DatumSnapshotData> = new Map();
        const closures: Map<ClosureIdentity, ClosureSnapshotData> = new Map();

        for (const [id, datum] of this.datum) {
            datums.set(id, {
                type: datum.type,
                fields: datum.fields ? { ...datum.fields } : null,
                items: datum.items ? [...datum.items] : null,
                availability: datum.availability,
            });
        }

        for (const [id, closure] of this.closures) {
            closures.set(id, {
                blueprint: closure.blueprint,
                parent: closure.parent_closure_id,
                children: [...closure.child_closure_ids],
                fields: { ...closure.fields },
            });
        }

        return { datums, closures };
    }
}

// Selector helpers
export const Everything: Selector = { type: 'Everything' };
export const Structural: Selector = { type: 'Structural' };
export const Carried: Selector = { type: 'Carried' };
export const Key = (key: string): Selector => ({ type: 'Key', key });
export const OneOf = (...keys: string[]): Selector => ({ type: 'OneOf', keys });
