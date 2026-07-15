use wasm_bindgen::prelude::*;

use crate::wasm_iter::ByteIterator;

/// Immutable byte buffer using the Compact strategy.
///
/// Stores up to 62 bytes inline with unique ownership for larger data (no reference counting).
/// Available as `CompactBytes` via `import { CompactBytes } from 'smol-bytes/compact'`.
#[allow(missing_docs)]
#[wasm_bindgen(js_name = "CompactBytes")]
pub struct WasmCompactBytes {
  inner: super::Bytes,
}

#[allow(missing_docs)]
#[wasm_bindgen(js_class = "CompactBytes")]
impl WasmCompactBytes {
  /// Create a new empty `CompactBytes`.
  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    Self {
      inner: super::Bytes::new(),
    }
  }

  /// Create a `CompactBytes` from a byte array.
  ///
  /// @param data - The source bytes to copy.
  #[wasm_bindgen(js_name = "fromBytes")]
  pub fn from_bytes(data: &[u8]) -> Self {
    Self {
      inner: super::Bytes::copy_from_slice(data),
    }
  }

  /// Create a `CompactBytes` from a UTF-8 string.
  ///
  /// @param s - The source string whose bytes are copied.
  #[wasm_bindgen(js_name = "fromString")]
  pub fn from_string(s: &str) -> Self {
    Self {
      inner: super::Bytes::from(s),
    }
  }

  /// Create a `CompactBytes` from a static string.
  ///
  /// Note: In WASM, this copies the data (true static references cannot cross the JS boundary).
  #[wasm_bindgen(js_name = "fromStatic")]
  pub fn from_static_wasm(s: &str) -> Self {
    // NOTE: we cannot truly take a &'static str from JS, so we copy.
    Self {
      inner: super::Bytes::copy_from_slice(s.as_bytes()),
    }
  }

  /// Return contents as a `Uint8Array` (copy).
  #[wasm_bindgen(js_name = "toBytes")]
  pub fn to_bytes(&self) -> Vec<u8> {
    self.inner.as_slice().to_vec()
  }

  /// Return contents as a UTF-8 string.
  ///
  /// @throws {Error} If the buffer contains invalid UTF-8.
  #[wasm_bindgen(js_name = "toString")]
  pub fn to_string_wasm(&self) -> Result<String, JsError> {
    core::str::from_utf8(self.inner.as_slice())
      .map(|s| s.to_string())
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Return the byte length of the buffer.
  #[wasm_bindgen(js_name = "len")]
  pub fn len(&self) -> usize {
    self.inner.len()
  }

  /// Return `true` if the buffer has no bytes.
  #[wasm_bindgen(js_name = "isEmpty")]
  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  /// Return `true` if data is stored inline (no heap allocation).
  #[wasm_bindgen(js_name = "isInline")]
  pub fn is_inline(&self) -> bool {
    self.inner.is_inline()
  }

  /// Return `true` if data is stored on the heap.
  #[wasm_bindgen(js_name = "isHeap")]
  pub fn is_heap(&self) -> bool {
    self.inner.is_heap()
  }

  /// Return the number of readable bytes remaining.
  #[wasm_bindgen(js_name = "remaining")]
  pub fn remaining(&self) -> usize {
    bytes::Buf::remaining(&self.inner)
  }

  /// Advance the read cursor by `cnt` bytes.
  ///
  /// @throws {Error} If `cnt` exceeds the number of remaining bytes.
  #[wasm_bindgen(js_name = "advance")]
  pub fn advance(&mut self, cnt: usize) -> Result<(), JsError> {
    self
      .inner
      .try_advance(cnt)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Clear the buffer, removing all data and resetting the cursor.
  #[wasm_bindgen(js_name = "clear")]
  pub fn clear(&mut self) {
    self.inner.clear();
  }

  /// Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
  #[wasm_bindgen(js_name = "truncate")]
  pub fn truncate(&mut self, new_len: usize) {
    self.inner.truncate(new_len);
  }

  /// Split the buffer at position `at`, returning bytes `[0, at)`.
  /// Self becomes `[at, len)`.
  ///
  /// @throws {Error} If `at` is out of bounds.
  #[wasm_bindgen(js_name = "splitTo")]
  pub fn split_to(&mut self, at: usize) -> Result<WasmCompactBytes, JsError> {
    self
      .inner
      .try_split_to(at)
      .map(|b| WasmCompactBytes { inner: b })
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Split the buffer at position `at`, returning bytes `[at, len)`.
  /// Self becomes `[0, at)`.
  ///
  /// @throws {Error} If `at` is out of bounds.
  #[wasm_bindgen(js_name = "splitOff")]
  pub fn split_off(&mut self, at: usize) -> Result<WasmCompactBytes, JsError> {
    self
      .inner
      .try_split_off(at)
      .map(|b| WasmCompactBytes { inner: b })
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Return a copy of bytes in range `[start, end)`.
  ///
  /// @throws {Error} If the range is out of bounds.
  #[wasm_bindgen(js_name = "slice")]
  pub fn slice(&self, start: usize, end: usize) -> Result<WasmCompactBytes, JsError> {
    self
      .inner
      .try_slice(start..end)
      .map(|b| WasmCompactBytes { inner: b })
      .map_err(|e| JsError::new(&e.to_string()))
  }

  // -- Buf getters --

  /// Read an unsigned 8-bit integer, advancing the cursor by 1 byte.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU8")]
  pub fn get_u8(&mut self) -> Result<u8, JsError> {
    if bytes::Buf::remaining(&self.inner) < 1 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u8(&mut self.inner))
  }

  /// Read a signed 8-bit integer, advancing the cursor by 1 byte.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI8")]
  pub fn get_i8(&mut self) -> Result<i8, JsError> {
    if bytes::Buf::remaining(&self.inner) < 1 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i8(&mut self.inner))
  }

  /// Read an unsigned 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU16")]
  pub fn get_u16(&mut self) -> Result<u16, JsError> {
    if bytes::Buf::remaining(&self.inner) < 2 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u16(&mut self.inner))
  }

  /// Read an unsigned 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU16Le")]
  pub fn get_u16_le(&mut self) -> Result<u16, JsError> {
    if bytes::Buf::remaining(&self.inner) < 2 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u16_le(&mut self.inner))
  }

  /// Read a signed 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI16")]
  pub fn get_i16(&mut self) -> Result<i16, JsError> {
    if bytes::Buf::remaining(&self.inner) < 2 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i16(&mut self.inner))
  }

  /// Read a signed 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI16Le")]
  pub fn get_i16_le(&mut self) -> Result<i16, JsError> {
    if bytes::Buf::remaining(&self.inner) < 2 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i16_le(&mut self.inner))
  }

  /// Read an unsigned 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU32")]
  pub fn get_u32(&mut self) -> Result<u32, JsError> {
    if bytes::Buf::remaining(&self.inner) < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u32(&mut self.inner))
  }

  /// Read an unsigned 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU32Le")]
  pub fn get_u32_le(&mut self) -> Result<u32, JsError> {
    if bytes::Buf::remaining(&self.inner) < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u32_le(&mut self.inner))
  }

  /// Read a signed 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI32")]
  pub fn get_i32(&mut self) -> Result<i32, JsError> {
    if bytes::Buf::remaining(&self.inner) < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i32(&mut self.inner))
  }

  /// Read a signed 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI32Le")]
  pub fn get_i32_le(&mut self) -> Result<i32, JsError> {
    if bytes::Buf::remaining(&self.inner) < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i32_le(&mut self.inner))
  }

  /// Read an unsigned 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU64")]
  pub fn get_u64(&mut self) -> Result<u64, JsError> {
    if bytes::Buf::remaining(&self.inner) < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u64(&mut self.inner))
  }

  /// Read an unsigned 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU64Le")]
  pub fn get_u64_le(&mut self) -> Result<u64, JsError> {
    if bytes::Buf::remaining(&self.inner) < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u64_le(&mut self.inner))
  }

  /// Read a signed 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI64")]
  pub fn get_i64(&mut self) -> Result<i64, JsError> {
    if bytes::Buf::remaining(&self.inner) < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i64(&mut self.inner))
  }

  /// Read a signed 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI64Le")]
  pub fn get_i64_le(&mut self) -> Result<i64, JsError> {
    if bytes::Buf::remaining(&self.inner) < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i64_le(&mut self.inner))
  }

  /// Read a 32-bit float in big-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getF32")]
  pub fn get_f32(&mut self) -> Result<f32, JsError> {
    if bytes::Buf::remaining(&self.inner) < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_f32(&mut self.inner))
  }

  /// Read a 32-bit float in little-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getF32Le")]
  pub fn get_f32_le(&mut self) -> Result<f32, JsError> {
    if bytes::Buf::remaining(&self.inner) < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_f32_le(&mut self.inner))
  }

  /// Read a 64-bit float in big-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getF64")]
  pub fn get_f64(&mut self) -> Result<f64, JsError> {
    if bytes::Buf::remaining(&self.inner) < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_f64(&mut self.inner))
  }

  /// Read a 64-bit float in little-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getF64Le")]
  pub fn get_f64_le(&mut self) -> Result<f64, JsError> {
    if bytes::Buf::remaining(&self.inner) < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_f64_le(&mut self.inner))
  }

  // -- Iterator --

  /// Return a JS-compatible byte iterator.
  ///
  /// Use `for (const b of buf)` after attaching `Symbol.iterator`.
  #[wasm_bindgen(js_name = "iter")]
  pub fn iter(&self) -> ByteIterator {
    ByteIterator::new(self.inner.as_slice().to_vec())
  }
}
