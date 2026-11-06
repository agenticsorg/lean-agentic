/**
 * lean-agentic TypeScript definitions
 *
 * Hash-consed dependent types with 150x faster equality. Under Node.js the
 * root entry point has the same API as `lean-agentic/node`, plus `init()`.
 * Browsers should use `lean-agentic/web`.
 *
 * @module lean-agentic
 * @author ruv.io
 * @license Apache-2.0
 */

import { LeanDemo, createDemo, quickStart, setDebug, wasm } from './node';

export { LeanDemo, createDemo, quickStart, setDebug, wasm };

/**
 * Kept for API compatibility. Resolves immediately: the Node WASM build is
 * ready as soon as it is loaded.
 */
export function init(): Promise<void>;

declare const _default: {
  init: typeof init;
  LeanDemo: typeof LeanDemo;
  createDemo: typeof createDemo;
  quickStart: typeof quickStart;
  setDebug: typeof setDebug;
  wasm: typeof wasm;
};

export default _default;
