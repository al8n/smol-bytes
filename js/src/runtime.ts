/**
 * The JavaScript boundary for the wasm classes.
 *
 * wasm-bindgen receives JavaScript numbers after WebAssembly coercion. Keep
 * every check here so all public entry points share the same, deterministic
 * TypeError/RangeError contract before that coercion can happen.
 */

type WasmClass = {
  prototype: object;
};

export type WasmClasses = {
  Buffer: WasmClass;
  BytesMut: WasmClass;
  SharedBytes: WasmClass;
  CompactBytes: WasmClass;
  Utf8Buffer: WasmClass;
  Utf8Bytes: WasmClass;
  CompactUtf8Bytes: WasmClass;
  Utf8BytesMut: WasmClass;
  ByteIterator: WasmClass;
  CharIterator: WasmClass;
};

const INITIALIZED = Symbol.for('smol-bytes.runtime.0.2');
const U32_MAX = 0xffff_ffff;
const U64_MAX = (1n << 64n) - 1n;
const I64_MIN = -(1n << 63n);
const I64_MAX = (1n << 63n) - 1n;
const freed = new WeakSet<object>();

type Method = (...args: unknown[]) => unknown;

function prototypeOf(cls: WasmClass): Record<PropertyKey, unknown> {
  return cls.prototype as Record<PropertyKey, unknown>;
}

function staticOf(cls: WasmClass): Record<PropertyKey, unknown> {
  return cls as unknown as Record<PropertyKey, unknown>;
}

function typeError(name: string, expected: string): never {
  throw new TypeError(`${name} must be ${expected}`);
}

function rangeError(name: string, detail: string): never {
  throw new RangeError(`${name} ${detail}`);
}

function natural(value: unknown, name: string, max = U32_MAX): number {
  if (typeof value !== 'number') typeError(name, 'a number');
  if (!Number.isFinite(value) || !Number.isInteger(value) || !Number.isSafeInteger(value)) {
    rangeError(name, 'must be a finite safe integer');
  }
  if (value < 0 || value > max) rangeError(name, `must be between 0 and ${max}`);
  return value;
}

function integer(value: unknown, name: string, min: number, max: number): number {
  if (typeof value !== 'number') typeError(name, 'a number');
  if (!Number.isFinite(value) || !Number.isInteger(value) || !Number.isSafeInteger(value)) {
    rangeError(name, 'must be a finite safe integer');
  }
  if (value < min || value > max) rangeError(name, `must be between ${min} and ${max}`);
  return value;
}

function finiteNumber(value: unknown, name: string, max = Number.MAX_VALUE): number {
  if (typeof value !== 'number') typeError(name, 'a number');
  if (!Number.isFinite(value)) rangeError(name, 'must be finite');
  if (value < -max || value > max) rangeError(name, `must be between ${-max} and ${max}`);
  return value;
}

function unsignedBigInt(value: unknown, name: string): bigint {
  if (typeof value !== 'bigint') typeError(name, 'a bigint');
  if (value < 0n || value > U64_MAX) rangeError(name, 'must fit in an unsigned 64-bit integer');
  return value;
}

function signedBigInt(value: unknown, name: string): bigint {
  if (typeof value !== 'bigint') typeError(name, 'a bigint');
  if (value < I64_MIN || value > I64_MAX) rangeError(name, 'must fit in a signed 64-bit integer');
  return value;
}

function byte(value: unknown, name: string): number {
  return integer(value, name, 0, 0xff);
}

function bytes(value: unknown, name: string): Uint8Array {
  if (!(value instanceof Uint8Array)) typeError(name, 'a Uint8Array');
  return value;
}

export function assertWellFormedString(value: unknown, name: string): string {
  if (typeof value !== 'string') typeError(name, 'a string');

  for (let index = 0; index < value.length; index += 1) {
    const codeUnit = value.charCodeAt(index);
    if (codeUnit >= 0xd800 && codeUnit <= 0xdbff) {
      const next = value.charCodeAt(index + 1);
      if (index + 1 >= value.length || next < 0xdc00 || next > 0xdfff) {
        throw new TypeError(`${name} must not contain isolated UTF-16 surrogates`);
      }
      index += 1;
    } else if (codeUnit >= 0xdc00 && codeUnit <= 0xdfff) {
      throw new TypeError(`${name} must not contain isolated UTF-16 surrogates`);
    }
  }
  return value;
}

