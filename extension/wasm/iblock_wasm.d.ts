/* tslint:disable */
/* eslint-disable */

/**
 * Request type flag constants exposed to JavaScript.
 */
export class RequestTypeFlags {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    static readonly firstParty: number;
    static readonly image: number;
    static readonly script: number;
    static readonly stylesheet: number;
    static readonly subdocument: number;
    static readonly thirdParty: number;
    static readonly xhr: number;
}

/**
 * Result of evaluating cosmetic selectors for a webpage.
 */
export class WasmCosmeticResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    hasSelectors(): boolean;
    readonly proceduralSelectors: string[];
    readonly stylesheet: string;
}

/**
 * WebAssembly-exported Static Network Filtering Engine.
 */
export class WasmEngine {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Configures a dynamic matrix firewall rule (e.g. `* * 3p-script block`).
     */
    addDynamicRule(rule_line: string): boolean;
    /**
     * Number of cosmetic rules currently active.
     */
    cosmeticRuleCount(): number;
    /**
     * Number of dynamic matrix firewall rules configured.
     */
    dynamicRuleCount(): number;
    /**
     * Hydrates a complete WasmEngine instantaneously from an LZ4-compressed binary buffer.
     */
    static fromCompressedBytes(bytes: Uint8Array): WasmEngine;
    /**
     * Ingests a text filter list (e.g. EasyList) and returns the number of loaded rules.
     */
    loadList(content: string): number;
    /**
     * Evaluates an HTTP network request against all loaded rules in sub-microsecond time.
     *
     * - `url`: Destination URL (e.g. "https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js")
     * - `source_domain`: Originating page domain (e.g. "example.com")
     * - `request_type`: Bitfield of RequestTypes (e.g. 1 for SCRIPT)
     */
    matchRequest(url: string, source_domain: string, request_type: number): WasmMatchResult;
    /**
     * Creates a new, empty WasmEngine instance.
     */
    constructor();
    /**
     * Evaluates all cosmetic CSS and procedural hiding rules for a given webpage hostname.
     */
    retrieveCosmetics(hostname: string): WasmCosmeticResult;
    /**
     * Total number of active rules compiled in the engine.
     */
    ruleCount(): number;
    /**
     * Serializes and compresses the entire compiled rule database into an LZ4 binary buffer.
     */
    toCompressedBytes(): Uint8Array;
}

/**
 * Result of evaluating an HTTP request.
 */
export class WasmMatchResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    isAllowed(): boolean;
    isBlocked(): boolean;
    isRedirected(): boolean;
    /**
     * 0 = NoMatch, 1 = Block, 2 = Allow, 3 = Redirect
     */
    readonly action: number;
    /**
     * Resolves the redirection target into a ready-to-use Data URI.
     */
    readonly redirectDataUri: string | undefined;
    readonly redirectTarget: string | undefined;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_requesttypeflags_free: (a: number, b: number) => void;
    readonly __wbg_wasmcosmeticresult_free: (a: number, b: number) => void;
    readonly __wbg_wasmengine_free: (a: number, b: number) => void;
    readonly __wbg_wasmmatchresult_free: (a: number, b: number) => void;
    readonly requesttypeflags_firstParty: () => number;
    readonly requesttypeflags_image: () => number;
    readonly requesttypeflags_script: () => number;
    readonly requesttypeflags_stylesheet: () => number;
    readonly requesttypeflags_subdocument: () => number;
    readonly requesttypeflags_thirdParty: () => number;
    readonly requesttypeflags_xhr: () => number;
    readonly wasmcosmeticresult_hasSelectors: (a: number) => number;
    readonly wasmcosmeticresult_proceduralSelectors: (a: number) => [number, number];
    readonly wasmcosmeticresult_stylesheet: (a: number) => [number, number];
    readonly wasmengine_addDynamicRule: (a: number, b: number, c: number) => number;
    readonly wasmengine_cosmeticRuleCount: (a: number) => number;
    readonly wasmengine_dynamicRuleCount: (a: number) => number;
    readonly wasmengine_fromCompressedBytes: (a: number, b: number) => [number, number, number];
    readonly wasmengine_loadList: (a: number, b: number, c: number) => number;
    readonly wasmengine_matchRequest: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly wasmengine_new: () => number;
    readonly wasmengine_retrieveCosmetics: (a: number, b: number, c: number) => number;
    readonly wasmengine_ruleCount: (a: number) => number;
    readonly wasmengine_toCompressedBytes: (a: number) => [number, number, number, number];
    readonly wasmmatchresult_action: (a: number) => number;
    readonly wasmmatchresult_isAllowed: (a: number) => number;
    readonly wasmmatchresult_isBlocked: (a: number) => number;
    readonly wasmmatchresult_isRedirected: (a: number) => number;
    readonly wasmmatchresult_redirectDataUri: (a: number) => [number, number];
    readonly wasmmatchresult_redirectTarget: (a: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_drop_slice: (a: number, b: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
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
