// Image Blueprint - Displays raster images

import { Runtime, Fragment } from '@frel/runtime';

export class ImageFragment extends Fragment {
  private element: HTMLImageElement | null = null;
  private src: string;

  constructor(runtime: Runtime, parent: Fragment | null, src: string) {
    super(runtime, parent);
    this.src = src;
  }

  build(): void {
    this.element = document.createElement('img');
    this.element.src = this.src;
    this.element.style.display = 'block';
  }

  updateSrc(src: string): void {
    this.src = src;
    if (this.element) {
      this.element.src = src;
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
