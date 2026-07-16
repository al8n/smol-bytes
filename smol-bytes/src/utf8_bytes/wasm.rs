use wasm_bindgen::prelude::*;

use crate::Utf8Buf as _;
use crate::bytes::strategy::compact::Compact;
use crate::bytes::strategy::shared::Shared;
use crate::wasm_iter::CharIterator;

/// Concrete shared Utf8Bytes type for WASM bindings.
type SharedUtf8Bytes = super::Utf8Bytes<Shared>;

/// Concrete compact Utf8Bytes type for WASM bindings.
type CompactUtf8BytesInner = super::Utf8Bytes<Compact>;

/// WASM bindings for the [`Utf8Bytes`](crate::Utf8Bytes) type (shared variant).
#[wasm_bindgen(js_name = "Utf8Bytes")]
pub struct WasmUtf8Bytes {
  inner: SharedUtf8Bytes,
}

impl From<SharedUtf8Bytes> for WasmUtf8Bytes {
  fn from(inner: SharedUtf8Bytes) -> Self {
    Self { inner }
  }
}

#[allow(missing_docs)]
#[wasm_bindgen(js_class = "Utf8Bytes")]
impl WasmUtf8Bytes {
  /// Create a new empty `Utf8Bytes`.
  #[wasm_bindgen(constructor)]
  pub fn new_wasm() -> Self {
    Self {
      inner: SharedUtf8Bytes::new(),
    }
  }

  /// Create a `Utf8Bytes` from a UTF-8 string.
  ///
  /// @param s - The source string whose bytes are copied.
  #[wasm_bindgen(js_name = "fromString")]
  pub fn from_string_wasm(s: &str) -> WasmUtf8Bytes {
    WasmUtf8Bytes {
      inner: SharedUtf8Bytes::from(s),
    }
  }

  /// Create a `Utf8Bytes` from a static string.
  ///
  /// Note: In WASM, this copies the data (true static references cannot cross the JS boundary).
  #[wasm_bindgen(js_name = "fromStatic")]
  pub fn from_static_wasm(s: &str) -> WasmUtf8Bytes {
    // NOTE: we cannot truly take a &'static str from JS, so we copy via From.
    WasmUtf8Bytes {
      inner: SharedUtf8Bytes::from(s),
    }
  }

  /// Return contents as a UTF-8 string.
  #[wasm_bindgen(js_name = "toString")]
  pub fn to_string_wasm(&self) -> String {
    self.inner.as_str().to_string()
  }

  /// Return contents as a `Uint8Array` (copy of the UTF-8 encoded bytes).
  #[wasm_bindgen(js_name = "toBytes")]
  pub fn to_bytes_wasm(&self) -> Vec<u8> {
    self.inner.as_str().as_bytes().to_vec()
  }

  /// Return the byte length of the buffer.
  #[wasm_bindgen(js_name = "len")]
  pub fn len_wasm(&self) -> usize {
    self.inner.len()
  }

  /// Return `true` if the buffer has no bytes.
  #[wasm_bindgen(js_name = "isEmpty")]
  pub fn is_empty_wasm(&self) -> bool {
    self.inner.is_empty()
  }

  /// Return `true` if data is stored inline (no heap allocation).
  #[wasm_bindgen(js_name = "isInline")]
  pub fn is_inline_wasm(&self) -> bool {
    self.inner.is_inline()
  }

  /// Return `true` if data is stored on the heap.
  #[wasm_bindgen(js_name = "isHeap")]
  pub fn is_heap_wasm(&self) -> bool {
    self.inner.is_heap()
  }

  /// Return the number of readable bytes remaining.
  #[wasm_bindgen(js_name = "remaining")]
  pub fn remaining_wasm(&self) -> usize {
    bytes::Buf::remaining(self.inner.as_inner())
  }

