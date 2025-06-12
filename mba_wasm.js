let wasm;

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_2.set(idx, obj);
    return idx;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

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
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_2.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}
/**
 * @param {string} poly
 * @param {Width} bits
 * @param {string} alg
 * @returns {string}
 */
export function invert_poly(poly, bits, alg) {
    let deferred4_0;
    let deferred4_1;
    try {
        const ptr0 = passStringToWasm0(poly, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(alg, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.invert_poly(ptr0, len0, bits, ptr1, len1);
        var ptr3 = ret[0];
        var len3 = ret[1];
        if (ret[3]) {
            ptr3 = 0; len3 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred4_0 = ptr3;
        deferred4_1 = len3;
        return getStringFromWasm0(ptr3, len3);
    } finally {
        wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
    }
}

/**
 * @param {Width} bits
 * @returns {string}
 */
export function rand_poly(bits) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.rand_poly(bits);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}
/**
 * @param {ObfuscationConfig} cfg
 * @returns {string}
 */
export function obfuscate(cfg) {
    let deferred2_0;
    let deferred2_1;
    try {
        _assertClass(cfg, ObfuscationConfig);
        const ret = wasm.obfuscate(cfg.__wbg_ptr);
        var ptr1 = ret[0];
        var len1 = ret[1];
        if (ret[3]) {
            ptr1 = 0; len1 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred2_0 = ptr1;
        deferred2_1 = len1;
        return getStringFromWasm0(ptr1, len1);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {ObfLinReq} req
 * @returns {string}
 */
export function obfuscate_linear(req) {
    let deferred3_0;
    let deferred3_1;
    try {
        _assertClass(req, ObfLinReq);
        var ptr0 = req.__destroy_into_raw();
        const ret = wasm.obfuscate_linear(ptr0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {string} expr
 * @param {Width} bits
 * @returns {string}
 */
export function normalize_op(expr, bits) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(expr, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.normalize_op(ptr0, len0, bits);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} matrix_str
 * @param {Width} bit
 * @returns {SolveTrace}
 */
export function solve_congruences(matrix_str, bit) {
    const ptr0 = passStringToWasm0(matrix_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.solve_congruences(ptr0, len0, bit);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return SolveTrace.__wrap(ret[0]);
}

/**
 * Determines how the result will be printed.
 * @enum {0 | 1 | 2 | 3}
 */
export const Printer = Object.freeze({
    /**
     * Some default.
     */
    Default: 0, "0": "Default",
    /**
     * Print a C function.
     */
    C: 1, "1": "C",
    /**
     * Print a rust function.
     */
    Rust: 2, "2": "Rust",
    /**
     * Tex expression.
     */
    Tex: 3, "3": "Tex",
});
/**
 * @enum {0 | 1 | 2 | 3 | 4}
 */
export const Width = Object.freeze({
    U8: 0, "0": "U8",
    U16: 1, "1": "U16",
    U32: 2, "2": "U32",
    U64: 3, "3": "U64",
    U128: 4, "4": "U128",
});

const ObfLinReqFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_obflinreq_free(ptr >>> 0, 1));
/**
 * Obfuscation settings.
 */
export class ObfLinReq {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ObfLinReqFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_obflinreq_free(ptr, 0);
    }
    /**
     * The integer width.
     * @returns {Width}
     */
    get bits() {
        const ret = wasm.__wbg_get_obflinreq_bits(this.__wbg_ptr);
        return ret;
    }
    /**
     * The integer width.
     * @param {Width} arg0
     */
    set bits(arg0) {
        wasm.__wbg_set_obflinreq_bits(this.__wbg_ptr, arg0);
    }
    /**
     * Should the solution be randomized.
     * @returns {boolean}
     */
    get randomize() {
        const ret = wasm.__wbg_get_obflinreq_randomize(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Should the solution be randomized.
     * @param {boolean} arg0
     */
    set randomize(arg0) {
        wasm.__wbg_set_obflinreq_randomize(this.__wbg_ptr, arg0);
    }
    /**
     * How to print the result.
     * @returns {Printer}
     */
    get printer() {
        const ret = wasm.__wbg_get_obflinreq_printer(this.__wbg_ptr);
        return ret;
    }
    /**
     * How to print the result.
     * @param {Printer} arg0
     */
    set printer(arg0) {
        wasm.__wbg_set_obflinreq_printer(this.__wbg_ptr, arg0);
    }
    constructor() {
        const ret = wasm.obflinreq_new();
        this.__wbg_ptr = ret >>> 0;
        ObfLinReqFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {string} expr
     */
    set expr(expr) {
        const ptr0 = passStringToWasm0(expr, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.obflinreq_set_expr(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {string} op
     */
    add_op(op) {
        const ptr0 = passStringToWasm0(op, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.obflinreq_add_op(this.__wbg_ptr, ptr0, len0);
    }
}

const ObfuscationConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_obfuscationconfig_free(ptr >>> 0, 1));

export class ObfuscationConfig {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ObfuscationConfigFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_obfuscationconfig_free(ptr, 0);
    }
    /**
     * The integer width.
     * @returns {Width}
     */
    get width() {
        const ret = wasm.__wbg_get_obfuscationconfig_width(this.__wbg_ptr);
        return ret;
    }
    /**
     * The integer width.
     * @param {Width} arg0
     */
    set width(arg0) {
        wasm.__wbg_set_obfuscationconfig_width(this.__wbg_ptr, arg0);
    }
    /**
     * How to print the result.
     * @returns {Printer}
     */
    get printer() {
        const ret = wasm.__wbg_get_obfuscationconfig_printer(this.__wbg_ptr);
        return ret;
    }
    /**
     * How to print the result.
     * @param {Printer} arg0
     */
    set printer(arg0) {
        wasm.__wbg_set_obfuscationconfig_printer(this.__wbg_ptr, arg0);
    }
    /**
     * The number of auxiliary variables to use.
     * @returns {number}
     */
    get aux_vars() {
        const ret = wasm.__wbg_get_obfuscationconfig_aux_vars(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * The number of auxiliary variables to use.
     * @param {number} arg0
     */
    set aux_vars(arg0) {
        wasm.__wbg_set_obfuscationconfig_aux_vars(this.__wbg_ptr, arg0);
    }
    /**
     * The depth of the rewrite expressions.
     * Ultimately, we should probably just generate a random truth table
     * and find a simple expression for it with `egg`.
     * @returns {number}
     */
    get rewrite_depth() {
        const ret = wasm.__wbg_get_obfuscationconfig_rewrite_depth(this.__wbg_ptr);
        return ret;
    }
    /**
     * The depth of the rewrite expressions.
     * Ultimately, we should probably just generate a random truth table
     * and find a simple expression for it with `egg`.
     * @param {number} arg0
     */
    set rewrite_depth(arg0) {
        wasm.__wbg_set_obfuscationconfig_rewrite_depth(this.__wbg_ptr, arg0);
    }
    /**
     * The number of rewrite expressions to use.
     * @returns {number}
     */
    get rewrite_count() {
        const ret = wasm.__wbg_get_obfuscationconfig_rewrite_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * The number of rewrite expressions to use.
     * @param {number} arg0
     */
    set rewrite_count(arg0) {
        wasm.__wbg_set_obfuscationconfig_rewrite_count(this.__wbg_ptr, arg0);
    }
    constructor() {
        const ret = wasm.obfuscationconfig_new();
        this.__wbg_ptr = ret >>> 0;
        ObfuscationConfigFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {string} expr
     */
    set expr(expr) {
        const ptr0 = passStringToWasm0(expr, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.obfuscationconfig_set_expr(this.__wbg_ptr, ptr0, len0);
    }
}

const SolveTraceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_solvetrace_free(ptr >>> 0, 1));
/**
 * Stores the intermediate results during the computation of the solution.
 */
export class SolveTrace {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(SolveTrace.prototype);
        obj.__wbg_ptr = ptr;
        SolveTraceFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SolveTraceFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_solvetrace_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get diag() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.solvetrace_diag(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get scalar_system() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.solvetrace_scalar_system(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get linear_solutions() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.solvetrace_linear_solutions(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get vector_solution() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.solvetrace_vector_solution(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get final_solution() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.solvetrace_final_solution(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_call_672a4d21634d4a24 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.call(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getRandomValues_3c9c0d586e575a16 = function() { return handleError(function (arg0, arg1) {
        globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
    }, arguments) };
    imports.wbg.__wbg_instanceof_Window_def73ea0955fc569 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Window;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_log_d4317785fc4ef9c8 = function(arg0, arg1) {
        console.log(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_newnoargs_105ed471475aaf50 = function(arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_now_d18023d54d4e5500 = function(arg0) {
        const ret = arg0.now();
        return ret;
    };
    imports.wbg.__wbg_performance_c185c0cdc2766575 = function(arg0) {
        const ret = arg0.performance;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_88a902d13a557d07 = function() {
        const ret = typeof global === 'undefined' ? null : global;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_THIS_56578be7e9f832b0 = function() {
        const ret = typeof globalThis === 'undefined' ? null : globalThis;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_SELF_37c5d418e4bf5819 = function() {
        const ret = typeof self === 'undefined' ? null : self;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_WINDOW_5de37043a91a9c40 = function() {
        const ret = typeof window === 'undefined' ? null : window;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_export_2;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
        ;
    };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        const ret = arg0 === undefined;
        return ret;
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };

    return imports;
}

function __wbg_init_memory(imports, memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('mba_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
