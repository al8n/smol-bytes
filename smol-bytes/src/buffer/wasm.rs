use wasm_bindgen::prelude::*;

use crate::wasm_iter::ByteIterator;

use super::Buffer;

#[allow(missing_docs)]
#[wasm_bindgen]
impl Buffer {
  /// Create a new empty Buffer.
  #[wasm_bindgen(constructor)]
  pub fn new_wasm() -> Self {
    Self::new()
  }

  /// Create a Buffer from a byte array.
  ///
  /// @param data - The source bytes to copy.
  /// @throws {Error} If the data exceeds inline capacity (62 bytes).
  #[wasm_bindgen(js_name = "fromBytes")]
  pub fn from_bytes_wasm(data: &[u8]) -> Result<Buffer, JsError> {
    Buffer::try_from(data).map_err(|e| JsError::new(&e.to_string()))
  }

  /// Create a Buffer from a UTF-8 string.
  ///
  /// @param s - The source string whose bytes are copied.
  /// @throws {Error} If the encoded bytes exceed inline capacity (62 bytes).
  #[wasm_bindgen(js_name = "fromUtf8")]
  pub fn from_utf8_wasm(s: js_sys::JsString) -> Result<Buffer, JsError> {
    let s = crate::wasm::js_string_to_string(&s)?;
    Buffer::try_from(s.as_bytes()).map_err(|e| JsError::new(&e.to_string()))
  }

  /// Return contents as a `Uint8Array` (copy).
  #[wasm_bindgen(js_name = "toBytes")]
  pub fn to_bytes_wasm(&self) -> Vec<u8> {
    self.as_slice().to_vec()
  }

  /// Return contents as a UTF-8 string.
  ///
  /// @throws {Error} If the buffer contains invalid UTF-8.
  #[wasm_bindgen(js_name = "decodeUtf8")]
  pub fn decode_utf8_wasm(&self) -> Result<String, JsError> {
    core::str::from_utf8(self.as_slice())
      .map(|s| s.to_string())
      .map_err(|e| JsError::new(&e.to_string()))
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

  /// Return the fixed inline capacity in bytes.
  #[wasm_bindgen(js_name = "capacity")]
  pub fn capacity_wasm(&self) -> usize {
    self.capacity()
  }

  /// Return the remaining writable capacity in bytes.
  #[wasm_bindgen(js_name = "remainingMut")]
  pub fn remaining_mut_wasm(&self) -> usize {
    self.remaining_mut()
  }

  /// Return the number of readable bytes remaining.
  #[wasm_bindgen(js_name = "remaining")]
  pub fn remaining_wasm(&self) -> usize {
    self.remaining()
  }

  /// Advance the read cursor by `cnt` bytes.
  ///
  /// @throws {Error} If `cnt` exceeds the number of remaining bytes.
  #[wasm_bindgen(js_name = "advance")]
  pub fn advance_wasm(&mut self, cnt: usize) -> Result<(), JsError> {
    self
      .try_advance(cnt)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Clear the buffer, removing all data and resetting the cursor.
  #[wasm_bindgen(js_name = "clear")]
  pub fn clear_wasm(&mut self) {
    self.clear();
  }

  /// Resize the buffer, filling new bytes with `value` if growing.
  #[wasm_bindgen(js_name = "resize")]
  pub fn resize_wasm(&mut self, new_len: usize, value: u8) -> Result<(), JsError> {
    self
      .try_resize(new_len, value)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
  #[wasm_bindgen(js_name = "truncate")]
  pub fn truncate_wasm(&mut self, new_len: usize) {
    self.truncate(new_len);
  }

  /// Split the buffer at position `at`, returning bytes `[0, at)`.
  /// Self becomes `[at, len)`.
  ///
  /// @throws {Error} If `at` is out of bounds.
  #[wasm_bindgen(js_name = "splitTo")]
  pub fn split_to_wasm(&mut self, at: usize) -> Result<Buffer, JsError> {
    self
      .try_split_to(at)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Split the buffer at position `at`, returning bytes `[at, len)`.
  /// Self becomes `[0, at)`.
  ///
  /// @throws {Error} If `at` is out of bounds.
  #[wasm_bindgen(js_name = "splitOff")]
  pub fn split_off_wasm(&mut self, at: usize) -> Result<Buffer, JsError> {
    self
      .try_split_off(at)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Return a copy of bytes in range `[start, end)`.
  ///
  /// @throws {Error} If the range is out of bounds.
  #[wasm_bindgen(js_name = "slice")]
  pub fn slice_wasm(&self, start: usize, end: usize) -> Result<Buffer, JsError> {
    self
      .try_slice(start..end)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Return an independent copy of this buffer.
  #[wasm_bindgen(js_name = "clone")]
  pub fn clone_wasm(&self) -> Buffer {
    *self
  }

  // -- Buf getters --

  /// Read an unsigned 8-bit integer, advancing the cursor by 1 byte.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU8")]
  pub fn get_u8_wasm(&mut self) -> Result<u8, JsError> {
    if self.remaining() < 1 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u8(self))
  }

  /// Read a signed 8-bit integer, advancing the cursor by 1 byte.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI8")]
  pub fn get_i8_wasm(&mut self) -> Result<i8, JsError> {
    if self.remaining() < 1 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i8(self))
  }

  /// Read an unsigned 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU16")]
  pub fn get_u16_wasm(&mut self) -> Result<u16, JsError> {
    if self.remaining() < 2 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u16(self))
  }

  /// Read an unsigned 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU16Le")]
  pub fn get_u16_le_wasm(&mut self) -> Result<u16, JsError> {
    if self.remaining() < 2 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u16_le(self))
  }

  /// Read a signed 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI16")]
  pub fn get_i16_wasm(&mut self) -> Result<i16, JsError> {
    if self.remaining() < 2 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i16(self))
  }

  /// Read a signed 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI16Le")]
  pub fn get_i16_le_wasm(&mut self) -> Result<i16, JsError> {
    if self.remaining() < 2 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i16_le(self))
  }

