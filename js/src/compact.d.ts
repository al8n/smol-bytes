import type { ByteInput, BytesMut, CharIterator, Utf8BytesMut } from './index.js';

export interface Bytes extends Iterable<number> {
  free(): void;
  [Symbol.dispose](): void;
  toBytes(): Uint8Array;
  decodeUtf8(): string;
  len(): number;
  isEmpty(): boolean;
  isInline(): boolean;
  isHeap(): boolean;
  remaining(): number;
  advance(count: number): void;
  clear(): void;
  truncate(newLen: number): void;
  splitTo(at: number): Bytes;
  splitOff(at: number): Bytes;
  slice(start: number, end: number): Bytes;
  clone(): Bytes;
  toMutable(): BytesMut;
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
  iter(): IterableIterator<number>;
  [Symbol.iterator](): IterableIterator<number>;
}

export declare const Bytes: {
  new (): Bytes;
  fromBytes(data: ByteInput): Bytes;
  fromUtf8(s: string): Bytes;
};

export interface Utf8Bytes extends Iterable<string> {
  free(): void;
  [Symbol.dispose](): void;
  toString(): string;
  toBytes(): Uint8Array;
  byteLength(): number;
  charLength(): number;
  isEmpty(): boolean;
  isInline(): boolean;
  isHeap(): boolean;
  remainingBytes(): number;
  advanceBytes(count: number): void;
  clear(): void;
  truncateBytes(newLen: number): void;
  splitToBytes(at: number): Utf8Bytes;
  splitOffBytes(at: number): Utf8Bytes;
  sliceBytes(start: number, end: number): Utf8Bytes;
  clone(): Utf8Bytes;
  toMutable(): Utf8BytesMut;
  iter(): CharIterator;
  [Symbol.iterator](): IterableIterator<string>;
}

export declare const Utf8Bytes: {
  new (): Utf8Bytes;
  fromString(s: string): Utf8Bytes;
};
