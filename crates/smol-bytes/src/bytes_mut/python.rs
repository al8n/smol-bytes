use super::*;
use crate::python::{PyBufCmp, PyBufExt, PyBufMutExt};
use bytes::BufMut;
use pyo3::{
  basic::CompareOp,
  exceptions::{PyTypeError, PyUnicodeDecodeError},
  prelude::{Bound, *},
  types::{PyAny, PyBytes, PyString},
};
use std::sync::Once;

type IntoIter = ::bytes::buf::IntoIter<BytesMut>;

const DOC: &str = r#"BytesMut

Growable mutable byte buffer. Matches `smol_bytes::BytesMut`, keeping ≤62 bytes inline with zero
allocations and automatically promoting larger payloads to the heap. Supports Python slicing,
mutation, rich comparisons, and the full suite of `Buf` getters (`get_u8`, `get_u32_le`, ...)."#;

static DOC_ONCE: Once = Once::new();

#[derive(Debug)]
#[pyclass]
struct Iter {
  inner: IntoIter,
}

#[pymethods]
impl Iter {
  #[allow(clippy::self_named_constructors)]
  fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
    slf
  }

  fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<u8> {
    slf.inner.next()
  }
}

impl BytesMut {
  fn convert_bytes_arg(arg: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
    if let Ok(bytes) = arg.extract::<Vec<u8>>() {
      Ok(bytes)
    } else if let Ok(s) = arg.extract::<String>() {
      Ok(s.into_bytes())
    } else {
      Err(PyTypeError::new_err("expected a bytes-like object or str"))
    }
  }
}

#[pymethods]
impl BytesMut {
  #[new]
  fn new_python(py: Python<'_>) -> Self {
    DOC_ONCE.call_once(|| {
      let ty = py.get_type::<Self>();
      let _ = ty.setattr("__doc__", DOC);
    });
    Self::new()
  }

  /// Creates a new `BytesMut` with the specified capacity.
  ///
  /// The returned `BytesMut` will be able to hold at least capacity bytes without reallocating.
  ///
  /// It is important to note that this function does not specify the length of the returned BytesMut, but only the capacity.
  #[staticmethod]
  #[pyo3(name = "with_capacity")]
  fn __python_with_capacity(capacity: usize) -> Self {
    Self::with_capacity(capacity)
  }

  /// Creates a new `BytesMut` containing `len` zeros.
  ///
  /// The resulting object has a length of `len` and a capacity greater
  /// than or equal to `len`. The entire length of the object will be filled
  /// with zeros.
  ///
  /// On some platforms or allocators this function may be faster than
  /// a manual implementation.
  #[staticmethod]
  #[pyo3(name = "zeroed")]
  fn __python_zeroed(len: usize) -> Self {
    Self::zeroed(len)
  }

  /// Create a new `BytesMut` by copying from a bytes-like object.
  #[staticmethod]
  #[pyo3(name = "from_bytes")]
  fn __python_from_bytes(py_bytes: &[u8]) -> Self {
    Self::from(py_bytes)
  }

  /// Create a new `BytesMut` by copying from a UTF-8 string.
  #[staticmethod]
  #[pyo3(name = "from_str")]
  fn __python_from_str(py_str: &str) -> Self {
    Self::from(py_str)
  }

