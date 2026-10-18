import { describe, expect, test } from 'vitest';

import {
  Buffer,
  ByteIterator,
  Bytes,
  BytesMut,
  CharIterator,
  Utf8Buffer,
  Utf8Bytes,
  Utf8BytesMut,
} from '../dist/index.js';
import {
  Bytes as SharedBytes,
  Utf8Bytes as SharedUtf8Bytes,
} from '../dist/shared.js';
import {
  Bytes as CompactBytes,
  Utf8Bytes as CompactUtf8Bytes,
} from '../dist/compact.js';

const encoder = new TextEncoder();

describe('entry points and aliases', () => {
  test('root exports canonical shared class objects', () => {
    expect(Bytes).toBe(SharedBytes);
    expect(Utf8Bytes).toBe(SharedUtf8Bytes);
    expect(Bytes).not.toBe(CompactBytes);
    expect(Utf8Bytes).not.toBe(CompactUtf8Bytes);
  });

  test('each class has iterable and disposable behavior', () => {
    const raw = Bytes.fromBytes(new Uint8Array([1, 2]));
    const text = Utf8Bytes.fromString('é');
    expect([...raw]).toEqual([1, 2]);
    expect([...text]).toEqual(['é']);
    expect(Symbol.iterator in raw).toBe(true);
    expect(Symbol.dispose in raw).toBe(true);
    raw[Symbol.dispose]();
    text[Symbol.dispose]();
  });

  test('iterator wrappers are iterable and disposable', () => {
    const iterator = Bytes.fromBytes(new Uint8Array([4, 5])).iter();
    expect([...iterator]).toEqual([4, 5]);
    expect(iterator[Symbol.iterator]()).toBe(iterator);
    expect(iterator).toBeInstanceOf(ByteIterator);
    expect(ByteIterator.name).toBe('ByteIterator');
    const chars = Utf8Bytes.fromString('x').iter();
    expect(chars).toBeInstanceOf(CharIterator);
    expect(CharIterator.name).toBe('CharIterator');
    iterator[Symbol.dispose]();
    expect(ByteIterator).toBeDefined();
    expect(CharIterator).toBeDefined();
  });

  test('free and dispose are idempotent for values and iterators', () => {
    const values = [
      new Buffer(),
      new BytesMut(),
      new Bytes(),
      new CompactBytes(),
      new Utf8Buffer(),
      new Utf8BytesMut(),
      new Utf8Bytes(),
      new CompactUtf8Bytes(),
      Buffer.fromBytes(new Uint8Array([1])).iter(),
      Utf8Buffer.fromString('x').iter(),
    ];

    for (const value of values) {
      expect(() => {
        value.free();
        value.free();
        value[Symbol.dispose]();
        value[Symbol.dispose]();
      }).not.toThrow();
    }

    const reverse = Bytes.fromBytes(new Uint8Array([1]));
    expect(() => {
      reverse[Symbol.dispose]();
      reverse.free();
      reverse[Symbol.dispose]();
    }).not.toThrow();
  });
});

