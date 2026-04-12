# wasm-bindgen Bindings — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add wasm-bindgen JavaScript/TypeScript bindings with full Buf/BufMut API, iterator support, npm package with subpath exports. Also fix missing Buf methods on Python UTF-8 types.

**Architecture:** Feature-gated `#[cfg(feature = "wasm")]` bindings in per-type `wasm.rs` submodules (following the existing pyo3 pattern). Wrapper types for generic `RawBytes<S>`. JS shim attaches `Symbol.iterator`. npm package with subpath exports for `shared` and `compact` strategies.

**Tech Stack:** wasm-bindgen 0.2, js-sys, wasm-pack, vitest (JS tests)

**Spec:** `docs/superpowers/specs/2026-04-12-wasm-bindgen-design.md`

---

## File Map

### New files

| File | Purpose |
|---|---|
| `crates/smol-bytes/src/buffer/wasm.rs` | `#[wasm_bindgen]` methods for Buffer |
| `crates/smol-bytes/src/bytes_mut/wasm.rs` | `#[wasm_bindgen]` methods for BytesMut |
| `crates/smol-bytes/src/utf8_buffer/wasm.rs` | `#[wasm_bindgen]` methods for Utf8Buffer |
| `crates/smol-bytes/src/utf8_bytes/wasm.rs` | `#[wasm_bindgen]` methods for Utf8Bytes |
| `crates/smol-bytes/src/utf8_bytes_mut/wasm.rs` | `#[wasm_bindgen]` methods for Utf8BytesMut |
| `crates/smol-bytes/src/bytes/strategy/shared/wasm.rs` | WasmSharedBytes wrapper |
| `crates/smol-bytes/src/bytes/strategy/compact/wasm.rs` | WasmCompactBytes wrapper |
| `js/package.json` | npm package config with subpath exports |
| `js/src/index.ts` | Re-exports + Symbol.iterator shim |
| `js/src/shared.ts` | Subpath export for shared.Bytes |
| `js/src/compact.ts` | Subpath export for compact.Bytes |
| `js/tests/smol_bytes.test.ts` | JS test suite |
| `.github/workflows/wasm.yml` | CI workflow |

### Modified files

| File | Changes |
|---|---|
| `crates/smol-bytes/Cargo.toml` | Add `wasm` feature, `js-sys` dep |
| `crates/smol-bytes/src/buffer.rs` | Add `#[cfg(feature = "wasm")] mod wasm;` |
| `crates/smol-bytes/src/bytes_mut.rs` | Same |
| `crates/smol-bytes/src/utf8_buffer.rs` | Same |
| `crates/smol-bytes/src/utf8_bytes.rs` | Same |
| `crates/smol-bytes/src/utf8_bytes_mut.rs` | Same |
| `crates/smol-bytes/src/bytes/strategy/shared.rs` | Same |
| `crates/smol-bytes/src/bytes/strategy/compact.rs` | Same |
| `crates/smol-bytes/src/utf8_buffer/python.rs` | Add Buf getter methods |
| `crates/smol-bytes/src/utf8_bytes/python.rs` | Same |
| `crates/smol-bytes/src/utf8_bytes_mut/python.rs` | Same |

---

## Task 1: Feature flag + dependencies + wasm target compat

**Files:**
- Modify: `crates/smol-bytes/Cargo.toml`

- [ ] **Step 1: Add wasm feature and js-sys dependency**

In `crates/smol-bytes/Cargo.toml`, add to `[features]`:

```toml
wasm = ["dep:wasm-bindgen", "dep:js-sys", "std"]
```

Add to `[dependencies]`:

```toml
js-sys = { version = "0.3", optional = true }
```

The `wasm` feature implies `std` because `bytes` (required for Bytes/BytesMut) needs `std` features.

- [ ] **Step 2: Verify wasm target compiles**

```bash
rustup target add wasm32-unknown-unknown
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm"
```

Expected: compiles (possibly with warnings about unused items). If it fails, fix target-specific issues (e.g., thread-related code behind `#[cfg(not(target_family = "wasm"))]`).

