/* tslint:disable */
/* eslint-disable */

export function apply_human_move(x: number, y: number, w: number, h: number): any;

export function compute_bot_move(d1: number, d2: number, effort?: string | null, include_telemetry?: boolean | null): Promise<any>;

export function get_legal_placements(w: number, h: number): any;

export function has_any_legal_move(d1: number, d2: number): boolean;

export function init_model(onnx_bytes: Uint8Array | number[]): Promise<void>;

export function init_nnue(nnue_bytes: Uint8Array | number[]): void;

export function inspect_position(): any;

export function is_nnue_loaded(): boolean;

export function new_game(komi?: number | null): any;

export function pass_turn(): any;

export function preview_move_capture(x: number, y: number, w: number, h: number): any;

export function restore_state(grid: Uint8Array | number[], p1_score: number, p2_score: number, turn: number, consecutive_passes: number, komi: number): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly apply_human_move: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly compute_bot_move: (a: number, b: number, c: number, d: number, e: number) => any;
    readonly get_legal_placements: (a: number, b: number) => [number, number, number];
    readonly has_any_legal_move: (a: number, b: number) => number;
    readonly init_model: (a: number, b: number) => any;
    readonly init_nnue: (a: number, b: number) => [number, number];
    readonly inspect_position: () => [number, number, number];
    readonly is_nnue_loaded: () => number;
    readonly new_game: (a: number) => [number, number, number];
    readonly pass_turn: () => [number, number, number];
    readonly preview_move_capture: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly restore_state: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly wasm_bindgen_bcebade823f86b51___convert__closures_____invoke___js_sys_829c32a5b7245d83___Function_fn_wasm_bindgen_bcebade823f86b51___JsValue_____wasm_bindgen_bcebade823f86b51___sys__Undefined___js_sys_829c32a5b7245d83___Function_fn_wasm_bindgen_bcebade823f86b51___JsValue_____wasm_bindgen_bcebade823f86b51___sys__Undefined_______true_: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen_bcebade823f86b51___convert__closures_____invoke___wasm_bindgen_bcebade823f86b51___JsValue__core_ed718c3d60ebd546___result__Result_____wasm_bindgen_bcebade823f86b51___JsError___true_: (a: number, b: number, c: any) => [number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
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