  /// Read an unsigned 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU32")]
  pub fn get_u32_wasm(&mut self) -> Result<u32, JsError> {
    if self.remaining() < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u32(self))
  }

  /// Read an unsigned 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU32Le")]
  pub fn get_u32_le_wasm(&mut self) -> Result<u32, JsError> {
    if self.remaining() < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u32_le(self))
  }

  /// Read a signed 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI32")]
  pub fn get_i32_wasm(&mut self) -> Result<i32, JsError> {
    if self.remaining() < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i32(self))
  }

  /// Read a signed 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI32Le")]
  pub fn get_i32_le_wasm(&mut self) -> Result<i32, JsError> {
    if self.remaining() < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i32_le(self))
  }

  /// Read an unsigned 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU64")]
  pub fn get_u64_wasm(&mut self) -> Result<u64, JsError> {
    if self.remaining() < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u64(self))
  }

  /// Read an unsigned 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU64Le")]
  pub fn get_u64_le_wasm(&mut self) -> Result<u64, JsError> {
    if self.remaining() < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u64_le(self))
  }

  /// Read a signed 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI64")]
  pub fn get_i64_wasm(&mut self) -> Result<i64, JsError> {
    if self.remaining() < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i64(self))
  }

  /// Read a signed 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI64Le")]
  pub fn get_i64_le_wasm(&mut self) -> Result<i64, JsError> {
    if self.remaining() < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i64_le(self))
  }

  /// Read a 32-bit float in big-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getF32")]
  pub fn get_f32_wasm(&mut self) -> Result<f32, JsError> {
    if self.remaining() < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_f32(self))
  }

  /// Read a 32-bit float in little-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getF32Le")]
  pub fn get_f32_le_wasm(&mut self) -> Result<f32, JsError> {
    if self.remaining() < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_f32_le(self))
  }

  /// Read a 64-bit float in big-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getF64")]
  pub fn get_f64_wasm(&mut self) -> Result<f64, JsError> {
    if self.remaining() < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_f64(self))
  }

  /// Read a 64-bit float in little-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getF64Le")]
  pub fn get_f64_le_wasm(&mut self) -> Result<f64, JsError> {
    if self.remaining() < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_f64_le(self))
  }

  /// Read an unsigned integer encoded in `width` bytes, big-endian.
  #[wasm_bindgen(js_name = "getUint")]
  pub fn get_uint_wasm(&mut self, width: usize) -> Result<u64, JsError> {
    self
      .try_get_uint(width)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Read an unsigned integer encoded in `width` bytes, little-endian.
  #[wasm_bindgen(js_name = "getUintLe")]
  pub fn get_uint_le_wasm(&mut self, width: usize) -> Result<u64, JsError> {
    self
      .try_get_uint_le(width)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Read an unsigned integer encoded in `width` bytes, native-endian.
  #[wasm_bindgen(js_name = "getUintNe")]
  pub fn get_uint_ne_wasm(&mut self, width: usize) -> Result<u64, JsError> {
    self
      .try_get_uint_ne(width)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Read a signed integer encoded in `width` bytes, big-endian.
  #[wasm_bindgen(js_name = "getInt")]
  pub fn get_int_wasm(&mut self, width: usize) -> Result<i64, JsError> {
    self
      .try_get_int(width)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Read a signed integer encoded in `width` bytes, little-endian.
  #[wasm_bindgen(js_name = "getIntLe")]
  pub fn get_int_le_wasm(&mut self, width: usize) -> Result<i64, JsError> {
    self
      .try_get_int_le(width)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Read a signed integer encoded in `width` bytes, native-endian.
  #[wasm_bindgen(js_name = "getIntNe")]
  pub fn get_int_ne_wasm(&mut self, width: usize) -> Result<i64, JsError> {
    self
      .try_get_int_ne(width)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  // -- BufMut putters --

  /// Write a byte slice into the buffer.
  ///
  /// @param data - The bytes to append.
  /// @throws {Error} If the data would exceed inline capacity (62 bytes).
  #[wasm_bindgen(js_name = "putSlice")]
  pub fn put_slice_wasm(&mut self, data: &[u8]) -> Result<(), JsError> {
    self
      .try_put_slice(data)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Append `count` copies of `value`.
  #[wasm_bindgen(js_name = "putBytes")]
  pub fn put_bytes_wasm(&mut self, value: u8, count: usize) -> Result<(), JsError> {
    self
      .try_put_bytes(value, count)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write an unsigned `width`-byte integer in big-endian byte order.
  #[wasm_bindgen(js_name = "putUint")]
  pub fn put_uint_wasm(&mut self, value: u64, width: usize) -> Result<(), JsError> {
    self
      .try_put_uint(value, width)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write an unsigned `width`-byte integer in little-endian byte order.
  #[wasm_bindgen(js_name = "putUintLe")]
  pub fn put_uint_le_wasm(&mut self, value: u64, width: usize) -> Result<(), JsError> {
    self
      .try_put_uint_le(value, width)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write an unsigned `width`-byte integer in native-endian byte order.
  #[wasm_bindgen(js_name = "putUintNe")]
  pub fn put_uint_ne_wasm(&mut self, value: u64, width: usize) -> Result<(), JsError> {
    self
      .try_put_uint_ne(value, width)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write a signed `width`-byte integer in big-endian byte order.
  #[wasm_bindgen(js_name = "putInt")]
  pub fn put_int_wasm(&mut self, value: i64, width: usize) -> Result<(), JsError> {
    self
      .try_put_int(value, width)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write a signed `width`-byte integer in little-endian byte order.
  #[wasm_bindgen(js_name = "putIntLe")]
  pub fn put_int_le_wasm(&mut self, value: i64, width: usize) -> Result<(), JsError> {
    self
      .try_put_int_le(value, width)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write a signed `width`-byte integer in native-endian byte order.
  #[wasm_bindgen(js_name = "putIntNe")]
  pub fn put_int_ne_wasm(&mut self, value: i64, width: usize) -> Result<(), JsError> {
    self
      .try_put_int_ne(value, width)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write an unsigned 8-bit integer.
  #[wasm_bindgen(js_name = "putU8")]
  pub fn put_u8_wasm(&mut self, val: u8) -> Result<(), JsError> {
    self
      .try_put_u8(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write a signed 8-bit integer.
  #[wasm_bindgen(js_name = "putI8")]
  pub fn put_i8_wasm(&mut self, val: i8) -> Result<(), JsError> {
    self
      .try_put_i8(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write an unsigned 16-bit integer in big-endian byte order.
  #[wasm_bindgen(js_name = "putU16")]
  pub fn put_u16_wasm(&mut self, val: u16) -> Result<(), JsError> {
    self
      .try_put_u16(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write an unsigned 16-bit integer in little-endian byte order.
  #[wasm_bindgen(js_name = "putU16Le")]
  pub fn put_u16_le_wasm(&mut self, val: u16) -> Result<(), JsError> {
    self
      .try_put_u16_le(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write a signed 16-bit integer in big-endian byte order.
  #[wasm_bindgen(js_name = "putI16")]
  pub fn put_i16_wasm(&mut self, val: i16) -> Result<(), JsError> {
    self
      .try_put_i16(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write a signed 16-bit integer in little-endian byte order.
  #[wasm_bindgen(js_name = "putI16Le")]
  pub fn put_i16_le_wasm(&mut self, val: i16) -> Result<(), JsError> {
    self
      .try_put_i16_le(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write an unsigned 32-bit integer in big-endian byte order.
  #[wasm_bindgen(js_name = "putU32")]
  pub fn put_u32_wasm(&mut self, val: u32) -> Result<(), JsError> {
    self
      .try_put_u32(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write an unsigned 32-bit integer in little-endian byte order.
  #[wasm_bindgen(js_name = "putU32Le")]
  pub fn put_u32_le_wasm(&mut self, val: u32) -> Result<(), JsError> {
    self
      .try_put_u32_le(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write a signed 32-bit integer in big-endian byte order.
  #[wasm_bindgen(js_name = "putI32")]
  pub fn put_i32_wasm(&mut self, val: i32) -> Result<(), JsError> {
    self
      .try_put_i32(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write a signed 32-bit integer in little-endian byte order.
  #[wasm_bindgen(js_name = "putI32Le")]
  pub fn put_i32_le_wasm(&mut self, val: i32) -> Result<(), JsError> {
    self
      .try_put_i32_le(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write an unsigned 64-bit integer in big-endian byte order.
  #[wasm_bindgen(js_name = "putU64")]
  pub fn put_u64_wasm(&mut self, val: u64) -> Result<(), JsError> {
    self
      .try_put_u64(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write an unsigned 64-bit integer in little-endian byte order.
  #[wasm_bindgen(js_name = "putU64Le")]
  pub fn put_u64_le_wasm(&mut self, val: u64) -> Result<(), JsError> {
    self
      .try_put_u64_le(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write a signed 64-bit integer in big-endian byte order.
  #[wasm_bindgen(js_name = "putI64")]
  pub fn put_i64_wasm(&mut self, val: i64) -> Result<(), JsError> {
    self
      .try_put_i64(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write a signed 64-bit integer in little-endian byte order.
  #[wasm_bindgen(js_name = "putI64Le")]
  pub fn put_i64_le_wasm(&mut self, val: i64) -> Result<(), JsError> {
    self
      .try_put_i64_le(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write a 32-bit float in big-endian byte order.
  #[wasm_bindgen(js_name = "putF32")]
  pub fn put_f32_wasm(&mut self, val: f32) -> Result<(), JsError> {
    self
      .try_put_f32(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write a 32-bit float in little-endian byte order.
  #[wasm_bindgen(js_name = "putF32Le")]
  pub fn put_f32_le_wasm(&mut self, val: f32) -> Result<(), JsError> {
    self
      .try_put_f32_le(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write a 64-bit float in big-endian byte order.
  #[wasm_bindgen(js_name = "putF64")]
  pub fn put_f64_wasm(&mut self, val: f64) -> Result<(), JsError> {
    self
      .try_put_f64(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Write a 64-bit float in little-endian byte order.
  #[wasm_bindgen(js_name = "putF64Le")]
  pub fn put_f64_le_wasm(&mut self, val: f64) -> Result<(), JsError> {
    self
      .try_put_f64_le(val)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  // -- Iterator --

  /// Return a JS-compatible byte iterator.
  ///
  /// Use `for (const b of buf)` after attaching `Symbol.iterator`.
  #[wasm_bindgen(js_name = "iter")]
  pub fn iter_wasm(&self) -> ByteIterator {
    ByteIterator::new(self.as_slice().to_vec())
  }
}
