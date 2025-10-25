
let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

function decodeText(ptr, len) {
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    }
}

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}
/**
 * Simple greeting function for testing WASM works
 * @param {string} name
 * @returns {string}
 */
exports.greet = function(name) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.greet(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
};

/**
 * Get version information
 * @returns {string}
 */
exports.getVersion = function() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.getVersion();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

const LeanDemoFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_leandemo_free(ptr >>> 0, 1));
/**
 * Demo struct showing hash-consing performance in WASM
 */
class LeanDemo {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LeanDemoFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_leandemo_free(ptr, 0);
    }
    /**
     * Create a new Lean demo instance
     */
    constructor() {
        const ret = wasm.leandemo_new();
        this.__wbg_ptr = ret >>> 0;
        LeanDemoFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Demonstrate hash-consing: creating same term twice reuses memory
     * Returns true if successful
     * @param {number} index
     * @returns {boolean}
     */
    createVariable(index) {
        const ret = wasm.leandemo_createVariable(this.__wbg_ptr, index);
        return ret !== 0;
    }
    /**
     * Create two identical variables and verify they share the same ID
     * Returns true if hash-consing worked (same IDs)
     * @returns {boolean}
     */
    demonstrateHashConsing() {
        const ret = wasm.leandemo_demonstrateHashConsing(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Get statistics about the arena (number of unique terms)
     * @returns {string}
     */
    getStats() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.leandemo_getStats(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Create a simple type (Type universe)
     * @returns {boolean}
     */
    createType() {
        const ret = wasm.leandemo_createType(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Create a lambda abstraction (x : Type) => x
     * @returns {boolean}
     */
    createIdentityFunction() {
        const ret = wasm.leandemo_createIdentityFunction(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Verify that hash-consing provides O(1) equality
     * @returns {string}
     */
    benchmarkEquality() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.leandemo_benchmarkEquality(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) LeanDemo.prototype[Symbol.dispose] = LeanDemo.prototype.free;

exports.LeanDemo = LeanDemo;

exports.__wbg_log_6c7b5f4f00b8ce3f = function(arg0) {
    console.log(arg0);
};

exports.__wbg_wbindgenthrow_451ec1a8469d7eb6 = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

exports.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
    // Cast intrinsic for `Ref(String) -> Externref`.
    const ret = getStringFromWasm0(arg0, arg1);
    return ret;
};

exports.__wbindgen_init_externref_table = function() {
    const table = wasm.__wbindgen_export_0;
    const offset = table.grow(4);
    table.set(0, undefined);
    table.set(offset + 0, undefined);
    table.set(offset + 1, null);
    table.set(offset + 2, true);
    table.set(offset + 3, false);
    ;
};

const wasmPath = `${__dirname}/leanr_wasm_bg.wasm`;
const wasmBytes = require('fs').readFileSync(wasmPath);
const wasmModule = new WebAssembly.Module(wasmBytes);
const wasm = exports.__wasm = new WebAssembly.Instance(wasmModule, imports).exports;

wasm.__wbindgen_start();

