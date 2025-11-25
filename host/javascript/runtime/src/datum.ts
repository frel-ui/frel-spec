// Datum - The fundamental unit of reactive data in Frel

import { DatumIdentity, Availability } from './types';

export class Datum {
  public readonly identityId: DatumIdentity;
  public readonly type: string;
  private structuralRev: number = 1;
  private carriedRev: number = 1;
  private setGeneration: number = 0;
  private availability: Availability = 'Ready';
  private error: string | null = null;
  private owner: DatumIdentity | null;
  private fields: Map<string, any> = new Map();
  private items: any[] | null = null;
  private entries: Map<any, any> | null = null;
  private subscriptions: Set<number> = new Set();

  constructor(identityId: DatumIdentity, type: string, owner: DatumIdentity | null) {
    this.identityId = identityId;
    this.type = type;
    this.owner = owner;
  }

  // Field access
  getField(name: string): any {
    return this.fields.get(name);
  }

  setField(name: string, value: any, generation: number): void {
    this.fields.set(name, value);
    this.structuralRev++;
    this.carriedRev++;
    this.setGeneration = generation;
  }

  // Collection access
  getItems(): any[] | null {
    return this.items;
  }

  setItems(items: any[], generation: number): void {
    this.items = items;
    this.structuralRev++;
    this.setGeneration = generation;
  }

  // Subscription management
  addSubscription(subscriptionId: number): void {
    this.subscriptions.add(subscriptionId);
  }

  removeSubscription(subscriptionId: number): void {
    this.subscriptions.delete(subscriptionId);
  }

  getSubscriptions(): Set<number> {
    return this.subscriptions;
  }

  // Revision tracking
  getStructuralRev(): number {
    return this.structuralRev;
  }

  getCarriedRev(): number {
    return this.carriedRev;
  }

  incrementCarriedRev(): void {
    this.carriedRev++;
  }

  // Generation tracking
  getSetGeneration(): number {
    return this.setGeneration;
  }

  // Availability
  getAvailability(): Availability {
    return this.availability;
  }

  setAvailability(availability: Availability, generation: number): void {
    if (this.availability !== availability) {
      this.availability = availability;
      this.structuralRev++;
      this.setGeneration = generation;
    }
  }

  // Error
  getError(): string | null {
    return this.error;
  }

  setError(error: string | null): void {
    this.error = error;
  }

  // Owner
  getOwner(): DatumIdentity | null {
    return this.owner;
  }
}