  /// Advance the read cursor by `cnt` bytes.
  ///
  /// @throws {Error} If `cnt` exceeds the number of remaining bytes or is not a UTF-8 character boundary.
  #[wasm_bindgen(js_name = "advance")]
  pub fn advance_wasm(&mut self, cnt: usize) -> Result<(), JsError> {
    self
      .inner
      .validate_char_boundary(cnt)
      .map_err(|e| JsError::new(&e.to_string()))?;
    self
      .inner
      .inner
      .try_advance(cnt)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Split the buffer at position `at`, returning bytes `[0, at)`.
  /// Self becomes `[at, len)`.
  ///
  /// @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
  #[wasm_bindgen(js_name = "splitTo")]
  pub fn split_to_wasm(&mut self, at: usize) -> Result<WasmUtf8Bytes, JsError> {
    self
      .inner
      .try_split_to(at)
      .map(Into::into)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Split the buffer at position `at`, returning bytes `[at, len)`.
  /// Self becomes `[0, at)`.
  ///
  /// @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
  #[wasm_bindgen(js_name = "splitOff")]
  pub fn split_off_wasm(&mut self, at: usize) -> Result<WasmUtf8Bytes, JsError> {
    self
      .inner
      .try_split_off(at)
      .map(Into::into)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Return a copy of bytes in range `[start, end)`.
  ///
  /// @throws {Error} If the range is out of bounds or not on UTF-8 char boundaries.
  #[wasm_bindgen(js_name = "slice")]
  pub fn slice_wasm(&self, start: usize, end: usize) -> Result<WasmUtf8Bytes, JsError> {
    self
      .inner
      .try_slice(start..end)
      .map(Into::into)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  // -- Iterator --

  /// Return a JS-compatible character iterator.
  ///
  /// Use `for (const ch of buf)` after attaching `Symbol.iterator`.
  #[wasm_bindgen(js_name = "iter")]
  pub fn iter_wasm(&self) -> CharIterator {
    CharIterator::new(self.inner.as_str())
  }
}

/// WASM bindings for the [`Utf8Bytes`](crate::Utf8Bytes) type (compact variant).
///
/// Available as `CompactUtf8Bytes` via `import { CompactUtf8Bytes } from 'smol-bytes/compact'`.
#[wasm_bindgen(js_name = "CompactUtf8Bytes")]
pub struct WasmCompactUtf8Bytes {
  inner: CompactUtf8BytesInner,
}

impl From<CompactUtf8BytesInner> for WasmCompactUtf8Bytes {
  fn from(inner: CompactUtf8BytesInner) -> Self {
    Self { inner }
  }
}

#[allow(missing_docs)]
#[wasm_bindgen(js_class = "CompactUtf8Bytes")]
impl WasmCompactUtf8Bytes {
  /// Create a new empty `CompactUtf8Bytes`.
  #[wasm_bindgen(constructor)]
  pub fn new_wasm() -> Self {
    Self {
      inner: CompactUtf8BytesInner::new(),
    }
  }

  /// Create a `CompactUtf8Bytes` from a UTF-8 string.
  ///
  /// @param s - The source string whose bytes are copied.
  #[wasm_bindgen(js_name = "fromString")]
  pub fn from_string_wasm(s: &str) -> WasmCompactUtf8Bytes {
    WasmCompactUtf8Bytes {
      inner: CompactUtf8BytesInner::from(s),
    }
  }

  /// Create a `CompactUtf8Bytes` from a static string.
  ///
  /// Note: In WASM, this copies the data (true static references cannot cross the JS boundary).
  #[wasm_bindgen(js_name = "fromStatic")]
  pub fn from_static_wasm(s: &str) -> WasmCompactUtf8Bytes {
    // NOTE: we cannot truly take a &'static str from JS, so we copy via From.
    WasmCompactUtf8Bytes {
      inner: CompactUtf8BytesInner::from(s),
    }
  }

  /// Return contents as a UTF-8 string.
  #[wasm_bindgen(js_name = "toString")]
  pub fn to_string_wasm(&self) -> String {
    self.inner.as_str().to_string()
  }

  /// Return contents as a `Uint8Array` (copy of the UTF-8 encoded bytes).
  #[wasm_bindgen(js_name = "toBytes")]
  pub fn to_bytes_wasm(&self) -> Vec<u8> {
    self.inner.as_str().as_bytes().to_vec()
  }

  /// Return the byte length of the buffer.
  #[wasm_bindgen(js_name = "len")]
  pub fn len_wasm(&self) -> usize {
    self.inner.len()
  }

  /// Return `true` if the buffer has no bytes.
  #[wasm_bindgen(js_name = "isEmpty")]
  pub fn is_empty_wasm(&self) -> bool {
    self.inner.is_empty()
  }

  /// Return `true` if data is stored inline (no heap allocation).
  #[wasm_bindgen(js_name = "isInline")]
  pub fn is_inline_wasm(&self) -> bool {
    self.inner.is_inline()
  }

  /// Return `true` if data is stored on the heap.
  #[wasm_bindgen(js_name = "isHeap")]
  pub fn is_heap_wasm(&self) -> bool {
    self.inner.is_heap()
  }

  /// Return the number of readable bytes remaining.
  #[wasm_bindgen(js_name = "remaining")]
  pub fn remaining_wasm(&self) -> usize {
    bytes::Buf::remaining(self.inner.as_inner())
  }

  /// Advance the read cursor by `cnt` bytes.
  ///
  /// @throws {Error} If `cnt` exceeds the number of remaining bytes or is not a UTF-8 character boundary.
  #[wasm_bindgen(js_name = "advance")]
  pub fn advance_wasm(&mut self, cnt: usize) -> Result<(), JsError> {
    self
      .inner
      .validate_char_boundary(cnt)
      .map_err(|e| JsError::new(&e.to_string()))?;
    self
      .inner
      .inner
      .try_advance(cnt)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Split the buffer at position `at`, returning bytes `[0, at)`.
  /// Self becomes `[at, len)`.
  ///
  /// @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
  #[wasm_bindgen(js_name = "splitTo")]
  pub fn split_to_wasm(&mut self, at: usize) -> Result<WasmCompactUtf8Bytes, JsError> {
    self
      .inner
      .try_split_to(at)
      .map(Into::into)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Split the buffer at position `at`, returning bytes `[at, len)`.
  /// Self becomes `[0, at)`.
  ///
  /// @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
  #[wasm_bindgen(js_name = "splitOff")]
  pub fn split_off_wasm(&mut self, at: usize) -> Result<WasmCompactUtf8Bytes, JsError> {
    self
      .inner
      .try_split_off(at)
      .map(Into::into)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Return a copy of bytes in range `[start, end)`.
  ///
  /// @throws {Error} If the range is out of bounds or not on UTF-8 char boundaries.
  #[wasm_bindgen(js_name = "slice")]
  pub fn slice_wasm(&self, start: usize, end: usize) -> Result<WasmCompactUtf8Bytes, JsError> {
    self
      .inner
      .try_slice(start..end)
      .map(Into::into)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  // -- Iterator --

  /// Return a JS-compatible character iterator.
  ///
  /// Use `for (const ch of buf)` after attaching `Symbol.iterator`.
  #[wasm_bindgen(js_name = "iter")]
  pub fn iter_wasm(&self) -> CharIterator {
    CharIterator::new(self.inner.as_str())
  }
}