  /// Return the mutable bytes as a Python `bytes` object.
  fn __bytes__<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    self.py_bytes(py)
  }

  /// Return `True` when any readable bytes remain.
  fn __bool__(&self) -> bool {
    !self.is_empty()
  }

  /// Return the number of readable bytes.
  fn __len__(&self) -> usize {
    self.py_len()
  }

  /// Check membership of a byte or bytes-like object.
  fn __contains__(&self, item: &Bound<'_, PyAny>) -> PyResult<bool> {
    self.py_contains(item)
  }

  /// Support indexing and slicing of the buffer contents.
  fn __getitem__(&self, index: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    self.py_getitem(index)
  }

  /// Assign to individual items or slices.
  fn __setitem__(&mut self, index: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>) -> PyResult<()> {
    self.py_setitem(index, value)
  }

  /// Iterate over the readable bytes.
  fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<Iter>> {
    let iter = Iter {
      inner: BytesMut::clone(&*slf).into_iter(),
    };
    Py::new(slf.py(), iter)
  }

  /// Perform rich comparisons (`==`, `<`, etc.) with other byte sequences.
  fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<bool> {
    self.py_richcmp(other, op)
  }

  /// Interpret the buffer as UTF-8, raising `UnicodeDecodeError` if invalid.
  fn __str__(&self) -> PyResult<&str> {
    core::str::from_utf8(self.as_slice()).map_err(|e| {
      PyUnicodeDecodeError::new_err(format!(
        "invalid utf-8 sequence at byte {}: {}",
        e.valid_up_to(),
        e
      ))
    })
  }

  fn __repr__(&self) -> String {
    format!("{:?}", self)
  }

  /// Support `copy.copy`, returning a shallow copy.
  #[pyo3(name = "__copy__")]
  fn __python_copy(&self) -> Self {
    self.clone()
  }

  /// Support `copy.deepcopy`, returning a new `BytesMut` clone.
  #[pyo3(name = "__deepcopy__")]
  fn __python_deepcopy(&self, _memo: &Bound<'_, PyAny>) -> Self {
    self.clone()
  }

  /// Return the buffer as a Python `bytes` object.
  #[pyo3(name = "as_bytes")]
  fn __python_as_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    self.py_bytes(py)
  }

  /// Interpret the bytes as UTF-8, returning a Python `str`.
  #[pyo3(name = "to_string")]
  fn __python_to_string<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyString>> {
    self.py_to_string(py)
  }

  /// Return whether the buffer currently uses inline storage.
  #[pyo3(name = "is_inline")]
  fn __python_is_inline(&self) -> bool {
    self.is_inline()
  }

  /// Return whether the buffer currently uses heap storage.
  #[pyo3(name = "is_heap")]
  fn __python_is_heap(&self) -> bool {
    self.is_heap()
  }

  /// Return the number of readable bytes.
  #[pyo3(name = "len")]
  fn __python_len(&self) -> usize {
    self.len()
  }

  /// Return the total capacity without reallocating.
  #[pyo3(name = "capacity")]
  fn __python_capacity(&self) -> usize {
    self.capacity()
  }

  /// Returns the number of bytes remaining to be read from the buffer.
  ///
  /// Returns:
  ///     int: Number of bytes available for reading.
  #[pyo3(name = "remaining")]
  fn __python_remaining(&self) -> usize {
    self.py_remaining()
  }

  /// Return the number of bytes available for writing.
  #[pyo3(name = "remaining_mut")]
  fn __python_remaining_mut(&self) -> usize {
    BufMut::remaining_mut(self)
  }

  /// Advance the read cursor by the specified number of bytes.
  ///
  /// Args:
  ///     cnt: Number of bytes to advance.
  ///
  /// Raises:
  ///     BufferError: If trying to advance beyond available data.
  #[pyo3(name = "advance")]
  fn __python_advance(&mut self, cnt: usize) -> PyResult<()> {
    self.py_advance(cnt)
  }

  // ==================== Get methods ====================

  /// Read an unsigned 8-bit integer and advance by one byte.
  #[pyo3(name = "get_u8")]
  fn __python_get_u8(&mut self) -> PyResult<u8> {
    self.py_get_u8()
  }

  /// Read a signed 8-bit integer and advance by one byte.
  #[pyo3(name = "get_i8")]
  fn __python_get_i8(&mut self) -> PyResult<i8> {
    self.py_get_i8()
  }

  /// Read an unsigned 16-bit integer in big-endian order.
  #[pyo3(name = "get_u16")]
  fn __python_get_u16(&mut self) -> PyResult<u16> {
    self.py_get_u16()
  }

  /// Read an unsigned 16-bit integer in little-endian order.
  #[pyo3(name = "get_u16_le")]
  fn __python_get_u16_le(&mut self) -> PyResult<u16> {
    self.py_get_u16_le()
  }

  /// Read a signed 16-bit integer in big-endian order.
  #[pyo3(name = "get_i16")]
  fn __python_get_i16(&mut self) -> PyResult<i16> {
    self.py_get_i16()
  }

  /// Read a signed 16-bit integer in little-endian order.
  #[pyo3(name = "get_i16_le")]
  fn __python_get_i16_le(&mut self) -> PyResult<i16> {
    self.py_get_i16_le()
  }

  /// Read an unsigned 32-bit integer in big-endian order.
  #[pyo3(name = "get_u32")]
  fn __python_get_u32(&mut self) -> PyResult<u32> {
    self.py_get_u32()
  }

  /// Read an unsigned 32-bit integer in little-endian order.
  #[pyo3(name = "get_u32_le")]
  fn __python_get_u32_le(&mut self) -> PyResult<u32> {
    self.py_get_u32_le()
  }

  /// Read a signed 32-bit integer in big-endian order.
  #[pyo3(name = "get_i32")]
  fn __python_get_i32(&mut self) -> PyResult<i32> {
    self.py_get_i32()
  }

  /// Read a signed 32-bit integer in little-endian order.
  #[pyo3(name = "get_i32_le")]
  fn __python_get_i32_le(&mut self) -> PyResult<i32> {
    self.py_get_i32_le()
  }

  /// Read a 32-bit float in big-endian order.
  #[pyo3(name = "get_f32")]
  fn __python_get_f32(&mut self) -> PyResult<f32> {
    self.py_get_f32()
  }

  /// Read a 32-bit float in little-endian order.
  #[pyo3(name = "get_f32_le")]
  fn __python_get_f32_le(&mut self) -> PyResult<f32> {
    self.py_get_f32_le()
  }

  /// Read an unsigned 64-bit integer in big-endian order.
  #[pyo3(name = "get_u64")]
  fn __python_get_u64(&mut self) -> PyResult<u64> {
    self.py_get_u64()
  }

  /// Read an unsigned 64-bit integer in little-endian order.
  #[pyo3(name = "get_u64_le")]
  fn __python_get_u64_le(&mut self) -> PyResult<u64> {
    self.py_get_u64_le()
  }

  /// Read a signed 64-bit integer in big-endian order.
  #[pyo3(name = "get_i64")]
  fn __python_get_i64(&mut self) -> PyResult<i64> {
    self.py_get_i64()
  }

  /// Read a signed 64-bit integer in little-endian order.
  #[pyo3(name = "get_i64_le")]
  fn __python_get_i64_le(&mut self) -> PyResult<i64> {
    self.py_get_i64_le()
  }

  /// Read a 64-bit float in big-endian order.
  #[pyo3(name = "get_f64")]
  fn __python_get_f64(&mut self) -> PyResult<f64> {
    self.py_get_f64()
  }

  /// Read a 64-bit float in little-endian order.
  #[pyo3(name = "get_f64_le")]
  fn __python_get_f64_le(&mut self) -> PyResult<f64> {
    self.py_get_f64_le()
  }

  /// Read an unsigned 128-bit integer in big-endian order.
  #[pyo3(name = "get_u128")]
  fn __python_get_u128(&mut self) -> PyResult<u128> {
    self.py_get_u128()
  }

  /// Read an unsigned 128-bit integer in little-endian order.
  #[pyo3(name = "get_u128_le")]
  fn __python_get_u128_le(&mut self) -> PyResult<u128> {
    self.py_get_u128_le()
  }

  /// Read a signed 128-bit integer in big-endian order.
  #[pyo3(name = "get_i128")]
  fn __python_get_i128(&mut self) -> PyResult<i128> {
    self.py_get_i128()
  }

  /// Read a signed 128-bit integer in little-endian order.
  #[pyo3(name = "get_i128_le")]
  fn __python_get_i128_le(&mut self) -> PyResult<i128> {
    self.py_get_i128_le()
  }

  /// Read an unsigned integer spanning `nbytes` in big-endian order.
  #[pyo3(name = "get_uint")]
  fn __python_get_uint(&mut self, nbytes: usize) -> PyResult<u64> {
    self.py_get_uint(nbytes)
  }

  /// Read an unsigned integer spanning `nbytes` in little-endian order.
  #[pyo3(name = "get_uint_le")]
  fn __python_get_uint_le(&mut self, nbytes: usize) -> PyResult<u64> {
    self.py_get_uint_le(nbytes)
  }

  /// Read a signed integer spanning `nbytes` in big-endian order.
  #[pyo3(name = "get_int")]
  fn __python_get_int(&mut self, nbytes: usize) -> PyResult<i64> {
    self.py_get_int(nbytes)
  }

  /// Read a signed integer spanning `nbytes` in little-endian order.
  #[pyo3(name = "get_int_le")]
  fn __python_get_int_le(&mut self, nbytes: usize) -> PyResult<i64> {
    self.py_get_int_le(nbytes)
  }

  /// Clear the buffer while retaining capacity.
  #[pyo3(name = "clear")]
  fn __python_clear(&mut self) {
    self.clear();
  }

  /// Truncate the buffer to the specified length.
  ///
  /// If the new length is greater than the current length, this has no effect.
  ///
  /// Args:
  ///     new_len: The new length of the buffer.
  #[pyo3(name = "truncate")]
  fn __python_truncate(&mut self, len: usize) {
    self.truncate(len);
  }

  /// Reserves capacity for at least `additional` more bytes to be inserted
  /// into the given `BytesMut`.
  ///
  /// More than `additional` bytes may be reserved in order to avoid frequent
  /// reallocations. A call to `reserve` may result in an allocation.
  ///
  /// Before allocating new buffer space, the function will attempt to reclaim
  /// space in the existing buffer. If the current handle references a view
  /// into a larger original buffer, and all other handles referencing part
  /// of the same original buffer have been dropped, then the current view
  /// can be copied/shifted to the front of the buffer and the handle can take
  /// ownership of the full buffer, provided that the full buffer is large
  /// enough to fit the requested additional capacity.
  ///
  /// This optimization will only happen if shifting the data from the current
  /// view to the front of the buffer is not too expensive in terms of the
  /// (amortized) time required. The precise condition is subject to change;
  /// as of now, the length of the data being shifted needs to be at least as
  /// large as the distance that it's shifted by. If the current view is empty
  /// and the original buffer is large enough to fit the requested additional
  /// capacity, then reallocations will never happen.
  #[pyo3(name = "reserve")]
  fn __python_reserve(&mut self, additional: usize) {
    self.reserve(additional);
  }

  /// Write a bytes-like object to the buffer.
  ///
  /// Args:
  ///    data: The bytes-like object to write.
  #[pyo3(name = "put_slice")]
  fn __python_put_slice(&mut self, data: &Bound<'_, PyBytes>) -> PyResult<()> {
    self.put_slice(data.as_ref());
    Ok(())
  }

  /// Write a byte value to the buffer multiple times.
  ///
  /// Args:
  ///    val: The byte value to write.
  ///    cnt: Number of times to write the byte.
  #[pyo3(name = "put_bytes")]
  fn __python_put_bytes(&mut self, value: u8, count: usize) {
    self.put_bytes(value, count)
  }

  /// Write a signed 8-bit integer to the buffer.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_i8")]
  fn __python_put_i8(&mut self, value: i8) {
    self.put_i8(value);
  }

  /// Write an unsigned 8-bit integer to the buffer.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_u8")]
  fn __python_put_u8(&mut self, value: u8) {
    self.put_u8(value);
  }

  /// Write an unsigned 16-bit integer in big-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_u16")]
  fn __python_put_u16(&mut self, value: u16) {
    self.put_u16(value);
  }

  /// Write an unsigned 16-bit integer in little-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_u16_le")]
  fn __python_put_u16_le(&mut self, value: u16) {
    self.put_u16_le(value);
  }

  /// Write a signed 16-bit integer in big-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_i16")]
  fn __python_put_i16(&mut self, value: i16) {
    self.put_i16(value);
  }

  /// Write a signed 16-bit integer in little-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_i16_le")]
  fn __python_put_i16_le(&mut self, value: i16) {
    self.put_i16_le(value);
  }

  /// Write an unsigned 32-bit integer in big-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_u32")]
  fn __python_put_u32(&mut self, value: u32) {
    self.put_u32(value);
  }

  /// Write an unsigned 32-bit integer in little-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_u32_le")]
  fn __python_put_u32_le(&mut self, value: u32) {
    self.put_u32_le(value);
  }

  /// Write a signed 32-bit integer in big-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_i32")]
  fn __python_put_i32(&mut self, value: i32) {
    self.put_i32(value);
  }

  /// Write a signed 32-bit integer in little-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_i32_le")]
  fn __python_put_i32_le(&mut self, value: i32) {
    self.put_i32_le(value);
  }

  /// Write a 32-bit floating point number in big-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_f32")]
  fn __python_put_f32(&mut self, value: f32) {
    self.put_f32(value);
  }

  /// Write a 32-bit floating point number in little-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_f32_le")]
  fn __python_put_f32_le(&mut self, value: f32) {
    self.put_f32_le(value);
  }

  /// Write a 64-bit floating point number in big-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_u64")]
  fn __python_put_u64(&mut self, value: u64) {
    self.put_u64(value);
  }

  /// Write a 64-bit floating point number in little-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_u64_le")]
  fn __python_put_u64_le(&mut self, value: u64) {
    self.put_u64_le(value);
  }

  /// Write a signed 64-bit integer in big-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_i64")]
  fn __python_put_i64(&mut self, value: i64) {
    self.put_i64(value);
  }

  /// Write a signed 64-bit integer in little-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_i64_le")]
  fn __python_put_i64_le(&mut self, value: i64) {
    self.put_i64_le(value);
  }

  /// Write a 64-bit float in big-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_f64")]
  fn __python_put_f64(&mut self, value: f64) {
    self.put_f64(value);
  }

  /// Write a 64-bit float in little-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_f64_le")]
  fn __python_put_f64_le(&mut self, value: f64) {
    self.put_f64_le(value);
  }

  /// Write an unsigned 128-bit integer in big-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_u128")]
  fn __python_put_u128(&mut self, value: u128) {
    self.put_u128(value);
  }

  /// Write an unsigned 128-bit integer in little-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_u128_le")]
  fn __python_put_u128_le(&mut self, value: u128) {
    self.put_u128_le(value);
  }

  /// Write a signed 128-bit integer in big-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_i128")]
  fn __python_put_i128(&mut self, value: i128) {
    self.put_i128(value);
  }

  /// Write a signed 128-bit integer in little-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  #[pyo3(name = "put_i128_le")]
  fn __python_put_i128_le(&mut self, value: i128) {
    self.put_i128_le(value);
  }

  /// Write an unsigned n-byte integer in big-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///     nbytes: Number of bytes to write (1-8).
  #[pyo3(name = "put_uint")]
  fn __python_put_uint(&mut self, value: u64, nbytes: usize) {
    self.put_uint(value, nbytes);
  }

  /// Write an unsigned n-byte integer in little-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///     nbytes: Number of bytes to write (1-8).
  #[pyo3(name = "put_uint_le")]
  fn __python_put_uint_le(&mut self, value: u64, nbytes: usize) {
    self.put_uint_le(value, nbytes);
  }

  /// Write a signed n-byte integer in big-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///     nbytes: Number of bytes to write (1-8).
  #[pyo3(name = "put_int")]
  fn __python_put_int(&mut self, value: i64, nbytes: usize) {
    self.put_int(value, nbytes);
  }

  /// Write a signed n-byte integer in little-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///     nbytes: Number of bytes to write (1-8).
  #[pyo3(name = "put_int_le")]
  fn __python_put_int_le(&mut self, value: i64, nbytes: usize) {
    self.put_int_le(value, nbytes);
  }

  /// Resize the buffer to the specified length, filling with zeros if expanding.
  ///
  /// Args:
  ///     new_len: The new length of the buffer.
  #[pyo3(name = "resize")]
  fn __python_resize(&mut self, new_len: usize, value: u8) {
    self.resize(new_len, value);
  }

  /// Split off the first `at` bytes, returning them as a new `BytesMut`.
  #[pyo3(name = "split_to")]
  fn __python_split_to(&mut self, at: usize) -> PyResult<Self> {
    self.py_split_to(at)
  }

  /// Split off the bytes after `at`, returning the tail portion.
  #[pyo3(name = "split_off")]
  fn __python_split_off(&mut self, at: usize) -> PyResult<Self> {
    self.py_split_off(at)
  }

  /// Split off all remaining bytes into a new buffer.
  #[pyo3(name = "split")]
  fn __python_split(&mut self) -> PyResult<Self> {
    let len = self.len();
    self.py_split_to(len)
  }

  /// Attempt to merge `other` back into this buffer.
  #[pyo3(name = "unsplit")]
  fn __python_unsplit(&mut self, other: Self) -> Option<Self> {
    self.unsplit(other)
  }

  /// Converts the `BytesMut` into a heap allocated buffer if it is currently inline.
  ///
  /// If the buffer is already heap allocated, this function does nothing.
  #[pyo3(name = "make_heap")]
  fn __python_make_heap(&mut self) {
    self.make_heap();
  }
}
