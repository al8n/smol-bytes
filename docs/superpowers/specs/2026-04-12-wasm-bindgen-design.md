# wasm-bindgen JavaScript/TypeScript Bindings — Design

**Date**: 2026-04-12
**Status**: Approved
**Scope**: Add wasm-bindgen bindings, npm package with subpath exports, full Buf/BufMut API, iterator support. Also fix missing Buf methods on Python UTF-8 types.

---

## 1. Feature Flag & Target Support

Add a `wasm` feature to `crates/smol-bytes/Cargo.toml`:

```toml
[features]
wasm = ["dep:wasm-bindgen", "dep:js-sys"]
```

Verify the crate compiles on `wasm32-unknown-unknown` with `--features wasm`. Gate all wasm code behind `#[cfg(feature = "wasm")]`.

---

## 2. Binding File Structure

Following the pyo3 pattern, each type gets a `wasm.rs` submodule:

| Type | Binding file | Notes |
|---|---|---|
| Buffer | `buffer/wasm.rs` | Direct `#[wasm_bindgen]` |
| BytesMut | `bytes_mut/wasm.rs` | Direct |
| Utf8Buffer | `utf8_buffer/wasm.rs` | Direct |
| Utf8Bytes | `utf8_bytes/wasm.rs` | Direct |
| Utf8BytesMut | `utf8_bytes_mut/wasm.rs` | Direct |
| shared::Bytes | `bytes/strategy/shared/wasm.rs` | `WasmSharedBytes` wrapper (like `PySharedBytes`) |
| compact::Bytes | `bytes/strategy/compact/wasm.rs` | `WasmCompactBytes` wrapper |

Each parent `.rs` file gets `#[cfg(feature = "wasm")] mod wasm;`.

Wrapper types are needed for `shared::Bytes` and `compact::Bytes` because `RawBytes<S>` is generic and wasm-bindgen doesn't support generic types.

---

## 3. JS API Surface

All methods use camelCase. Byte data crosses the boundary as `Uint8Array`. Strings cross as JS `string`.

### 3a. Construction — all types

```typescript
new Buffer()
Buffer.fromBytes(data: Uint8Array): Buffer
Buffer.fromString(s: string): Buffer
// BytesMut also: BytesMut.withCapacity(cap: number)
// Utf8* types: Utf8Buffer.fromString(s: string)
```

### 3b. Core — all types

```typescript
toBytes(): Uint8Array
toString(): string       // UTF-8 decode; throws on invalid for byte types
len(): number
isEmpty(): boolean
```

### 3c. Buf trait getters — all types

```typescript
remaining(): number
advance(cnt: number): void

// Big-endian
getU8(): number
getI8(): number
getU16(): number
getI16(): number
getU32(): number
getI32(): number
getU64(): bigint
getI64(): bigint
getF32(): number
getF64(): number

// Little-endian
getU16Le(): number
getI16Le(): number
getU32Le(): number
getI32Le(): number
getU64Le(): bigint
getI64Le(): bigint
getF32Le(): number
getF64Le(): number

// Variable-length
getUint(nbytes: number): bigint
getUintLe(nbytes: number): bigint
getInt(nbytes: number): bigint
getIntLe(nbytes: number): bigint
```

### 3d. BufMut trait putters — Buffer, BytesMut only

Same pattern: `putU8(val)`, `putU16(val)`, `putU16Le(val)`, `putSlice(data: Uint8Array)`, `putBytes(val: number, cnt: number)`, etc.

### 3e. Storage info

```typescript
isInline(): boolean    // BytesMut, Bytes, CompactBytes, Utf8Bytes, Utf8BytesMut
isHeap(): boolean      // same set
```

Not on Buffer (always inline) or Utf8Buffer (always inline).

### 3f. Split/slice — all types

```typescript
splitTo(at: number): Self
splitOff(at: number): Self
slice(start: number, end: number): Self
```

