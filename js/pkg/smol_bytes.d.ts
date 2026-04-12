/* tslint:disable */
/* eslint-disable */

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
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Advance the read cursor by `cnt` bytes.
     *
     * @throws {Error} If `cnt` exceeds the number of remaining bytes.
     */
    advance(cnt: number): void;
    /**
     * Clear the buffer, removing all data and resetting the cursor.
     */
    clear(): void;
    /**
     * Create a Buffer from a byte array.
     *
     * @param data - The source bytes to copy.
     * @throws {Error} If the data exceeds inline capacity (62 bytes).
     */
    static fromBytes(data: Uint8Array): Buffer;
    /**
     * Create a Buffer from a UTF-8 string.
     *
     * @param s - The source string whose bytes are copied.
     * @throws {Error} If the encoded bytes exceed inline capacity (62 bytes).
     */
    static fromString(s: string): Buffer;
    /**
     * Read a 32-bit float in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF32(): number;
    /**
     * Read a 32-bit float in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF32Le(): number;
    /**
     * Read a 64-bit float in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF64(): number;
    /**
     * Read a 64-bit float in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF64Le(): number;
    /**
     * Read a signed 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI16(): number;
    /**
     * Read a signed 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI16Le(): number;
    /**
     * Read a signed 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI32(): number;
    /**
     * Read a signed 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI32Le(): number;
    /**
     * Read a signed 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI64(): bigint;
    /**
     * Read a signed 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI64Le(): bigint;
    /**
     * Read a signed 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     */
    getI8(): number;
    /**
     * Read an unsigned 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU16(): number;
    /**
     * Read an unsigned 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU16Le(): number;
    /**
     * Read an unsigned 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU32(): number;
    /**
     * Read an unsigned 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU32Le(): number;
    /**
     * Read an unsigned 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU64(): bigint;
    /**
     * Read an unsigned 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU64Le(): bigint;
    /**
     * Read an unsigned 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     */
    getU8(): number;
    /**
     * Return `true` if the buffer has no bytes.
     */
    isEmpty(): boolean;
    /**
     * Return a JS-compatible byte iterator.
     *
     * Use `for (const b of buf)` after attaching `Symbol.iterator`.
     */
    iter(): ByteIterator;
    /**
     * Return the byte length of the buffer.
     */
    len(): number;
    /**
     * Create a new empty Buffer.
     */
    constructor();
    /**
     * Write a 32-bit float in big-endian byte order.
     */
    putF32(val: number): void;
    /**
     * Write a 32-bit float in little-endian byte order.
     */
    putF32Le(val: number): void;
    /**
     * Write a 64-bit float in big-endian byte order.
     */
    putF64(val: number): void;
    /**
     * Write a 64-bit float in little-endian byte order.
     */
    putF64Le(val: number): void;
    /**
     * Write a signed 16-bit integer in big-endian byte order.
     */
    putI16(val: number): void;
    /**
     * Write a signed 16-bit integer in little-endian byte order.
     */
    putI16Le(val: number): void;
    /**
     * Write a signed 32-bit integer in big-endian byte order.
     */
    putI32(val: number): void;
    /**
     * Write a signed 32-bit integer in little-endian byte order.
     */
    putI32Le(val: number): void;
    /**
     * Write a signed 64-bit integer in big-endian byte order.
     */
    putI64(val: bigint): void;
    /**
     * Write a signed 64-bit integer in little-endian byte order.
     */
    putI64Le(val: bigint): void;
    /**
     * Write a signed 8-bit integer.
     */
    putI8(val: number): void;
    /**
     * Write a byte slice into the buffer.
     *
     * @param data - The bytes to append.
     * @throws {Error} If the data would exceed inline capacity (62 bytes).
     */
    putSlice(data: Uint8Array): void;
    /**
     * Write an unsigned 16-bit integer in big-endian byte order.
     */
    putU16(val: number): void;
    /**
     * Write an unsigned 16-bit integer in little-endian byte order.
     */
    putU16Le(val: number): void;
    /**
     * Write an unsigned 32-bit integer in big-endian byte order.
     */
    putU32(val: number): void;
    /**
     * Write an unsigned 32-bit integer in little-endian byte order.
     */
    putU32Le(val: number): void;
    /**
     * Write an unsigned 64-bit integer in big-endian byte order.
     */
    putU64(val: bigint): void;
    /**
     * Write an unsigned 64-bit integer in little-endian byte order.
     */
    putU64Le(val: bigint): void;
    /**
     * Write an unsigned 8-bit integer.
     */
    putU8(val: number): void;
    /**
     * Return the number of readable bytes remaining.
     */
    remaining(): number;
    /**
     * Return a copy of bytes in range `[start, end)`.
     *
     * @throws {Error} If the range is out of bounds.
     */
    slice(start: number, end: number): Buffer;
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds.
     */
    splitOff(at: number): Buffer;
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds.
     */
    splitTo(at: number): Buffer;
    /**
     * Return contents as a `Uint8Array` (copy).
     */
    toBytes(): Uint8Array;
    /**
     * Return contents as a UTF-8 string.
     *
     * @throws {Error} If the buffer contains invalid UTF-8.
     */
    toString(): string;
    /**
     * Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
     */
    truncate(new_len: number): void;
}

