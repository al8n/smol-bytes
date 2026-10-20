use wasm_bindgen::prelude::*;

use crate::wasm_iter::ByteIterator;

use super::BytesMut;

#[allow(missing_docs)]
#[wasm_bindgen]
impl BytesMut {
  /// Create a new empty `BytesMut`.
  #[wasm_bindgen(constructor)]
  pub fn new_wasm() -> Self {
    Self::new()
  }

  /// Create a new `BytesMut` with the given capacity pre-allocated.
  ///
  /// @param capacity - Number of bytes to pre-allocate.
  #[wasm_bindgen(js_name = "withCapacity")]
  pub fn with_capacity_wasm(capacity: usize) -> Self {
    Self::with_capacity(capacity)
  }

  /// Create a `BytesMut` containing `len` zero bytes.
  #[wasm_bindgen(js_name = "zeroed")]
  pub fn zeroed_wasm(len: usize) -> Self {
    Self::zeroed(len)
  }

  /// Create a `BytesMut` from a byte array.
  ///
  /// @param data - The source bytes to copy.
  #[wasm_bindgen(js_name = "fromBytes")]
  pub fn from_bytes_wasm(data: &[u8]) -> BytesMut {
    BytesMut::from(data)
  }

  /// Create a `BytesMut` from a UTF-8 string.
  ///
  /// @param s - The source string whose bytes are copied.
  #[wasm_bindgen(js_name = "fromUtf8")]
  pub fn from_utf8_wasm(s: js_sys::JsString) -> Result<BytesMut, JsError> {
    let s = crate::wasm::js_string_to_string(&s)?;
    Ok(BytesMut::from(s.as_bytes()))
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

  /// Return the number of readable bytes remaining.
  #[wasm_bindgen(js_name = "remaining")]
  pub fn remaining_wasm(&self) -> usize {
    bytes::Buf::remaining(self)
  }

  /// Return the growable `bytes::BufMut` writable limit.
  ///
  /// This is `usize::MAX - len`, not the current allocation's spare capacity.
  /// On Wasm it is bounded by the wasm32 `usize` range.
  #[wasm_bindgen(js_name = "remainingMut")]
  pub fn remaining_mut_wasm(&self) -> usize {
    bytes::BufMut::remaining_mut(self)
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

  /// Truncate the buffer to `new_len` bytes, discarding any data beyond that point.
  #[wasm_bindgen(js_name = "truncate")]
  pub fn truncate_wasm(&mut self, new_len: usize) {
    self.truncate(new_len);
  }

  /// Reserve capacity for at least `additional` more bytes.
  #[wasm_bindgen(js_name = "reserve")]
  pub fn reserve_wasm(&mut self, additional: usize) {
    self.reserve(additional);
  }

  /// Ensure this value is backed by a heap allocation.
  #[wasm_bindgen(js_name = "makeHeap")]
  pub fn make_heap_wasm(&mut self) {
    self.make_heap();
  }

  /// Copy a small heap value back into inline storage when possible.
  #[wasm_bindgen(js_name = "tryMakeInline")]
  pub fn try_make_inline_wasm(&mut self) -> bool {
    self.try_make_inline()
  }

  /// Resize the buffer to `new_len`, filling new bytes with `value` if growing.
  #[wasm_bindgen(js_name = "resize")]
  pub fn resize_wasm(&mut self, new_len: usize, value: u8) {
    self.resize(new_len, value);
  }

  /// Split the buffer at position `at`, returning bytes `[0, at)`.
  /// Self becomes `[at, len)`.
  ///
  /// @throws {Error} If `at` is out of bounds.
  #[wasm_bindgen(js_name = "splitTo")]
  pub fn split_to_wasm(&mut self, at: usize) -> Result<BytesMut, JsError> {
    self
      .try_split_to(at)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Split the buffer at position `at`, returning bytes `[at, len)`.
  /// Self becomes `[0, at)`.
  ///
  /// @throws {Error} If `at` is out of bounds.
  #[wasm_bindgen(js_name = "splitOff")]
  pub fn split_off_wasm(&mut self, at: usize) -> Result<BytesMut, JsError> {
    self
      .try_split_off(at)
      .map_err(|e| JsError::new(&e.to_string()))
  }

  /// Split all data out of this buffer, returning it. Self becomes empty.
  #[wasm_bindgen(js_name = "split")]
  pub fn split_wasm(&mut self) -> BytesMut {
    self.split()
  }

  /// Return an independent mutable copy of bytes in range `[start, end)`.
  #[wasm_bindgen(js_name = "slice")]
  pub fn slice_wasm(&self, start: usize, end: usize) -> Result<BytesMut, JsError> {
    if start > end || end > self.len() {
      return Err(JsError::new("slice range is out of bounds"));
    }
    Ok(BytesMut::from(&self.as_slice()[start..end]))
  }

  /// Return an independent mutable copy of this value.
  #[wasm_bindgen(js_name = "clone")]
  pub fn clone_wasm(&self) -> BytesMut {
    self.clone()
  }

  /// Freeze the current contents using the shared strategy and reset this buffer.
  #[wasm_bindgen(js_name = "freezeShared")]
  pub fn freeze_shared_wasm(&mut self) -> crate::bytes::strategy::shared::wasm::WasmSharedBytes {
    let value = core::mem::take(self);
    crate::bytes::strategy::shared::wasm::WasmSharedBytes::from_inner(value.freeze_shared())
  }

  /// Freeze the current contents using the compact strategy and reset this buffer.
  #[wasm_bindgen(js_name = "freezeCompact")]
  pub fn freeze_compact_wasm(&mut self) -> crate::bytes::strategy::compact::wasm::WasmCompactBytes {
    let value = core::mem::take(self);
    crate::bytes::strategy::compact::wasm::WasmCompactBytes::from_inner(value.freeze_compact())
  }

  // -- Buf getters --

  /// Read an unsigned 8-bit integer, advancing the cursor by 1 byte.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU8")]
  pub fn get_u8_wasm(&mut self) -> Result<u8, JsError> {
    if bytes::Buf::remaining(self) < 1 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u8(self))
  }

  /// Read a signed 8-bit integer, advancing the cursor by 1 byte.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI8")]
  pub fn get_i8_wasm(&mut self) -> Result<i8, JsError> {
    if bytes::Buf::remaining(self) < 1 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i8(self))
  }

  /// Read an unsigned 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU16")]
  pub fn get_u16_wasm(&mut self) -> Result<u16, JsError> {
    if bytes::Buf::remaining(self) < 2 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u16(self))
  }

