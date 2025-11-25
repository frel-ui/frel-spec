// Counter example bootstrap
// This file would import the compiled Frel code once the compiler is complete

import { BrowserAdapter } from '@frel/browser';

// TODO: Import compiled Counter blueprint
// import { Counter } from './src/counter.js';

console.log('Frel Counter Example');
console.log('Waiting for compiler implementation...');

const root = document.getElementById('app');
const adapter = new BrowserAdapter(root);

// TODO: Mount Counter blueprint
// const counter = new Counter(adapter.getRuntime(), null);
// adapter.mount(counter);

// For now, show a placeholder
root.innerHTML = `
  <div style="padding: 32px; text-align: center;">
    <h1>Frel Counter Example</h1>
    <p style="margin-top: 16px; color: #666;">
      Compiler implementation in progress...
    </p>
    <p style="margin-top: 8px; color: #999; font-size: 14px;">
      See <code>examples/counter/src/counter.frel</code> for the source code.
    </p>
  </div>
`;
