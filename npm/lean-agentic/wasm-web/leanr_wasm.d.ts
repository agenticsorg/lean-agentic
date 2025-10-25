/* tslint:disable */
/* eslint-disable */
/**
 * Simple greeting function for testing WASM works
 */
export function greet(name: string): string;
/**
 * Get version information
 */
export function getVersion(): string;
/**
 * Demo struct showing hash-consing performance in WASM
 */
export class LeanDemo {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Create a new Lean demo instance
   */
  constructor();
  /**
   * Demonstrate hash-consing: creating same term twice reuses memory
   * Returns true if successful
   */
  createVariable(index: number): boolean;
  /**
   * Create two identical variables and verify they share the same ID
   * Returns true if hash-consing worked (same IDs)
   */
  demonstrateHashConsing(): boolean;
  /**
   * Get statistics about the arena (number of unique terms)
   */
  getStats(): string;
  /**
   * Create a simple type (Type universe)
   */
  createType(): boolean;
  /**
   * Create a lambda abstraction (x : Type) => x
   */
  createIdentityFunction(): boolean;
  /**
   * Verify that hash-consing provides O(1) equality
   */
  benchmarkEquality(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_leandemo_free: (a: number, b: number) => void;
  readonly leandemo_new: () => number;
  readonly leandemo_createVariable: (a: number, b: number) => number;
  readonly leandemo_demonstrateHashConsing: (a: number) => number;
  readonly leandemo_getStats: (a: number) => [number, number];
  readonly leandemo_createType: (a: number) => number;
  readonly leandemo_createIdentityFunction: (a: number) => number;
  readonly leandemo_benchmarkEquality: (a: number) => [number, number];
  readonly greet: (a: number, b: number) => [number, number];
  readonly getVersion: () => [number, number];
  readonly __wbindgen_export_0: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