- [ ] **Step 3: Commit**

```bash
git add crates/smol-bytes/Cargo.toml
git commit -m "feat(wasm): add wasm feature flag and js-sys dependency"
```

---

## Task 2: Fix Python UTF-8 types — add Buf getter methods

**Files:**
- Modify: `crates/smol-bytes/src/utf8_buffer/python.rs`
- Modify: `crates/smol-bytes/src/utf8_bytes/python.rs`
- Modify: `crates/smol-bytes/src/utf8_bytes_mut/python.rs`

The Python UTF-8 types are missing all Buf getter methods that the byte types have. This is needed for API parity across Python and JS.

- [ ] **Step 1: Understand the pattern**

Read `crates/smol-bytes/src/buffer/python.rs` and find the block of Buf getter `#[pymethods]` (around lines 188-900). These use the `PyBufExt` trait methods like `self.py_get_u8()`, `self.py_get_u16()`, etc.

The UTF-8 types need the SAME methods. However, the UTF-8 types don't implement `Buf` directly — their inner type does. The types implement `AsRef<[u8]>` via `self.as_str().as_bytes()`.

For Buf getters to work, the UTF-8 types need to implement the `PyBufExt` trait (which requires `Buf + AsRef<[u8]> + PyBufCommon`). Since Utf8 types don't implement `Buf`, we need to either:
- Add `Buf` impl for Utf8 types (delegates to inner), OR
- Add the getter methods directly without the trait

The simplest: add `remaining()` and `advance()` as direct methods, and delegate the get_* methods through the inner buffer. Check how the inner types work:
- `Utf8Buffer` wraps `Buffer` (which implements `Buf` via the io module)
- `Utf8Bytes` wraps `shared::Bytes` (which implements `Buf`)
- `Utf8BytesMut` wraps `BytesMut` (which implements `Buf` via `BufMut`)

For the Python side, add methods that delegate to the inner's Buf impl. For each UTF-8 python.rs, add inside the `#[pymethods]` block:

```rust
#[pyo3(name = "remaining")]
fn __python_remaining(&self) -> usize {
    self.inner.remaining()
}

#[pyo3(name = "advance")]
fn __python_advance(&mut self, cnt: usize) -> PyResult<()> {
    if cnt > self.inner.remaining() {
        return Err(pyo3::exceptions::PyBufferError::new_err("advance past end"));
    }
    bytes::Buf::advance(&mut self.inner, cnt);
    Ok(())
}

#[pyo3(name = "get_u8")]
fn __python_get_u8(&mut self) -> PyResult<u8> {
    if self.inner.remaining() < 1 {
        return Err(pyo3::exceptions::PyBufferError::new_err("not enough data"));
    }
    Ok(bytes::Buf::get_u8(&mut self.inner))
}
// ... all other get_* methods following the same pattern
```

