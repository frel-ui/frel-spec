// Core types for the Frel runtime

export type DatumIdentity = number;

export type Availability = 'Loading' | 'Ready' | 'Error';

export type Selector =
  | { type: 'Everything' }
  | { type: 'Structural' }
  | { type: 'Carried' }
  | { type: 'Key'; key: string | number };

export type Callback = (subscriptionId: number) => void;

export interface DatumData {
  identityId: DatumIdentity;
  type: string;
  structuralRev: number;
  carriedRev: number;
  setGeneration: number;
  availability: Availability;
  error: string | null;
  owner: DatumIdentity | null;
  fields: Record<string, any> | null;
  items: any[] | null;
  entries: Map<any, any> | null;
  subscriptions: number[];
}

export interface SubscriptionData {
  subscriptionId: number;
  sourceId: DatumIdentity;
  targetId: DatumIdentity;
  selector: Selector;
  callbackId: number;
}