describe('Buffer', () => {
  test('raw UTF-8 API uses explicit names', () => {
    const buffer = Buffer.fromUtf8('hello');
    expect(buffer.decodeUtf8()).toBe('hello');
    expect(Object.hasOwn(Buffer, 'toString')).toBe(false);
    expect('fromString' in Buffer).toBe(false);
    expect('fromStatic' in Buffer).toBe(false);
    expect('__remainingMut' in Utf8Buffer.prototype).toBe(false);
  });

  test('roundtrips bytes, split operations, and iteration', () => {
    const buffer = Buffer.fromBytes(new Uint8Array([1, 2, 3, 4]));
    expect(buffer.toBytes()).toEqual(new Uint8Array([1, 2, 3, 4]));
    expect(buffer.getU8()).toBe(1);
    expect(buffer.getU16()).toBe(0x0203);
    expect(buffer.remaining()).toBe(1);
    expect([...buffer]).toEqual([4]);

    const words = Buffer.fromUtf8('hello world');
    const head = words.splitTo(5);
    expect(head.decodeUtf8()).toBe('hello');
    expect(words.decodeUtf8()).toBe(' world');
  });

  test('preserves fixed little-endian getter and putter behavior', () => {
    const read = Buffer.fromBytes(new Uint8Array([0x01, 0x02]));
    expect(read.getU16Le()).toBe(0x0201);

    const write = new Buffer();
    write.putU16Le(0x0102);
    write.putI16Le(-2);
    expect(write.toBytes()).toEqual(new Uint8Array([0x02, 0x01, 0xfe, 0xff]));
  });

  test('fixed capacity parity is explicit and atomic', () => {
    const buffer = new Buffer();
    expect(buffer.capacity()).toBe(62);
    expect(buffer.remainingMut()).toBe(62);
    buffer.putBytes(0x61, 3);
    buffer.resize(5);
    expect(buffer.toBytes()).toEqual(new Uint8Array([0x61, 0x61, 0x61, 0, 0]));
    const before = buffer.toBytes();
    expect(() => buffer.resize(63)).toThrow(RangeError);
    expect(buffer.toBytes()).toEqual(before);
  });

  test('retains empty-state and fixed putter coverage', () => {
    const buffer = new Buffer();
    expect(buffer.isEmpty()).toBe(true);
    buffer.putU8(42);
    buffer.putU16(0x0102);
    expect(buffer.isEmpty()).toBe(false);
    expect(buffer.toBytes()).toEqual(new Uint8Array([42, 1, 2]));
  });

  test('variable-width raw integers are available', () => {
    const buffer = new Buffer();
    buffer.putUint(0x010203n, 3);
    buffer.putInt(-2n, 1);
    expect(buffer.getUint(3)).toBe(0x010203n);
    expect(buffer.getInt(1)).toBe(-2n);
    expect(() => buffer.putUint(1n, 9)).toThrow(RangeError);
  });

  test('fixed constructors and writes fail before mutation at capacity', () => {
    expect(() => Buffer.fromBytes(new Uint8Array(63))).toThrow(RangeError);
    expect(() => Buffer.fromUtf8('x'.repeat(63))).toThrow(RangeError);

    const buffer = Buffer.fromBytes(new Uint8Array(62));
    const before = buffer.toBytes();
    expect(() => buffer.putU8(1)).toThrow(RangeError);
    expect(buffer.toBytes()).toEqual(before);
  });
});

describe('immutable byte strategies', () => {
  test.each([
    ['shared', Bytes],
    ['compact', CompactBytes],
  ])('%s raw bytes keep strategy and clone cheaply', (_name, Type) => {
    const value = Type.fromBytes(new Uint8Array(100));
    const clone = value.clone();
    expect(clone.constructor).toBe(value.constructor);
    expect(clone.isHeap()).toBe(true);
    expect([...clone]).toHaveLength(100);

    const mutable = value.toMutable();
    mutable.putU8(7);
    expect(value.len()).toBe(100);
    expect(mutable.len()).toBe(101);
  });

  test('raw names are migrated across strategy entry points', () => {
    const raw = CompactBytes.fromUtf8('ok');
    expect(raw.decodeUtf8()).toBe('ok');
    expect('fromString' in CompactBytes).toBe(false);
    expect('fromStatic' in CompactBytes).toBe(false);
  });

  test.each([
    ['shared', Bytes],
    ['compact', CompactBytes],
  ])('%s keeps small immutable raw values inline', (_name, Type) => {
    expect(Type.fromBytes(new Uint8Array([1, 2, 3])).isInline()).toBe(true);
  });
});

