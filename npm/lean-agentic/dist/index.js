/**
 * lean-agentic - Hash-consed dependent types with 150x faster equality
 *
 * @module lean-agentic
 * @author ruv.io
 * @license Apache-2.0
 */

const wasm = require('../wasm/leanr_wasm.js');

/**
 * Initialize the WASM module
 * @returns {Promise<void>}
 */
export async function init() {
  // WASM is already initialized when imported
  return Promise.resolve();
}

/**
 * LeanDemo - Main interface for lean-agentic
 */
class LeanDemo {
  constructor() {
    this._inner = new wasm.LeanDemo();
  }

  /**
   * Create identity function: λx:Type. x
   * @returns {string} JSON representation of the identity function
   */
  createIdentity() {
    return this._inner.create_identity();
  }

  /**
   * Create and verify an application
   * @returns {string} JSON representation of the application
   */
  createApplication() {
    return this._inner.create_application();
  }

  /**
   * Demonstrate hash-consing by creating identical terms
   * @returns {string} JSON showing term equality
   */
  demonstrateHashConsing() {
    return this._inner.demonstrate_hash_consing();
  }
}

/**
 * Create a new LeanDemo instance
 * @returns {LeanDemo}
 */
function createDemo() {
  return new LeanDemo();
}

/**
 * Quick start: Create identity function
 * @returns {Promise<string>}
 */
export async function quickStart() {
  await init();
  const demo = createDemo();
  return demo.createIdentity();
}

// Re-export WASM types
module.exports = { wasm };

// Default export
module.exports = {
  init,
  LeanDemo,
  createDemo,
  quickStart,
  wasm
};
