use wasm_bindgen::prelude::*;

use crate::wasm_iter::ByteIterator;

/// Immutable byte buffer using the Shared strategy.
///
/// Stores up to 62 bytes inline with zero-copy reference-counted cloning for larger data.
/// Available as `SharedBytes` via `import { SharedBytes } from 'smol-bytes/shared'`.
#[allow(missing_docs)]
#[wasm_bindgen(js_name = "SharedBytes")]
pub struct WasmSharedBytes {
  inner: super::Bytes,
}

impl WasmSharedBytes {
  pub(crate) fn from_inner(inner: super::Bytes) -> Self {
    Self { inner }
  }
}

#[allow(missing_docs)]
#[wasm_bindgen(js_class = "SharedBytes")]
impl WasmSharedBytes {
  /// Create a new empty `SharedBytes`.
  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    Self {
      inner: super::Bytes::new(),
    }
  }

  /// Create a `SharedBytes` from a byte array.
  ///
  /// @param data - The source bytes to copy.
  #[wasm_bindgen(js_name = "fromBytes")]
  pub fn from_bytes(data: &[u8]) -> Self {
    Self {
      inner: super::Bytes::copy_from_slice(data),
    }
  }

  /// Create a `SharedBytes` from a UTF-8 string.
  ///
  /// @param s - The source string whose bytes are copied.
  #[wasm_bindgen(js_name = "fromUtf8")]
  pub fn from_utf8(s: js_sys::JsString) -> Result<Self, JsError> {
    let s = crate::wasm::js_string_to_string(&s)?;
    Ok(Self {
      inner: super::Bytes::from(s),
    })
  }

  /// Return contents as a `Uint8Array` (copy).
  #[wasm_bindgen(js_name = "toBytes")]
  pub fn to_bytes(&self) -> Vec<u8> {
    self.inner.as_slice().to_vec()
  }

  /// Return contents as a UTF-8 string.
  ///
  /// @throws {Error} If the buffer contains invalid UTF-8.
  #[wasm_bindgen(js_name = "decodeUtf8")]
  pub fn decode_utf8_wasm(&self) -> Result<String, JsError> {
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
  pub fn split_to(&mut self, at: usize) -> Result<WasmSharedBytes, JsError> {
    self
      .inner
      .try_split_to(at)
      .map(|b| WasmSharedBytes { inner: b })
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Split the buffer at position `at`, returning bytes `[at, len)`.
  /// Self becomes `[0, at)`.
  ///
  /// @throws {Error} If `at` is out of bounds.
  #[wasm_bindgen(js_name = "splitOff")]
  pub fn split_off(&mut self, at: usize) -> Result<WasmSharedBytes, JsError> {
    self
      .inner
      .try_split_off(at)
      .map(|b| WasmSharedBytes { inner: b })
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Return a copy of bytes in range `[start, end)`.
  ///
  /// @throws {Error} If the range is out of bounds.
  #[wasm_bindgen(js_name = "slice")]
  pub fn slice(&self, start: usize, end: usize) -> Result<WasmSharedBytes, JsError> {
    self
      .inner
      .try_slice(start..end)
      .map(|b| WasmSharedBytes { inner: b })
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Return a cheap strategy-preserving immutable clone.
  #[wasm_bindgen(js_name = "clone")]
  pub fn clone_wasm(&self) -> WasmSharedBytes {
    WasmSharedBytes {
      inner: self.inner.clone(),
    }
  }

  /// Return an independent mutable copy of these bytes.
  #[wasm_bindgen(js_name = "toMutable")]
  pub fn to_mutable_wasm(&self) -> crate::BytesMut {
    crate::BytesMut::from(self.inner.as_slice())
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

  /// Read an unsigned integer encoded in `width` bytes, big-endian.
  #[wasm_bindgen(js_name = "getUint")]
  pub fn get_uint(&mut self, width: usize) -> Result<u64, JsError> {
    bytes::Buf::try_get_uint(&mut self.inner, width).map_err(|e| JsError::new(&e.to_string()))
  }

  /// Read an unsigned integer encoded in `width` bytes, little-endian.
  #[wasm_bindgen(js_name = "getUintLe")]
  pub fn get_uint_le(&mut self, width: usize) -> Result<u64, JsError> {
    bytes::Buf::try_get_uint_le(&mut self.inner, width).map_err(|e| JsError::new(&e.to_string()))
  }

  /// Read an unsigned integer encoded in `width` bytes, native-endian.
  #[wasm_bindgen(js_name = "getUintNe")]
  pub fn get_uint_ne(&mut self, width: usize) -> Result<u64, JsError> {
    bytes::Buf::try_get_uint_ne(&mut self.inner, width).map_err(|e| JsError::new(&e.to_string()))
  }

  /// Read a signed integer encoded in `width` bytes, big-endian.
  #[wasm_bindgen(js_name = "getInt")]
  pub fn get_int(&mut self, width: usize) -> Result<i64, JsError> {
    bytes::Buf::try_get_int(&mut self.inner, width).map_err(|e| JsError::new(&e.to_string()))
  }

  /// Read a signed integer encoded in `width` bytes, little-endian.
  #[wasm_bindgen(js_name = "getIntLe")]
  pub fn get_int_le(&mut self, width: usize) -> Result<i64, JsError> {
    bytes::Buf::try_get_int_le(&mut self.inner, width).map_err(|e| JsError::new(&e.to_string()))
  }

  /// Read a signed integer encoded in `width` bytes, native-endian.
  #[wasm_bindgen(js_name = "getIntNe")]
  pub fn get_int_ne(&mut self, width: usize) -> Result<i64, JsError> {
    bytes::Buf::try_get_int_ne(&mut self.inner, width).map_err(|e| JsError::new(&e.to_string()))
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
