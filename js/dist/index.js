export { Buffer, BytesMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut, ByteIterator, CharIterator, } from '../pkg/smol_bytes.js';
// Attach Symbol.iterator to all types
import { Buffer, BytesMut, SharedBytes, CompactBytes, Utf8Buffer, Utf8Bytes, CompactUtf8Bytes, Utf8BytesMut, } from '../pkg/smol_bytes.js';
for (const Cls of [Buffer, BytesMut, SharedBytes, CompactBytes]) {
    Cls.prototype[Symbol.iterator] = function () {
        return this.iter();
    };
}
for (const Cls of [Utf8Buffer, Utf8Bytes, CompactUtf8Bytes, Utf8BytesMut]) {
    Cls.prototype[Symbol.iterator] = function () {
        return this.iter();
    };
}
