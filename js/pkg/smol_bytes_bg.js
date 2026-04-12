/**
 * A fixed-size buffer for inline storage.
 *
 * This type can hold at most `62` bytes on stack without heap allocation.
 *
 * @example
 * ```typescript
 * import { Buffer } from 'smol-bytes';
 * const buf = Buffer.fromBytes(new Uint8Array([1, 2, 3]));
 * console.log(buf.len()); // 3
 * ```
 */
export class Buffer {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Buffer.prototype);
        obj.__wbg_ptr = ptr;
        BufferFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BufferFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_buffer_free(ptr, 0);
    }
    /**
     * Advance the read cursor by `cnt` bytes.
     *
     * @throws {Error} If `cnt` exceeds the number of remaining bytes.
     * @param {number} cnt
     */
    advance(cnt) {
        const ret = wasm.buffer_advance(this.__wbg_ptr, cnt);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Clear the buffer, removing all data and resetting the cursor.
     */
    clear() {
        wasm.buffer_clear(this.__wbg_ptr);
    }
    /**
     * Create a Buffer from a byte array.
     *
     * @param data - The source bytes to copy.
     * @throws {Error} If the data exceeds inline capacity (62 bytes).
     * @param {Uint8Array} data
     * @returns {Buffer}
     */
    static fromBytes(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.buffer_fromBytes(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Buffer.__wrap(ret[0]);
    }
    /**
     * Create a Buffer from a UTF-8 string.
     *
     * @param s - The source string whose bytes are copied.
     * @throws {Error} If the encoded bytes exceed inline capacity (62 bytes).
     * @param {string} s
     * @returns {Buffer}
     */
    static fromString(s) {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.buffer_fromString(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Buffer.__wrap(ret[0]);
    }
    /**
     * Read a 32-bit float in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF32() {
        const ret = wasm.buffer_getF32(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a 32-bit float in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF32Le() {
        const ret = wasm.buffer_getF32Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a 64-bit float in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF64() {
        const ret = wasm.buffer_getF64(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a 64-bit float in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF64Le() {
        const ret = wasm.buffer_getF64Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI16() {
        const ret = wasm.buffer_getI16(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI16Le() {
        const ret = wasm.buffer_getI16Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI32() {
        const ret = wasm.buffer_getI32(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI32Le() {
        const ret = wasm.buffer_getI32Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getI64() {
        const ret = wasm.buffer_getI64(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getI64Le() {
        const ret = wasm.buffer_getI64Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI8() {
        const ret = wasm.buffer_getI8(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read an unsigned 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU16() {
        const ret = wasm.buffer_getU16(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read an unsigned 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU16Le() {
        const ret = wasm.buffer_getU16Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read an unsigned 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU32() {
        const ret = wasm.buffer_getU32(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Read an unsigned 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU32Le() {
        const ret = wasm.buffer_getU32Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Read an unsigned 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getU64() {
        const ret = wasm.buffer_getU64(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return BigInt.asUintN(64, ret[0]);
    }
    /**
     * Read an unsigned 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getU64Le() {
        const ret = wasm.buffer_getU64Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return BigInt.asUintN(64, ret[0]);
    }
    /**
     * Read an unsigned 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU8() {
        const ret = wasm.buffer_getU8(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Return `true` if the buffer has no bytes.
     * @returns {boolean}
     */
    isEmpty() {
        const ret = wasm.buffer_isEmpty(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return a JS-compatible byte iterator.
     *
     * Use `for (const b of buf)` after attaching `Symbol.iterator`.
     * @returns {ByteIterator}
     */
    iter() {
        const ret = wasm.buffer_iter(this.__wbg_ptr);
        return ByteIterator.__wrap(ret);
    }
    /**
     * Return the byte length of the buffer.
     * @returns {number}
     */
    len() {
        const ret = wasm.buffer_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a new empty Buffer.
     */
    constructor() {
        const ret = wasm.buffer_new_wasm();
        this.__wbg_ptr = ret >>> 0;
        BufferFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Write a 32-bit float in big-endian byte order.
     * @param {number} val
     */
    putF32(val) {
        wasm.buffer_putF32(this.__wbg_ptr, val);
    }
    /**
     * Write a 32-bit float in little-endian byte order.
     * @param {number} val
     */
    putF32Le(val) {
        wasm.buffer_putF32Le(this.__wbg_ptr, val);
    }
    /**
     * Write a 64-bit float in big-endian byte order.
     * @param {number} val
     */
    putF64(val) {
        wasm.buffer_putF64(this.__wbg_ptr, val);
    }
    /**
     * Write a 64-bit float in little-endian byte order.
     * @param {number} val
     */
    putF64Le(val) {
        wasm.buffer_putF64Le(this.__wbg_ptr, val);
    }
    /**
     * Write a signed 16-bit integer in big-endian byte order.
     * @param {number} val
     */
    putI16(val) {
        wasm.buffer_putI16(this.__wbg_ptr, val);
    }
    /**
     * Write a signed 16-bit integer in little-endian byte order.
     * @param {number} val
     */
    putI16Le(val) {
        wasm.buffer_putI16Le(this.__wbg_ptr, val);
    }
    /**
     * Write a signed 32-bit integer in big-endian byte order.
     * @param {number} val
     */
    putI32(val) {
        wasm.buffer_putI32(this.__wbg_ptr, val);
    }
    /**
     * Write a signed 32-bit integer in little-endian byte order.
     * @param {number} val
     */
    putI32Le(val) {
        wasm.buffer_putI32Le(this.__wbg_ptr, val);
    }
    /**
     * Write a signed 64-bit integer in big-endian byte order.
     * @param {bigint} val
     */
    putI64(val) {
        wasm.buffer_putI64(this.__wbg_ptr, val);
    }
    /**
     * Write a signed 64-bit integer in little-endian byte order.
     * @param {bigint} val
     */
    putI64Le(val) {
        wasm.buffer_putI64Le(this.__wbg_ptr, val);
    }
    /**
     * Write a signed 8-bit integer.
     * @param {number} val
     */
    putI8(val) {
        wasm.buffer_putI8(this.__wbg_ptr, val);
    }
    /**
     * Write a byte slice into the buffer.
     *
     * @param data - The bytes to append.
     * @throws {Error} If the data would exceed inline capacity (62 bytes).
     * @param {Uint8Array} data
     */
    putSlice(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.buffer_putSlice(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Write an unsigned 16-bit integer in big-endian byte order.
     * @param {number} val
     */
    putU16(val) {
        wasm.buffer_putU16(this.__wbg_ptr, val);
    }
    /**
     * Write an unsigned 16-bit integer in little-endian byte order.
     * @param {number} val
     */
    putU16Le(val) {
        wasm.buffer_putU16Le(this.__wbg_ptr, val);
    }
    /**
     * Write an unsigned 32-bit integer in big-endian byte order.
     * @param {number} val
     */
    putU32(val) {
        wasm.buffer_putU32(this.__wbg_ptr, val);
    }
    /**
     * Write an unsigned 32-bit integer in little-endian byte order.
     * @param {number} val
     */
    putU32Le(val) {
        wasm.buffer_putU32Le(this.__wbg_ptr, val);
    }
    /**
     * Write an unsigned 64-bit integer in big-endian byte order.
     * @param {bigint} val
     */
    putU64(val) {
        wasm.buffer_putU64(this.__wbg_ptr, val);
    }
    /**
     * Write an unsigned 64-bit integer in little-endian byte order.
     * @param {bigint} val
     */
    putU64Le(val) {
        wasm.buffer_putU64Le(this.__wbg_ptr, val);
    }
    /**
     * Write an unsigned 8-bit integer.
     * @param {number} val
     */
    putU8(val) {
        wasm.buffer_putU8(this.__wbg_ptr, val);
    }
    /**
     * Return the number of readable bytes remaining.
     * @returns {number}
     */
    remaining() {
        const ret = wasm.buffer_remaining(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Return a copy of bytes in range `[start, end)`.
     *
     * @throws {Error} If the range is out of bounds.
     * @param {number} start
     * @param {number} end
     * @returns {Buffer}
     */
    slice(start, end) {
        const ret = wasm.buffer_slice(this.__wbg_ptr, start, end);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Buffer.__wrap(ret[0]);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds.
     * @param {number} at
     * @returns {Buffer}
     */
    splitOff(at) {
        const ret = wasm.buffer_splitOff(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Buffer.__wrap(ret[0]);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds.
     * @param {number} at
     * @returns {Buffer}
     */
    splitTo(at) {
        const ret = wasm.buffer_splitTo(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Buffer.__wrap(ret[0]);
    }
    /**
     * Return contents as a `Uint8Array` (copy).
     * @returns {Uint8Array}
     */
    toBytes() {
        const ret = wasm.buffer_toBytes(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Return contents as a UTF-8 string.
     *
     * @throws {Error} If the buffer contains invalid UTF-8.
     * @returns {string}
     */
    toString() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.buffer_toString(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
     * @param {number} new_len
     */
    truncate(new_len) {
        wasm.buffer_truncate(this.__wbg_ptr, new_len);
    }
}
if (Symbol.dispose) Buffer.prototype[Symbol.dispose] = Buffer.prototype.free;

/**
 * Iterator over bytes, compatible with the JS iterator protocol.
 *
 * Each call to `next()` returns `{ value: number, done: false }` or `{ done: true }`.
 * Attach `Symbol.iterator` to use with `for...of` loops.
 */
export class ByteIterator {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ByteIterator.prototype);
        obj.__wbg_ptr = ptr;
        ByteIteratorFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ByteIteratorFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_byteiterator_free(ptr, 0);
    }
    /**
     * Return the next `{ value, done }` object per the JS iterator protocol.
     *
     * Returns `{ value: number, done: false }` while bytes remain,
     * then `{ done: true }` when exhausted.
     * @returns {any}
     */
    next() {
        const ret = wasm.byteiterator_next(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) ByteIterator.prototype[Symbol.dispose] = ByteIterator.prototype.free;

/**
 * Growable mutable byte buffer with inline/heap storage.
 *
 * Stores up to 62 bytes inline on the stack. Larger data automatically
 * promotes to heap allocation. Once on heap, stays on heap.
 *
 * @example
 * ```typescript
 * import { BytesMut } from 'smol-bytes';
 * const buf = BytesMut.withCapacity(100);
 * buf.putSlice(new Uint8Array([1, 2, 3]));
 * console.log(buf.len()); // 3
 * ```
 */
export class BytesMut {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(BytesMut.prototype);
        obj.__wbg_ptr = ptr;
        BytesMutFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BytesMutFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_bytesmut_free(ptr, 0);
    }
    /**
     * Advance the read cursor by `cnt` bytes.
     *
     * @throws {Error} If `cnt` exceeds the number of remaining bytes.
     * @param {number} cnt
     */
    advance(cnt) {
        const ret = wasm.bytesmut_advance(this.__wbg_ptr, cnt);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Return the total allocated capacity in bytes.
     * @returns {number}
     */
    capacity() {
        const ret = wasm.bytesmut_capacity(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Clear the buffer, removing all data and resetting the cursor.
     */
    clear() {
        wasm.bytesmut_clear(this.__wbg_ptr);
    }
    /**
     * Create a `BytesMut` from a byte array.
     *
     * @param data - The source bytes to copy.
     * @param {Uint8Array} data
     * @returns {BytesMut}
     */
    static fromBytes(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bytesmut_fromBytes(ptr0, len0);
        return BytesMut.__wrap(ret);
    }
    /**
     * Create a `BytesMut` from a UTF-8 string.
     *
     * @param s - The source string whose bytes are copied.
     * @param {string} s
     * @returns {BytesMut}
     */
    static fromString(s) {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bytesmut_fromString(ptr0, len0);
        return BytesMut.__wrap(ret);
    }
    /**
     * Read a 32-bit float in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF32() {
        const ret = wasm.bytesmut_getF32(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a 32-bit float in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF32Le() {
        const ret = wasm.bytesmut_getF32Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a 64-bit float in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF64() {
        const ret = wasm.bytesmut_getF64(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a 64-bit float in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF64Le() {
        const ret = wasm.bytesmut_getF64Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI16() {
        const ret = wasm.bytesmut_getI16(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI16Le() {
        const ret = wasm.bytesmut_getI16Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI32() {
        const ret = wasm.bytesmut_getI32(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI32Le() {
        const ret = wasm.bytesmut_getI32Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getI64() {
        const ret = wasm.bytesmut_getI64(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getI64Le() {
        const ret = wasm.bytesmut_getI64Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI8() {
        const ret = wasm.bytesmut_getI8(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read an unsigned 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU16() {
        const ret = wasm.bytesmut_getU16(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read an unsigned 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU16Le() {
        const ret = wasm.bytesmut_getU16Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read an unsigned 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU32() {
        const ret = wasm.bytesmut_getU32(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Read an unsigned 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU32Le() {
        const ret = wasm.bytesmut_getU32Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Read an unsigned 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getU64() {
        const ret = wasm.bytesmut_getU64(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return BigInt.asUintN(64, ret[0]);
    }
    /**
     * Read an unsigned 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getU64Le() {
        const ret = wasm.bytesmut_getU64Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return BigInt.asUintN(64, ret[0]);
    }
    /**
     * Read an unsigned 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU8() {
        const ret = wasm.bytesmut_getU8(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Return `true` if the buffer has no bytes.
     * @returns {boolean}
     */
    isEmpty() {
        const ret = wasm.bytesmut_isEmpty(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return `true` if data is stored on the heap.
     * @returns {boolean}
     */
    isHeap() {
        const ret = wasm.bytesmut_isHeap(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return `true` if data is stored inline (no heap allocation).
     * @returns {boolean}
     */
    isInline() {
        const ret = wasm.bytesmut_isInline(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return a JS-compatible byte iterator.
     *
     * Use `for (const b of buf)` after attaching `Symbol.iterator`.
     * @returns {ByteIterator}
     */
    iter() {
        const ret = wasm.bytesmut_iter(this.__wbg_ptr);
        return ByteIterator.__wrap(ret);
    }
    /**
     * Return the byte length of the buffer.
     * @returns {number}
     */
    len() {
        const ret = wasm.bytesmut_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a new empty `BytesMut`.
     */
    constructor() {
        const ret = wasm.bytesmut_new_wasm();
        this.__wbg_ptr = ret >>> 0;
        BytesMutFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Write a 32-bit float in big-endian byte order.
     * @param {number} val
     */
    putF32(val) {
        wasm.bytesmut_putF32(this.__wbg_ptr, val);
    }
    /**
     * Write a 32-bit float in little-endian byte order.
     * @param {number} val
     */
    putF32Le(val) {
        wasm.bytesmut_putF32Le(this.__wbg_ptr, val);
    }
    /**
     * Write a 64-bit float in big-endian byte order.
     * @param {number} val
     */
    putF64(val) {
        wasm.bytesmut_putF64(this.__wbg_ptr, val);
    }
    /**
     * Write a 64-bit float in little-endian byte order.
     * @param {number} val
     */
    putF64Le(val) {
        wasm.bytesmut_putF64Le(this.__wbg_ptr, val);
    }
    /**
     * Write a signed 16-bit integer in big-endian byte order.
     * @param {number} val
     */
    putI16(val) {
        wasm.bytesmut_putI16(this.__wbg_ptr, val);
    }
    /**
     * Write a signed 16-bit integer in little-endian byte order.
     * @param {number} val
     */
    putI16Le(val) {
        wasm.bytesmut_putI16Le(this.__wbg_ptr, val);
    }
    /**
     * Write a signed 32-bit integer in big-endian byte order.
     * @param {number} val
     */
    putI32(val) {
        wasm.bytesmut_putI32(this.__wbg_ptr, val);
    }
    /**
     * Write a signed 32-bit integer in little-endian byte order.
     * @param {number} val
     */
    putI32Le(val) {
        wasm.bytesmut_putI32Le(this.__wbg_ptr, val);
    }
    /**
     * Write a signed 64-bit integer in big-endian byte order.
     * @param {bigint} val
     */
    putI64(val) {
        wasm.bytesmut_putI64(this.__wbg_ptr, val);
    }
    /**
     * Write a signed 64-bit integer in little-endian byte order.
     * @param {bigint} val
     */
    putI64Le(val) {
        wasm.bytesmut_putI64Le(this.__wbg_ptr, val);
    }
    /**
     * Write a signed 8-bit integer.
     * @param {number} val
     */
    putI8(val) {
        wasm.bytesmut_putI8(this.__wbg_ptr, val);
    }
    /**
     * Write a byte slice into the buffer.
     *
     * @param data - The bytes to append.
     * @param {Uint8Array} data
     */
    putSlice(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.bytesmut_putSlice(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Write an unsigned 16-bit integer in big-endian byte order.
     * @param {number} val
     */
    putU16(val) {
        wasm.bytesmut_putU16(this.__wbg_ptr, val);
    }
    /**
     * Write an unsigned 16-bit integer in little-endian byte order.
     * @param {number} val
     */
    putU16Le(val) {
        wasm.bytesmut_putU16Le(this.__wbg_ptr, val);
    }
    /**
     * Write an unsigned 32-bit integer in big-endian byte order.
     * @param {number} val
     */
    putU32(val) {
        wasm.bytesmut_putU32(this.__wbg_ptr, val);
    }
    /**
     * Write an unsigned 32-bit integer in little-endian byte order.
     * @param {number} val
     */
    putU32Le(val) {
        wasm.bytesmut_putU32Le(this.__wbg_ptr, val);
    }
    /**
     * Write an unsigned 64-bit integer in big-endian byte order.
     * @param {bigint} val
     */
    putU64(val) {
        wasm.bytesmut_putU64(this.__wbg_ptr, val);
    }
    /**
     * Write an unsigned 64-bit integer in little-endian byte order.
     * @param {bigint} val
     */
    putU64Le(val) {
        wasm.bytesmut_putU64Le(this.__wbg_ptr, val);
    }
    /**
     * Write an unsigned 8-bit integer.
     * @param {number} val
     */
    putU8(val) {
        wasm.bytesmut_putU8(this.__wbg_ptr, val);
    }
    /**
     * Return the number of readable bytes remaining.
     * @returns {number}
     */
    remaining() {
        const ret = wasm.bytesmut_remaining(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Reserve capacity for at least `additional` more bytes.
     * @param {number} additional
     */
    reserve(additional) {
        wasm.bytesmut_reserve(this.__wbg_ptr, additional);
    }
    /**
     * Resize the buffer to `new_len`, filling new bytes with `value` if growing.
     * @param {number} new_len
     * @param {number} value
     */
    resize(new_len, value) {
        wasm.bytesmut_resize(this.__wbg_ptr, new_len, value);
    }
    /**
     * Split all data out of this buffer, returning it. Self becomes empty.
     * @returns {BytesMut}
     */
    split() {
        const ret = wasm.bytesmut_split(this.__wbg_ptr);
        return BytesMut.__wrap(ret);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds.
     * @param {number} at
     * @returns {BytesMut}
     */
    splitOff(at) {
        const ret = wasm.bytesmut_splitOff(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return BytesMut.__wrap(ret[0]);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds.
     * @param {number} at
     * @returns {BytesMut}
     */
    splitTo(at) {
        const ret = wasm.bytesmut_splitTo(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return BytesMut.__wrap(ret[0]);
    }
    /**
     * Return contents as a `Uint8Array` (copy).
     * @returns {Uint8Array}
     */
    toBytes() {
        const ret = wasm.bytesmut_toBytes(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Return contents as a UTF-8 string.
     *
     * @throws {Error} If the buffer contains invalid UTF-8.
     * @returns {string}
     */
    toString() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.bytesmut_toString(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
     * @param {number} new_len
     */
    truncate(new_len) {
        wasm.bytesmut_truncate(this.__wbg_ptr, new_len);
    }
    /**
     * Attempt to re-merge a previously split buffer back into this one.
     *
     * Returns `other` unchanged if the two buffers are not contiguous.
     * @param {BytesMut} other
     * @returns {BytesMut | undefined}
     */
    unsplit(other) {
        _assertClass(other, BytesMut);
        var ptr0 = other.__destroy_into_raw();
        const ret = wasm.bytesmut_unsplit(this.__wbg_ptr, ptr0);
        return ret === 0 ? undefined : BytesMut.__wrap(ret);
    }
    /**
     * Create a new `BytesMut` with the given capacity pre-allocated.
     *
     * @param capacity - Number of bytes to pre-allocate.
     * @param {number} capacity
     * @returns {BytesMut}
     */
    static withCapacity(capacity) {
        const ret = wasm.bytesmut_withCapacity(capacity);
        return BytesMut.__wrap(ret);
    }
}
if (Symbol.dispose) BytesMut.prototype[Symbol.dispose] = BytesMut.prototype.free;

/**
 * Iterator over characters, compatible with the JS iterator protocol.
 *
 * Each call to `next()` returns `{ value: string, done: false }` or `{ done: true }`.
 * Attach `Symbol.iterator` to use with `for...of` loops.
 */
export class CharIterator {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(CharIterator.prototype);
        obj.__wbg_ptr = ptr;
        CharIteratorFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CharIteratorFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_chariterator_free(ptr, 0);
    }
    /**
     * Return the next `{ value, done }` object per the JS iterator protocol.
     *
     * Returns `{ value: string, done: false }` while characters remain,
     * then `{ done: true }` when exhausted.
     * @returns {any}
     */
    next() {
        const ret = wasm.chariterator_next(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) CharIterator.prototype[Symbol.dispose] = CharIterator.prototype.free;

/**
 * Immutable byte buffer using the Compact strategy.
 *
 * Stores up to 62 bytes inline with unique ownership for larger data (no reference counting).
 * Available as `CompactBytes` via `import { CompactBytes } from 'smol-bytes/compact'`.
 */
export class CompactBytes {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(CompactBytes.prototype);
        obj.__wbg_ptr = ptr;
        CompactBytesFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CompactBytesFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_compactbytes_free(ptr, 0);
    }
    /**
     * Advance the read cursor by `cnt` bytes.
     *
     * @throws {Error} If `cnt` exceeds the number of remaining bytes.
     * @param {number} cnt
     */
    advance(cnt) {
        const ret = wasm.compactbytes_advance(this.__wbg_ptr, cnt);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Clear the buffer, removing all data and resetting the cursor.
     */
    clear() {
        wasm.compactbytes_clear(this.__wbg_ptr);
    }
    /**
     * Create a `CompactBytes` from a byte array.
     *
     * @param data - The source bytes to copy.
     * @param {Uint8Array} data
     * @returns {CompactBytes}
     */
    static fromBytes(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.compactbytes_fromBytes(ptr0, len0);
        return CompactBytes.__wrap(ret);
    }
    /**
     * Create a `CompactBytes` from a static string.
     *
     * Note: In WASM, this copies the data (true static references cannot cross the JS boundary).
     * @param {string} s
     * @returns {CompactBytes}
     */
    static fromStatic(s) {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.compactbytes_fromStatic(ptr0, len0);
        return CompactBytes.__wrap(ret);
    }
    /**
     * Create a `CompactBytes` from a UTF-8 string.
     *
     * @param s - The source string whose bytes are copied.
     * @param {string} s
     * @returns {CompactBytes}
     */
    static fromString(s) {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.compactbytes_fromString(ptr0, len0);
        return CompactBytes.__wrap(ret);
    }
    /**
     * Read a 32-bit float in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF32() {
        const ret = wasm.compactbytes_getF32(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a 32-bit float in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF32Le() {
        const ret = wasm.compactbytes_getF32Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a 64-bit float in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF64() {
        const ret = wasm.compactbytes_getF64(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a 64-bit float in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF64Le() {
        const ret = wasm.compactbytes_getF64Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI16() {
        const ret = wasm.compactbytes_getI16(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI16Le() {
        const ret = wasm.compactbytes_getI16Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI32() {
        const ret = wasm.compactbytes_getI32(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI32Le() {
        const ret = wasm.compactbytes_getI32Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getI64() {
        const ret = wasm.compactbytes_getI64(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getI64Le() {
        const ret = wasm.compactbytes_getI64Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI8() {
        const ret = wasm.compactbytes_getI8(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read an unsigned 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU16() {
        const ret = wasm.compactbytes_getU16(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read an unsigned 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU16Le() {
        const ret = wasm.compactbytes_getU16Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read an unsigned 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU32() {
        const ret = wasm.compactbytes_getU32(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Read an unsigned 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU32Le() {
        const ret = wasm.compactbytes_getU32Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Read an unsigned 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getU64() {
        const ret = wasm.compactbytes_getU64(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return BigInt.asUintN(64, ret[0]);
    }
    /**
     * Read an unsigned 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getU64Le() {
        const ret = wasm.compactbytes_getU64Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return BigInt.asUintN(64, ret[0]);
    }
    /**
     * Read an unsigned 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU8() {
        const ret = wasm.compactbytes_getU8(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Return `true` if the buffer has no bytes.
     * @returns {boolean}
     */
    isEmpty() {
        const ret = wasm.compactbytes_isEmpty(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return `true` if data is stored on the heap.
     * @returns {boolean}
     */
    isHeap() {
        const ret = wasm.compactbytes_isHeap(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return `true` if data is stored inline (no heap allocation).
     * @returns {boolean}
     */
    isInline() {
        const ret = wasm.compactbytes_isInline(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return a JS-compatible byte iterator.
     *
     * Use `for (const b of buf)` after attaching `Symbol.iterator`.
     * @returns {ByteIterator}
     */
    iter() {
        const ret = wasm.compactbytes_iter(this.__wbg_ptr);
        return ByteIterator.__wrap(ret);
    }
    /**
     * Return the byte length of the buffer.
     * @returns {number}
     */
    len() {
        const ret = wasm.compactbytes_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a new empty `CompactBytes`.
     */
    constructor() {
        const ret = wasm.compactbytes_new();
        this.__wbg_ptr = ret >>> 0;
        CompactBytesFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Return the number of readable bytes remaining.
     * @returns {number}
     */
    remaining() {
        const ret = wasm.compactbytes_remaining(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Return a copy of bytes in range `[start, end)`.
     *
     * @throws {Error} If the range is out of bounds.
     * @param {number} start
     * @param {number} end
     * @returns {CompactBytes}
     */
    slice(start, end) {
        const ret = wasm.compactbytes_slice(this.__wbg_ptr, start, end);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return CompactBytes.__wrap(ret[0]);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds.
     * @param {number} at
     * @returns {CompactBytes}
     */
    splitOff(at) {
        const ret = wasm.compactbytes_splitOff(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return CompactBytes.__wrap(ret[0]);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds.
     * @param {number} at
     * @returns {CompactBytes}
     */
    splitTo(at) {
        const ret = wasm.compactbytes_splitTo(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return CompactBytes.__wrap(ret[0]);
    }
    /**
     * Return contents as a `Uint8Array` (copy).
     * @returns {Uint8Array}
     */
    toBytes() {
        const ret = wasm.compactbytes_toBytes(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Return contents as a UTF-8 string.
     *
     * @throws {Error} If the buffer contains invalid UTF-8.
     * @returns {string}
     */
    toString() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.compactbytes_toString(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
     * @param {number} new_len
     */
    truncate(new_len) {
        wasm.compactbytes_truncate(this.__wbg_ptr, new_len);
    }
}
if (Symbol.dispose) CompactBytes.prototype[Symbol.dispose] = CompactBytes.prototype.free;

/**
 * WASM bindings for the [`Utf8Bytes`](crate::Utf8Bytes) type (compact variant).
 *
 * Available as `CompactUtf8Bytes` via `import { CompactUtf8Bytes } from 'smol-bytes/compact'`.
 */
export class CompactUtf8Bytes {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(CompactUtf8Bytes.prototype);
        obj.__wbg_ptr = ptr;
        CompactUtf8BytesFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CompactUtf8BytesFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_compactutf8bytes_free(ptr, 0);
    }
    /**
     * Advance the read cursor by `cnt` bytes.
     *
     * @throws {Error} If `cnt` exceeds the number of remaining bytes.
     * @param {number} cnt
     */
    advance(cnt) {
        const ret = wasm.compactutf8bytes_advance(this.__wbg_ptr, cnt);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Create a `CompactUtf8Bytes` from a static string.
     *
     * Note: In WASM, this copies the data (true static references cannot cross the JS boundary).
     * @param {string} s
     * @returns {CompactUtf8Bytes}
     */
    static fromStatic(s) {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.compactutf8bytes_fromStatic(ptr0, len0);
        return CompactUtf8Bytes.__wrap(ret);
    }
    /**
     * Create a `CompactUtf8Bytes` from a UTF-8 string.
     *
     * @param s - The source string whose bytes are copied.
     * @param {string} s
     * @returns {CompactUtf8Bytes}
     */
    static fromString(s) {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.compactutf8bytes_fromString(ptr0, len0);
        return CompactUtf8Bytes.__wrap(ret);
    }
    /**
     * Return `true` if the buffer has no bytes.
     * @returns {boolean}
     */
    isEmpty() {
        const ret = wasm.compactutf8bytes_isEmpty(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return `true` if data is stored on the heap.
     * @returns {boolean}
     */
    isHeap() {
        const ret = wasm.compactutf8bytes_isHeap(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return `true` if data is stored inline (no heap allocation).
     * @returns {boolean}
     */
    isInline() {
        const ret = wasm.compactutf8bytes_isInline(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return a JS-compatible character iterator.
     *
     * Use `for (const ch of buf)` after attaching `Symbol.iterator`.
     * @returns {CharIterator}
     */
    iter() {
        const ret = wasm.compactutf8bytes_iter(this.__wbg_ptr);
        return CharIterator.__wrap(ret);
    }
    /**
     * Return the byte length of the buffer.
     * @returns {number}
     */
    len() {
        const ret = wasm.compactutf8bytes_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a new empty `CompactUtf8Bytes`.
     */
    constructor() {
        const ret = wasm.compactutf8bytes_new_wasm();
        this.__wbg_ptr = ret >>> 0;
        CompactUtf8BytesFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Return the number of readable bytes remaining.
     * @returns {number}
     */
    remaining() {
        const ret = wasm.compactutf8bytes_remaining(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Return a copy of bytes in range `[start, end)`.
     *
     * @throws {Error} If the range is out of bounds or not on UTF-8 char boundaries.
     * @param {number} start
     * @param {number} end
     * @returns {CompactUtf8Bytes}
     */
    slice(start, end) {
        const ret = wasm.compactutf8bytes_slice(this.__wbg_ptr, start, end);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return CompactUtf8Bytes.__wrap(ret[0]);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     * @param {number} at
     * @returns {CompactUtf8Bytes}
     */
    splitOff(at) {
        const ret = wasm.compactutf8bytes_splitOff(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return CompactUtf8Bytes.__wrap(ret[0]);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     * @param {number} at
     * @returns {CompactUtf8Bytes}
     */
    splitTo(at) {
        const ret = wasm.compactutf8bytes_splitTo(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return CompactUtf8Bytes.__wrap(ret[0]);
    }
    /**
     * Return contents as a `Uint8Array` (copy of the UTF-8 encoded bytes).
     * @returns {Uint8Array}
     */
    toBytes() {
        const ret = wasm.compactutf8bytes_toBytes(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Return contents as a UTF-8 string.
     * @returns {string}
     */
    toString() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.compactutf8bytes_toString(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) CompactUtf8Bytes.prototype[Symbol.dispose] = CompactUtf8Bytes.prototype.free;

/**
 * Immutable byte buffer using the Shared strategy.
 *
 * Stores up to 62 bytes inline with zero-copy reference-counted cloning for larger data.
 * Available as `SharedBytes` via `import { SharedBytes } from 'smol-bytes/shared'`.
 */
export class SharedBytes {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(SharedBytes.prototype);
        obj.__wbg_ptr = ptr;
        SharedBytesFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SharedBytesFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_sharedbytes_free(ptr, 0);
    }
    /**
     * Advance the read cursor by `cnt` bytes.
     *
     * @throws {Error} If `cnt` exceeds the number of remaining bytes.
     * @param {number} cnt
     */
    advance(cnt) {
        const ret = wasm.sharedbytes_advance(this.__wbg_ptr, cnt);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Clear the buffer, removing all data and resetting the cursor.
     */
    clear() {
        wasm.sharedbytes_clear(this.__wbg_ptr);
    }
    /**
     * Create a `SharedBytes` from a byte array.
     *
     * @param data - The source bytes to copy.
     * @param {Uint8Array} data
     * @returns {SharedBytes}
     */
    static fromBytes(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.sharedbytes_fromBytes(ptr0, len0);
        return SharedBytes.__wrap(ret);
    }
    /**
     * Create a `SharedBytes` from a static string.
     *
     * Note: In WASM, this copies the data (true static references cannot cross the JS boundary).
     * @param {string} s
     * @returns {SharedBytes}
     */
    static fromStatic(s) {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.sharedbytes_fromStatic(ptr0, len0);
        return SharedBytes.__wrap(ret);
    }
    /**
     * Create a `SharedBytes` from a UTF-8 string.
     *
     * @param s - The source string whose bytes are copied.
     * @param {string} s
     * @returns {SharedBytes}
     */
    static fromString(s) {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.sharedbytes_fromString(ptr0, len0);
        return SharedBytes.__wrap(ret);
    }
    /**
     * Read a 32-bit float in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF32() {
        const ret = wasm.sharedbytes_getF32(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a 32-bit float in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF32Le() {
        const ret = wasm.sharedbytes_getF32Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a 64-bit float in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF64() {
        const ret = wasm.sharedbytes_getF64(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a 64-bit float in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getF64Le() {
        const ret = wasm.sharedbytes_getF64Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI16() {
        const ret = wasm.sharedbytes_getI16(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI16Le() {
        const ret = wasm.sharedbytes_getI16Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI32() {
        const ret = wasm.sharedbytes_getI32(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI32Le() {
        const ret = wasm.sharedbytes_getI32Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getI64() {
        const ret = wasm.sharedbytes_getI64(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getI64Le() {
        const ret = wasm.sharedbytes_getI64Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read a signed 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getI8() {
        const ret = wasm.sharedbytes_getI8(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read an unsigned 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU16() {
        const ret = wasm.sharedbytes_getU16(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read an unsigned 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU16Le() {
        const ret = wasm.sharedbytes_getU16Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Read an unsigned 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU32() {
        const ret = wasm.sharedbytes_getU32(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Read an unsigned 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU32Le() {
        const ret = wasm.sharedbytes_getU32Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Read an unsigned 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getU64() {
        const ret = wasm.sharedbytes_getU64(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return BigInt.asUintN(64, ret[0]);
    }
    /**
     * Read an unsigned 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     * @returns {bigint}
     */
    getU64Le() {
        const ret = wasm.sharedbytes_getU64Le(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return BigInt.asUintN(64, ret[0]);
    }
    /**
     * Read an unsigned 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     * @returns {number}
     */
    getU8() {
        const ret = wasm.sharedbytes_getU8(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Return `true` if the buffer has no bytes.
     * @returns {boolean}
     */
    isEmpty() {
        const ret = wasm.sharedbytes_isEmpty(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return `true` if data is stored on the heap.
     * @returns {boolean}
     */
    isHeap() {
        const ret = wasm.sharedbytes_isHeap(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return `true` if data is stored inline (no heap allocation).
     * @returns {boolean}
     */
    isInline() {
        const ret = wasm.sharedbytes_isInline(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return a JS-compatible byte iterator.
     *
     * Use `for (const b of buf)` after attaching `Symbol.iterator`.
     * @returns {ByteIterator}
     */
    iter() {
        const ret = wasm.sharedbytes_iter(this.__wbg_ptr);
        return ByteIterator.__wrap(ret);
    }
    /**
     * Return the byte length of the buffer.
     * @returns {number}
     */
    len() {
        const ret = wasm.sharedbytes_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a new empty `SharedBytes`.
     */
    constructor() {
        const ret = wasm.sharedbytes_new();
        this.__wbg_ptr = ret >>> 0;
        SharedBytesFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Return the number of readable bytes remaining.
     * @returns {number}
     */
    remaining() {
        const ret = wasm.sharedbytes_remaining(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Return a copy of bytes in range `[start, end)`.
     *
     * @throws {Error} If the range is out of bounds.
     * @param {number} start
     * @param {number} end
     * @returns {SharedBytes}
     */
    slice(start, end) {
        const ret = wasm.sharedbytes_slice(this.__wbg_ptr, start, end);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return SharedBytes.__wrap(ret[0]);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds.
     * @param {number} at
     * @returns {SharedBytes}
     */
    splitOff(at) {
        const ret = wasm.sharedbytes_splitOff(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return SharedBytes.__wrap(ret[0]);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds.
     * @param {number} at
     * @returns {SharedBytes}
     */
    splitTo(at) {
        const ret = wasm.sharedbytes_splitTo(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return SharedBytes.__wrap(ret[0]);
    }
    /**
     * Return contents as a `Uint8Array` (copy).
     * @returns {Uint8Array}
     */
    toBytes() {
        const ret = wasm.sharedbytes_toBytes(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Return contents as a UTF-8 string.
     *
     * @throws {Error} If the buffer contains invalid UTF-8.
     * @returns {string}
     */
    toString() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.sharedbytes_toString(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
     * @param {number} new_len
     */
    truncate(new_len) {
        wasm.sharedbytes_truncate(this.__wbg_ptr, new_len);
    }
}
if (Symbol.dispose) SharedBytes.prototype[Symbol.dispose] = SharedBytes.prototype.free;

/**
 * UTF-8 validated wrapper around `Buffer` with a String-like interface.
 *
 * Guarantees valid UTF-8. Split/slice operations check char boundaries.
 * Fixed inline capacity of 62 bytes.
 *
 * @example
 * ```typescript
 * import { Utf8Buffer } from 'smol-bytes';
 * const buf = new Utf8Buffer();
 * buf.pushStr('hello world');
 * console.log(buf.toString()); // 'hello world'
 * ```
 */
export class Utf8Buffer {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Utf8Buffer.prototype);
        obj.__wbg_ptr = ptr;
        Utf8BufferFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Utf8BufferFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_utf8buffer_free(ptr, 0);
    }
    /**
     * Return the total inline capacity in bytes.
     * @returns {number}
     */
    capacity() {
        const ret = wasm.utf8buffer_capacity(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Clear the buffer, removing all data.
     */
    clear() {
        wasm.utf8buffer_clear(this.__wbg_ptr);
    }
    /**
     * Create a `Utf8Buffer` from a UTF-8 string.
     *
     * @param s - The source string to copy.
     * @throws {Error} If the encoded bytes exceed inline capacity (62 bytes).
     * @param {string} s
     * @returns {Utf8Buffer}
     */
    static fromString(s) {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.utf8buffer_fromString(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Utf8Buffer.__wrap(ret[0]);
    }
    /**
     * Return `true` if the buffer has no bytes.
     * @returns {boolean}
     */
    isEmpty() {
        const ret = wasm.utf8buffer_isEmpty(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return a JS-compatible character iterator.
     *
     * Use `for (const ch of buf)` after attaching `Symbol.iterator`.
     * @returns {CharIterator}
     */
    iter() {
        const ret = wasm.utf8buffer_iter(this.__wbg_ptr);
        return CharIterator.__wrap(ret);
    }
    /**
     * Return the byte length of the buffer.
     * @returns {number}
     */
    len() {
        const ret = wasm.utf8buffer_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a new empty `Utf8Buffer`.
     */
    constructor() {
        const ret = wasm.utf8buffer_new_wasm();
        this.__wbg_ptr = ret >>> 0;
        Utf8BufferFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Append a single character.
     *
     * @param ch - A single-character string.
     * @throws {Error} If the string is empty or the character would exceed inline capacity.
     * @param {string} ch
     */
    push(ch) {
        const ptr0 = passStringToWasm0(ch, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.utf8buffer_push(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Append a string.
     *
     * @param s - The string to append.
     * @throws {Error} If the combined length would exceed inline capacity.
     * @param {string} s
     */
    pushStr(s) {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.utf8buffer_pushStr(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Return the number of readable bytes remaining.
     * @returns {number}
     */
    remaining() {
        const ret = wasm.utf8buffer_remaining(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Return a copy of bytes in range `[start, end)`.
     *
     * @throws {Error} If the range is out of bounds or not on UTF-8 char boundaries.
     * @param {number} start
     * @param {number} end
     * @returns {Utf8Buffer}
     */
    slice(start, end) {
        const ret = wasm.utf8buffer_slice(this.__wbg_ptr, start, end);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Utf8Buffer.__wrap(ret[0]);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     * @param {number} at
     * @returns {Utf8Buffer}
     */
    splitOff(at) {
        const ret = wasm.utf8buffer_splitOff(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Utf8Buffer.__wrap(ret[0]);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     * @param {number} at
     * @returns {Utf8Buffer}
     */
    splitTo(at) {
        const ret = wasm.utf8buffer_splitTo(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Utf8Buffer.__wrap(ret[0]);
    }
    /**
     * Return contents as a `Uint8Array` (copy of the UTF-8 encoded bytes).
     * @returns {Uint8Array}
     */
    toBytes() {
        const ret = wasm.utf8buffer_toBytes(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Return contents as a UTF-8 string.
     * @returns {string}
     */
    toString() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.utf8buffer_toString(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
     * @param {number} new_len
     */
    truncate(new_len) {
        wasm.utf8buffer_truncate(this.__wbg_ptr, new_len);
    }
}
if (Symbol.dispose) Utf8Buffer.prototype[Symbol.dispose] = Utf8Buffer.prototype.free;

/**
 * WASM bindings for the [`Utf8Bytes`](crate::Utf8Bytes) type (shared variant).
 */
export class Utf8Bytes {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Utf8Bytes.prototype);
        obj.__wbg_ptr = ptr;
        Utf8BytesFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Utf8BytesFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_utf8bytes_free(ptr, 0);
    }
    /**
     * Advance the read cursor by `cnt` bytes.
     *
     * @throws {Error} If `cnt` exceeds the number of remaining bytes.
     * @param {number} cnt
     */
    advance(cnt) {
        const ret = wasm.utf8bytes_advance(this.__wbg_ptr, cnt);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Create a `Utf8Bytes` from a static string.
     *
     * Note: In WASM, this copies the data (true static references cannot cross the JS boundary).
     * @param {string} s
     * @returns {Utf8Bytes}
     */
    static fromStatic(s) {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.utf8bytes_fromStatic(ptr0, len0);
        return Utf8Bytes.__wrap(ret);
    }
    /**
     * Create a `Utf8Bytes` from a UTF-8 string.
     *
     * @param s - The source string whose bytes are copied.
     * @param {string} s
     * @returns {Utf8Bytes}
     */
    static fromString(s) {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.utf8bytes_fromString(ptr0, len0);
        return Utf8Bytes.__wrap(ret);
    }
    /**
     * Return `true` if the buffer has no bytes.
     * @returns {boolean}
     */
    isEmpty() {
        const ret = wasm.utf8bytes_isEmpty(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return `true` if data is stored on the heap.
     * @returns {boolean}
     */
    isHeap() {
        const ret = wasm.utf8bytes_isHeap(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return `true` if data is stored inline (no heap allocation).
     * @returns {boolean}
     */
    isInline() {
        const ret = wasm.utf8bytes_isInline(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return a JS-compatible character iterator.
     *
     * Use `for (const ch of buf)` after attaching `Symbol.iterator`.
     * @returns {CharIterator}
     */
    iter() {
        const ret = wasm.utf8bytes_iter(this.__wbg_ptr);
        return CharIterator.__wrap(ret);
    }
    /**
     * Return the byte length of the buffer.
     * @returns {number}
     */
    len() {
        const ret = wasm.utf8bytes_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a new empty `Utf8Bytes`.
     */
    constructor() {
        const ret = wasm.utf8bytes_new_wasm();
        this.__wbg_ptr = ret >>> 0;
        Utf8BytesFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Return the number of readable bytes remaining.
     * @returns {number}
     */
    remaining() {
        const ret = wasm.utf8bytes_remaining(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Return a copy of bytes in range `[start, end)`.
     *
     * @throws {Error} If the range is out of bounds or not on UTF-8 char boundaries.
     * @param {number} start
     * @param {number} end
     * @returns {Utf8Bytes}
     */
    slice(start, end) {
        const ret = wasm.utf8bytes_slice(this.__wbg_ptr, start, end);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Utf8Bytes.__wrap(ret[0]);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     * @param {number} at
     * @returns {Utf8Bytes}
     */
    splitOff(at) {
        const ret = wasm.utf8bytes_splitOff(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Utf8Bytes.__wrap(ret[0]);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     * @param {number} at
     * @returns {Utf8Bytes}
     */
    splitTo(at) {
        const ret = wasm.utf8bytes_splitTo(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Utf8Bytes.__wrap(ret[0]);
    }
    /**
     * Return contents as a `Uint8Array` (copy of the UTF-8 encoded bytes).
     * @returns {Uint8Array}
     */
    toBytes() {
        const ret = wasm.utf8bytes_toBytes(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Return contents as a UTF-8 string.
     * @returns {string}
     */
    toString() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.utf8bytes_toString(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) Utf8Bytes.prototype[Symbol.dispose] = Utf8Bytes.prototype.free;

/**
 * Growable mutable UTF-8 string with inline/heap storage.
 *
 * Guarantees valid UTF-8. Split operations check char boundaries.
 *
 * @example
 * ```typescript
 * import { Utf8BytesMut } from 'smol-bytes';
 * const buf = new Utf8BytesMut();
 * buf.pushStr('hello world');
 * console.log(buf.toString()); // 'hello world'
 * ```
 */
export class Utf8BytesMut {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Utf8BytesMut.prototype);
        obj.__wbg_ptr = ptr;
        Utf8BytesMutFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        Utf8BytesMutFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_utf8bytesmut_free(ptr, 0);
    }
    /**
     * Return the total allocated capacity in bytes.
     * @returns {number}
     */
    capacity() {
        const ret = wasm.utf8bytesmut_capacity(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Clear the buffer, removing all data.
     */
    clear() {
        wasm.utf8bytesmut_clear(this.__wbg_ptr);
    }
    /**
     * Create a `Utf8BytesMut` from a UTF-8 string.
     *
     * @param s - The source string whose bytes are copied.
     * @param {string} s
     * @returns {Utf8BytesMut}
     */
    static fromString(s) {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.utf8bytesmut_fromString(ptr0, len0);
        return Utf8BytesMut.__wrap(ret);
    }
    /**
     * Return `true` if the buffer has no bytes.
     * @returns {boolean}
     */
    isEmpty() {
        const ret = wasm.utf8bytesmut_isEmpty(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return `true` if data is stored on the heap.
     * @returns {boolean}
     */
    isHeap() {
        const ret = wasm.utf8bytesmut_isHeap(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return `true` if data is stored inline (no heap allocation).
     * @returns {boolean}
     */
    isInline() {
        const ret = wasm.utf8bytesmut_isInline(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Return a JS-compatible character iterator.
     *
     * Use `for (const ch of buf)` after attaching `Symbol.iterator`.
     * @returns {CharIterator}
     */
    iter() {
        const ret = wasm.utf8bytesmut_iter(this.__wbg_ptr);
        return CharIterator.__wrap(ret);
    }
    /**
     * Return the byte length of the buffer.
     * @returns {number}
     */
    len() {
        const ret = wasm.utf8bytesmut_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a new empty `Utf8BytesMut`.
     */
    constructor() {
        const ret = wasm.utf8bytesmut_new_wasm();
        this.__wbg_ptr = ret >>> 0;
        Utf8BytesMutFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Append a single character.
     *
     * @param ch - A single-character string.
     * @throws {Error} If the string is empty.
     * @param {string} ch
     */
    push(ch) {
        const ptr0 = passStringToWasm0(ch, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.utf8bytesmut_push(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Append a string.
     *
     * @param s - The string to append.
     * @param {string} s
     */
    pushStr(s) {
        const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.utf8bytesmut_pushStr(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Reserve capacity for at least `additional` more bytes.
     * @param {number} additional
     */
    reserve(additional) {
        wasm.utf8bytesmut_reserve(this.__wbg_ptr, additional);
    }
    /**
     * Return a copy of bytes in range `[start, end)`.
     *
     * @throws {Error} If the range is out of bounds or not on UTF-8 char boundaries.
     * @param {number} start
     * @param {number} end
     * @returns {Utf8BytesMut}
     */
    slice(start, end) {
        const ret = wasm.utf8bytesmut_slice(this.__wbg_ptr, start, end);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Utf8BytesMut.__wrap(ret[0]);
    }
    /**
     * Split all data out of this buffer, returning it. Self becomes empty.
     * @returns {Utf8BytesMut}
     */
    split() {
        const ret = wasm.utf8bytesmut_split(this.__wbg_ptr);
        return Utf8BytesMut.__wrap(ret);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     * @param {number} at
     * @returns {Utf8BytesMut}
     */
    splitOff(at) {
        const ret = wasm.utf8bytesmut_splitOff(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Utf8BytesMut.__wrap(ret[0]);
    }
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     * @param {number} at
     * @returns {Utf8BytesMut}
     */
    splitTo(at) {
        const ret = wasm.utf8bytesmut_splitTo(this.__wbg_ptr, at);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Utf8BytesMut.__wrap(ret[0]);
    }
    /**
     * Return contents as a `Uint8Array` (copy of the UTF-8 encoded bytes).
     * @returns {Uint8Array}
     */
    toBytes() {
        const ret = wasm.utf8bytesmut_toBytes(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Return contents as a UTF-8 string.
     * @returns {string}
     */
    toString() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.utf8bytesmut_toString(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
     * @param {number} new_len
     */
    truncate(new_len) {
        wasm.utf8bytesmut_truncate(this.__wbg_ptr, new_len);
    }
    /**
     * Attempt to re-merge a previously split buffer back into this one.
     *
     * Returns `other` unchanged if the two buffers are not contiguous.
     * @param {Utf8BytesMut} other
     * @returns {Utf8BytesMut | undefined}
     */
    unsplit(other) {
        _assertClass(other, Utf8BytesMut);
        var ptr0 = other.__destroy_into_raw();
        const ret = wasm.utf8bytesmut_unsplit(this.__wbg_ptr, ptr0);
        return ret === 0 ? undefined : Utf8BytesMut.__wrap(ret);
    }
    /**
     * Create a new `Utf8BytesMut` with the given capacity pre-allocated.
     *
     * @param capacity - Number of bytes to pre-allocate.
     * @param {number} capacity
     * @returns {Utf8BytesMut}
     */
    static withCapacity(capacity) {
        const ret = wasm.utf8bytesmut_withCapacity(capacity);
        return Utf8BytesMut.__wrap(ret);
    }
}
if (Symbol.dispose) Utf8BytesMut.prototype[Symbol.dispose] = Utf8BytesMut.prototype.free;
export function __wbg_Error_2e59b1b37a9a34c3(arg0, arg1) {
    const ret = Error(getStringFromWasm0(arg0, arg1));
    return ret;
}
export function __wbg___wbindgen_throw_81fc77679af83bc6(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
}
export function __wbg_new_4f9fafbb3909af72() {
    const ret = new Object();
    return ret;
}
export function __wbg_set_8ee2d34facb8466e() { return handleError(function (arg0, arg1, arg2) {
    const ret = Reflect.set(arg0, arg1, arg2);
    return ret;
}, arguments); }
export function __wbindgen_cast_0000000000000001(arg0) {
    // Cast intrinsic for `F64 -> Externref`.
    const ret = arg0;
    return ret;
}
export function __wbindgen_cast_0000000000000002(arg0, arg1) {
    // Cast intrinsic for `Ref(String) -> Externref`.
    const ret = getStringFromWasm0(arg0, arg1);
    return ret;
}
export function __wbindgen_init_externref_table() {
    const table = wasm.__wbindgen_externrefs;
    const offset = table.grow(4);
    table.set(0, undefined);
    table.set(offset + 0, undefined);
    table.set(offset + 1, null);
    table.set(offset + 2, true);
    table.set(offset + 3, false);
}
const BufferFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_buffer_free(ptr >>> 0, 1));
const ByteIteratorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_byteiterator_free(ptr >>> 0, 1));
const BytesMutFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_bytesmut_free(ptr >>> 0, 1));
const CharIteratorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_chariterator_free(ptr >>> 0, 1));
const Utf8BufferFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_utf8buffer_free(ptr >>> 0, 1));
const Utf8BytesMutFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_utf8bytesmut_free(ptr >>> 0, 1));
const CompactBytesFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_compactbytes_free(ptr >>> 0, 1));
const CompactUtf8BytesFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_compactutf8bytes_free(ptr >>> 0, 1));
const SharedBytesFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_sharedbytes_free(ptr >>> 0, 1));
const Utf8BytesFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_utf8bytes_free(ptr >>> 0, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
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


let wasm;
export function __wbg_set_wasm(val) {
    wasm = val;
}