function scalar(value: unknown, name: string): string {
  const string = assertWellFormedString(value, name);
  if (Array.from(string).length !== 1) rangeError(name, 'must contain exactly one Unicode scalar value');
  return string;
}

function defineMethod(target: Record<PropertyKey, unknown>, key: PropertyKey, value: Method): void {
  Object.defineProperty(target, key, {
    configurable: true,
    enumerable: false,
    writable: true,
    value,
  });
}

function ownMethod(target: Record<PropertyKey, unknown>, name: string): Method | undefined {
  const value = target[name];
  return typeof value === 'function' ? (value as Method) : undefined;
}

function wrapInstance(
  cls: WasmClass,
  name: string,
  validate: (self: Record<PropertyKey, unknown>, args: unknown[]) => void,
): void {
  const prototype = prototypeOf(cls);
  const original = ownMethod(prototype, name);
  if (!original) return;
  defineMethod(prototype, name, function wrapped(this: Record<PropertyKey, unknown>, ...args: unknown[]) {
    validate(this, args);
    return original.apply(this, args);
  });
}

function wrapStatic(
  cls: WasmClass,
  name: string,
  validate: (args: unknown[]) => void,
): void {
  const target = staticOf(cls);
  const original = ownMethod(target, name);
  if (!original) return;
  defineMethod(target, name, function wrapped(this: WasmClass, ...args: unknown[]) {
    validate(args);
    return original.apply(this, args);
  });
}

function defaultResizeValue(cls: WasmClass): void {
  const prototype = prototypeOf(cls);
  const original = ownMethod(prototype, 'resize');
  if (!original) return;
  defineMethod(prototype, 'resize', function resize(
    this: Record<PropertyKey, unknown>,
    newLength: unknown,
    value: unknown = 0,
  ) {
    return original.call(this, newLength, value);
  });
}

function numberFrom(self: Record<PropertyKey, unknown>, method: string): number {
  const fn = ownMethod(self, method);
  if (!fn) throw new Error(`missing wasm method ${method}`);
  return fn.call(self) as number;
}

function fixedUtf8Writable(self: Record<PropertyKey, unknown>, count: number, name: string): void {
  if (count > numberFrom(self, 'capacity') - numberFrom(self, 'byteLength')) {
    rangeError(name, 'exceeds fixed buffer capacity');
  }
}

function rawRange(self: Record<PropertyKey, unknown>, args: unknown[], method: string): void {
  const start = natural(args[0], 'start');
  const end = natural(args[1], 'end');
  const length = numberFrom(self, method);
  if (start > end) rangeError('range', 'must have start <= end');
  if (end > length) rangeError('range', 'is out of bounds');
}

function rawOffset(self: Record<PropertyKey, unknown>, value: unknown, name: string, method: string): void {
  const offset = natural(value, name);
  if (offset > numberFrom(self, method)) rangeError(name, 'is out of bounds');
}

function readable(self: Record<PropertyKey, unknown>, count: number, name: string): void {
  if (count > numberFrom(self, 'remaining')) rangeError(name, 'requires more readable bytes');
}

function writable(self: Record<PropertyKey, unknown>, count: number, method: string, name: string): void {
  if (count > numberFrom(self, method)) rangeError(name, 'exceeds remaining capacity');
}

function variableUnsigned(value: unknown, width: unknown): number {
  const bytes = natural(width, 'width', 8);
  const number = unsignedBigInt(value, 'value');
  const max = bytes === 0 ? 0n : (1n << BigInt(bytes * 8)) - 1n;
  if (number > max) rangeError('value', `does not fit in ${bytes} byte${bytes === 1 ? '' : 's'}`);
  return bytes;
}

function variableSigned(value: unknown, width: unknown): number {
  const bytes = natural(width, 'width', 8);
  const number = signedBigInt(value, 'value');
  if (bytes === 0) {
    if (number !== 0n) rangeError('value', 'must be 0n when width is 0');
    return bytes;
  }
  const bits = BigInt(bytes * 8 - 1);
  const min = -(1n << bits);
  const max = (1n << bits) - 1n;
  if (number < min || number > max) rangeError('value', `does not fit in ${bytes} byte${bytes === 1 ? '' : 's'}`);
  return bytes;
}

function utf8ByteLength(value: string): number {
  let length = 0;
  for (const scalarValue of value) {
    const codePoint = scalarValue.codePointAt(0)!;
    length += codePoint <= 0x7f ? 1 : codePoint <= 0x7ff ? 2 : codePoint <= 0xffff ? 3 : 4;
  }
  return length;
}

