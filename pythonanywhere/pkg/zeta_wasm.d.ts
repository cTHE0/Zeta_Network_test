/* tslint:disable */
/* eslint-disable */

/**
 * Récupère les infos du nœud local
 */
export function get_node_info(): any;

/**
 * Récupère la liste des peers
 */
export function get_peers(): any;

/**
 * Récupère la liste des posts
 */
export function get_posts(): any;

/**
 * Initialise le nœud P2P dans le navigateur
 */
export function init(relay_url?: string | null): any;

/**
 * Définit le callback pour les nouveaux messages
 */
export function on_message(callback: Function): void;

/**
 * Définit le callback pour les changements de peers
 */
export function on_peers_change(callback: Function): void;

/**
 * Définit le callback pour les changements de statut
 */
export function on_status_change(callback: Function): void;

/**
 * Publie un post sur le réseau
 */
export function publish_post(content: string, author_name: string): any;

export function start_heartbeat(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly get_node_info: () => [number, number, number];
    readonly get_peers: () => [number, number, number];
    readonly get_posts: () => [number, number, number];
    readonly init: (a: number, b: number) => [number, number, number];
    readonly on_message: (a: any) => void;
    readonly on_peers_change: (a: any) => void;
    readonly on_status_change: (a: any) => void;
    readonly publish_post: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly start_heartbeat: () => void;
    readonly wasm_bindgen__closure__destroy__h45166842240c659f: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h34e697c653f38917: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h2677813e22131529: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
