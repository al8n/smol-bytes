/** Final 0.2 JavaScript API for the shared immutable strategy. */

export type ByteInput = Uint8Array;

/** Runtime byte iterator. Instances are returned by `iter()`. */
export declare class ByteIterator implements IterableIterator<number> {
  private constructor();
  next(): IteratorResult<number>;
  free(): void;
  [Symbol.iterator](): IterableIterator<number>;
  [Symbol.dispose](): void;
}

/** Runtime Unicode-scalar iterator. Instances are returned by `iter()`. */
export declare class CharIterator implements IterableIterator<string> {
  private constructor();
  next(): IteratorResult<string>;
  free(): void;
  [Symbol.iterator](): IterableIterator<string>;
  [Symbol.dispose](): void;
}

interface Disposable {
  free(): void;
  [Symbol.dispose](): void;
}

interface RawReadable<TSelf> extends Disposable, Iterable<number> {
  toBytes(): Uint8Array;
  decodeUtf8(): string;
  len(): number;
  isEmpty(): boolean;
  remaining(): number;
  advance(count: number): void;
  clear(): void;
  truncate(newLen: number): void;
  splitTo(at: number): TSelf;
  splitOff(at: number): TSelf;
  slice(start: number, end: number): TSelf;
  clone(): TSelf;
  getU8(): number;
  getI8(): number;
  getU16(): number;
  getU16Le(): number;
  getI16(): number;
  getI16Le(): number;
  getU32(): number;
  getU32Le(): number;
  getI32(): number;
  getI32Le(): number;
  getU64(): bigint;
  getU64Le(): bigint;
  getI64(): bigint;
  getI64Le(): bigint;
  getF32(): number;
  getF32Le(): number;
  getF64(): number;
  getF64Le(): number;
  getUint(width: number): bigint;
  getUintLe(width: number): bigint;
  getUintNe(width: number): bigint;
  getInt(width: number): bigint;
  getIntLe(width: number): bigint;
  getIntNe(width: number): bigint;
  iter(): ByteIterator;
  [Symbol.iterator](): IterableIterator<number>;
}

interface RawWritable {
  putSlice(data: ByteInput): void;
  putBytes(value: number, count: number): void;
  putU8(value: number): void;
  putI8(value: number): void;
  putU16(value: number): void;
  putU16Le(value: number): void;
  putI16(value: number): void;
  putI16Le(value: number): void;
  putU32(value: number): void;
  putU32Le(value: number): void;
  putI32(value: number): void;
  putI32Le(value: number): void;
  putU64(value: bigint): void;
  putU64Le(value: bigint): void;
  putI64(value: bigint): void;
  putI64Le(value: bigint): void;
  putF32(value: number): void;
  putF32Le(value: number): void;
  putF64(value: number): void;
  putF64Le(value: number): void;
  putUint(value: bigint, width: number): void;
  putUintLe(value: bigint, width: number): void;
  putUintNe(value: bigint, width: number): void;
  putInt(value: bigint, width: number): void;
  putIntLe(value: bigint, width: number): void;
  putIntNe(value: bigint, width: number): void;
}

export interface Buffer extends RawReadable<Buffer>, RawWritable {
  capacity(): number;
  remainingMut(): number;
  resize(newLen: number, value?: number): void;
}

export declare const Buffer: {
  new (): Buffer;
  fromBytes(data: ByteInput): Buffer;
  fromUtf8(s: string): Buffer;
};

export interface BytesMut extends RawReadable<BytesMut>, RawWritable {
  capacity(): number;
  remainingMut(): number;
  isInline(): boolean;
  isHeap(): boolean;
  reserve(additional: number): void;
  resize(newLen: number, value?: number): void;
  makeHeap(): void;
  split(): BytesMut;
  unsplit(other: BytesMut): void;
  freezeShared(): Bytes;
  freezeCompact(): import('./compact.js').Bytes;
}

export declare const BytesMut: {
  new (): BytesMut;
  withCapacity(capacity: number): BytesMut;
  zeroed(length: number): BytesMut;
  fromBytes(data: ByteInput): BytesMut;
  fromUtf8(s: string): BytesMut;
};

interface ImmutableRaw<TSelf> extends RawReadable<TSelf> {
  isInline(): boolean;
  isHeap(): boolean;
  toMutable(): BytesMut;
}

export interface Bytes extends ImmutableRaw<Bytes> {}

export declare const Bytes: {
  new (): Bytes;
  fromBytes(data: ByteInput): Bytes;
  fromUtf8(s: string): Bytes;
};

interface Utf8Common<TSelf> extends Disposable, Iterable<string> {
  toString(): string;
  toBytes(): Uint8Array;
  byteLength(): number;
  charLength(): number;
  isEmpty(): boolean;
  remainingBytes(): number;
  advanceBytes(count: number): void;
  clear(): void;
  truncateBytes(newLen: number): void;
  splitToBytes(at: number): TSelf;
  splitOffBytes(at: number): TSelf;
  sliceBytes(start: number, end: number): TSelf;
  clone(): TSelf;
  iter(): CharIterator;
  [Symbol.iterator](): IterableIterator<string>;
}

export interface Utf8Buffer extends Utf8Common<Utf8Buffer> {
  capacity(): number;
  push(ch: string): void;
  pushStr(s: string): void;
}

export declare const Utf8Buffer: {
  new (): Utf8Buffer;
  fromString(s: string): Utf8Buffer;
};

export interface Utf8BytesMut extends Utf8Common<Utf8BytesMut> {
  capacity(): number;
  isInline(): boolean;
  isHeap(): boolean;
  push(ch: string): void;
  pushStr(s: string): void;
  reserve(additional: number): void;
  split(): Utf8BytesMut;
  unsplit(other: Utf8BytesMut): void;
  freezeShared(): Utf8Bytes;
  freezeCompact(): import('./compact.js').Utf8Bytes;
}

export declare const Utf8BytesMut: {
  new (): Utf8BytesMut;
  withCapacity(capacity: number): Utf8BytesMut;
  fromString(s: string): Utf8BytesMut;
};

interface ImmutableUtf8<TSelf> extends Utf8Common<TSelf> {
  isInline(): boolean;
  isHeap(): boolean;
  toMutable(): Utf8BytesMut;
}

export interface Utf8Bytes extends ImmutableUtf8<Utf8Bytes> {}

export declare const Utf8Bytes: {
  new (): Utf8Bytes;
  fromString(s: string): Utf8Bytes;
};