/**
 * Iterator over bytes, compatible with the JS iterator protocol.
 *
 * Each call to `next()` returns `{ value: number, done: false }` or `{ done: true }`.
 * Attach `Symbol.iterator` to use with `for...of` loops.
 */
export class ByteIterator {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Return the next `{ value, done }` object per the JS iterator protocol.
     *
     * Returns `{ value: number, done: false }` while bytes remain,
     * then `{ done: true }` when exhausted.
     */
    next(): any;
}

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
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Advance the read cursor by `cnt` bytes.
     *
     * @throws {Error} If `cnt` exceeds the number of remaining bytes.
     */
    advance(cnt: number): void;
    /**
     * Return the total allocated capacity in bytes.
     */
    capacity(): number;
    /**
     * Clear the buffer, removing all data and resetting the cursor.
     */
    clear(): void;
    /**
     * Create a `BytesMut` from a byte array.
     *
     * @param data - The source bytes to copy.
     */
    static fromBytes(data: Uint8Array): BytesMut;
    /**
     * Create a `BytesMut` from a UTF-8 string.
     *
     * @param s - The source string whose bytes are copied.
     */
    static fromString(s: string): BytesMut;
    /**
     * Read a 32-bit float in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF32(): number;
    /**
     * Read a 32-bit float in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF32Le(): number;
    /**
     * Read a 64-bit float in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF64(): number;
    /**
     * Read a 64-bit float in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF64Le(): number;
    /**
     * Read a signed 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI16(): number;
    /**
     * Read a signed 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI16Le(): number;
    /**
     * Read a signed 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI32(): number;
    /**
     * Read a signed 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI32Le(): number;
    /**
     * Read a signed 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI64(): bigint;
    /**
     * Read a signed 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI64Le(): bigint;
    /**
     * Read a signed 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     */
    getI8(): number;
    /**
     * Read an unsigned 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU16(): number;
    /**
     * Read an unsigned 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU16Le(): number;
    /**
     * Read an unsigned 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU32(): number;
    /**
     * Read an unsigned 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU32Le(): number;
    /**
     * Read an unsigned 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU64(): bigint;
    /**
     * Read an unsigned 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU64Le(): bigint;
    /**
     * Read an unsigned 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     */
    getU8(): number;
    /**
     * Return `true` if the buffer has no bytes.
     */
    isEmpty(): boolean;
    /**
     * Return `true` if data is stored on the heap.
     */
    isHeap(): boolean;
    /**
     * Return `true` if data is stored inline (no heap allocation).
     */
    isInline(): boolean;
    /**
     * Return a JS-compatible byte iterator.
     *
     * Use `for (const b of buf)` after attaching `Symbol.iterator`.
     */
    iter(): ByteIterator;
    /**
     * Return the byte length of the buffer.
     */
    len(): number;
    /**
     * Create a new empty `BytesMut`.
     */
    constructor();
    /**
     * Write a 32-bit float in big-endian byte order.
     */
    putF32(val: number): void;
    /**
     * Write a 32-bit float in little-endian byte order.
     */
    putF32Le(val: number): void;
    /**
     * Write a 64-bit float in big-endian byte order.
     */
    putF64(val: number): void;
    /**
     * Write a 64-bit float in little-endian byte order.
     */
    putF64Le(val: number): void;
    /**
     * Write a signed 16-bit integer in big-endian byte order.
     */
    putI16(val: number): void;
    /**
     * Write a signed 16-bit integer in little-endian byte order.
     */
    putI16Le(val: number): void;
    /**
     * Write a signed 32-bit integer in big-endian byte order.
     */
    putI32(val: number): void;
    /**
     * Write a signed 32-bit integer in little-endian byte order.
     */
    putI32Le(val: number): void;
    /**
     * Write a signed 64-bit integer in big-endian byte order.
     */
    putI64(val: bigint): void;
    /**
     * Write a signed 64-bit integer in little-endian byte order.
     */
    putI64Le(val: bigint): void;
    /**
     * Write a signed 8-bit integer.
     */
    putI8(val: number): void;
    /**
     * Write a byte slice into the buffer.
     *
     * @param data - The bytes to append.
     */
    putSlice(data: Uint8Array): void;
    /**
     * Write an unsigned 16-bit integer in big-endian byte order.
     */
    putU16(val: number): void;
    /**
     * Write an unsigned 16-bit integer in little-endian byte order.
     */
    putU16Le(val: number): void;
    /**
     * Write an unsigned 32-bit integer in big-endian byte order.
     */
    putU32(val: number): void;
    /**
     * Write an unsigned 32-bit integer in little-endian byte order.
     */
    putU32Le(val: number): void;
    /**
     * Write an unsigned 64-bit integer in big-endian byte order.
     */
    putU64(val: bigint): void;
    /**
     * Write an unsigned 64-bit integer in little-endian byte order.
     */
    putU64Le(val: bigint): void;
    /**
     * Write an unsigned 8-bit integer.
     */
    putU8(val: number): void;
    /**
     * Return the number of readable bytes remaining.
     */
    remaining(): number;
    /**
     * Reserve capacity for at least `additional` more bytes.
     */
    reserve(additional: number): void;
    /**
     * Resize the buffer to `new_len`, filling new bytes with `value` if growing.
     */
    resize(new_len: number, value: number): void;
    /**
     * Split all data out of this buffer, returning it. Self becomes empty.
     */
    split(): BytesMut;
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds.
     */
    splitOff(at: number): BytesMut;
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds.
     */
    splitTo(at: number): BytesMut;
    /**
     * Return contents as a `Uint8Array` (copy).
     */
    toBytes(): Uint8Array;
    /**
     * Return contents as a UTF-8 string.
     *
     * @throws {Error} If the buffer contains invalid UTF-8.
     */
    toString(): string;
    /**
     * Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
     */
    truncate(new_len: number): void;
    /**
     * Attempt to re-merge a previously split buffer back into this one.
     *
     * Returns `other` unchanged if the two buffers are not contiguous.
     */
    unsplit(other: BytesMut): BytesMut | undefined;
    /**
     * Create a new `BytesMut` with the given capacity pre-allocated.
     *
     * @param capacity - Number of bytes to pre-allocate.
     */
    static withCapacity(capacity: number): BytesMut;
}

