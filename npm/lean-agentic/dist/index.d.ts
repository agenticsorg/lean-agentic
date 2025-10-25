/**
 * lean-agentic TypeScript definitions
 *
 * Hash-consed dependent types with 150x faster equality
 *
 * @module lean-agentic
 * @author ruv.io
 * @license Apache-2.0
 */

/**
 * Initialize the WASM module
 */
export function init(): Promise<void>;

/**
 * Main interface for lean-agentic theorem prover
 */
export class LeanDemo {
  constructor();

  /**
   * Create identity function: λx:Type. x
   * @returns JSON representation of the identity function
   */
  createIdentity(): string;

  /**
   * Create and verify an application
   * @returns JSON representation of the application
   */
  createApplication(): string;

  /**
   * Demonstrate hash-consing by creating identical terms
   * @returns JSON showing term equality
   */
  demonstrateHashConsing(): string;
}

/**
 * Create a new LeanDemo instance
 */
export function createDemo(): LeanDemo;

/**
 * Quick start: Create identity function
 */
export function quickStart(): Promise<string>;

/**
 * WASM module exports
 */
export namespace wasm {
  export class LeanDemo {
    constructor();
    create_identity(): string;
    create_application(): string;
    demonstrate_hash_consing(): string;
  }
}

/**
 * Default export
 */
declare const _default: {
  init: typeof init;
  LeanDemo: typeof LeanDemo;
  createDemo: typeof createDemo;
  quickStart: typeof quickStart;
  wasm: typeof wasm;
};

export default _default;
