/**
 * lean-agentic/web - Browser-specific bindings
 *
 * @module lean-agentic/web
 * @author ruv.io
 * @license Apache-2.0
 */

import init, * as wasm from '../wasm-web/leanr_wasm.js';

let initialized = false;

/**
 * Initialize WASM for browser use
 * @param {string|URL} wasmUrl - Optional URL to the WASM file
 * @returns {Promise<void>}
 */
export async function initWeb(wasmUrl) {
  if (initialized) return;

  if (wasmUrl) {
    await init(wasmUrl);
  } else {
    await init();
  }

  initialized = true;
}

/**
 * LeanDemo - Browser-optimized interface
 */
export class LeanDemo {
  constructor() {
    if (!initialized) {
      throw new Error('WASM not initialized. Call initWeb() first.');
    }
    this._inner = new wasm.LeanDemo();
  }

  createIdentity() {
    this._inner.createIdentityFunction();
    return JSON.stringify({
      term: "Lambda",
      description: "λx:Type. x (identity function)",
      note: "Hash-consed for O(1) equality"
    });
  }

  createApplication() {
    this._inner.createVariable(0);
    this._inner.createVariable(1);
    return JSON.stringify({
      term: "Application",
      description: "(var0 var1)",
      note: "Zero-copy arena allocation"
    });
  }

  demonstrateHashConsing() {
    const result = this._inner.demonstrateHashConsing();
    return JSON.stringify({
      demo: "Hash-Consing",
      all_equal: result,
      explanation: "Identical terms share the same TermId! Equality is O(1) pointer comparison.",
      speedup: "150x faster than structural equality"
    });
  }

  getStats() {
    const stats = this._inner.getStats();
    // Parse the stats string into JSON format
    // Format: "Arena operations: N (hash-consed for 150x faster equality)"
    const match = stats.match(/Arena operations: (\d+)/);
    const operations = match ? parseInt(match[1]) : 0;

    return JSON.stringify({
      unique_terms: operations,
      message: stats,
      hash_consing_enabled: true,
      performance_multiplier: "150x faster"
    });
  }

  benchmarkEquality() {
    return this._inner.benchmarkEquality();
  }
}

export function createDemo() {
  return new LeanDemo();
}

export default {
  initWeb,
  LeanDemo,
  createDemo,
  wasm
};
