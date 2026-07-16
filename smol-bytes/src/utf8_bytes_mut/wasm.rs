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
  pub fn from_string_wasm(s: &str) -> Utf8BytesMut {
    Utf8BytesMut::from(s)
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
  #[wasm_bindgen(js_name = "len")]
  pub fn len_wasm(&self) -> usize {
    self.len()
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
  pub fn push_str_wasm(&mut self, s: &str) {
    self.push_str(s);
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
  #[wasm_bindgen(js_name = "truncate")]
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
  #[wasm_bindgen(js_name = "splitTo")]
  pub fn split_to_wasm(&mut self, at: usize) -> Result<Utf8BytesMut, JsError> {
    self
      .try_split_to(at)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Split the buffer at position `at`, returning bytes `[at, len)`.
  /// Self becomes `[0, at)`.
  ///
  /// @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
  #[wasm_bindgen(js_name = "splitOff")]
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

  /// Attempt to re-merge a previously split buffer back into this one.
  ///
  /// Returns `other` unchanged if the two buffers are not contiguous.
  #[wasm_bindgen(js_name = "unsplit")]
  pub fn unsplit_wasm(&mut self, other: Utf8BytesMut) -> Option<Utf8BytesMut> {
    self.unsplit(other)
  }

  /// Return a copy of bytes in range `[start, end)`.
  ///
  /// @throws {Error} If the range is out of bounds or not on UTF-8 char boundaries.
  #[wasm_bindgen(js_name = "slice")]
  pub fn slice_wasm(&self, start: usize, end: usize) -> Result<Utf8BytesMut, JsError> {
    self
      .try_slice(start..end)
      .map_err(|e| JsError::new(&e.to_string()))
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