/**
 * Iterator over characters, compatible with the JS iterator protocol.
 *
 * Each call to `next()` returns `{ value: string, done: false }` or `{ done: true }`.
 * Attach `Symbol.iterator` to use with `for...of` loops.
 */
export class CharIterator {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Return the next `{ value, done }` object per the JS iterator protocol.
     *
     * Returns `{ value: string, done: false }` while characters remain,
     * then `{ done: true }` when exhausted.
     */
    next(): any;
}

/**
 * Immutable byte buffer using the Compact strategy.
 *
 * Stores up to 62 bytes inline with unique ownership for larger data (no reference counting).
 * Available as `CompactBytes` via `import { CompactBytes } from 'smol-bytes/compact'`.
 */
export class CompactBytes {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Advance the read cursor by `cnt` bytes.
     *
     * @throws {Error} If `cnt` exceeds the number of remaining bytes.
     */
    advance(cnt: number): void;
    /**
     * Clear the buffer, removing all data and resetting the cursor.
     */
    clear(): void;
    /**
     * Create a `CompactBytes` from a byte array.
     *
     * @param data - The source bytes to copy.
     */
    static fromBytes(data: Uint8Array): CompactBytes;
    /**
     * Create a `CompactBytes` from a static string.
     *
     * Note: In WASM, this copies the data (true static references cannot cross the JS boundary).
     */
    static fromStatic(s: string): CompactBytes;
    /**
     * Create a `CompactBytes` from a UTF-8 string.
     *
     * @param s - The source string whose bytes are copied.
     */
    static fromString(s: string): CompactBytes;
    /**
     * Read a 32-bit float in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF32(): number;
    /**
     * Read a 32-bit float in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF32Le(): number;
    /**
     * Read a 64-bit float in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF64(): number;
    /**
     * Read a 64-bit float in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF64Le(): number;
    /**
     * Read a signed 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI16(): number;
    /**
     * Read a signed 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI16Le(): number;
    /**
     * Read a signed 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI32(): number;
    /**
     * Read a signed 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI32Le(): number;
    /**
     * Read a signed 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI64(): bigint;
    /**
     * Read a signed 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI64Le(): bigint;
    /**
     * Read a signed 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     */
    getI8(): number;
    /**
     * Read an unsigned 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU16(): number;
    /**
     * Read an unsigned 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU16Le(): number;
    /**
     * Read an unsigned 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU32(): number;
    /**
     * Read an unsigned 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU32Le(): number;
    /**
     * Read an unsigned 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU64(): bigint;
    /**
     * Read an unsigned 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU64Le(): bigint;
    /**
     * Read an unsigned 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     */
    getU8(): number;
    /**
     * Return `true` if the buffer has no bytes.
     */
    isEmpty(): boolean;
    /**
     * Return `true` if data is stored on the heap.
     */
    isHeap(): boolean;
    /**
     * Return `true` if data is stored inline (no heap allocation).
     */
    isInline(): boolean;
    /**
     * Return a JS-compatible byte iterator.
     *
     * Use `for (const b of buf)` after attaching `Symbol.iterator`.
     */
    iter(): ByteIterator;
    /**
     * Return the byte length of the buffer.
     */
    len(): number;
    /**
     * Create a new empty `CompactBytes`.
     */
    constructor();
    /**
     * Return the number of readable bytes remaining.
     */
    remaining(): number;
    /**
     * Return a copy of bytes in range `[start, end)`.
     *
     * @throws {Error} If the range is out of bounds.
     */
    slice(start: number, end: number): CompactBytes;
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds.
     */
    splitOff(at: number): CompactBytes;
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds.
     */
    splitTo(at: number): CompactBytes;
    /**
     * Return contents as a `Uint8Array` (copy).
     */
    toBytes(): Uint8Array;
    /**
     * Return contents as a UTF-8 string.
     *
     * @throws {Error} If the buffer contains invalid UTF-8.
     */
    toString(): string;
    /**
     * Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
     */
    truncate(new_len: number): void;
}

