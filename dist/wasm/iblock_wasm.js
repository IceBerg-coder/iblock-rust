/* @ts-self-types="./iblock_wasm.d.ts" */

/**
 * Request type flag constants exposed to JavaScript.
 */
export class RequestTypeFlags {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        RequestTypeFlagsFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_requesttypeflags_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    static get firstParty() {
        const ret = wasm.requesttypeflags_firstParty();
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    static get image() {
        const ret = wasm.requesttypeflags_image();
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    static get script() {
        const ret = wasm.requesttypeflags_script();
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    static get stylesheet() {
        const ret = wasm.requesttypeflags_stylesheet();
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    static get subdocument() {
        const ret = wasm.requesttypeflags_subdocument();
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    static get thirdParty() {
        const ret = wasm.requesttypeflags_thirdParty();
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    static get xhr() {
        const ret = wasm.requesttypeflags_xhr();
        return ret >>> 0;
    }
}
if (Symbol.dispose) RequestTypeFlags.prototype[Symbol.dispose] = RequestTypeFlags.prototype.free;

/**
 * Result of evaluating cosmetic selectors for a webpage.
 */
export class WasmCosmeticResult {
    static __wrap(ptr) {
        const obj = Object.create(WasmCosmeticResult.prototype);
        obj.__wbg_ptr = ptr;
        WasmCosmeticResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmCosmeticResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmcosmeticresult_free(ptr, 0);
    }
    /**
     * @returns {boolean}
     */
    hasSelectors() {
        const ret = wasm.wasmcosmeticresult_hasSelectors(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {string[]}
     */
    get proceduralSelectors() {
        const ret = wasm.wasmcosmeticresult_proceduralSelectors(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]);
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {string}
     */
    get stylesheet() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcosmeticresult_stylesheet(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) WasmCosmeticResult.prototype[Symbol.dispose] = WasmCosmeticResult.prototype.free;

/**
 * WebAssembly-exported Static Network Filtering Engine.
 */
export class WasmEngine {
    static __wrap(ptr) {
        const obj = Object.create(WasmEngine.prototype);
        obj.__wbg_ptr = ptr;
        WasmEngineFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmEngineFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmengine_free(ptr, 0);
    }
    /**
     * Configures a dynamic matrix firewall rule (e.g. `* * 3p-script block`).
     * @param {string} rule_line
     * @returns {boolean}
     */
    addDynamicRule(rule_line) {
        const ptr0 = passStringToWasm0(rule_line, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmengine_addDynamicRule(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Number of cosmetic rules currently active.
     * @returns {number}
     */
    cosmeticRuleCount() {
        const ret = wasm.wasmengine_cosmeticRuleCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Number of dynamic matrix firewall rules configured.
     * @returns {number}
     */
    dynamicRuleCount() {
        const ret = wasm.wasmengine_dynamicRuleCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Hydrates a complete WasmEngine instantaneously from an LZ4-compressed binary buffer.
     * @param {Uint8Array} bytes
     * @returns {WasmEngine}
     */
    static fromCompressedBytes(bytes) {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmengine_fromCompressedBytes(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmEngine.__wrap(ret[0]);
    }
    /**
     * Ingests a text filter list (e.g. EasyList) and returns the number of loaded rules.
     * @param {string} content
     * @returns {number}
     */
    loadList(content) {
        const ptr0 = passStringToWasm0(content, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmengine_loadList(this.__wbg_ptr, ptr0, len0);
        return ret >>> 0;
    }
    /**
     * Evaluates an HTTP network request against all loaded rules in sub-microsecond time.
     *
     * - `url`: Destination URL (e.g. "https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js")
     * - `source_domain`: Originating page domain (e.g. "example.com")
     * - `request_type`: Bitfield of RequestTypes (e.g. 1 for SCRIPT)
     * @param {string} url
     * @param {string} source_domain
     * @param {number} request_type
     * @returns {WasmMatchResult}
     */
    matchRequest(url, source_domain, request_type) {
        const ptr0 = passStringToWasm0(url, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(source_domain, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmengine_matchRequest(this.__wbg_ptr, ptr0, len0, ptr1, len1, request_type);
        return WasmMatchResult.__wrap(ret);
    }
    /**
     * Creates a new, empty WasmEngine instance.
     */
    constructor() {
        const ret = wasm.wasmengine_new();
        this.__wbg_ptr = ret;
        WasmEngineFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Evaluates all cosmetic CSS and procedural hiding rules for a given webpage hostname.
     * @param {string} hostname
     * @returns {WasmCosmeticResult}
     */
    retrieveCosmetics(hostname) {
        const ptr0 = passStringToWasm0(hostname, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmengine_retrieveCosmetics(this.__wbg_ptr, ptr0, len0);
        return WasmCosmeticResult.__wrap(ret);
    }
    /**
     * Total number of active rules compiled in the engine.
     * @returns {number}
     */
    ruleCount() {
        const ret = wasm.wasmengine_ruleCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Serializes and compresses the entire compiled rule database into an LZ4 binary buffer.
     * @returns {Uint8Array}
     */
    toCompressedBytes() {
        const ret = wasm.wasmengine_toCompressedBytes(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
}
if (Symbol.dispose) WasmEngine.prototype[Symbol.dispose] = WasmEngine.prototype.free;

/**
 * Result of evaluating an HTTP request.
 */
export class WasmMatchResult {
    static __wrap(ptr) {
        const obj = Object.create(WasmMatchResult.prototype);
        obj.__wbg_ptr = ptr;
        WasmMatchResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmMatchResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmmatchresult_free(ptr, 0);
    }
    /**
     * 0 = NoMatch, 1 = Block, 2 = Allow, 3 = Redirect
     * @returns {number}
     */
    get action() {
        const ret = wasm.wasmmatchresult_action(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {boolean}
     */
    isAllowed() {
        const ret = wasm.wasmmatchresult_isAllowed(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    isBlocked() {
        const ret = wasm.wasmmatchresult_isBlocked(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    isRedirected() {
        const ret = wasm.wasmmatchresult_isRedirected(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Resolves the redirection target into a ready-to-use Data URI.
     * @returns {string | undefined}
     */
    get redirectDataUri() {
        const ret = wasm.wasmmatchresult_redirectDataUri(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]);
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @returns {string | undefined}
     */
    get redirectTarget() {
        const ret = wasm.wasmmatchresult_redirectTarget(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]);
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
}
if (Symbol.dispose) WasmMatchResult.prototype[Symbol.dispose] = WasmMatchResult.prototype.free;
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_41e9ee4f547fc59a: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbindgen_generic_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./iblock_wasm_bg.js": import0,
    };
}

const RequestTypeFlagsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_requesttypeflags_free(ptr, 1));
const WasmCosmeticResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmcosmeticresult_free(ptr, 1));
const WasmEngineFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmengine_free(ptr, 1));
const WasmMatchResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmmatchresult_free(ptr, 1));

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_externrefs.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
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

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
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

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('iblock_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