Copy the FULL set of get_* methods from `buffer/python.rs` (they're ~700 lines). Each one checks `remaining()` then calls the `Buf` trait method on `self.inner`.

IMPORTANT: For `Utf8Buffer`, `self.inner` is `Buffer`. For `Utf8Bytes`, it's `shared::Bytes`. For `Utf8BytesMut`, it's `BytesMut`. All implement `Buf`. But after calling Buf methods that advance the cursor, the UTF-8 invariant still holds (the remaining bytes are a suffix of valid UTF-8).

- [ ] **Step 2: Add Buf getters to Utf8Buffer python.rs**

Add the full set of `remaining`, `advance`, and `get_*` methods inside the existing `#[pymethods] impl Utf8Buffer` block. Use `bytes::Buf` trait calls on `self.inner`.

- [ ] **Step 3: Add Buf getters to Utf8Bytes python.rs**

Same pattern, but `self.inner` is `shared::Bytes`.

- [ ] **Step 4: Add Buf getters to Utf8BytesMut python.rs**

Same pattern, but `self.inner` is `BytesMut`.

- [ ] **Step 5: Verify compilation**

```bash
cargo check --no-default-features --features "std pyo3"
```

- [ ] **Step 6: Rebuild and test Python**

```bash
source .venv/bin/activate
maturin develop --features pyo3 --manifest-path crates/smol-bytes/Cargo.toml
pytest tests/python/ -v
```

Add a quick test to `tests/python/test_smol_bytes.py`:

```python
def test_utf8_bytes_buf_getters():
    from smol_bytes import Utf8Bytes
    buf = Utf8Bytes.from_str("hello")
    assert buf.remaining() == 5
    assert buf.get_u8() == ord('h')
    assert buf.remaining() == 4
```

- [ ] **Step 7: Commit**

```bash
git add crates/smol-bytes/src/utf8_*/python.rs tests/python/
git commit -m "feat(pyo3): add Buf getter methods to UTF-8 types"
```

---

## Task 3: Buffer + BytesMut wasm bindings

**Files:**
- Create: `crates/smol-bytes/src/buffer/wasm.rs`
- Create: `crates/smol-bytes/src/bytes_mut/wasm.rs`
- Modify: `crates/smol-bytes/src/buffer.rs` (add `mod wasm`)
- Modify: `crates/smol-bytes/src/bytes_mut.rs` (add `mod wasm`)

For wasm-bindgen, we use `#[wasm_bindgen]` on `impl` blocks with `js_name` attributes. Since `Buffer` and `BytesMut` are concrete (non-generic) types, we can add `#[wasm_bindgen]` directly.

IMPORTANT: wasm-bindgen requires the type to be `pub`. Both `Buffer` and `BytesMut` are already `pub`.

### buffer/wasm.rs pattern:

```rust
use wasm_bindgen::prelude::*;
use super::Buffer;

#[wasm_bindgen]
impl Buffer {
    /// Create a new empty Buffer.
    #[wasm_bindgen(constructor)]
    pub fn new_wasm() -> Self {
        Self::new()
    }

    /// Create a Buffer from a byte array.
    #[wasm_bindgen(js_name = "fromBytes")]
    pub fn from_bytes_wasm(data: &[u8]) -> Result<Buffer, JsError> {
        Buffer::try_from(data).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Create a Buffer from a string.
    #[wasm_bindgen(js_name = "fromString")]
    pub fn from_string_wasm(s: &str) -> Result<Buffer, JsError> {
        Buffer::try_from(s).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Return contents as Uint8Array.
    #[wasm_bindgen(js_name = "toBytes")]
    pub fn to_bytes_wasm(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }

    /// Return contents as string (UTF-8).
    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string_wasm(&self) -> Result<String, JsError> {
        core::str::from_utf8(self.as_slice())
            .map(|s| s.to_string())
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "len")]
    pub fn len_wasm(&self) -> usize { self.len() }

    #[wasm_bindgen(js_name = "isEmpty")]
    pub fn is_empty_wasm(&self) -> bool { self.is_empty() }

    #[wasm_bindgen(js_name = "remaining")]
    pub fn remaining_wasm(&self) -> usize { self.remaining() }

    #[wasm_bindgen(js_name = "advance")]
    pub fn advance_wasm(&mut self, cnt: usize) -> Result<(), JsError> {
        self.try_advance(cnt).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "clear")]
    pub fn clear_wasm(&mut self) { self.clear(); }

    #[wasm_bindgen(js_name = "truncate")]
    pub fn truncate_wasm(&mut self, new_len: usize) { self.truncate(new_len); }

    // --- Buf getters ---
    #[wasm_bindgen(js_name = "getU8")]
    pub fn get_u8_wasm(&mut self) -> Result<u8, JsError> {
        if self.remaining() < 1 { return Err(JsError::new("not enough data")); }
        Ok(bytes::Buf::get_u8(self))
    }

    #[wasm_bindgen(js_name = "getI8")]
    pub fn get_i8_wasm(&mut self) -> Result<i8, JsError> {
        if self.remaining() < 1 { return Err(JsError::new("not enough data")); }
        Ok(bytes::Buf::get_i8(self))
    }

    #[wasm_bindgen(js_name = "getU16")]
    pub fn get_u16_wasm(&mut self) -> Result<u16, JsError> {
        if self.remaining() < 2 { return Err(JsError::new("not enough data")); }
        Ok(bytes::Buf::get_u16(self))
    }

    #[wasm_bindgen(js_name = "getU16Le")]
    pub fn get_u16_le_wasm(&mut self) -> Result<u16, JsError> {
        if self.remaining() < 2 { return Err(JsError::new("not enough data")); }
        Ok(bytes::Buf::get_u16_le(self))
    }

    // ... continue for all get_i16, get_u32, get_u32_le, get_i32, get_i32_le,
    // get_u64 (returns u64), get_u64_le, get_i64, get_i64_le,
    // get_f32, get_f32_le, get_f64, get_f64_le,
    // get_uint(nbytes), get_uint_le(nbytes), get_int(nbytes), get_int_le(nbytes)
    //
    // For u64/i64: wasm-bindgen maps them to JS bigint automatically.
    // For get_uint/get_int: take nbytes: usize, return u64/i64.

    // --- BufMut putters ---
    #[wasm_bindgen(js_name = "putU8")]
    pub fn put_u8_wasm(&mut self, val: u8) {
        bytes::BufMut::put_u8(self, val);
    }

    #[wasm_bindgen(js_name = "putSlice")]
    pub fn put_slice_wasm(&mut self, data: &[u8]) -> Result<(), JsError> {
        self.try_put_slice(data).map_err(|e| JsError::new(&e.to_string()))
    }

    // ... continue for all put_i8, put_u16, put_u16_le, etc.

    // --- Split ---
    #[wasm_bindgen(js_name = "splitTo")]
    pub fn split_to_wasm(&mut self, at: usize) -> Result<Buffer, JsError> {
        self.try_split_to(at)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "splitOff")]
    pub fn split_off_wasm(&mut self, at: usize) -> Result<Buffer, JsError> {
        self.try_split_off(at)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "slice")]
    pub fn slice_wasm(&self, start: usize, end: usize) -> Result<Buffer, JsError> {
        self.try_slice(start..end)
            .map_err(|e| JsError::new(&e.to_string()))
    }
}
```

Apply the same pattern to `bytes_mut/wasm.rs` with BytesMut-specific methods (`withCapacity`, `reserve`, `split`, `unsplit`, `isInline`, `isHeap`).

- [ ] **Step 1: Create buffer/wasm.rs with all methods**
- [ ] **Step 2: Add `#[cfg(feature = "wasm")] mod wasm;` to buffer.rs**
- [ ] **Step 3: Create bytes_mut/wasm.rs with all methods (including BytesMut extras)**
- [ ] **Step 4: Add `#[cfg(feature = "wasm")] mod wasm;` to bytes_mut.rs**
- [ ] **Step 5: Verify compilation**

```bash
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm"
```

- [ ] **Step 6: Commit**

```bash
git add crates/smol-bytes/src/buffer/wasm.rs crates/smol-bytes/src/buffer.rs \
       crates/smol-bytes/src/bytes_mut/wasm.rs crates/smol-bytes/src/bytes_mut.rs
git commit -m "feat(wasm): add Buffer and BytesMut wasm-bindgen bindings"
```

---

## Task 4: WasmSharedBytes + WasmCompactBytes wrappers

**Files:**
- Create: `crates/smol-bytes/src/bytes/strategy/shared/wasm.rs`
- Create: `crates/smol-bytes/src/bytes/strategy/compact/wasm.rs`
- Modify: `crates/smol-bytes/src/bytes/strategy/shared.rs` (add `mod wasm`)
- Modify: `crates/smol-bytes/src/bytes/strategy/compact.rs` (add `mod wasm`)

These need wrapper types because `RawBytes<S>` is generic. Pattern (for shared):

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "Bytes")]
pub struct WasmSharedBytes {
    inner: super::Bytes,
}

