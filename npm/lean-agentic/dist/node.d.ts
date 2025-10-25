/**
 * lean-agentic/node TypeScript definitions
 * Node.js-specific bindings
 *
 * @module lean-agentic/node
 * @author ruv.io
 * @license Apache-2.0
 */

/**
 * Node.js interface for lean-agentic
 */
export class LeanDemo {
  constructor();
  createIdentity(): string;
  createApplication(): string;
  demonstrateHashConsing(): string;
}

export function createDemo(): LeanDemo;
export function quickStart(): Promise<string>;

export namespace wasm {
  export class LeanDemo {
    constructor();
    create_identity(): string;
    create_application(): string;
    demonstrate_hash_consing(): string;
  }
}
