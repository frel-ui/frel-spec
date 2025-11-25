// Subscription - Represents a reactive dependency between two data

import { DatumIdentity, Selector } from './types';

export class Subscription {
  public readonly subscriptionId: number;
  public readonly sourceId: DatumIdentity;
  public readonly targetId: DatumIdentity;
  public readonly selector: Selector;
  public readonly callbackId: number;

  constructor(
    subscriptionId: number,
    sourceId: DatumIdentity,
    targetId: DatumIdentity,
    selector: Selector,
    callbackId: number
  ) {
    this.subscriptionId = subscriptionId;
    this.sourceId = sourceId;
    this.targetId = targetId;
    this.selector = selector;
    this.callbackId = callbackId;
  }

  matches(change: { type: 'structural' | 'carried'; key?: string | number }): boolean {
    switch (this.selector.type) {
      case 'Everything':
        return true;

      case 'Structural':
        return change.type === 'structural';

      case 'Carried':
        return change.type === 'carried';

      case 'Key':
        return change.key !== undefined && change.key === this.selector.key;

      default:
        return false;
    }
  }
}
