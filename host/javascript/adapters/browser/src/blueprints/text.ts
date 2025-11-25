// Text Blueprint - Renders text content

import { Runtime, Fragment } from '@frel/runtime';

export class TextFragment extends Fragment {
  private element: HTMLElement | null = null;
  private content: string;

  constructor(runtime: Runtime, parent: Fragment | null, content: string) {
    super(runtime, parent);
    this.content = content;
  }

  build(): void {
    // Create DOM element
    this.element = document.createElement('span');
    this.element.textContent = this.content;
    this.element.style.display = 'inline-block';

    // TODO: Subscribe to content changes
  }

  updateContent(content: string): void {
    this.content = content;
    if (this.element) {
      this.element.textContent = content;
    }
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