UTF-8 types check char boundaries; throw `Error` on mid-char splits.

### 3g. Mutation — mutable types

```typescript
// Buffer, BytesMut
clear(): void
truncate(newLen: number): void

// Utf8Buffer, Utf8BytesMut
push(ch: string): void
pushStr(s: string): void
clear(): void

// BytesMut extras
reserve(additional: number): void
split(): BytesMut
unsplit(other: BytesMut): BytesMut | undefined

// Utf8BytesMut extras
reserve(additional: number): void
```

### 3h. Iterator

Each type exposes an `iter()` method returning a wasm-bindgen struct with a `next()` method returning `{ value, done }`.

A JS shim attaches `Symbol.iterator`:

```javascript
proto[Symbol.iterator] = function() { return this.iter(); };
```

**Byte types** iterate over `number` (byte values).
**UTF-8 types** iterate over `string` (single characters).

This enables `for..of`, spread (`[...buf]`), and `Array.from(buf)`.

---

## 4. npm Package Structure

```
js/
├── package.json          # name: "smol-bytes", exports with subpaths
├── tsconfig.json
├── src/
│   ├── index.ts          # re-exports + Symbol.iterator shim for all types
│   ├── shared.ts         # export { Bytes } (WasmSharedBytes renamed)
│   └── compact.ts        # export { Bytes } (WasmCompactBytes renamed)
└── tests/
    └── smol_bytes.test.ts
```

### package.json exports

```json
{
  "name": "smol-bytes",
  "type": "module",
  "exports": {
    ".": "./pkg/index.js",
    "./shared": "./pkg/shared.js",
    "./compact": "./pkg/compact.js"
  },
  "types": "./pkg/index.d.ts"
}
```

Usage:
```typescript
import { Buffer, BytesMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut } from 'smol-bytes';
import { Bytes } from 'smol-bytes/shared';
import { Bytes } from 'smol-bytes/compact';
```

### Build pipeline

`wasm-pack build --target bundler` compiles Rust → wasm + JS glue in `js/pkg/`. TypeScript sources in `js/src/` add the `Symbol.iterator` shim and re-export.

---

## 5. Fix Python UTF-8 Bindings

The Python UTF-8 types (`Utf8Buffer`, `Utf8Bytes`, `Utf8BytesMut`) currently lack Buf/BufMut getter methods that the byte types have (`get_u8`, `remaining`, `advance`, etc.).

Add the full Buf getter method set to all three UTF-8 python.rs files, using the same `PyBufExt` trait pattern that `Buffer` and `BytesMut` use.

This is a prerequisite for API consistency: both Python and JS should expose the same methods on all types.

---

## 6. Tests

`js/tests/smol_bytes.test.ts` using `vitest` (or `node:test`):

1. Import paths: all types from correct subpaths
2. Construction: `fromBytes`, `fromString`, `withCapacity`
3. `toBytes()` / `toString()` round-trips
4. Buf getters: `getU8`, `getU16Le`, `getU64` → bigint, etc.
5. BufMut putters: `putU8`, `putSlice`, overflow errors
6. Iterator: `for..of` on byte types yields numbers, on UTF-8 types yields chars
7. Split operations: `splitTo`, `splitOff`, char boundary errors on UTF-8
8. Storage info: `isInline` / `isHeap` transitions
9. `len`, `isEmpty`, `clear`, `truncate`

---

## 7. CI Workflow

`.github/workflows/wasm.yml`:

1. Install `wasm-pack`
2. `wasm-pack build` with `--features wasm`
3. `npm install && npm test` in `js/`
4. On tag: `wasm-pack publish` to npm

---

## 8. NOT in scope

- `wasm-bindgen-futures` / async support
- Streaming / `ReadableStream` integration
- Web Workers / `SharedArrayBuffer` interop
- Custom JS error types (use standard `Error` via `JsError`)
- Browser-specific APIs (`fetch`, `WebSocket` integration)
