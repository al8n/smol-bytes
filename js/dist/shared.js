import { Buffer, BytesMut, SharedBytes, CompactBytes, Utf8Buffer, Utf8Bytes, CompactUtf8Bytes, Utf8BytesMut, ByteIterator, CharIterator, } from '../pkg/smol_bytes.js';
import { installRuntime } from './runtime.js';
installRuntime({
    Buffer,
    BytesMut,
    SharedBytes,
    CompactBytes,
    Utf8Buffer,
    Utf8Bytes,
    CompactUtf8Bytes,
    Utf8BytesMut,
    ByteIterator,
    CharIterator,
});
export { SharedBytes as Bytes, Utf8Bytes };
