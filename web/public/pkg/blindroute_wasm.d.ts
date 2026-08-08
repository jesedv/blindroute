/* tslint:disable */
/* eslint-disable */

export function ckks_engine_info(): any;

export function demo_bfv_calc(a: bigint, b: bigint, op: string): any;

export function demo_ckks_calc(a: number, b: number, op: string): any;

export function engine_info(): any;

export function init(): void;

export function private_mean(values: BigUint64Array, seed: bigint): any;

export function run_ckks_self_test(): any;

export function run_self_test(): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly ckks_engine_info: () => any;
    readonly demo_bfv_calc: (a: bigint, b: bigint, c: number, d: number) => any;
    readonly demo_ckks_calc: (a: number, b: number, c: number, d: number) => any;
    readonly engine_info: () => any;
    readonly private_mean: (a: number, b: number, c: bigint) => any;
    readonly run_ckks_self_test: () => any;
    readonly run_self_test: () => any;
    readonly init: () => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
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
