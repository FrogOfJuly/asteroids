/* tslint:disable */
/* eslint-disable */
/**
 * Type of hex color which is r,g,b
 */
export class Color {
  private constructor();
  free(): void;
}
export class Cursive {
  private constructor();
  free(): void;
  static asteroids(): Promise<Cursive>;
  static asteroids_with_canvas(canvas: HTMLCanvasElement): Promise<Cursive>;
}
export class ScreenChar {
  private constructor();
  free(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_cursive_free: (a: number, b: number) => void;
  readonly cursive_asteroids: () => any;
  readonly cursive_asteroids_with_canvas: (a: any) => any;
  readonly __wbg_screenchar_free: (a: number, b: number) => void;
  readonly __wbg_color_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_6: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__he1e540767b799e4b: (a: number, b: number) => void;
  readonly closure60_externref_shim: (a: number, b: number, c: any) => void;
  readonly closure138_externref_shim: (a: number, b: number, c: any) => void;
  readonly closure171_externref_shim: (a: number, b: number, c: any, d: any) => void;
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