describe('BytesMut ownership and copies', () => {
  test('withCapacity and putSlice preserve byte roundtrips', () => {
    const value = BytesMut.withCapacity(100);
    expect(value.isEmpty()).toBe(true);
    value.putSlice(new Uint8Array([1, 2, 3]));
    expect(value.toBytes()).toEqual(new Uint8Array([1, 2, 3]));
  });

  test('slice returns an independent mutable copy', () => {
    const value = BytesMut.fromUtf8('hello');
    const copy = value.slice(1, 4);
    copy.putU8(0x21);
    expect(value.decodeUtf8()).toBe('hello');
    expect(copy.decodeUtf8()).toBe('ell!');
  });

  test('freeze drains and reuses the source while selecting strategy', () => {
    const sharedSource = BytesMut.fromUtf8('shared');
    const shared = sharedSource.freezeShared();
    expect(shared.constructor).toBe(Bytes);
    expect(shared.decodeUtf8()).toBe('shared');
    expect(sharedSource.isEmpty()).toBe(true);
    sharedSource.putU8(1);
    expect(shared.toBytes()).toEqual(encoder.encode('shared'));

    const compactSource = BytesMut.fromUtf8('compact');
    const compact = compactSource.freezeCompact();
    expect(compact.constructor).toBe(CompactBytes);
    expect(compactSource.isEmpty()).toBe(true);
  });

  test('unsplit snapshots other, preserves it, and doubles on self alias', () => {
    const value = BytesMut.fromUtf8('left');
    const other = BytesMut.fromUtf8('right');
    expect(value.unsplit(other)).toBeUndefined();
    expect(value.decodeUtf8()).toBe('leftright');
    expect(other.decodeUtf8()).toBe('right');

    const self = BytesMut.fromUtf8('x');
    self.unsplit(self);
    expect(self.decodeUtf8()).toBe('xx');
  });

  test('split drains the source and returns reusable mutable bytes', () => {
    const value = BytesMut.fromUtf8('hello');
    const split = value.split();
    expect(value.isEmpty()).toBe(true);
    expect(split.decodeUtf8()).toBe('hello');
    value.putU8(0x21);
    expect(value.decodeUtf8()).toBe('!');
  });

  test('remaining mutable capacity and heap promotion are exposed', () => {
    const value = BytesMut.zeroed(4);
    expect(value.remainingMut()).toBeGreaterThanOrEqual(0);
    value.makeHeap();
    expect(value.isHeap()).toBe(true);
    value.resize(6);
    expect(value.toBytes()).toEqual(new Uint8Array([0, 0, 0, 0, 0, 0]));
  });
});

describe('variable-width integer fit validation', () => {
  const writers = [
    ['putUint', false],
    ['putUintLe', false],
    ['putUintNe', false],
    ['putInt', true],
    ['putIntLe', true],
    ['putIntNe', true],
  ] as const;
  const targets = [
    ['Buffer', () => new Buffer()],
    ['BytesMut', () => new BytesMut()],
  ] as const;

  test.each(targets.flatMap(([targetName, make]) => writers.map(([method, signed]) => [targetName, make, method, signed] as const)))(
    '%s %s validates every byte width before writing',
    (_targetName, make, method, signed) => {
      for (let width = 0; width <= 8; width += 1) {
        const target = make();
        const bits = BigInt(width * 8);
        const min = signed && width > 0 ? -(1n << (bits - 1n)) : 0n;
        const max = width === 0 ? 0n : signed ? (1n << (bits - 1n)) - 1n : (1n << bits) - 1n;
        const write = (target as unknown as Record<string, (value: bigint, count: number) => void>)[method];

        write.call(target, min, width);
        write.call(target, max, width);

        for (const invalid of signed ? [min - 1n, max + 1n] : [max + 1n]) {
          const before = target.toBytes();
          expect(() => write.call(target, invalid, width)).toThrow(RangeError);
          expect(target.toBytes()).toEqual(before);
        }
      }
    },
  );
});