#[wasm_bindgen(js_class = "Bytes")]
impl WasmSharedBytes {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self { Self { inner: super::Bytes::new() } }

    #[wasm_bindgen(js_name = "fromBytes")]
    pub fn from_bytes(data: &[u8]) -> Self {
        Self { inner: super::Bytes::copy_from_slice(data) }
    }

    // ... all the same methods as Buffer wasm.rs,
    // but no BufMut putters (immutable type),
    // plus isInline() and isHeap()
}
```

For compact, identical but wraps `compact::Bytes` and the JS class name is `"CompactBytes"` (to avoid collision — the subpath export renames it).

Actually, since we use subpath exports, both can be named `"Bytes"` in their respective wasm.rs. But wasm-bindgen puts all exports in one wasm module. Two types named `"Bytes"` would collide. So:
- shared: `#[wasm_bindgen(js_name = "SharedBytes")]`
- compact: `#[wasm_bindgen(js_name = "CompactBytes")]`

The JS re-export renames them to `Bytes` in each subpath.

- [ ] **Step 1: Create shared/wasm.rs with WasmSharedBytes**
- [ ] **Step 2: Add `mod wasm` to shared.rs**
- [ ] **Step 3: Create compact/wasm.rs with WasmCompactBytes**
- [ ] **Step 4: Add `mod wasm` to compact.rs**
- [ ] **Step 5: Verify compilation**