/**
 * WASM bindings for the [`Utf8Bytes`](crate::Utf8Bytes) type (compact variant).
 *
 * Available as `CompactUtf8Bytes` via `import { CompactUtf8Bytes } from 'smol-bytes/compact'`.
 */
export class CompactUtf8Bytes {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Advance the read cursor by `cnt` bytes.
     *
     * @throws {Error} If `cnt` exceeds the number of remaining bytes.
     */
    advance(cnt: number): void;
    /**
     * Create a `CompactUtf8Bytes` from a static string.
     *
     * Note: In WASM, this copies the data (true static references cannot cross the JS boundary).
     */
    static fromStatic(s: string): CompactUtf8Bytes;
    /**
     * Create a `CompactUtf8Bytes` from a UTF-8 string.
     *
     * @param s - The source string whose bytes are copied.
     */
    static fromString(s: string): CompactUtf8Bytes;
    /**
     * Return `true` if the buffer has no bytes.
     */
    isEmpty(): boolean;
    /**
     * Return `true` if data is stored on the heap.
     */
    isHeap(): boolean;
    /**
     * Return `true` if data is stored inline (no heap allocation).
     */
    isInline(): boolean;
    /**
     * Return a JS-compatible character iterator.
     *
     * Use `for (const ch of buf)` after attaching `Symbol.iterator`.
     */
    iter(): CharIterator;
    /**
     * Return the byte length of the buffer.
     */
    len(): number;
    /**
     * Create a new empty `CompactUtf8Bytes`.
     */
    constructor();
    /**
     * Return the number of readable bytes remaining.
     */
    remaining(): number;
    /**
     * Return a copy of bytes in range `[start, end)`.
     *
     * @throws {Error} If the range is out of bounds or not on UTF-8 char boundaries.
     */
    slice(start: number, end: number): CompactUtf8Bytes;
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     */
    splitOff(at: number): CompactUtf8Bytes;
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     */
    splitTo(at: number): CompactUtf8Bytes;
    /**
     * Return contents as a `Uint8Array` (copy of the UTF-8 encoded bytes).
     */
    toBytes(): Uint8Array;
    /**
     * Return contents as a UTF-8 string.
     */
    toString(): string;
}

