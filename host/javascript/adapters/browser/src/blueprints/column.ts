// Column Blueprint - Vertical layout container

import { Runtime, Fragment } from '@frel/runtime';

export class ColumnFragment extends Fragment {
  private element: HTMLElement | null = null;

  build(): void {
    this.element = document.createElement('div');
    this.element.style.display = 'flex';
    this.element.style.flexDirection = 'column';
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