```bash
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm"
```

- [ ] **Step 6: Commit**

```bash
git add crates/smol-bytes/src/bytes/strategy/shared/wasm.rs \
       crates/smol-bytes/src/bytes/strategy/shared.rs \
       crates/smol-bytes/src/bytes/strategy/compact/wasm.rs \
       crates/smol-bytes/src/bytes/strategy/compact.rs
git commit -m "feat(wasm): add SharedBytes and CompactBytes wasm-bindgen wrappers"
```

---

## Task 5: UTF-8 type wasm bindings

**Files:**
- Create: `crates/smol-bytes/src/utf8_buffer/wasm.rs`
- Create: `crates/smol-bytes/src/utf8_bytes/wasm.rs`
- Create: `crates/smol-bytes/src/utf8_bytes_mut/wasm.rs`
- Modify: `crates/smol-bytes/src/utf8_buffer.rs`, `utf8_bytes.rs`, `utf8_bytes_mut.rs` (add `mod wasm`)

UTF-8 types expose string-oriented methods:

```rust
#[wasm_bindgen]
impl Utf8Buffer {
    #[wasm_bindgen(constructor)]
    pub fn new_wasm() -> Self { Self::new() }

    #[wasm_bindgen(js_name = "fromString")]
    pub fn from_string_wasm(s: &str) -> Result<Utf8Buffer, JsError> {
        Utf8Buffer::try_from_str(s).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string_wasm(&self) -> String { self.as_str().to_string() }

    #[wasm_bindgen(js_name = "toBytes")]
    pub fn to_bytes_wasm(&self) -> Vec<u8> { self.as_str().as_bytes().to_vec() }

    #[wasm_bindgen(js_name = "len")]
    pub fn len_wasm(&self) -> usize { self.len() }

    #[wasm_bindgen(js_name = "isEmpty")]
    pub fn is_empty_wasm(&self) -> bool { self.is_empty() }

    #[wasm_bindgen(js_name = "push")]
    pub fn push_wasm(&mut self, ch: &str) -> Result<(), JsError> {
        let c = ch.chars().next().ok_or_else(|| JsError::new("empty string"))?;
        self.try_push(c).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "pushStr")]
    pub fn push_str_wasm(&mut self, s: &str) -> Result<(), JsError> {
        self.try_push_str(s).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "clear")]
    pub fn clear_wasm(&mut self) { self.clear(); }

    #[wasm_bindgen(js_name = "splitTo")]
    pub fn split_to_wasm(&mut self, at: usize) -> Result<Utf8Buffer, JsError> {
        self.try_split_to(at).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "splitOff")]
    pub fn split_off_wasm(&mut self, at: usize) -> Result<Utf8Buffer, JsError> {
        self.try_split_off(at).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "slice")]
    pub fn slice_wasm(&self, start: usize, end: usize) -> Result<Utf8Buffer, JsError> {
        self.try_slice(start..end).map_err(|e| JsError::new(&e.to_string()))
    }

    // Buf getters — same as buffer/wasm.rs, delegating through self.inner
    #[wasm_bindgen(js_name = "remaining")]
    pub fn remaining_wasm(&self) -> usize { self.inner.remaining() }

    #[wasm_bindgen(js_name = "advance")]
    pub fn advance_wasm(&mut self, cnt: usize) -> Result<(), JsError> {
        // ... bounds check + Buf::advance on inner
    }

    #[wasm_bindgen(js_name = "getU8")]
    pub fn get_u8_wasm(&mut self) -> Result<u8, JsError> {
        // ... delegate to inner
    }
    // ... all get_* methods
}
```

