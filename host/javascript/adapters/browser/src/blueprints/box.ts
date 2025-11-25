// Box Blueprint - Positional container

import { Runtime, Fragment } from '@frel/runtime';

export class BoxFragment extends Fragment {
  private element: HTMLElement | null = null;

  build(): void {
    this.element = document.createElement('div');
    this.element.style.position = 'relative';
    this.element.style.display = 'block';
  }

  getElement(): HTMLElement | null {
    return this.element;
  }

  destroy(): void {
    super.destroy();
    if (this.element) {
      this.element.remove();
      this.element = null;
    }
  }
}
