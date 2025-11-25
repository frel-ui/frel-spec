// Frel JavaScript Runtime
//
// Platform-independent reactive core that manages:
// - Datum storage and identity
// - Subscription management
// - Change propagation
// - Event and notification queues

export { Runtime } from './runtime';
export { Datum } from './datum';
export { Fragment } from './fragment';
export { Subscription } from './subscription';
export { Arena } from './arena';

export type {
  DatumIdentity,
  Availability,
  Selector,
  Callback,
} from './types';
