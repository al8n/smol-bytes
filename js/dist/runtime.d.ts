/**
 * The JavaScript boundary for the wasm classes.
 *
 * wasm-bindgen receives JavaScript numbers after WebAssembly coercion. Keep
 * every check here so all public entry points share the same, deterministic
 * TypeError/RangeError contract before that coercion can happen.
 */
type WasmClass = {
    prototype: object;
};
export type WasmClasses = {
    Buffer: WasmClass;
    BytesMut: WasmClass;
    SharedBytes: WasmClass;
    CompactBytes: WasmClass;
    Utf8Buffer: WasmClass;
    Utf8Bytes: WasmClass;
    CompactUtf8Bytes: WasmClass;
    Utf8BytesMut: WasmClass;
    ByteIterator: WasmClass;
    CharIterator: WasmClass;
};
export declare function assertWellFormedString(value: unknown, name: string): string;
/** Install the shared facade once for the generated wasm class objects. */
export declare function installRuntime(classes: WasmClasses): void;
export {};