function utf8Boundary(value: string, offset: number): boolean {
  let current = 0;
  if (offset === 0) return true;
  for (const scalarValue of value) {
    current += utf8ByteLength(scalarValue);
    if (current === offset) return true;
    if (current > offset) return false;
  }
  return offset === current;
}

function utf8Offset(self: Record<PropertyKey, unknown>, value: unknown, name: string): number {
  const offset = natural(value, name);
  const text = (ownMethod(self, 'toString') as Method).call(self) as string;
  const length = utf8ByteLength(text);
  if (offset > length) rangeError(name, 'is out of bounds');
  if (!utf8Boundary(text, offset)) rangeError(name, 'must be on a UTF-8 character boundary');
  return offset;
}

function utf8Range(self: Record<PropertyKey, unknown>, args: unknown[]): void {
  const start = utf8Offset(self, args[0], 'start');
  const end = utf8Offset(self, args[1], 'end');
  if (start > end) rangeError('range', 'must have start <= end');
}

const fatalUtf8 = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });

function wrapRawCommon(cls: WasmClass, options: { mutable: boolean; fixed: boolean }): void {
  wrapStatic(cls, 'fromBytes', ([value]) => {
    const data = bytes(value, 'data');
    if (options.fixed && data.byteLength > 62) rangeError('data', 'exceeds fixed buffer capacity');
  });
  wrapStatic(cls, 'fromUtf8', ([value]) => {
    const string = assertWellFormedString(value, 's');
    if (options.fixed && utf8ByteLength(string) > 62) rangeError('s', 'exceeds fixed buffer capacity');
  });

  const decode = ownMethod(prototypeOf(cls), 'decodeUtf8');
  if (decode) {
    defineMethod(prototypeOf(cls), 'decodeUtf8', function decodeUtf8(this: Record<PropertyKey, unknown>) {
      const data = (ownMethod(this, 'toBytes') as Method).call(this) as Uint8Array;
      return fatalUtf8.decode(data);
    });
  }

  wrapInstance(cls, 'advance', (self, [count]) => rawOffset(self, count, 'count', 'remaining'));
  wrapInstance(cls, 'splitTo', (self, [at]) => rawOffset(self, at, 'at', 'len'));
  wrapInstance(cls, 'splitOff', (self, [at]) => rawOffset(self, at, 'at', 'len'));
  wrapInstance(cls, 'slice', (self, args) => rawRange(self, args, 'len'));
  wrapInstance(cls, 'truncate', (_self, [newLength]) => natural(newLength, 'newLen'));

  const fixedReads = [
    ['getU8', 1],
    ['getI8', 1],
    ['getU16', 2],
    ['getU16Le', 2],
    ['getI16', 2],
    ['getI16Le', 2],
    ['getU32', 4],
    ['getU32Le', 4],
    ['getI32', 4],
    ['getI32Le', 4],
    ['getU64', 8],
    ['getU64Le', 8],
    ['getI64', 8],
    ['getI64Le', 8],
    ['getF32', 4],
    ['getF32Le', 4],
    ['getF64', 8],
    ['getF64Le', 8],
  ] as const;
  for (const [name, count] of fixedReads) {
    wrapInstance(cls, name, (self) => readable(self, count, name));
  }
  for (const name of ['getUint', 'getUintLe', 'getUintNe', 'getInt', 'getIntLe', 'getIntNe']) {
    wrapInstance(cls, name, (self, [width]) => {
      const count = natural(width, 'width', 8);
      readable(self, count, name);
    });
  }

  if (!options.mutable) return;

  wrapInstance(cls, 'putSlice', (self, [value]) => {
    const data = bytes(value, 'data');
    if (options.fixed) writable(self, data.byteLength, 'remainingMut', 'data');
  });
  wrapInstance(cls, 'putBytes', (self, [value, count]) => {
    byte(value, 'value');
    const length = natural(count, 'count');
    if (options.fixed) writable(self, length, 'remainingMut', 'count');
  });
  for (const name of ['putUint', 'putUintLe', 'putUintNe']) {
    wrapInstance(cls, name, (self, [value, width]) => {
      const count = variableUnsigned(value, width);
      if (options.fixed) writable(self, count, 'remainingMut', 'width');
    });
  }
  for (const name of ['putInt', 'putIntLe', 'putIntNe']) {
    wrapInstance(cls, name, (self, [value, width]) => {
      const count = variableSigned(value, width);
      if (options.fixed) writable(self, count, 'remainingMut', 'width');
    });
  }

  const unsigned = [
    ['putU8', 0xff],
    ['putU16', 0xffff],
    ['putU16Le', 0xffff],
    ['putU32', U32_MAX],
    ['putU32Le', U32_MAX],
  ] as const;
  for (const [name, max] of unsigned) {
    const count = name.includes('8') ? 1 : name.includes('16') ? 2 : 4;
    wrapInstance(cls, name, (self, [value]) => {
      integer(value, 'value', 0, max);
      if (options.fixed) writable(self, count, 'remainingMut', name);
    });
  }
  const signed = [
    ['putI8', -0x80, 0x7f],
    ['putI16', -0x8000, 0x7fff],
    ['putI16Le', -0x8000, 0x7fff],
    ['putI32', -0x8000_0000, 0x7fff_ffff],
    ['putI32Le', -0x8000_0000, 0x7fff_ffff],
  ] as const;
  for (const [name, min, max] of signed) {
    const count = name.includes('8') ? 1 : name.includes('16') ? 2 : 4;
    wrapInstance(cls, name, (self, [value]) => {
      integer(value, 'value', min, max);
      if (options.fixed) writable(self, count, 'remainingMut', name);
    });
  }
  for (const name of ['putU64', 'putU64Le']) {
    wrapInstance(cls, name, (self, [value]) => {
      unsignedBigInt(value, 'value');
      if (options.fixed) writable(self, 8, 'remainingMut', name);
    });
  }
  for (const name of ['putI64', 'putI64Le']) {
    wrapInstance(cls, name, (self, [value]) => {
      signedBigInt(value, 'value');
      if (options.fixed) writable(self, 8, 'remainingMut', name);
    });
  }
  for (const name of ['putF32', 'putF32Le']) {
    wrapInstance(cls, name, (self, [value]) => {
      finiteNumber(value, 'value', 3.4028234663852886e38);
      if (options.fixed) writable(self, 4, 'remainingMut', name);
    });
  }
  for (const name of ['putF64', 'putF64Le']) {
    wrapInstance(cls, name, (self, [value]) => {
      finiteNumber(value, 'value');
      if (options.fixed) writable(self, 8, 'remainingMut', name);
    });
  }
}

