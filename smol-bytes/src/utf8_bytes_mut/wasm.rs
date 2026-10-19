use wasm_bindgen::prelude::*;

use crate::wasm_iter::CharIterator;

use super::Utf8BytesMut;

#[allow(missing_docs)]
#[wasm_bindgen]
impl Utf8BytesMut {
  /// Create a new empty `Utf8BytesMut`.
  #[wasm_bindgen(constructor)]
  pub fn new_wasm() -> Self {
    Self::new()
  }

  /// Create a new `Utf8BytesMut` with the given capacity pre-allocated.
  ///
  /// @param capacity - Number of bytes to pre-allocate.
  #[wasm_bindgen(js_name = "withCapacity")]
  pub fn with_capacity_wasm(capacity: usize) -> Self {
    Self::with_capacity(capacity)
  }

  /// Create a `Utf8BytesMut` from a UTF-8 string.
  ///
  /// @param s - The source string whose bytes are copied.
  #[wasm_bindgen(js_name = "fromString")]
  pub fn from_string_wasm(s: js_sys::JsString) -> Result<Utf8BytesMut, JsError> {
    let s = crate::wasm::js_string_to_string(&s)?;
    Ok(Utf8BytesMut::from(s))
  }

  /// Decode bytes as UTF-8, replacing malformed sequences with `U+FFFD`.
  #[wasm_bindgen(js_name = "fromBytesLossy")]
  pub fn from_bytes_lossy_wasm(data: &[u8]) -> Utf8BytesMut {
    Utf8BytesMut::from_utf8_lossy(crate::BytesMut::from(data))
  }

  /// Return contents as a UTF-8 string.
  #[wasm_bindgen(js_name = "toString")]
  pub fn to_string_wasm(&self) -> String {
    self.as_str().to_string()
  }

  /// Return contents as a `Uint8Array` (copy of the UTF-8 encoded bytes).
  #[wasm_bindgen(js_name = "toBytes")]
  pub fn to_bytes_wasm(&self) -> Vec<u8> {
    self.as_str().as_bytes().to_vec()
  }

  /// Return the byte length of the buffer.
  #[wasm_bindgen(js_name = "byteLength")]
  pub fn len_wasm(&self) -> usize {
    self.len()
  }

  /// Return the number of Unicode scalar values in the buffer.
  #[wasm_bindgen(js_name = "charLength")]
  pub fn char_length_wasm(&self) -> usize {
    self.as_str().chars().count()
  }

  /// Return `true` if the buffer has no bytes.
  #[wasm_bindgen(js_name = "isEmpty")]
  pub fn is_empty_wasm(&self) -> bool {
    self.is_empty()
  }

  /// Return the total allocated capacity in bytes.
  #[wasm_bindgen(js_name = "capacity")]
  pub fn capacity_wasm(&self) -> usize {
    self.capacity()
  }

  /// Return `true` if data is stored inline (no heap allocation).
  #[wasm_bindgen(js_name = "isInline")]
  pub fn is_inline_wasm(&self) -> bool {
    self.is_inline()
  }

  /// Return `true` if data is stored on the heap.
  #[wasm_bindgen(js_name = "isHeap")]
  pub fn is_heap_wasm(&self) -> bool {
    self.is_heap()
  }

  /// Copy a small heap value back into inline storage when possible.
  #[wasm_bindgen(js_name = "tryMakeInline")]
  pub fn try_make_inline_wasm(&mut self) -> bool {
    self.try_make_inline()
  }

  /// Return the number of readable UTF-8 bytes remaining.
  #[wasm_bindgen(js_name = "remainingBytes")]
  pub fn remaining_bytes_wasm(&self) -> usize {
    bytes::Buf::remaining(&self.inner)
  }

