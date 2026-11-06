/**
 * lean-agentic - Hash-consed dependent types with 150x faster equality
 *
 * Root entry point. Under Node.js this is the same API as `lean-agentic/node`,
 * plus an `init()` that resolves immediately (the Node WASM build loads
 * synchronously). Browsers should use `lean-agentic/web`.
 *
 * @module lean-agentic
 * @author ruv.io
 * @license Apache-2.0
 */

const node = require('./node.js');

/**
 * Kept for API compatibility: the Node WASM build is ready as soon as it is
 * required, so there is nothing to wait for.
 * @returns {Promise<void>}
 */
async function init() {}

module.exports = {
  init,
  LeanDemo: node.LeanDemo,
  createDemo: node.createDemo,
  quickStart: node.quickStart,
  setDebug: node.setDebug,
  wasm: node.wasm
};