/**
 * Immutable byte buffer using the Shared strategy.
 *
 * Stores up to 62 bytes inline with zero-copy reference-counted cloning for larger data.
 * Available as `SharedBytes` via `import { SharedBytes } from 'smol-bytes/shared'`.
 */
export class SharedBytes {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Advance the read cursor by `cnt` bytes.
     *
     * @throws {Error} If `cnt` exceeds the number of remaining bytes.
     */
    advance(cnt: number): void;
    /**
     * Clear the buffer, removing all data and resetting the cursor.
     */
    clear(): void;
    /**
     * Create a `SharedBytes` from a byte array.
     *
     * @param data - The source bytes to copy.
     */
    static fromBytes(data: Uint8Array): SharedBytes;
    /**
     * Create a `SharedBytes` from a static string.
     *
     * Note: In WASM, this copies the data (true static references cannot cross the JS boundary).
     */
    static fromStatic(s: string): SharedBytes;
    /**
     * Create a `SharedBytes` from a UTF-8 string.
     *
     * @param s - The source string whose bytes are copied.
     */
    static fromString(s: string): SharedBytes;
    /**
     * Read a 32-bit float in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF32(): number;
    /**
     * Read a 32-bit float in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF32Le(): number;
    /**
     * Read a 64-bit float in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF64(): number;
    /**
     * Read a 64-bit float in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getF64Le(): number;
    /**
     * Read a signed 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI16(): number;
    /**
     * Read a signed 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI16Le(): number;
    /**
     * Read a signed 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI32(): number;
    /**
     * Read a signed 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI32Le(): number;
    /**
     * Read a signed 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI64(): bigint;
    /**
     * Read a signed 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getI64Le(): bigint;
    /**
     * Read a signed 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     */
    getI8(): number;
    /**
     * Read an unsigned 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU16(): number;
    /**
     * Read an unsigned 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU16Le(): number;
    /**
     * Read an unsigned 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU32(): number;
    /**
     * Read an unsigned 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU32Le(): number;
    /**
     * Read an unsigned 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU64(): bigint;
    /**
     * Read an unsigned 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
     *
     * @throws {Error} If not enough data remains.
     */
    getU64Le(): bigint;
    /**
     * Read an unsigned 8-bit integer, advancing the cursor by 1 byte.
     *
     * @throws {Error} If not enough data remains.
     */
    getU8(): number;
    /**
     * Return `true` if the buffer has no bytes.
     */
    isEmpty(): boolean;
    /**
     * Return `true` if data is stored on the heap.
     */
    isHeap(): boolean;
    /**
     * Return `true` if data is stored inline (no heap allocation).
     */
    isInline(): boolean;
    /**
     * Return a JS-compatible byte iterator.
     *
     * Use `for (const b of buf)` after attaching `Symbol.iterator`.
     */
    iter(): ByteIterator;
    /**
     * Return the byte length of the buffer.
     */
    len(): number;
    /**
     * Create a new empty `SharedBytes`.
     */
    constructor();
    /**
     * Return the number of readable bytes remaining.
     */
    remaining(): number;
    /**
     * Return a copy of bytes in range `[start, end)`.
     *
     * @throws {Error} If the range is out of bounds.
     */
    slice(start: number, end: number): SharedBytes;
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds.
     */
    splitOff(at: number): SharedBytes;
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds.
     */
    splitTo(at: number): SharedBytes;
    /**
     * Return contents as a `Uint8Array` (copy).
     */
    toBytes(): Uint8Array;
    /**
     * Return contents as a UTF-8 string.
     *
     * @throws {Error} If the buffer contains invalid UTF-8.
     */
    toString(): string;
    /**
     * Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
     */
    truncate(new_len: number): void;
}

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
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Return the total inline capacity in bytes.
     */
    capacity(): number;
    /**
     * Clear the buffer, removing all data.
     */
    clear(): void;
    /**
     * Create a `Utf8Buffer` from a UTF-8 string.
     *
     * @param s - The source string to copy.
     * @throws {Error} If the encoded bytes exceed inline capacity (62 bytes).
     */
    static fromString(s: string): Utf8Buffer;
    /**
     * Return `true` if the buffer has no bytes.
     */
    isEmpty(): boolean;
    /**
     * Return a JS-compatible character iterator.
     *
     * Use `for (const ch of buf)` after attaching `Symbol.iterator`.
     */
    iter(): CharIterator;
    /**
     * Return the byte length of the buffer.
     */
    len(): number;
    /**
     * Create a new empty `Utf8Buffer`.
     */
    constructor();
    /**
     * Append a single character.
     *
     * @param ch - A single-character string.
     * @throws {Error} If the string is empty or the character would exceed inline capacity.
     */
    push(ch: string): void;
    /**
     * Append a string.
     *
     * @param s - The string to append.
     * @throws {Error} If the combined length would exceed inline capacity.
     */
    pushStr(s: string): void;
    /**
     * Return the number of readable bytes remaining.
     */
    remaining(): number;
    /**
     * Return a copy of bytes in range `[start, end)`.
     *
     * @throws {Error} If the range is out of bounds or not on UTF-8 char boundaries.
     */
    slice(start: number, end: number): Utf8Buffer;
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     */
    splitOff(at: number): Utf8Buffer;
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     */
    splitTo(at: number): Utf8Buffer;
    /**
     * Return contents as a `Uint8Array` (copy of the UTF-8 encoded bytes).
     */
    toBytes(): Uint8Array;
    /**
     * Return contents as a UTF-8 string.
     */
    toString(): string;
    /**
     * Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
     */
    truncate(new_len: number): void;
}