function wrapUtf8(cls: WasmClass, options: { fixed: boolean; immutable: boolean }): void {
  wrapStatic(cls, 'fromString', ([value]) => {
    const string = assertWellFormedString(value, 's');
    if (options.fixed && utf8ByteLength(string) > 62) rangeError('s', 'exceeds fixed buffer capacity');
  });
  wrapInstance(cls, 'push', (self, [value]) => {
    const string = scalar(value, 'ch');
    if (options.fixed) fixedUtf8Writable(self, utf8ByteLength(string), 'ch');
  });
  wrapInstance(cls, 'pushStr', (self, [value]) => {
    const string = assertWellFormedString(value, 's');
    if (options.fixed) fixedUtf8Writable(self, utf8ByteLength(string), 's');
  });
  wrapInstance(cls, 'advanceBytes', (self, [count]) => {
    utf8Offset(self, count, 'count');
  });
  wrapInstance(cls, 'splitToBytes', (self, [at]) => {
    utf8Offset(self, at, 'at');
  });
  wrapInstance(cls, 'splitOffBytes', (self, [at]) => {
    utf8Offset(self, at, 'at');
  });
  wrapInstance(cls, 'sliceBytes', (self, args) => utf8Range(self, args));
  wrapInstance(cls, 'truncateBytes', (self, [newLength]) => {
    const length = natural(newLength, 'newLen');
    const text = (ownMethod(self, 'toString') as Method).call(self) as string;
    if (length <= utf8ByteLength(text) && !utf8Boundary(text, length)) {
      rangeError('newLen', 'must be on a UTF-8 character boundary');
    }
  });

  if (options.immutable) return;
  wrapStatic(cls, 'withCapacity', ([capacity]) => natural(capacity, 'capacity'));
  wrapInstance(cls, 'reserve', (_self, [additional]) => natural(additional, 'additional'));
}

