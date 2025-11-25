// Fragment - Runtime instance of a blueprint

import { Runtime } from './runtime';
import { DatumIdentity } from './types';

export abstract class Fragment {
  protected runtime: Runtime;
  protected parent: Fragment | null;
  protected closureId: DatumIdentity;
  protected children: Fragment[] = [];

  constructor(runtime: Runtime, parent: Fragment | null) {
    this.runtime = runtime;
    this.parent = parent;

    // Create closure datum for this fragment
    const closure = runtime.createDatum('Closure', parent?.closureId ?? null);
    this.closureId = closure.identityId;
  }

  abstract build(): void;

  destroy(): void {
    // Destroy children first
    for (const child of this.children) {
      child.destroy();
    }
    this.children = [];

    // TODO: Unsubscribe from all subscriptions
    // TODO: Remove closure datum
  }

  getClosureId(): DatumIdentity {
    return this.closureId;
  }

  protected addChild(child: Fragment): void {
    this.children.push(child);
  }

  protected removeChild(child: Fragment): void {
    const index = this.children.indexOf(child);
    if (index !== -1) {
      this.children.splice(index, 1);
    }
  }
}
