export class Bytes {
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
     * Create a `Bytes` from a byte array.
     *
     * @param data - The source bytes to copy.
     */
    static fromBytes(data: Uint8Array): Bytes;
    /**
     * Create a `Bytes` from a static string.
     *
     * Note: In WASM, this copies the data (true static references cannot cross the JS boundary).
     */
    static fromStatic(s: string): Bytes;
    /**
     * Create a `Bytes` from a UTF-8 string.
     *
     * @param s - The source string whose bytes are copied.
     */
    static fromString(s: string): Bytes;
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
     * Create a new empty `Bytes`.
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
    slice(start: number, end: number): Bytes;
    /**
     * Split the buffer at position `at`, returning bytes `[at, len)`.
     * Self becomes `[0, at)`.
     *
     * @throws {Error} If `at` is out of bounds.
     */
    splitOff(at: number): Bytes;
    /**
     * Split the buffer at position `at`, returning bytes `[0, at)`.
     * Self becomes `[at, len)`.
     *
     * @throws {Error} If `at` is out of bounds.
     */
    splitTo(at: number): Bytes;
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
export class Utf8Bytes {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Advance the read cursor by `cnt` bytes.
     *
     * @throws {Error} If `cnt` exceeds the number of remaining bytes or does not end on a UTF-8 character boundary.
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
