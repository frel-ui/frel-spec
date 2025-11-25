// Arena - Collection container for schemes with identity

import { DatumIdentity } from './types';

export class Arena {
  private schemeName: string;
  private items: Map<DatumIdentity, any> = new Map();

  constructor(schemeName: string) {
    this.schemeName = schemeName;
  }

  add(id: DatumIdentity, item: any): void {
    this.items.set(id, item);
  }

  remove(id: DatumIdentity): boolean {
    return this.items.delete(id);
  }

  get(id: DatumIdentity): any | undefined {
    return this.items.get(id);
  }

  has(id: DatumIdentity): boolean {
    return this.items.has(id);
  }

  all(): any[] {
    return Array.from(this.items.values());
  }

  clear(): void {
    this.items.clear();
  }

  get size(): number {
    return this.items.size;
  }
}
