// Icon Blueprint - Displays vector icons

import { Runtime, Fragment } from '@frel/runtime';

export class IconFragment extends Fragment {
  private element: HTMLElement | null = null;
  private iconName: string;

  constructor(runtime: Runtime, parent: Fragment | null, iconName: string) {
    super(runtime, parent);
    this.iconName = iconName;
  }

  build(): void {
    // For POC, use a simple span with text
    // In production, this would load SVG icons
    this.element = document.createElement('span');
    this.element.className = `icon icon-${this.iconName}`;
    this.element.textContent = '◆'; // Placeholder
    this.element.style.display = 'inline-block';
  }

  updateIcon(iconName: string): void {
    this.iconName = iconName;
    if (this.element) {
      this.element.className = `icon icon-${iconName}`;
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
