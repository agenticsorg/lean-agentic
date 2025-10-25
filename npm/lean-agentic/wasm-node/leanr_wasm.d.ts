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