function installIterationAndDisposal(classes: WasmClasses): void {
  const raw = [classes.Buffer, classes.BytesMut, classes.SharedBytes, classes.CompactBytes];
  const utf8 = [classes.Utf8Buffer, classes.Utf8BytesMut, classes.Utf8Bytes, classes.CompactUtf8Bytes];

  for (const cls of [...raw, ...utf8]) {
    defineMethod(prototypeOf(cls), Symbol.iterator, function iterator(this: Record<PropertyKey, unknown>) {
      const iter = ownMethod(this, 'iter');
      if (!iter) throw new Error('missing wasm iterator');
      return iter.call(this);
    });
  }
  for (const cls of [classes.ByteIterator, classes.CharIterator]) {
    defineMethod(prototypeOf(cls), Symbol.iterator, function iterator(this: unknown) {
      return this;
    });
  }

  for (const cls of [...raw, ...utf8, classes.ByteIterator, classes.CharIterator]) {
    const prototype = prototypeOf(cls);
    const original = ownMethod(prototype, 'free');
    if (!original) continue;
    defineMethod(prototype, 'free', function free(this: Record<PropertyKey, unknown>, ...args: unknown[]) {
      if (freed.has(this)) return;
      const result = original.apply(this, args);
      freed.add(this);
      return result;
    });
  }

  if (typeof Symbol.dispose !== 'symbol') return;
  for (const cls of [...raw, ...utf8, classes.ByteIterator, classes.CharIterator]) {
    defineMethod(prototypeOf(cls), Symbol.dispose, function dispose(this: Record<PropertyKey, unknown>) {
      const free = ownMethod(this, 'free');
      if (free) free.call(this);
    });
  }
}

function installOwnership(classes: WasmClasses): void {
  defineMethod(prototypeOf(classes.BytesMut), 'unsplit', function unsplit(this: Record<PropertyKey, unknown>, other: unknown) {
    if (!(other instanceof (classes.BytesMut as unknown as Function))) typeError('other', 'a BytesMut');
    const data = (ownMethod(other as Record<PropertyKey, unknown>, 'toBytes') as Method).call(other) as Uint8Array;
    (ownMethod(this, 'putSlice') as Method).call(this, data);
  });
  defineMethod(prototypeOf(classes.Utf8BytesMut), 'unsplit', function unsplit(this: Record<PropertyKey, unknown>, other: unknown) {
    if (!(other instanceof (classes.Utf8BytesMut as unknown as Function))) typeError('other', 'a Utf8BytesMut');
    const text = (ownMethod(other as Record<PropertyKey, unknown>, 'toString') as Method).call(other) as string;
    (ownMethod(this, 'pushStr') as Method).call(this, text);
  });
}

/** Install the shared facade once for the generated wasm class objects. */
export function installRuntime(classes: WasmClasses): void {
  if (Object.prototype.hasOwnProperty.call(classes.Buffer, INITIALIZED)) return;

  wrapRawCommon(classes.Buffer, { mutable: true, fixed: true });
  wrapRawCommon(classes.BytesMut, { mutable: true, fixed: false });
  wrapRawCommon(classes.SharedBytes, { mutable: false, fixed: false });
  wrapRawCommon(classes.CompactBytes, { mutable: false, fixed: false });
  wrapStatic(classes.BytesMut, 'withCapacity', ([capacity]) => natural(capacity, 'capacity'));
  wrapStatic(classes.BytesMut, 'zeroed', ([length]) => natural(length, 'length'));
  wrapInstance(classes.BytesMut, 'reserve', (_self, [additional]) => natural(additional, 'additional'));
  wrapInstance(classes.BytesMut, 'resize', (_self, [newLength, value]) => {
    natural(newLength, 'newLen');
    byte(value, 'value');
  });
  wrapInstance(classes.Buffer, 'resize', (_self, [newLength, value]) => {
    const length = natural(newLength, 'newLen');
    byte(value, 'value');
    if (length > numberFrom(_self, 'len') + numberFrom(_self, 'remainingMut')) {
      rangeError('newLen', 'exceeds fixed buffer capacity');
    }
  });
  defaultResizeValue(classes.Buffer);
  defaultResizeValue(classes.BytesMut);

  wrapUtf8(classes.Utf8Buffer, { fixed: true, immutable: false });
  wrapUtf8(classes.Utf8BytesMut, { fixed: false, immutable: false });
  wrapUtf8(classes.Utf8Bytes, { fixed: false, immutable: true });
  wrapUtf8(classes.CompactUtf8Bytes, { fixed: false, immutable: true });
  installOwnership(classes);
  installIterationAndDisposal(classes);

  Object.defineProperty(classes.Buffer, INITIALIZED, { value: true });
}