Apply same pattern to Utf8Bytes and Utf8BytesMut. Utf8BytesMut also gets `withCapacity`, `reserve`, `split`, `unsplit`, `isInline`, `isHeap`.

- [ ] **Step 1: Create utf8_buffer/wasm.rs**
- [ ] **Step 2: Create utf8_bytes/wasm.rs**
- [ ] **Step 3: Create utf8_bytes_mut/wasm.rs**
- [ ] **Step 4: Add `mod wasm` to all three parent files**
- [ ] **Step 5: Verify compilation**

```bash
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm"
```

- [ ] **Step 6: Commit**

```bash
git add crates/smol-bytes/src/utf8_*/wasm.rs crates/smol-bytes/src/utf8_*.rs
git commit -m "feat(wasm): add UTF-8 type wasm-bindgen bindings"
```

---

## Task 6: Iterator support

**Files:**
- Modify: all 7 wasm.rs files (add `iter()` method + iterator struct)
- Create (later in Task 7): `js/src/index.ts` (Symbol.iterator shim)

For each byte type, add a `BytesIterator` struct:

```rust
#[wasm_bindgen]
pub struct BufferIterator {
    data: Vec<u8>,
    index: usize,
}

#[wasm_bindgen]
impl BufferIterator {
    pub fn next(&mut self) -> JsValue {
        if self.index < self.data.len() {
            let val = self.data[self.index];
            self.index += 1;
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &"value".into(), &JsValue::from(val)).unwrap();
            js_sys::Reflect::set(&obj, &"done".into(), &JsValue::FALSE).unwrap();
            obj.into()
        } else {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &"done".into(), &JsValue::TRUE).unwrap();
            obj.into()
        }
    }
}

// In Buffer's #[wasm_bindgen] impl:
#[wasm_bindgen(js_name = "iter")]
pub fn iter_wasm(&self) -> BufferIterator {
    BufferIterator { data: self.as_slice().to_vec(), index: 0 }
}
```

For UTF-8 types, the iterator yields strings (single chars):

```rust
#[wasm_bindgen]
pub struct Utf8CharIterator {
    chars: Vec<String>,
    index: usize,
}

#[wasm_bindgen]
impl Utf8CharIterator {
    pub fn next(&mut self) -> JsValue {
        if self.index < self.chars.len() {
            let val = &self.chars[self.index];
            self.index += 1;
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &"value".into(), &JsValue::from_str(val)).unwrap();
            js_sys::Reflect::set(&obj, &"done".into(), &JsValue::FALSE).unwrap();
            obj.into()
        } else {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &"done".into(), &JsValue::TRUE).unwrap();
            obj.into()
        }
    }
}
```

- [ ] **Step 1: Add iterator structs and `iter()` methods to all 7 wasm.rs files**

Each byte type gets a `{Type}Iterator` struct; each UTF-8 type gets a `Utf8{Type}Iterator`. Or share a common `ByteIterator` and `CharIterator` if possible (they can be defined in a shared wasm helper module).

