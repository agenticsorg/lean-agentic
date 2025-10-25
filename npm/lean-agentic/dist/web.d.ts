/**
 * lean-agentic/web TypeScript definitions
 * Browser-specific bindings
 *
 * @module lean-agentic/web
 * @author ruv.io
 * @license Apache-2.0
 */

/**
 * Initialize WASM for browser use
 * @param wasmUrl - Optional URL to the WASM file
 */
export function initWeb(wasmUrl?: string | URL): Promise<void>;

/**
 * Browser-optimized interface for lean-agentic
 */
export class LeanDemo {
  constructor();
  createIdentity(): string;
  createApplication(): string;
  demonstrateHashConsing(): string;
}

export function createDemo(): LeanDemo;

export namespace wasm {
  export class LeanDemo {
    constructor();
    create_identity(): string;
    create_application(): string;
    demonstrate_hash_consing(): string;
  }
}

declare const _default: {
  initWeb: typeof initWeb;
  LeanDemo: typeof LeanDemo;
  createDemo: typeof createDemo;
  wasm: typeof wasm;
};

export default _default;
