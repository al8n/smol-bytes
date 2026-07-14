/* @ts-self-types="./smol_bytes.d.ts" */
import * as wasm from "./smol_bytes_bg.wasm";
import { __wbg_set_wasm } from "./smol_bytes_bg.js";

__wbg_set_wasm(wasm);
wasm.__wbindgen_start();
export {
    Buffer, ByteIterator, BytesMut, CharIterator, CompactBytes, CompactUtf8Bytes, SharedBytes, Utf8Buffer, Utf8Bytes, Utf8BytesMut
} from "./smol_bytes_bg.js";
