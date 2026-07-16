import { describe, test, expect } from 'vitest';

// These imports assume the build has been run (npm run build)
import {
  Buffer,
  BytesMut,
  Utf8Buffer,
  Utf8Bytes,
  Utf8BytesMut,
} from '../src/index.js';
import { Bytes as SharedBytes } from '../src/shared.js';
import { Bytes as CompactBytes } from '../src/compact.js';
import { Utf8Bytes as CompactUtf8Bytes } from '../src/compact.js';

describe('imports', () => {
  test('all types are defined', () => {
    expect(Buffer).toBeDefined();
    expect(BytesMut).toBeDefined();
    expect(SharedBytes).toBeDefined();
    expect(CompactBytes).toBeDefined();
    expect(Utf8Buffer).toBeDefined();
    expect(Utf8Bytes).toBeDefined();
    expect(Utf8BytesMut).toBeDefined();
  });
});

describe('Buffer', () => {
  test('fromBytes + toBytes roundtrip', () => {
    const buf = Buffer.fromBytes(new Uint8Array([1, 2, 3]));
    expect(buf.len()).toBe(3);
    const out = buf.toBytes();
    expect(out).toEqual(new Uint8Array([1, 2, 3]));
  });

  test('fromString + toString', () => {
    const buf = Buffer.fromString('hello');
    expect(buf.toString()).toBe('hello');
  });

  test('Buf getters', () => {
    const buf = Buffer.fromBytes(new Uint8Array([0x01, 0x02, 0x03, 0x04]));
    expect(buf.getU8()).toBe(1);
    expect(buf.remaining()).toBe(3);
    expect(buf.getU16()).toBe(0x0203);
    expect(buf.remaining()).toBe(1);
  });

  test('getU16Le', () => {
    const buf = Buffer.fromBytes(new Uint8Array([0x01, 0x02]));
    expect(buf.getU16Le()).toBe(0x0201);
  });

  test('BufMut putters', () => {
    const buf = new Buffer();
    buf.putU8(42);
    buf.putU16(0x0102);
    expect(buf.len()).toBe(3);
    expect(buf.toBytes()).toEqual(new Uint8Array([42, 1, 2]));
  });

  test('splitTo', () => {
    const buf = Buffer.fromString('hello world');
    const head = buf.splitTo(5);
    expect(head.toString()).toBe('hello');
    expect(buf.toString()).toBe(' world');
  });

  test('iteration', () => {
    const buf = Buffer.fromBytes(new Uint8Array([1, 2, 3]));
    const result = [...buf];
    expect(result).toEqual([1, 2, 3]);
  });

  test('isEmpty', () => {
    const buf = new Buffer();
    expect(buf.isEmpty()).toBe(true);
    buf.putU8(1);
    expect(buf.isEmpty()).toBe(false);
  });
});

describe('SharedBytes', () => {
  test('fromBytes + toBytes', () => {
    const b = SharedBytes.fromBytes(new Uint8Array([1, 2, 3]));
    expect(b.toBytes()).toEqual(new Uint8Array([1, 2, 3]));
  });

  test('isInline / isHeap', () => {
    const small = SharedBytes.fromBytes(new Uint8Array([1, 2, 3]));
    expect(small.isInline()).toBe(true);
    const large = SharedBytes.fromBytes(new Uint8Array(100));
    expect(large.isHeap()).toBe(true);
  });

  test('iteration', () => {
    const b = SharedBytes.fromBytes(new Uint8Array([10, 20, 30]));
    expect([...b]).toEqual([10, 20, 30]);
  });
});

describe('Utf8Bytes', () => {
  test('fromString + toString', () => {
    const s = Utf8Bytes.fromString('cafe\u0301 \uD83E\uDD80');
    expect(s.toString()).toBe('cafe\u0301 \uD83E\uDD80');
  });

  test('len is byte length', () => {
    const s = Utf8Bytes.fromString('caf\u00e9');
    expect(s.len()).toBe(5); // c(1) + a(1) + f(1) + e\u0301(2)
  });

  test('char iteration', () => {
    const s = Utf8Bytes.fromString('caf\u00e9');
    expect([...s]).toEqual(['c', 'a', 'f', '\u00e9']);
  });

  test('splitTo on char boundary', () => {
    const s = Utf8Bytes.fromString('caf\u00e9');
    const head = s.splitTo(3);
    expect(head.toString()).toBe('caf');
    expect(s.toString()).toBe('\u00e9');
  });

  test('splitOff mid-char throws', () => {
    const s = Utf8Bytes.fromString('caf\u00e9');
    expect(() => s.splitOff(4)).toThrow();
  });

  test('toBytes returns UTF-8', () => {
    const s = Utf8Bytes.fromString('caf\u00e9');
    const bytes = s.toBytes();
    expect(bytes).toEqual(new TextEncoder().encode('caf\u00e9'));
  });

  test.each([
    ['shared', () => Utf8Bytes.fromString('\u00e9x')],
    ['compact', () => CompactUtf8Bytes.fromString('\u00e9x')],
  ])('%s advance requires a character boundary', (_strategy, makeBytes) => {
    const s = makeBytes();

    expect(() => s.advance(1)).toThrow();
    expect(s.toString()).toBe('\u00e9x');
    s.advance(2);
    expect(s.toString()).toBe('x');
  });
});

describe('BytesMut', () => {
  test('withCapacity', () => {
    const buf = BytesMut.withCapacity(100);
    expect(buf.len()).toBe(0);
    expect(buf.isEmpty()).toBe(true);
  });

  test('putSlice + toBytes', () => {
    const buf = BytesMut.withCapacity(100);
    buf.putSlice(new Uint8Array([1, 2, 3]));
    expect(buf.toBytes()).toEqual(new Uint8Array([1, 2, 3]));
  });
});

describe('Utf8BytesMut', () => {
  test('push + pushStr', () => {
    const buf = Utf8BytesMut.withCapacity(100);
    buf.pushStr('hello');
    buf.push(' ');
    buf.pushStr('world');
    expect(buf.toString()).toBe('hello world');
  });
});

describe.each([
  ['Utf8Buffer', () => new Utf8Buffer()],
  ['Utf8BytesMut', () => new Utf8BytesMut()],
])('%s Unicode scalar operations', (_name, make) => {
  test('push accepts exactly one Unicode scalar value', () => {
    const value = make();

    value.push('a');
    value.push('é');
    value.push('🦀');

    expect(value.toString()).toBe('aé🦀');
  });

  test.each(['', 'ab', 'éx', '🦀x', 'e\u0301', '\uD800', '\uDC00'])(
    'push rejects %j without mutation',
    (input) => {
      const value = make();
      value.pushStr('ok');

      expect(() => value.push(input)).toThrow(
        'push expects exactly one Unicode scalar value',
      );
      expect(value.toString()).toBe('ok');
    },
  );

  test('truncate validates UTF-8 boundaries and preserves state on error', () => {
    const value = make();
    value.pushStr('éx');

    expect(() => value.truncate(1)).toThrow();
    expect(value.toString()).toBe('éx');
    value.truncate(2);
    expect(value.toString()).toBe('é');
    value.truncate(99);
    expect(value.toString()).toBe('é');
  });
});
