// Browser Adapter - Entry point for Frel in browser environment

import { Runtime } from '@frel/runtime';
import { DOMRenderer } from './renderer';

export class BrowserAdapter {
  private runtime: Runtime;
  private renderer: DOMRenderer;
  private rootElement: HTMLElement;

  constructor(rootElement: HTMLElement) {
    this.runtime = new Runtime();
    this.rootElement = rootElement;
    this.renderer = new DOMRenderer(this.runtime, rootElement);

    this.setupEventHandlers();
  }

  private setupEventHandlers(): void {
    // Set up DOM event listeners that feed into runtime
    document.addEventListener('click', (e) => this.handlePointerEvent('click', e));
    document.addEventListener('input', (e) => this.handleInputEvent(e));
    // TODO: Add more event listeners
  }

  private handlePointerEvent(type: string, event: MouseEvent): void {
    this.runtime.putEvent({
      type: 'pointer',
      subtype: type,
      target: event.target,
      x: event.clientX,
      y: event.clientY,
      button: event.button,
    });

    this.runtime.drainEvents();
  }

  private handleInputEvent(event: Event): void {
    this.runtime.putEvent({
      type: 'input',
      target: event.target,
      value: (event.target as HTMLInputElement).value,
    });

    this.runtime.drainEvents();
  }

  getRuntime(): Runtime {
    return this.runtime;
  }

  getRenderer(): DOMRenderer {
    return this.renderer;
  }

  mount(rootFragment: any): void {
    this.renderer.mountRoot(rootFragment);
  }

  unmount(): void {
    this.renderer.unmountRoot();
  }
}