- [ ] **Step 2: Verify compilation**

```bash
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm"
```

- [ ] **Step 3: Commit**

```bash
git add crates/smol-bytes/src/*/wasm.rs crates/smol-bytes/src/bytes/strategy/*/wasm.rs
git commit -m "feat(wasm): add JS-compatible iterator protocol to all types"
```

---

## Task 7: npm package setup

**Files:**
- Create: `js/package.json`
- Create: `js/tsconfig.json`
- Create: `js/src/index.ts`
- Create: `js/src/shared.ts`
- Create: `js/src/compact.ts`

- [ ] **Step 1: Build wasm module**

```bash
cd crates/smol-bytes
wasm-pack build --target bundler --out-dir ../../js/pkg -- --features wasm
```

This generates `js/pkg/` with the wasm binary, JS glue, and `.d.ts` types.

- [ ] **Step 2: Create js/package.json**

```json
{
  "name": "smol-bytes",
  "version": "0.1.0",
  "type": "module",
  "description": "High-performance, clone-efficient byte buffers for JavaScript/TypeScript",
  "license": "MIT OR Apache-2.0",
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "import": "./dist/index.js"
    },
    "./shared": {
      "types": "./dist/shared.d.ts",
      "import": "./dist/shared.js"
    },
    "./compact": {
      "types": "./dist/compact.d.ts",
      "import": "./dist/compact.js"
    }
  },
  "files": ["dist/", "pkg/"],
  "scripts": {
    "build:wasm": "cd ../crates/smol-bytes && wasm-pack build --target bundler --out-dir ../../js/pkg -- --features wasm",
    "build:js": "tsc",
    "build": "npm run build:wasm && npm run build:js",
    "test": "vitest run"
  },
  "devDependencies": {
    "typescript": "^5.0",
    "vitest": "^3.0"
  }
}
```

- [ ] **Step 3: Create js/tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ES2020",
    "moduleResolution": "bundler",
    "outDir": "./dist",
    "rootDir": "./src",
    "declaration": true,
    "strict": true,
    "esModuleInterop": true
  },
  "include": ["src/**/*.ts"]
}
```

- [ ] **Step 4: Create js/src/index.ts**

```typescript
// Re-export all types from wasm pkg
export { Buffer, BytesMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut } from '../pkg/smol_bytes.js';
export type { SharedBytes, CompactBytes } from '../pkg/smol_bytes.js';

// Attach Symbol.iterator to all types
import { Buffer, BytesMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut, SharedBytes, CompactBytes } from '../pkg/smol_bytes.js';

for (const Cls of [Buffer, BytesMut, SharedBytes, CompactBytes]) {
  (Cls.prototype as any)[Symbol.iterator] = function() { return this.iter(); };
}

for (const Cls of [Utf8Buffer, Utf8Bytes, Utf8BytesMut]) {
  (Cls.prototype as any)[Symbol.iterator] = function() { return this.iter(); };
}
```

- [ ] **Step 5: Create js/src/shared.ts**

```typescript
export { SharedBytes as Bytes } from '../pkg/smol_bytes.js';
```

- [ ] **Step 6: Create js/src/compact.ts**

```typescript
export { CompactBytes as Bytes } from '../pkg/smol_bytes.js';
```

- [ ] **Step 7: Commit**

```bash
git add js/package.json js/tsconfig.json js/src/
git commit -m "feat(wasm): add npm package with subpath exports and Symbol.iterator shim"
```

---

## Task 8: JS tests

**Files:**
- Create: `js/tests/smol_bytes.test.ts`

- [ ] **Step 1: Install deps and build**

```bash
cd js && npm install && npm run build
```

- [ ] **Step 2: Write test suite**

`js/tests/smol_bytes.test.ts`:

```typescript
import { describe, test, expect } from 'vitest';
import { Buffer, BytesMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut } from '../src/index.js';
import { Bytes as SharedBytes } from '../src/shared.js';
import { Bytes as CompactBytes } from '../src/compact.js';