  /// Advance the read cursor by a UTF-8 byte count at a scalar boundary.
  #[wasm_bindgen(js_name = "advanceBytes")]
  pub fn advance_bytes_wasm(&mut self, count: usize) -> Result<(), JsError> {
    crate::utf8_buf::validate_char_boundary(self.as_str(), count)
      .map_err(|e| JsError::new(&e.to_string()))?;
    self
      .inner
      .try_advance(count)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Append a single character.
  ///
  /// @param ch - A string containing exactly one Unicode scalar value.
  /// @throws {Error} If `ch` is not exactly one scalar value or contains an
  /// unpaired surrogate.
  #[wasm_bindgen(js_name = "push")]
  pub fn push_wasm(&mut self, ch: js_sys::JsString) -> Result<(), JsError> {
    let c = crate::wasm::js_string_to_char(&ch)?;
    self.push(c);
    Ok(())
  }

  /// Append a string.
  ///
  /// @param s - The string to append.
  #[wasm_bindgen(js_name = "pushStr")]
  pub fn push_str_wasm(&mut self, s: js_sys::JsString) -> Result<(), JsError> {
    let s = crate::wasm::js_string_to_string(&s)?;
    self.push_str(&s);
    Ok(())
  }

  /// Insert one Unicode scalar at a UTF-8 byte boundary.
  #[wasm_bindgen(js_name = "insertBytes")]
  pub fn insert_wasm(&mut self, index: usize, ch: js_sys::JsString) -> Result<(), JsError> {
    let ch = crate::wasm::js_string_to_char(&ch)?;
    self
      .try_insert(index, ch)
      .map_err(|error| JsError::new(&error.to_string()))
  }

  /// Insert a string at a UTF-8 byte boundary.
  #[wasm_bindgen(js_name = "insertStrBytes")]
  pub fn insert_str_wasm(&mut self, index: usize, s: js_sys::JsString) -> Result<(), JsError> {
    let s = crate::wasm::js_string_to_string(&s)?;
    self
      .try_insert_str(index, &s)
      .map_err(|error| JsError::new(&error.to_string()))
  }

  /// Replace the UTF-8 byte range `[start, end)` with a string.
  #[wasm_bindgen(js_name = "replaceRangeBytes")]
  pub fn replace_range_wasm(
    &mut self,
    start: usize,
    end: usize,
    replacement: js_sys::JsString,
  ) -> Result<(), JsError> {
    let replacement = crate::wasm::js_string_to_string(&replacement)?;
    self
      .try_replace_range(start..end, &replacement)
      .map_err(|error| JsError::new(&error.to_string()))
  }

  /// Remove and return the final Unicode scalar, if present.
  #[wasm_bindgen(js_name = "pop")]
  pub fn pop_wasm(&mut self) -> Option<String> {
    self.pop().map(|ch| ch.to_string())
  }

  /// Clear the buffer, removing all data.
  #[wasm_bindgen(js_name = "clear")]
  pub fn clear_wasm(&mut self) {
    self.clear();
  }

  /// Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
  ///
  /// Throws if `new_len` is inside a UTF-8 scalar. Lengths beyond the current
  /// byte length leave the value unchanged.
  #[wasm_bindgen(js_name = "truncateBytes")]
  pub fn truncate_wasm(&mut self, new_len: usize) -> Result<(), JsError> {
    self
      .try_truncate(new_len)
      .map_err(|error| JsError::new(&error.to_string()))
  }

  /// Reserve capacity for at least `additional` more bytes.
  #[wasm_bindgen(js_name = "reserve")]
  pub fn reserve_wasm(&mut self, additional: usize) {
    self.reserve(additional);
  }

  /// Split the buffer at position `at`, returning bytes `[0, at)`.
  /// Self becomes `[at, len)`.
  ///
  /// @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
  #[wasm_bindgen(js_name = "splitToBytes")]
  pub fn split_to_wasm(&mut self, at: usize) -> Result<Utf8BytesMut, JsError> {
    self
      .try_split_to(at)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Split the buffer at position `at`, returning bytes `[at, len)`.
  /// Self becomes `[0, at)`.
  ///
  /// @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
  #[wasm_bindgen(js_name = "splitOffBytes")]
  pub fn split_off_wasm(&mut self, at: usize) -> Result<Utf8BytesMut, JsError> {
    self
      .try_split_off(at)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Split all data out of this buffer, returning it. Self becomes empty.
  #[wasm_bindgen(js_name = "split")]
  pub fn split_wasm(&mut self) -> Utf8BytesMut {
    self.split()
  }

  /// Return a copy of bytes in range `[start, end)`.
  ///
  /// @throws {Error} If the range is out of bounds or not on UTF-8 char boundaries.
  #[wasm_bindgen(js_name = "sliceBytes")]
  pub fn slice_wasm(&self, start: usize, end: usize) -> Result<Utf8BytesMut, JsError> {
    self
      .try_slice(start..end)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Return an independent mutable clone.
  #[wasm_bindgen(js_name = "clone")]
  pub fn clone_wasm(&self) -> Utf8BytesMut {
    self.clone()
  }

  /// Freeze the current contents using the shared strategy and reset this buffer.
  #[wasm_bindgen(js_name = "freezeShared")]
  pub fn freeze_shared_wasm(&mut self) -> crate::utf8_bytes::wasm::WasmUtf8Bytes {
    core::mem::take(self).freeze_shared().into()
  }

  /// Freeze the current contents using the compact strategy and reset this buffer.
  #[wasm_bindgen(js_name = "freezeCompact")]
  pub fn freeze_compact_wasm(&mut self) -> crate::utf8_bytes::wasm::WasmCompactUtf8Bytes {
    core::mem::take(self).freeze_compact().into()
  }

  // -- Iterator --

  /// Return a JS-compatible character iterator.
  ///
  /// Use `for (const ch of buf)` after attaching `Symbol.iterator`.
  #[wasm_bindgen(js_name = "iter")]
  pub fn iter_wasm(&self) -> CharIterator {
    CharIterator::new(self.as_str())
  }
}
