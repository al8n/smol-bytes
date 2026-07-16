use wasm_bindgen::prelude::*;

use crate::wasm_iter::CharIterator;

use super::Utf8Buffer;

#[allow(missing_docs)]
#[wasm_bindgen]
impl Utf8Buffer {
  /// Create a new empty `Utf8Buffer`.
  #[wasm_bindgen(constructor)]
  pub fn new_wasm() -> Self {
    Self::new()
  }

  /// Create a `Utf8Buffer` from a UTF-8 string.
  ///
  /// @param s - The source string to copy.
  /// @throws {Error} If the encoded bytes exceed inline capacity (62 bytes).
  #[wasm_bindgen(js_name = "fromString")]
  pub fn from_string_wasm(s: &str) -> Result<Utf8Buffer, JsError> {
    Utf8Buffer::try_from_str(s).map_err(|e| JsError::new(&e.to_string()))
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

  /// Return the number of readable bytes remaining.
  #[wasm_bindgen(js_name = "remaining")]
  pub fn remaining_wasm(&self) -> usize {
    self.remaining()
  }

  /// Return the total inline capacity in bytes.
  #[wasm_bindgen(js_name = "capacity")]
  pub fn capacity_wasm(&self) -> usize {
    self.capacity()
  }

  /// Append a single character.
  ///
  /// @param ch - A string containing exactly one Unicode scalar value.
  /// @throws {Error} If `ch` is not exactly one scalar value, contains an
  /// unpaired surrogate, or the character would exceed inline capacity.
  #[wasm_bindgen(js_name = "push")]
  pub fn push_wasm(&mut self, ch: js_sys::JsString) -> Result<(), JsError> {
    let c = crate::wasm::js_string_to_char(&ch)?;
    self.try_push(c).map_err(|e| JsError::new(&e.to_string()))
  }

  /// Append a string.
  ///
  /// @param s - The string to append.
  /// @throws {Error} If the combined length would exceed inline capacity.
  #[wasm_bindgen(js_name = "pushStr")]
  pub fn push_str_wasm(&mut self, s: &str) -> Result<(), JsError> {
    self
      .try_push_str(s)
      .map_err(|e| JsError::new(&e.to_string()))
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

  /// Split the buffer at position `at`, returning bytes `[0, at)`.
  /// Self becomes `[at, len)`.
  ///
  /// @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
  #[wasm_bindgen(js_name = "splitTo")]
  pub fn split_to_wasm(&mut self, at: usize) -> Result<Utf8Buffer, JsError> {
    self
      .try_split_to(at)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Split the buffer at position `at`, returning bytes `[at, len)`.
  /// Self becomes `[0, at)`.
  ///
  /// @throws {Error} If `at` is out of bounds or not on a UTF-8 char boundary.
  #[wasm_bindgen(js_name = "splitOff")]
  pub fn split_off_wasm(&mut self, at: usize) -> Result<Utf8Buffer, JsError> {
    self
      .try_split_off(at)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Return a copy of bytes in range `[start, end)`.
  ///
  /// @throws {Error} If the range is out of bounds or not on UTF-8 char boundaries.
  #[wasm_bindgen(js_name = "slice")]
  pub fn slice_wasm(&self, start: usize, end: usize) -> Result<Utf8Buffer, JsError> {
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