describe('imports', () => {
  test('all types importable', () => {
    expect(Buffer).toBeDefined();
    expect(BytesMut).toBeDefined();
    expect(SharedBytes).toBeDefined();
    expect(CompactBytes).toBeDefined();
    expect(Utf8Buffer).toBeDefined();
    expect(Utf8Bytes).toBeDefined();
    expect(Utf8BytesMut).toBeDefined();
  });
});

describe('construction', () => {
  test('Buffer.fromBytes', () => {
    const buf = Buffer.fromBytes(new Uint8Array([1, 2, 3]));
    expect(buf.len()).toBe(3);
    expect(buf.toBytes()).toEqual(new Uint8Array([1, 2, 3]));
  });

  test('Utf8Bytes.fromString', () => {
    const s = Utf8Bytes.fromString('café 🦀');
    expect(s.toString()).toBe('café 🦀');
  });
});

describe('Buf getters', () => {
  test('getU8', () => {
    const buf = Buffer.fromBytes(new Uint8Array([0x42]));
    expect(buf.getU8()).toBe(0x42);
  });

  test('getU16Le', () => {
    const buf = Buffer.fromBytes(new Uint8Array([0x01, 0x02]));
    expect(buf.getU16Le()).toBe(0x0201);
  });

  test('getU64 returns bigint', () => {
    const data = new Uint8Array(8);
    data[0] = 1;
    const buf = Buffer.fromBytes(data);
    expect(typeof buf.getU64()).toBe('bigint');
  });
});

describe('iteration', () => {
  test('Buffer iterates bytes', () => {
    const buf = Buffer.fromBytes(new Uint8Array([1, 2, 3]));
    expect([...buf]).toEqual([1, 2, 3]);
  });

  test('Utf8Bytes iterates chars', () => {
    const s = Utf8Bytes.fromString('café');
    expect([...s]).toEqual(['c', 'a', 'f', 'é']);
  });
});

describe('split', () => {
  test('splitTo', () => {
    const buf = SharedBytes.fromBytes(new TextEncoder().encode('hello world'));
    const head = buf.splitTo(5);
    expect(head.toBytes()).toEqual(new TextEncoder().encode('hello'));
  });

  test('UTF-8 mid-char split throws', () => {
    const s = Utf8Bytes.fromString('café');
    expect(() => s.splitOff(4)).toThrow();
  });
});

describe('storage', () => {
  test('small SharedBytes is inline', () => {
    const b = SharedBytes.fromBytes(new Uint8Array([1, 2, 3]));
    expect(b.isInline()).toBe(true);
  });

  test('large SharedBytes is heap', () => {
    const b = SharedBytes.fromBytes(new Uint8Array(100));
    expect(b.isHeap()).toBe(true);
  });
});
```

- [ ] **Step 3: Run tests**

```bash
cd js && npm test
```

- [ ] **Step 4: Commit**

```bash
git add js/tests/
git commit -m "test(wasm): add JS/TS integration test suite"
```

---

## Task 9: CI workflow

**Files:**
- Create: `.github/workflows/wasm.yml`

- [ ] **Step 1: Create workflow**

```yaml
name: WebAssembly

on:
  push:
    branches: [main, "*.*.x"]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: jetli/wasm-pack-action@v0.4.0
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - name: Build wasm
        run: |
          cd crates/smol-bytes
          wasm-pack build --target bundler --out-dir ../../js/pkg -- --features wasm
      - name: Install JS deps and build
        run: cd js && npm install && npm run build:js
      - name: Run JS tests
        run: cd js && npm test

  publish:
    if: startsWith(github.ref, 'refs/tags/')
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: jetli/wasm-pack-action@v0.4.0
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          registry-url: https://registry.npmjs.org
      - run: |
          cd crates/smol-bytes
          wasm-pack build --target bundler --out-dir ../../js/pkg -- --features wasm
      - run: cd js && npm install && npm run build:js
      - run: cd js && npm publish
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/wasm.yml
git commit -m "ci: add WebAssembly build and publish workflow"
```