  /// Read an unsigned 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU16Le")]
  pub fn get_u16_le_wasm(&mut self) -> Result<u16, JsError> {
    if bytes::Buf::remaining(self) < 2 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u16_le(self))
  }

  /// Read a signed 16-bit integer in big-endian byte order, advancing the cursor by 2 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI16")]
  pub fn get_i16_wasm(&mut self) -> Result<i16, JsError> {
    if bytes::Buf::remaining(self) < 2 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i16(self))
  }

  /// Read a signed 16-bit integer in little-endian byte order, advancing the cursor by 2 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI16Le")]
  pub fn get_i16_le_wasm(&mut self) -> Result<i16, JsError> {
    if bytes::Buf::remaining(self) < 2 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i16_le(self))
  }

  /// Read an unsigned 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU32")]
  pub fn get_u32_wasm(&mut self) -> Result<u32, JsError> {
    if bytes::Buf::remaining(self) < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u32(self))
  }

  /// Read an unsigned 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU32Le")]
  pub fn get_u32_le_wasm(&mut self) -> Result<u32, JsError> {
    if bytes::Buf::remaining(self) < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u32_le(self))
  }

  /// Read a signed 32-bit integer in big-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI32")]
  pub fn get_i32_wasm(&mut self) -> Result<i32, JsError> {
    if bytes::Buf::remaining(self) < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i32(self))
  }

  /// Read a signed 32-bit integer in little-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI32Le")]
  pub fn get_i32_le_wasm(&mut self) -> Result<i32, JsError> {
    if bytes::Buf::remaining(self) < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i32_le(self))
  }

  /// Read an unsigned 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU64")]
  pub fn get_u64_wasm(&mut self) -> Result<u64, JsError> {
    if bytes::Buf::remaining(self) < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u64(self))
  }

  /// Read an unsigned 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getU64Le")]
  pub fn get_u64_le_wasm(&mut self) -> Result<u64, JsError> {
    if bytes::Buf::remaining(self) < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_u64_le(self))
  }

  /// Read a signed 64-bit integer in big-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI64")]
  pub fn get_i64_wasm(&mut self) -> Result<i64, JsError> {
    if bytes::Buf::remaining(self) < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i64(self))
  }

  /// Read a signed 64-bit integer in little-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getI64Le")]
  pub fn get_i64_le_wasm(&mut self) -> Result<i64, JsError> {
    if bytes::Buf::remaining(self) < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_i64_le(self))
  }

  /// Read a 32-bit float in big-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getF32")]
  pub fn get_f32_wasm(&mut self) -> Result<f32, JsError> {
    if bytes::Buf::remaining(self) < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_f32(self))
  }

  /// Read a 32-bit float in little-endian byte order, advancing the cursor by 4 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getF32Le")]
  pub fn get_f32_le_wasm(&mut self) -> Result<f32, JsError> {
    if bytes::Buf::remaining(self) < 4 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_f32_le(self))
  }

  /// Read a 64-bit float in big-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getF64")]
  pub fn get_f64_wasm(&mut self) -> Result<f64, JsError> {
    if bytes::Buf::remaining(self) < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_f64(self))
  }

  /// Read a 64-bit float in little-endian byte order, advancing the cursor by 8 bytes.
  ///
  /// @throws {Error} If not enough data remains.
  #[wasm_bindgen(js_name = "getF64Le")]
  pub fn get_f64_le_wasm(&mut self) -> Result<f64, JsError> {
    if bytes::Buf::remaining(self) < 8 {
      return Err(JsError::new("not enough data"));
    }
    Ok(bytes::Buf::get_f64_le(self))
  }

  /// Read an unsigned integer encoded in `width` bytes, big-endian.
  #[wasm_bindgen(js_name = "getUint")]
  pub fn get_uint_wasm(&mut self, width: usize) -> Result<u64, JsError> {
    bytes::Buf::try_get_uint(self, width).map_err(|e| JsError::new(&e.to_string()))
  }

  /// Read an unsigned integer encoded in `width` bytes, little-endian.
  #[wasm_bindgen(js_name = "getUintLe")]
  pub fn get_uint_le_wasm(&mut self, width: usize) -> Result<u64, JsError> {
    bytes::Buf::try_get_uint_le(self, width).map_err(|e| JsError::new(&e.to_string()))
  }

  /// Read an unsigned integer encoded in `width` bytes, native-endian.
  #[wasm_bindgen(js_name = "getUintNe")]
  pub fn get_uint_ne_wasm(&mut self, width: usize) -> Result<u64, JsError> {
    bytes::Buf::try_get_uint_ne(self, width).map_err(|e| JsError::new(&e.to_string()))
  }

  /// Read a signed integer encoded in `width` bytes, big-endian.
  #[wasm_bindgen(js_name = "getInt")]
  pub fn get_int_wasm(&mut self, width: usize) -> Result<i64, JsError> {
    bytes::Buf::try_get_int(self, width).map_err(|e| JsError::new(&e.to_string()))
  }

  /// Read a signed integer encoded in `width` bytes, little-endian.
  #[wasm_bindgen(js_name = "getIntLe")]
  pub fn get_int_le_wasm(&mut self, width: usize) -> Result<i64, JsError> {
    bytes::Buf::try_get_int_le(self, width).map_err(|e| JsError::new(&e.to_string()))
  }

  /// Read a signed integer encoded in `width` bytes, native-endian.
  #[wasm_bindgen(js_name = "getIntNe")]
  pub fn get_int_ne_wasm(&mut self, width: usize) -> Result<i64, JsError> {
    bytes::Buf::try_get_int_ne(self, width).map_err(|e| JsError::new(&e.to_string()))
  }

  // -- BufMut putters --

  /// Write a byte slice into the buffer.
  ///
  /// @param data - The bytes to append.
  #[wasm_bindgen(js_name = "putSlice")]
  pub fn put_slice_wasm(&mut self, data: &[u8]) {
    self.extend_from_slice(data);
  }

  /// Append `count` copies of `value`.
  #[wasm_bindgen(js_name = "putBytes")]
  pub fn put_bytes_wasm(&mut self, value: u8, count: usize) {
    bytes::BufMut::put_bytes(self, value, count);
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
  pub fn put_u8_wasm(&mut self, val: u8) {
    bytes::BufMut::put_u8(self, val);
  }

  /// Write a signed 8-bit integer.
  #[wasm_bindgen(js_name = "putI8")]
  pub fn put_i8_wasm(&mut self, val: i8) {
    bytes::BufMut::put_i8(self, val);
  }

  /// Write an unsigned 16-bit integer in big-endian byte order.
  #[wasm_bindgen(js_name = "putU16")]
  pub fn put_u16_wasm(&mut self, val: u16) {
    bytes::BufMut::put_u16(self, val);
  }

  /// Write an unsigned 16-bit integer in little-endian byte order.
  #[wasm_bindgen(js_name = "putU16Le")]
  pub fn put_u16_le_wasm(&mut self, val: u16) {
    bytes::BufMut::put_u16_le(self, val);
  }

  /// Write a signed 16-bit integer in big-endian byte order.
  #[wasm_bindgen(js_name = "putI16")]
  pub fn put_i16_wasm(&mut self, val: i16) {
    bytes::BufMut::put_i16(self, val);
  }

  /// Write a signed 16-bit integer in little-endian byte order.
  #[wasm_bindgen(js_name = "putI16Le")]
  pub fn put_i16_le_wasm(&mut self, val: i16) {
    bytes::BufMut::put_i16_le(self, val);
  }

  /// Write an unsigned 32-bit integer in big-endian byte order.
  #[wasm_bindgen(js_name = "putU32")]
  pub fn put_u32_wasm(&mut self, val: u32) {
    bytes::BufMut::put_u32(self, val);
  }

  /// Write an unsigned 32-bit integer in little-endian byte order.
  #[wasm_bindgen(js_name = "putU32Le")]
  pub fn put_u32_le_wasm(&mut self, val: u32) {
    bytes::BufMut::put_u32_le(self, val);
  }

  /// Write a signed 32-bit integer in big-endian byte order.
  #[wasm_bindgen(js_name = "putI32")]
  pub fn put_i32_wasm(&mut self, val: i32) {
    bytes::BufMut::put_i32(self, val);
  }

  /// Write a signed 32-bit integer in little-endian byte order.
  #[wasm_bindgen(js_name = "putI32Le")]
  pub fn put_i32_le_wasm(&mut self, val: i32) {
    bytes::BufMut::put_i32_le(self, val);
  }

  /// Write an unsigned 64-bit integer in big-endian byte order.
  #[wasm_bindgen(js_name = "putU64")]
  pub fn put_u64_wasm(&mut self, val: u64) {
    bytes::BufMut::put_u64(self, val);
  }

  /// Write an unsigned 64-bit integer in little-endian byte order.
  #[wasm_bindgen(js_name = "putU64Le")]
  pub fn put_u64_le_wasm(&mut self, val: u64) {
    bytes::BufMut::put_u64_le(self, val);
  }

  /// Write a signed 64-bit integer in big-endian byte order.
  #[wasm_bindgen(js_name = "putI64")]
  pub fn put_i64_wasm(&mut self, val: i64) {
    bytes::BufMut::put_i64(self, val);
  }

  /// Write a signed 64-bit integer in little-endian byte order.
  #[wasm_bindgen(js_name = "putI64Le")]
  pub fn put_i64_le_wasm(&mut self, val: i64) {
    bytes::BufMut::put_i64_le(self, val);
  }

  /// Write a 32-bit float in big-endian byte order.
  #[wasm_bindgen(js_name = "putF32")]
  pub fn put_f32_wasm(&mut self, val: f32) {
    bytes::BufMut::put_f32(self, val);
  }

  /// Write a 32-bit float in little-endian byte order.
  #[wasm_bindgen(js_name = "putF32Le")]
  pub fn put_f32_le_wasm(&mut self, val: f32) {
    bytes::BufMut::put_f32_le(self, val);
  }

  /// Write a 64-bit float in big-endian byte order.
  #[wasm_bindgen(js_name = "putF64")]
  pub fn put_f64_wasm(&mut self, val: f64) {
    bytes::BufMut::put_f64(self, val);
  }

  /// Write a 64-bit float in little-endian byte order.
  #[wasm_bindgen(js_name = "putF64Le")]
  pub fn put_f64_le_wasm(&mut self, val: f64) {
    bytes::BufMut::put_f64_le(self, val);
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