describe('UTF-8 byte-unit API', () => {
  test.each([
    ['fixed', Utf8Buffer.fromString('é🦀')],
    ['mutable', Utf8BytesMut.fromString('é🦀')],
    ['shared', Utf8Bytes.fromString('é🦀')],
    ['compact', CompactUtf8Bytes.fromString('é🦀')],
  ])('%s exposes byte and scalar lengths without ambiguous byte names', (_name, value) => {
    expect(value.byteLength()).toBe(6);
    expect(value.charLength()).toBe(2);
    expect('len' in value).toBe(false);
    expect('remaining' in value).toBe(false);
    expect('splitTo' in value).toBe(false);
    expect([...value]).toEqual(['é', '🦀']);
  });

  test('byte operations enforce UTF-8 boundaries before mutation', () => {
    const value = Utf8BytesMut.fromString('é🦀x');
    expect(() => value.splitToBytes(1)).toThrow(RangeError);
    expect(value.toString()).toBe('é🦀x');
    const head = value.splitToBytes(2);
    expect(head.toString()).toBe('é');
    expect(value.toString()).toBe('🦀x');
    expect(() => value.truncateBytes(1)).toThrow(RangeError);
    expect(value.sliceBytes(0, 4).toString()).toBe('🦀');
  });

  test.each([
    ['shared', Utf8Bytes],
    ['compact', CompactUtf8Bytes],
  ])('%s immutable UTF-8 values split on boundaries and reject interior bytes', (_name, Type) => {
    const value = Type.fromString('café');
    const head = value.splitToBytes(3);
    expect(head.toString()).toBe('caf');
    expect(value.toString()).toBe('é');

    const boundary = Type.fromString('café');
    expect(() => boundary.splitOffBytes(4)).toThrow(RangeError);
    expect(boundary.toString()).toBe('café');
  });

  test.each([
    ['fixed', () => new Utf8Buffer()],
    ['mutable', () => new Utf8BytesMut()],
  ])('%s accepts valid scalar push and string append operations', (_name, make) => {
    const value = make();
    value.push('a');
    value.push('é');
    value.push('🦀');
    value.pushStr('!');
    expect(value.toString()).toBe('aé🦀!');
  });

  test.each([
    ['fixed', () => new Utf8Buffer()],
    ['mutable', () => new Utf8BytesMut()],
  ])('%s rejects empty and multi-scalar push without mutation', (_name, make) => {
    const value = make();
    value.pushStr('ok');
    for (const input of ['', 'ab', 'éx', '🦀x', 'e\u0301']) {
      expect(() => value.push(input)).toThrow(RangeError);
      expect(value.toString()).toBe('ok');
    }
  });

  test.each([
    ['fixed', () => Utf8Buffer.fromString('éx')],
    ['mutable', () => Utf8BytesMut.fromString('éx')],
  ])('%s truncation keeps boundary and no-op semantics', (_name, make) => {
    const value = make();
    expect(() => value.truncateBytes(1)).toThrow(RangeError);
    expect(value.toString()).toBe('éx');
    value.truncateBytes(2);
    expect(value.toString()).toBe('é');
    value.truncateBytes(99);
    expect(value.toString()).toBe('é');
  });

  test('explicit advance tracks remaining byte views', () => {
    const value = Utf8Bytes.fromString('éx');
    expect(() => value.advanceBytes(1)).toThrow(RangeError);
    value.advanceBytes(2);
    expect(value.toString()).toBe('x');
    expect(value.remainingBytes()).toBe(1);
  });

  test('UTF-8 byte conversion roundtrips encoded text', () => {
    const value = Utf8Bytes.fromString('café');
    expect(value.toBytes()).toEqual(encoder.encode('café'));
  });

  test('mutable UTF-8 ownership is non-consuming for unsplit and draining for freeze', () => {
    const value = Utf8BytesMut.fromString('hi');
    const other = Utf8BytesMut.fromString('🦀');
    value.unsplit(other);
    expect(value.toString()).toBe('hi🦀');
    expect(other.toString()).toBe('🦀');
    value.unsplit(value);
    expect(value.toString()).toBe('hi🦀hi🦀');

    const frozen = value.freezeCompact();
    expect(frozen.constructor).toBe(CompactUtf8Bytes);
    expect(value.isEmpty()).toBe(true);
    const mutable = frozen.toMutable();
    mutable.pushStr('!');
    expect(frozen.toString()).toBe('hi🦀hi🦀');
    expect(mutable.toString()).toBe('hi🦀hi🦀!');
  });

  test('malformed JavaScript UTF-16 is rejected before mutation', () => {
    for (const Type of [Buffer, BytesMut, Bytes, CompactBytes]) {
      expect(() => Type.fromUtf8('\ud800')).toThrow(TypeError);
    }
    for (const Type of [Utf8Buffer, Utf8BytesMut, Utf8Bytes, CompactUtf8Bytes]) {
      expect(() => Type.fromString('\ud800')).toThrow(TypeError);
    }
    const value = Utf8BytesMut.fromString('ok');
    expect(() => value.pushStr('\udc00')).toThrow(TypeError);
    expect(value.toString()).toBe('ok');
    expect(() => value.push('\ud800')).toThrow(TypeError);
    expect(value.toString()).toBe('ok');

    const fixed = Utf8Buffer.fromString('ok');
    expect(() => fixed.pushStr('\udc00')).toThrow(TypeError);
    expect(fixed.toString()).toBe('ok');
  });

  test('fixed UTF-8 capacity failures are deterministic and atomic', () => {
    expect(() => Utf8Buffer.fromString('x'.repeat(63))).toThrow(RangeError);
    const value = Utf8Buffer.fromString('x'.repeat(62));
    expect(() => value.push('!')).toThrow(RangeError);
    expect(value.toString()).toBe('x'.repeat(62));

    value.advanceBytes(1);
    expect(() => value.push('!')).toThrow(RangeError);
    expect(value.toString()).toBe('x'.repeat(61));
  });
});

