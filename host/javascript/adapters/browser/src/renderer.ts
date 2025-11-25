// DOM Renderer - Manages the actual DOM tree

import { Runtime, Fragment } from '@frel/runtime';

export class DOMRenderer {
  private runtime: Runtime;
  private rootElement: HTMLElement;
  private rootFragment: Fragment | null = null;
  private nodeMap: Map<number, HTMLElement> = new Map();

  constructor(runtime: Runtime, rootElement: HTMLElement) {
    this.runtime = runtime;
    this.rootElement = rootElement;
  }

  mountRoot(fragment: Fragment): void {
    if (this.rootFragment) {
      this.unmountRoot();
    }

    this.rootFragment = fragment;
    fragment.build();

    // Initial render
    this.render();
  }

  unmountRoot(): void {
    if (this.rootFragment) {
      this.rootFragment.destroy();
      this.rootFragment = null;
    }

    this.rootElement.innerHTML = '';
    this.nodeMap.clear();
  }

  render(): void {
    // TODO: Implement actual rendering logic
    console.log('[Renderer] Rendering...');
  }

  // Node management
  createNode(fragmentId: number, tagName: string): HTMLElement {
    const element = document.createElement(tagName);
    this.nodeMap.set(fragmentId, element);
    return element;
  }

  getNode(fragmentId: number): HTMLElement | undefined {
    return this.nodeMap.get(fragmentId);
  }

  removeNode(fragmentId: number): void {
    const node = this.nodeMap.get(fragmentId);
    if (node) {
      node.remove();
      this.nodeMap.delete(fragmentId);
    }
  }

  // Style application
  applyStyle(element: HTMLElement, property: string, value: string): void {
    (element.style as any)[property] = value;
  }

  applyStyles(element: HTMLElement, styles: Record<string, string>): void {
    Object.entries(styles).forEach(([property, value]) => {
      this.applyStyle(element, property, value);
    });
  }

  // Layout application
  applyLayout(element: HTMLElement, layout: any): void {
    // TODO: Apply layout based on instructions
    if (layout.width !== undefined) {
      element.style.width = `${layout.width}px`;
    }
    if (layout.height !== undefined) {
      element.style.height = `${layout.height}px`;
    }
  }
}