/**
 * WASM bindings for the [`Utf8Bytes`](crate::Utf8Bytes) type (shared variant).
 */
export class Utf8Bytes {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Advance the read cursor by `cnt` bytes.
     *
     * @throws {Error} If `cnt` exceeds the number of remaining bytes.
     */
    advance(cnt: number): void;
    /**
     * Create a `Utf8Bytes` from a static string.
     *
     * Note: In WASM, this copies the data (true static references cannot cross the JS boundary).
     */
    static fromStatic(s: string): Utf8Bytes;
    /**
     * Create a `Utf8Bytes` from a UTF-8 string.
     *
     * @param s - The source string whose bytes are copied.
     */
    static fromString(s: string): Utf8Bytes;
    /**
     * Return `true` if the buffer has no bytes.
     */
    isEmpty(): boolean;
    /**
     * Return `true` if data is stored on the heap.
     */
    isHeap(): boolean;
    /**
     * Return `true` if data is stored inline (no heap allocation).
     */
    isInline(): boolean;
    /**
     * Return a JS-compatible character iterator.
     *
     * Use `for (const ch of buf)` after attaching `Symbol.iterator`.
     */
    iter(): CharIterator;
    /**
     * Return the byte length of the buffer.
     */
    len(): number;
    /**
     * Create a new empty `Utf8Bytes`.
     */
    constructor();
    /**
     * Return the number of readable bytes remaining.
     */
    remaining(): number;
    /**
     * Return a copy of bytes in range `[start, end)`.
     *
     * @throws {Error} If the range is out of bounds or not on UTF-8 char boundaries.
     */
    slice(start: number, end: number): Utf8Bytes;
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     */
    splitOff(at: number): Utf8Bytes;
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     */
    splitTo(at: number): Utf8Bytes;
    /**
     * Return contents as a `Uint8Array` (copy of the UTF-8 encoded bytes).
     */
    toBytes(): Uint8Array;
    /**
     * Return contents as a UTF-8 string.
     */
    toString(): string;
}

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
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Return the total allocated capacity in bytes.
     */
    capacity(): number;
    /**
     * Clear the buffer, removing all data.
     */
    clear(): void;
    /**
     * Create a `Utf8BytesMut` from a UTF-8 string.
     *
     * @param s - The source string whose bytes are copied.
     */
    static fromString(s: string): Utf8BytesMut;
    /**
     * Return `true` if the buffer has no bytes.
     */
    isEmpty(): boolean;
    /**
     * Return `true` if data is stored on the heap.
     */
    isHeap(): boolean;
    /**
     * Return `true` if data is stored inline (no heap allocation).
     */
    isInline(): boolean;
    /**
     * Return a JS-compatible character iterator.
     *
     * Use `for (const ch of buf)` after attaching `Symbol.iterator`.
     */
    iter(): CharIterator;
    /**
     * Return the byte length of the buffer.
     */
    len(): number;
    /**
     * Create a new empty `Utf8BytesMut`.
     */
    constructor();
    /**
     * Append a single character.
     *
     * @param ch - A single-character string.
     * @throws {Error} If the string is empty.
     */
    push(ch: string): void;
    /**
     * Append a string.
     *
     * @param s - The string to append.
     */
    pushStr(s: string): void;
    /**
     * Reserve capacity for at least `additional` more bytes.
     */
    reserve(additional: number): void;
    /**
     * Return a copy of bytes in range `[start, end)`.
     *
     * @throws {Error} If the range is out of bounds or not on UTF-8 char boundaries.
     */
    slice(start: number, end: number): Utf8BytesMut;
    /**
     * Split all data out of this buffer, returning it. Self becomes empty.
     */
    split(): Utf8BytesMut;
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     */
    splitOff(at: number): Utf8BytesMut;
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
     */
    splitTo(at: number): Utf8BytesMut;
    /**
     * Return contents as a `Uint8Array` (copy of the UTF-8 encoded bytes).
     */
    toBytes(): Uint8Array;
    /**
     * Return contents as a UTF-8 string.
     */
    toString(): string;
    /**
     * Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
     */
    truncate(new_len: number): void;
    /**
     * Attempt to re-merge a previously split buffer back into this one.
     *
     * Returns `other` unchanged if the two buffers are not contiguous.
     */
    unsplit(other: Utf8BytesMut): Utf8BytesMut | undefined;
    /**
     * Create a new `Utf8BytesMut` with the given capacity pre-allocated.
     *
     * @param capacity - Number of bytes to pre-allocate.
     */
    static withCapacity(capacity: number): Utf8BytesMut;
}