describe('public numeric validation', () => {
  test.each([NaN, Infinity, -1, 1.5, Number.MAX_SAFE_INTEGER + 1])(
    'rejects invalid byte offsets before wasm coercion: %p',
    (value) => {
      const buffer = BytesMut.fromBytes(new Uint8Array([1]));
      expect(() => buffer.splitTo(value)).toThrow(RangeError);
      expect(buffer.toBytes()).toEqual(new Uint8Array([1]));
    },
  );

  test('separates type and range failures', () => {
    const buffer = new Buffer();
    expect(() => buffer.putU8('1' as unknown as number)).toThrow(TypeError);
    expect(() => buffer.putU8(256)).toThrow(RangeError);
    expect(() => buffer.putF32(Number.MAX_VALUE)).toThrow(RangeError);
    expect(() => BytesMut.withCapacity(-1)).toThrow(RangeError);
    expect(() => BytesMut.fromBytes([1] as unknown as Uint8Array)).toThrow(TypeError);
  });

  test.each([
    ['Buffer', () => Buffer.fromBytes(new Uint8Array([1]))],
    ['BytesMut', () => BytesMut.fromBytes(new Uint8Array([1]))],
    ['Bytes', () => Bytes.fromBytes(new Uint8Array([1]))],
    ['CompactBytes', () => CompactBytes.fromBytes(new Uint8Array([1]))],
  ])('%s preflights fixed and variable getter underflow without advancing', (_name, make) => {
    const fixed = make();
    expect(() => fixed.getU16()).toThrow(RangeError);
    expect(fixed.remaining()).toBe(1);

    const variable = make();
    expect(() => variable.getUint(2)).toThrow(RangeError);
    expect(variable.remaining()).toBe(1);
  });

  test.each([
    ['Buffer', () => Buffer.fromBytes(new Uint8Array([0xff]))],
    ['BytesMut', () => BytesMut.fromBytes(new Uint8Array([0xff]))],
    ['Bytes', () => Bytes.fromBytes(new Uint8Array([0xff]))],
    ['CompactBytes', () => CompactBytes.fromBytes(new Uint8Array([0xff]))],
  ])('%s maps recognizable invalid UTF-8 decode to TypeError', (_name, make) => {
    expect(() => make().decodeUtf8()).toThrow(TypeError);
  });

  test.each([
    ['Buffer', Buffer],
    ['BytesMut', BytesMut],
    ['Bytes', Bytes],
    ['CompactBytes', CompactBytes],
  ])('%s decodeUtf8 preserves a leading BOM and source bytes', (_name, Type) => {
    const value = Type.fromUtf8('\ufeffa');
    const before = value.toBytes();
    expect(value.decodeUtf8()).toBe('\ufeffa');
    expect(value.toBytes()).toEqual(before);
  });

  test('does not relabel an unexpected use-after-free failure', () => {
    const value = Bytes.fromUtf8('ok');
    value.free();
    let failure: unknown;
    try {
      value.decodeUtf8();
    } catch (error) {
      failure = error;
    }
    expect(failure).toBeInstanceOf(Error);
    expect(failure).not.toBeInstanceOf(TypeError);
    expect(failure).not.toBeInstanceOf(RangeError);
  });
});
