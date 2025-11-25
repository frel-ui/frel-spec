// Frel Runtime
//
// Main runtime class that manages the entire reactive system

import { Datum } from './datum';
import { Subscription } from './subscription';
import { DatumIdentity, Callback, Selector } from './types';

export class Runtime {
  private nextDatumId: number = 1;
  private nextSubscriptionId: number = 1;
  private nextCallbackId: number = 1;
  private currentGeneration: number = 0;

  private data: Map<DatumIdentity, Datum> = new Map();
  private subscriptions: Map<number, Subscription> = new Map();
  private callbacks: Map<number, Callback> = new Map();

  private eventQueue: any[] = [];
  private notificationQueue: Set<number> = new Set();

  constructor() {
    console.log('[Frel Runtime] Initialized');
  }

  // Datum management
  createDatum(type: string, owner: DatumIdentity | null = null): Datum {
    const identityId = this.nextDatumId++;
    const datum = new Datum(identityId, type, owner);
    this.data.set(identityId, datum);
    return datum;
  }

  getDatum(id: DatumIdentity): Datum | undefined {
    return this.data.get(id);
  }

  // Subscription management
  subscribe(
    sourceId: DatumIdentity,
    targetId: DatumIdentity,
    selector: Selector,
    callback: Callback
  ): number {
    const subscriptionId = this.nextSubscriptionId++;
    const callbackId = this.nextCallbackId++;

    this.callbacks.set(callbackId, callback);

    const subscription = new Subscription(
      subscriptionId,
      sourceId,
      targetId,
      selector,
      callbackId
    );

    this.subscriptions.set(subscriptionId, subscription);

    // Add subscription to source datum
    const source = this.getDatum(sourceId);
    if (source) {
      source.addSubscription(subscriptionId);

      // If source was set in current generation, queue notification
      if (source.getSetGeneration() === this.currentGeneration) {
        this.notificationQueue.add(subscriptionId);
      }
    }

    return subscriptionId;
  }

  unsubscribe(subscriptionId: number): void {
    const subscription = this.subscriptions.get(subscriptionId);
    if (subscription) {
      const source = this.getDatum(subscription.sourceId);
      if (source) {
        source.removeSubscription(subscriptionId);
      }
      this.subscriptions.delete(subscriptionId);
      this.callbacks.delete(subscription.callbackId);
    }
  }

  // Event and notification processing
  putEvent(event: any): void {
    this.eventQueue.push(event);
  }

  drainEvents(): void {
    while (this.eventQueue.length > 0) {
      const event = this.eventQueue.shift();
      this.processEvent(event);
    }

    this.drainNotifications();
  }

  private processEvent(event: any): void {
    // TODO: Process event and update data
    console.log('[Runtime] Processing event:', event);
  }

  private drainNotifications(): void {
    const GEN_LIMIT = 1000;
    let generation = 0;

    while (this.notificationQueue.size > 0 && generation < GEN_LIMIT) {
      generation++;

      // Take current notifications
      const currentNotifications = Array.from(this.notificationQueue);
      this.notificationQueue.clear();
      this.currentGeneration++;

      // Process each notification
      for (const subscriptionId of currentNotifications) {
        const subscription = this.subscriptions.get(subscriptionId);
        if (subscription) {
          const callback = this.callbacks.get(subscription.callbackId);
          if (callback) {
            callback(subscriptionId);
          }
        }
      }
    }

    if (generation >= GEN_LIMIT) {
      console.error('[Runtime] Notification drain exceeded generation limit!');
    }
  }

  // Notification queue management
  queueNotification(subscriptionId: number): void {
    this.notificationQueue.add(subscriptionId);
  }

  getCurrentGeneration(): number {
    return this.currentGeneration;
  }
}
