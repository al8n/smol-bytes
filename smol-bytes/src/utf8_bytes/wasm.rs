use wasm_bindgen::prelude::*;

use crate::bytes::strategy::compact::Compact;
use crate::bytes::strategy::shared::Shared;
use crate::utf8_buf::validate_char_boundary;
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
  pub fn from_string_wasm(s: js_sys::JsString) -> Result<WasmUtf8Bytes, JsError> {
    let s = crate::wasm::js_string_to_string(&s)?;
    Ok(WasmUtf8Bytes {
      inner: SharedUtf8Bytes::from(s),
    })
  }

  /// Decode bytes as UTF-8, replacing malformed sequences with `U+FFFD`.
  #[wasm_bindgen(js_name = "fromBytesLossy")]
  pub fn from_bytes_lossy_wasm(data: &[u8]) -> WasmUtf8Bytes {
    WasmUtf8Bytes {
      inner: SharedUtf8Bytes::from_utf8_lossy(crate::bytes::strategy::shared::Bytes::from(data)),
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
  #[wasm_bindgen(js_name = "byteLength")]
  pub fn len_wasm(&self) -> usize {
    self.inner.len()
  }

  /// Return the number of Unicode scalar values in this view.
  #[wasm_bindgen(js_name = "charLength")]
  pub fn char_length_wasm(&self) -> usize {
    self.inner.as_str().chars().count()
  }

  /// Return `true` if the buffer has no bytes.
  #[wasm_bindgen(js_name = "isEmpty")]
  pub fn is_empty_wasm(&self) -> bool {
    self.inner.is_empty()
  }

  /// Remove and return the final Unicode scalar, if present.
  #[wasm_bindgen(js_name = "pop")]
  pub fn pop_wasm(&mut self) -> Option<String> {
    self.inner.pop().map(|ch| ch.to_string())
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
  #[wasm_bindgen(js_name = "remainingBytes")]
  pub fn remaining_wasm(&self) -> usize {
    bytes::Buf::remaining(self.inner.as_inner())
  }

  /// Advance the read cursor by `cnt` bytes.
  ///
  /// @throws {Error} If `cnt` exceeds the number of remaining bytes or is not a UTF-8 character boundary.
  #[wasm_bindgen(js_name = "advanceBytes")]
  pub fn advance_wasm(&mut self, cnt: usize) -> Result<(), JsError> {
    validate_char_boundary(self.inner.as_str(), cnt).map_err(|e| JsError::new(&e.to_string()))?;
    self
      .inner
      .inner
      .try_advance(cnt)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Clear this visible UTF-8 view.
  #[wasm_bindgen(js_name = "clear")]
  pub fn clear_wasm(&mut self) {
    self.inner.clear();
  }

  /// Truncate this view at a UTF-8 byte boundary.
  #[wasm_bindgen(js_name = "truncateBytes")]
  pub fn truncate_wasm(&mut self, new_len: usize) -> Result<(), JsError> {
    self
      .inner
      .try_truncate(new_len)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Split the buffer at position `at`, returning bytes `[0, at)`.
  /// Self becomes `[at, len)`.
  ///
  /// @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
  #[wasm_bindgen(js_name = "splitToBytes")]
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
  #[wasm_bindgen(js_name = "splitOffBytes")]
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
  #[wasm_bindgen(js_name = "sliceBytes")]
  pub fn slice_wasm(&self, start: usize, end: usize) -> Result<WasmUtf8Bytes, JsError> {
    self
      .inner
      .try_slice(start..end)
      .map(Into::into)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Return a cheap strategy-preserving clone of this immutable view.
  #[wasm_bindgen(js_name = "clone")]
  pub fn clone_wasm(&self) -> WasmUtf8Bytes {
    WasmUtf8Bytes {
      inner: self.inner.clone(),
    }
  }

  /// Return an independent mutable copy of this view.
  #[wasm_bindgen(js_name = "toMutable")]
  pub fn to_mutable_wasm(&self) -> crate::Utf8BytesMut {
    crate::Utf8BytesMut::from(self.inner.as_str())
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
  pub fn from_string_wasm(s: js_sys::JsString) -> Result<WasmCompactUtf8Bytes, JsError> {
    let s = crate::wasm::js_string_to_string(&s)?;
    Ok(WasmCompactUtf8Bytes {
      inner: CompactUtf8BytesInner::from(s),
    })
  }

  /// Decode bytes as UTF-8, replacing malformed sequences with `U+FFFD`.
  #[wasm_bindgen(js_name = "fromBytesLossy")]
  pub fn from_bytes_lossy_wasm(data: &[u8]) -> WasmCompactUtf8Bytes {
    WasmCompactUtf8Bytes {
      inner: CompactUtf8BytesInner::from_utf8_lossy(crate::bytes::strategy::compact::Bytes::from(
        data,
      )),
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
  #[wasm_bindgen(js_name = "byteLength")]
  pub fn len_wasm(&self) -> usize {
    self.inner.len()
  }

  /// Return the number of Unicode scalar values in this view.
  #[wasm_bindgen(js_name = "charLength")]
  pub fn char_length_wasm(&self) -> usize {
    self.inner.as_str().chars().count()
  }

  /// Return `true` if the buffer has no bytes.
  #[wasm_bindgen(js_name = "isEmpty")]
  pub fn is_empty_wasm(&self) -> bool {
    self.inner.is_empty()
  }

  /// Remove and return the final Unicode scalar, if present.
  #[wasm_bindgen(js_name = "pop")]
  pub fn pop_wasm(&mut self) -> Option<String> {
    self.inner.pop().map(|ch| ch.to_string())
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
  #[wasm_bindgen(js_name = "remainingBytes")]
  pub fn remaining_wasm(&self) -> usize {
    bytes::Buf::remaining(self.inner.as_inner())
  }

  /// Advance the read cursor by `cnt` bytes.
  ///
  /// @throws {Error} If `cnt` exceeds the number of remaining bytes or is not a UTF-8 character boundary.
  #[wasm_bindgen(js_name = "advanceBytes")]
  pub fn advance_wasm(&mut self, cnt: usize) -> Result<(), JsError> {
    validate_char_boundary(self.inner.as_str(), cnt).map_err(|e| JsError::new(&e.to_string()))?;
    self
      .inner
      .inner
      .try_advance(cnt)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Clear this visible UTF-8 view.
  #[wasm_bindgen(js_name = "clear")]
  pub fn clear_wasm(&mut self) {
    self.inner.clear();
  }

  /// Truncate this view at a UTF-8 byte boundary.
  #[wasm_bindgen(js_name = "truncateBytes")]
  pub fn truncate_wasm(&mut self, new_len: usize) -> Result<(), JsError> {
    self
      .inner
      .try_truncate(new_len)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Split the buffer at position `at`, returning bytes `[0, at)`.
  /// Self becomes `[at, len)`.
  ///
  /// @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
  #[wasm_bindgen(js_name = "splitToBytes")]
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
  #[wasm_bindgen(js_name = "splitOffBytes")]
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
  #[wasm_bindgen(js_name = "sliceBytes")]
  pub fn slice_wasm(&self, start: usize, end: usize) -> Result<WasmCompactUtf8Bytes, JsError> {
    self
      .inner
      .try_slice(start..end)
      .map(Into::into)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Return a cheap strategy-preserving clone of this immutable view.
  #[wasm_bindgen(js_name = "clone")]
  pub fn clone_wasm(&self) -> WasmCompactUtf8Bytes {
    WasmCompactUtf8Bytes {
      inner: self.inner.clone(),
    }
  }

  /// Return an independent mutable copy of this view.
  #[wasm_bindgen(js_name = "toMutable")]
  pub fn to_mutable_wasm(&self) -> crate::Utf8BytesMut {
    crate::Utf8BytesMut::from(self.inner.as_str())
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
