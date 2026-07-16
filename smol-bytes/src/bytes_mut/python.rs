use super::*;
use crate::python::{PyBufCmp, PyBufExt, PyBufMutExt, py_check_alloc};
use bytes::BufMut;
use pyo3::{
  basic::CompareOp,
  exceptions::PyUnicodeDecodeError,
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

  /// Create a new empty buffer with pre-allocated capacity.
  ///
  /// Pre-allocates space for at least `capacity` bytes, but the buffer starts empty
  /// (length 0). This is useful when you know how much data you'll be writing and
  /// want to avoid multiple reallocations.
  ///
  /// Args:
  ///     capacity: Minimum number of bytes to pre-allocate.
  ///
  /// Returns:
  ///     BytesMut: An empty buffer with the specified capacity.
  ///
  /// Example:
  ///     >>> buf = BytesMut.with_capacity(100)
  ///     >>> len(buf)
  ///     0
  ///     >>> buf.capacity()
  ///     100
  #[staticmethod]
  #[pyo3(name = "with_capacity")]
  fn __python_with_capacity(capacity: usize) -> PyResult<Self> {
    py_check_alloc(capacity)?;
    Ok(Self::with_capacity(capacity))
  }

  /// Create a new buffer filled with zeros.
  ///
  /// Creates a buffer of the specified length, with all bytes initialized to zero.
  /// This is more efficient than manually creating a buffer and filling it with zeros.
  ///
  /// Args:
  ///     len: The number of zero bytes to allocate.
  ///
  /// Returns:
  ///     BytesMut: A buffer containing `len` zero bytes.
  ///
  /// Example:
  ///     >>> buf = BytesMut.zeroed(5)
  ///     >>> bytes(buf)
  ///     b'\x00\x00\x00\x00\x00'
  ///     >>> len(buf)
  ///     5
  #[staticmethod]
  #[pyo3(name = "zeroed")]
  fn __python_zeroed(len: usize) -> PyResult<Self> {
    py_check_alloc(len)?;
    Ok(Self::zeroed(len))
  }

  /// Create a new buffer by copying from a bytes-like object.
  ///
  /// Args:
  ///     data: A bytes-like object (bytes, bytearray, etc.) to copy from.
  ///
  /// Returns:
  ///     BytesMut: A new mutable buffer containing a copy of the data.
  ///
  /// Example:
  ///     >>> buf = BytesMut.from_bytes(b"Hello")
  ///     >>> bytes(buf)
  ///     b'Hello'
  #[staticmethod]
  #[pyo3(name = "from_bytes")]
  fn __python_from_bytes(py_bytes: &[u8]) -> Self {
    Self::from(py_bytes)
  }

  /// Create a new buffer from a UTF-8 string.
  ///
  /// Encodes the string as UTF-8 bytes and creates a mutable buffer containing them.
  ///
  /// Args:
  ///     s: A string to encode as UTF-8.
  ///
  /// Returns:
  ///     BytesMut: A new buffer containing the UTF-8 encoded string.
  ///
  /// Example:
  ///     >>> buf = BytesMut.from_str("Hello")
  ///     >>> bytes(buf)
  ///     b'Hello'
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

  #[allow(non_upper_case_globals)]
  const __hash__: Option<Py<PyAny>> = None;

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
  fn __setitem__(
    mut slf: PyRefMut<'_, Self>,
    index: &Bound<'_, PyAny>,
    value: &Bound<'_, PyAny>,
  ) -> PyResult<()> {
    let self_object = (&slf)
      .into_pyobject(slf.py())?
      .to_owned()
      .into_any()
      .unbind();
    let self_assignment = value.is(&self_object).then(|| slf.as_ref().to_vec());
    slf.py_setitem(index, value, self_assignment)
  }

  /// Iterate over the readable bytes.
  fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<Iter>> {
    let iter = Iter {
      inner: BytesMut::clone(&*slf).into_iter(),
    };
    Py::new(slf.py(), iter)
  }

  /// Perform rich comparisons (`==`, `<`, etc.) with other byte sequences.
  fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<Py<PyAny>> {
    self.py_richcmp(other, op)
  }

  /// Interpret the buffer as UTF-8, raising `UnicodeDecodeError` if invalid.
  fn __str__(&self, py: Python<'_>) -> PyResult<&str> {
    core::str::from_utf8(self.as_slice())
      .map_err(|err| PyUnicodeDecodeError::new_err_from_utf8(py, self.as_slice(), err))
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

  /// Check if the buffer is using inline (stack) storage.
  ///
  /// Small buffers (≤62 bytes) are stored inline for better performance.
  /// Larger buffers are automatically moved to the heap.
  ///
  /// Returns:
  ///     bool: True if the buffer is stored inline, False if on the heap.
  ///
  /// Example:
  ///     >>> small = BytesMut.from_bytes(b"small")
  ///     >>> small.is_inline()
  ///     True
  ///     >>> large = BytesMut.from_bytes(b"x" * 100)
  ///     >>> large.is_inline()
  ///     False
  #[pyo3(name = "is_inline")]
  fn __python_is_inline(&self) -> bool {
    self.is_inline()
  }

  /// Check if the buffer is using heap storage.
  ///
  /// Buffers larger than 62 bytes are stored on the heap.
  /// This is the opposite of `is_inline()`.
  ///
  /// Returns:
  ///     bool: True if the buffer is on the heap, False if inline.
  ///
  /// Example:
  ///     >>> large = BytesMut.from_bytes(b"x" * 100)
  ///     >>> large.is_heap()
  ///     True
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
    self.try_advance(cnt).map_err(Into::into)
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
  fn __python_get_uint(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<u64> {
    self.py_get_uint_object(nbytes)
  }

  /// Read an unsigned integer spanning `nbytes` in little-endian order.
  #[pyo3(name = "get_uint_le")]
  fn __python_get_uint_le(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<u64> {
    self.py_get_uint_le_object(nbytes)
  }

  /// Read a signed integer spanning `nbytes` in big-endian order.
  #[pyo3(name = "get_int")]
  fn __python_get_int(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<i64> {
    self.py_get_int_object(nbytes)
  }

  /// Read a signed integer spanning `nbytes` in little-endian order.
  #[pyo3(name = "get_int_le")]
  fn __python_get_int_le(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<i64> {
    self.py_get_int_le_object(nbytes)
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

  /// Reserve capacity for at least `additional` more bytes.
  ///
  /// Ensures the buffer can hold at least `additional` more bytes beyond its
  /// current length without reallocating. The actual capacity reserved may be
  /// larger to avoid frequent reallocations.
  ///
  /// This method may attempt to reclaim unused space from previous operations
  /// before allocating new memory, making it efficient for reused buffers.
  ///
  /// Args:
  ///     additional: Minimum number of additional bytes to reserve.
  ///
  /// Example:
  ///     >>> buf = BytesMut.from_bytes(b"Hello")
  ///     >>> buf.reserve(100)
  ///     >>> buf.capacity() >= 105  # At least current len + additional
  ///     True
  #[pyo3(name = "reserve")]
  fn __python_reserve(&mut self, additional: usize) -> PyResult<()> {
    py_check_alloc(additional)?;
    self.reserve(additional);
    Ok(())
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
  fn __python_put_bytes(&mut self, value: u8, count: usize) -> PyResult<()> {
    py_check_alloc(count)?;
    self.put_bytes(value, count);
    Ok(())
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
  ///     nbytes: Number of bytes to write (<= 8).
  #[pyo3(name = "put_uint")]
  fn __python_put_uint(&mut self, value: u64, nbytes: usize) -> PyResult<()> {
    self.try_put_uint(value, nbytes).map_err(Into::into)
  }

  /// Write an unsigned n-byte integer in little-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///     nbytes: Number of bytes to write (<= 8).
  #[pyo3(name = "put_uint_le")]
  fn __python_put_uint_le(&mut self, value: u64, nbytes: usize) -> PyResult<()> {
    self.try_put_uint_le(value, nbytes).map_err(Into::into)
  }

  /// Write a signed n-byte integer in big-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///     nbytes: Number of bytes to write (<= 8).
  #[pyo3(name = "put_int")]
  fn __python_put_int(&mut self, value: i64, nbytes: usize) -> PyResult<()> {
    self.try_put_int(value, nbytes).map_err(Into::into)
  }

  /// Write a signed n-byte integer in little-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///     nbytes: Number of bytes to write (<= 8).
  #[pyo3(name = "put_int_le")]
  fn __python_put_int_le(&mut self, value: i64, nbytes: usize) -> PyResult<()> {
    self.try_put_int_le(value, nbytes).map_err(Into::into)
  }

  /// Resize the buffer to the specified length, filling with `value` if expanding.
  ///
  /// Args:
  ///     new_len: The new length of the buffer.
  #[pyo3(name = "resize")]
  fn __python_resize(&mut self, new_len: usize, value: u8) -> PyResult<()> {
    py_check_alloc(new_len.saturating_sub(self.len()))?;
    self.resize(new_len, value);
    Ok(())
  }

  /// Split off and return the first `at` bytes as a new buffer.
  ///
  /// After this operation, `self` will contain the remaining bytes starting from
  /// index `at`, and the returned buffer will contain bytes `[0:at)`.
  ///
  /// This is similar to doing `head, self = self[:at], self[at:]` but more efficient.
  /// For large buffers, this can be a zero-copy operation.
  ///
  /// Args:
  ///     at: The split index. Must be <= len(self).
  ///
  /// Returns:
  ///     BytesMut: A new buffer containing the first `at` bytes.
  ///
  /// Raises:
  ///     ValueError: If `at` is greater than the buffer length.
  ///
  /// Example:
  ///     >>> buf = BytesMut.from_bytes(b"Hello, World!")
  ///     >>> head = buf.split_to(7)
  ///     >>> bytes(head)
  ///     b'Hello, '
  ///     >>> bytes(buf)
  ///     b'World!'
  #[pyo3(name = "split_to")]
  fn __python_split_to(&mut self, at: usize) -> PyResult<Self> {
    self
      .try_split_to(at)
      .map(|s| match s {
        Ok(b) => b,
        Err(buf) => BytesMut::from(buf),
      })
      .map_err(Into::into)
  }

  /// Split off and return bytes from index `at` onwards as a new buffer.
  ///
  /// After this operation, `self` will contain bytes `[0:at)`, and the returned
  /// buffer will contain the remaining bytes from index `at` onwards.
  ///
  /// This is similar to doing `self, tail = self[:at], self[at:]` but more efficient.
  /// For large buffers, this can be a zero-copy operation.
  ///
  /// Args:
  ///     at: The split index. Must be <= len(self).
  ///
  /// Returns:
  ///     BytesMut: A new buffer containing bytes from index `at` onwards.
  ///
  /// Raises:
  ///     ValueError: If `at` is greater than the buffer length.
  ///
  /// Example:
  ///     >>> buf = BytesMut.from_bytes(b"Hello, World!")
  ///     >>> tail = buf.split_off(7)
  ///     >>> bytes(buf)
  ///     b'Hello, '
  ///     >>> bytes(tail)
  ///     b'World!'
  #[pyo3(name = "split_off")]
  fn __python_split_off(&mut self, at: usize) -> PyResult<Self> {
    self
      .try_split_off(at)
      .map(|s| match s {
        Ok(b) => b,
        Err(buf) => BytesMut::from(buf),
      })
      .map_err(Into::into)
  }

  /// Take all bytes from the buffer, leaving it empty.
  ///
  /// Returns all bytes from the buffer in a new `BytesMut`, leaving `self` empty
  /// but retaining its capacity for future writes. This is equivalent to calling
  /// `split_to(len(self))`.
  ///
  /// This is useful when you want to move data out of a buffer while keeping the
  /// buffer itself around for reuse.
  ///
  /// Returns:
  ///     BytesMut: A new buffer containing all bytes that were in `self`.
  ///
  /// Example:
  ///     >>> buf = BytesMut.from_bytes(b"Hello!")
  ///     >>> cap = buf.capacity()
  ///     >>> data = buf.split()
  ///     >>> bytes(data)
  ///     b'Hello!'
  ///     >>> len(buf)
  ///     0
  ///     >>> buf.capacity() >= cap  # Capacity is retained
  ///     True
  #[pyo3(name = "split")]
  fn __python_split(&mut self) -> PyResult<Self> {
    self
      .try_split()
      .map(|s| match s {
        Ok(b) => b,
        Err(buf) => BytesMut::from(buf),
      })
      .map_err(Into::into)
  }

  /// Merge another buffer back into this one.
  ///
  /// Attempts to recombine a buffer that was previously split off. This is particularly
  /// efficient when merging buffers that were recently split and haven't been modified
  /// in ways that would cause reallocation.
  ///
  /// If the two buffers were originally contiguous (e.g., `other` was created via
  /// `split_to()` or `split_off()` on this buffer), this can be a very efficient
  /// zero-copy operation. Otherwise, this method will copy the data from `other`
  /// and append it to `self`.
  ///
  /// Args:
  ///     other: The buffer to merge into this one.
  ///
  /// Returns:
  ///     None: If the merge was successful.
  ///     BytesMut: Returns `other` unchanged if the buffers cannot be efficiently merged
  ///         (e.g., when using inline storage). In this case, `self` remains unchanged.
  ///
  /// Example:
  ///     >>> buf = BytesMut.from_bytes(b"Hello, World!")
  ///     >>> tail = buf.split_off(7)
  ///     >>> bytes(buf)
  ///     b'Hello, '
  ///     >>> bytes(tail)
  ///     b'World!'
  ///     >>> result = buf.unsplit(tail)
  ///     >>> result is None  # Merge succeeded
  ///     True
  ///     >>> bytes(buf)
  ///     b'Hello, World!'
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

  /// Support pickling via `pickle.dumps` / `pickle.loads`.
  fn __reduce__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<(Py<PyAny>, (Py<PyBytes>,))> {
    let cls = py.get_type::<Self>();
    let from_bytes = cls.getattr("from_bytes")?;
    let data = PyBytes::new(py, slf.as_ref());
    Ok((from_bytes.unbind(), (data.unbind(),)))
  }
}
