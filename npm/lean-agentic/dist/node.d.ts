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
  /** JSON describing the identity function λx:Type. x */
  createIdentity(): string;
  /** JSON describing an application (var0 var1) */
  createApplication(): string;
  /** JSON showing that identical terms share one TermId */
  demonstrateHashConsing(): string;
  /** JSON with arena statistics */
  getStats(): string;
  benchmarkEquality(): string;
}

export function createDemo(): LeanDemo;
export function quickStart(): Promise<string>;

/**
 * Turn the WASM module's diagnostic logging on or off. Logs go to stderr.
 * Off by default; LEAN_AGENTIC_DEBUG=1 turns it on at load time.
 */
export function setDebug(enabled: boolean): void;

/** Raw wasm-bindgen module (types: wasm-node/leanr_wasm.d.ts) */
export const wasm: {
  LeanDemo: new () => {
    free(): void;
    createVariable(index: number): boolean;
    demonstrateHashConsing(): boolean;
    getStats(): string;
    createType(): boolean;
    createIdentityFunction(): boolean;
    benchmarkEquality(): string;
  };
  greet(name: string): string;
  getVersion(): string;
  setDebug(enabled: boolean): void;
};

declare const _default: {
  LeanDemo: typeof LeanDemo;
  createDemo: typeof createDemo;
  quickStart: typeof quickStart;
  setDebug: typeof setDebug;
  wasm: typeof wasm;
};

export default _default;
