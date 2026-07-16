use crate::python::{PyBufCmp, PyBufExt, PyBufMutExt, py_check_alloc};
use pyo3::{
  basic::CompareOp,
  exceptions::{PyBufferError, PyUnicodeDecodeError},
  prelude::{Bound, *},
  types::{PyBytes, PyString},
};

use super::*;

#[cfg(not(any(feature = "std", feature = "alloc")))]
type IntoIter = super::iter::IntoIter<Buffer>;

#[cfg(any(feature = "std", feature = "alloc"))]
type IntoIter = ::bytes::buf::IntoIter<Buffer>;

/// Iterator over the bytes contained by the buffer.
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
impl Buffer {
  #[new]
  fn new_python() -> Self {
    Self::new()
  }

  /// Create a new buffer by copying from a bytes-like object.
  ///
  /// Creates a fixed-size inline buffer (max 62 bytes) containing a copy of the data.
  ///
  /// Args:
  ///     data: A bytes-like object (bytes, bytearray, etc.) to copy from.
  ///
  /// Returns:
  ///     Buffer: A new buffer containing a copy of the data.
  ///
  /// Raises:
  ///     BufferError: If the data exceeds the 62-byte inline capacity.
  ///
  /// Example:
  ///     >>> buf = Buffer.from_bytes(b"Hello")
  ///     >>> bytes(buf)
  ///     b'Hello'
  #[staticmethod]
  #[pyo3(name = "from_bytes")]
  fn __python_from_bytes(py_bytes: &[u8]) -> PyResult<Self> {
    Self::try_from(py_bytes).map_err(|e| {
      PyBufferError::new_err(format!(
        "overflow the buffer capacity: requested {} but only {} available",
        e.requested, e.available
      ))
    })
  }

  /// Create a new buffer from a UTF-8 string.
  ///
  /// Encodes the string as UTF-8 bytes and creates a fixed-size inline buffer containing them.
  ///
  /// Args:
  ///     s: A string to encode as UTF-8.
  ///
  /// Returns:
  ///     Buffer: A new buffer containing the UTF-8 encoded string.
  ///
  /// Raises:
  ///     BufferError: If the encoded string exceeds the 62-byte inline capacity.
  ///
  /// Example:
  ///     >>> buf = Buffer.from_str("Hello")
  ///     >>> bytes(buf)
  ///     b'Hello'
  #[staticmethod]
  #[pyo3(name = "from_str")]
  fn __python_from_str(py_str: &str) -> PyResult<Self> {
    Self::try_from(py_str.as_bytes()).map_err(|e| {
      PyBufferError::new_err(format!(
        "overflow the buffer capacity: requested {} but only {} available",
        e.requested, e.available
      ))
    })
  }

  /// Return the buffer contents as a Python `bytes` object.
  fn __bytes__<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    self.py_bytes(py)
  }

  /// Return `True` when the buffer contains any readable bytes.
  fn __bool__(&self) -> bool {
    !self.is_empty()
  }

  #[allow(non_upper_case_globals)]
  const __hash__: Option<Py<PyAny>> = None;

  /// Iterate over the readable bytes, yielding one `int` per byte.
  fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<Iter>> {
    let iter = Iter {
      inner: slf.into_iter(),
    };
    Py::new(slf.py(), iter)
  }

  /// Perform rich comparisons (`==`, `<`, etc.) with other byte sequences.
  fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<Py<PyAny>> {
    self.py_richcmp(other, op)
  }

  /// Interpret the buffer as UTF-8, raising `UnicodeDecodeError` if invalid.
  fn __str__(&self, py: Python<'_>) -> PyResult<&str> {
    <&str>::try_from(self)
      .map_err(|err| PyUnicodeDecodeError::new_err_from_utf8(py, self.as_ref(), err))
  }

  /// Return a debug representation of the buffer.
  fn __repr__(&self) -> String {
    format!("{:?}", self)
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
    let self_assignment = if value.is(&self_object) {
      py_check_alloc(slf.as_ref().len())?;
      Some(slf.as_ref().to_vec())
    } else {
      None
    };
    slf.py_setitem(index, value, self_assignment)
  }

  /// Return the buffer contents as Python bytes.
  ///
  /// This returns a view of the remaining readable data in the buffer
  /// (from the current cursor position to the end).
  ///
  /// Returns:
  ///     bytes: The buffer contents as a bytes object.
  #[pyo3(name = "as_bytes")]
  fn __python_as_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    self.py_bytes(py)
  }

  /// Convert the buffer contents to a UTF-8 string.
  ///
  /// This validates that all bytes in the buffer (from the current cursor
  /// position to the end) are valid UTF-8 and returns a string.
  ///
  /// Returns:
  ///     str: The buffer contents as a UTF-8 string.
  ///
  /// Raises:
  ///     UnicodeDecodeError: If the buffer contains invalid UTF-8.
  #[pyo3(name = "to_string")]
  fn __python_to_string<'py>(&self, py: Python<'py>) -> ::pyo3::PyResult<Bound<'py, PyString>> {
    self.py_to_string(py)
  }

  /// Read an unsigned 8-bit integer from the buffer.
  ///
  /// The current position is advanced by 1 byte.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u8")]
  fn __python_get_u8(&mut self) -> ::pyo3::PyResult<u8> {
    self.py_get_u8()
  }

  /// Read a signed 8-bit integer from the buffer.
  ///
  /// The current position is advanced by 1 byte.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i8")]
  fn __python_get_i8(&mut self) -> ::pyo3::PyResult<i8> {
    self.py_get_i8()
  }

  // ==================== 16-bit methods ====================

  /// Read an unsigned 16-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 2 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u16")]
  fn __python_get_u16(&mut self) -> ::pyo3::PyResult<u16> {
    self.py_get_u16()
  }

  /// Read an unsigned 16-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 2 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u16_le")]
  fn __python_get_u16_le(&mut self) -> ::pyo3::PyResult<u16> {
    self.py_get_u16_le()
  }

  /// Read a signed 16-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 2 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i16")]
  fn __python_get_i16(&mut self) -> ::pyo3::PyResult<i16> {
    self.py_get_i16()
  }

  /// Read a signed 16-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 2 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i16_le")]
  fn __python_get_i16_le(&mut self) -> ::pyo3::PyResult<i16> {
    self.py_get_i16_le()
  }

  // ==================== 32-bit methods ====================

  /// Read an unsigned 32-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u32")]
  fn __python_get_u32(&mut self) -> ::pyo3::PyResult<u32> {
    self.py_get_u32()
  }

  /// Read an unsigned 32-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u32_le")]
  fn __python_get_u32_le(&mut self) -> ::pyo3::PyResult<u32> {
    self.py_get_u32_le()
  }

  /// Read a signed 32-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i32")]
  fn __python_get_i32(&mut self) -> ::pyo3::PyResult<i32> {
    self.py_get_i32()
  }

  /// Read a signed 32-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i32_le")]
  fn __python_get_i32_le(&mut self) -> ::pyo3::PyResult<i32> {
    self.py_get_i32_le()
  }

  /// Read a 32-bit floating point number in big-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_f32")]
  fn __python_get_f32(&mut self) -> ::pyo3::PyResult<f32> {
    self.py_get_f32()
  }

  /// Read a 32-bit floating point number in little-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_f32_le")]
  fn __python_get_f32_le(&mut self) -> ::pyo3::PyResult<f32> {
    self.py_get_f32_le()
  }

  // ==================== 64-bit methods ====================

  /// Read an unsigned 64-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u64")]
  fn __python_get_u64(&mut self) -> ::pyo3::PyResult<u64> {
    self.py_get_u64()
  }

  /// Read an unsigned 64-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u64_le")]
  fn __python_get_u64_le(&mut self) -> ::pyo3::PyResult<u64> {
    self.py_get_u64_le()
  }

  /// Read a signed 64-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i64")]
  fn __python_get_i64(&mut self) -> ::pyo3::PyResult<i64> {
    self.py_get_i64()
  }

  /// Read a signed 64-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i64_le")]
  fn __python_get_i64_le(&mut self) -> ::pyo3::PyResult<i64> {
    self.py_get_i64_le()
  }

  /// Read a 64-bit floating point number in big-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_f64")]
  fn __python_get_f64(&mut self) -> ::pyo3::PyResult<f64> {
    self.py_get_f64()
  }

  /// Read a 64-bit floating point number in little-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_f64_le")]
  fn __python_get_f64_le(&mut self) -> ::pyo3::PyResult<f64> {
    self.py_get_f64_le()
  }

  // ==================== 128-bit methods ====================

  /// Read an unsigned 128-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 16 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u128")]
  fn __python_get_u128(&mut self) -> ::pyo3::PyResult<u128> {
    self.py_get_u128()
  }

  /// Read an unsigned 128-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 16 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u128_le")]
  fn __python_get_u128_le(&mut self) -> ::pyo3::PyResult<u128> {
    self.py_get_u128_le()
  }

  /// Read a signed 128-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 16 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i128")]
  fn __python_get_i128(&mut self) -> ::pyo3::PyResult<i128> {
    self.py_get_i128()
  }

  /// Read a signed 128-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 16 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i128_le")]
  fn __python_get_i128_le(&mut self) -> ::pyo3::PyResult<i128> {
    self.py_get_i128_le()
  }

  // ==================== Variable-length methods ====================

  /// Read an unsigned n-byte integer in big-endian byte order.
  ///
  /// The current position is advanced by nbytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (0-8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_uint")]
  fn __python_get_uint(&mut self, nbytes: &Bound<'_, PyAny>) -> ::pyo3::PyResult<u64> {
    self.py_get_uint_object(nbytes)
  }

  /// Read an unsigned n-byte integer in little-endian byte order.
  ///
  /// The current position is advanced by nbytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (0-8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_uint_le")]
  fn __python_get_uint_le(&mut self, nbytes: &Bound<'_, PyAny>) -> ::pyo3::PyResult<u64> {
    self.py_get_uint_le_object(nbytes)
  }

  /// Read a signed n-byte integer in big-endian byte order.
  ///
  /// The current position is advanced by nbytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (0-8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_int")]
  fn __python_get_int(&mut self, nbytes: &Bound<'_, PyAny>) -> ::pyo3::PyResult<i64> {
    self.py_get_int_object(nbytes)
  }

  /// Read a signed n-byte integer in little-endian byte order.
  ///
  /// The current position is advanced by nbytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (0-8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_int_le")]
  fn __python_get_int_le(&mut self, nbytes: &Bound<'_, PyAny>) -> ::pyo3::PyResult<i64> {
    self.py_get_int_le_object(nbytes)
  }

  // ==================== Buffer control methods ====================

  /// Returns the number of bytes remaining to be read from the buffer.
  ///
  /// Returns:
  ///     int: Number of bytes available for reading.
  #[pyo3(name = "remaining")]
  fn __python_remaining(&self) -> usize {
    self.py_remaining()
  }

  /// Returns the number of bytes that can be written to the buffer.
  ///
  /// Returns:
  ///     int: Number of bytes available for writing.
  #[pyo3(name = "remaining_mut")]
  fn __python_remaining_mut(&self) -> usize {
    self.remaining_mut()
  }

  /// Advance the read cursor by the specified number of bytes.
  ///
  /// Args:
  ///     cnt: Number of bytes to advance.
  ///
  /// Raises:
  ///     BufferError: If trying to advance beyond available data.
  #[pyo3(name = "advance")]
  fn __python_advance(&mut self, cnt: usize) -> ::pyo3::PyResult<()> {
    self.py_advance(cnt)
  }

  // ==================== Put methods ====================
  /// Write a bytes-like object to the buffer.
  ///
  /// Args:
  ///    data: The bytes-like object to write.
  ///
  /// Raises:
  ///   BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_slice")]
  fn __python_put_slice(&mut self, data: &Bound<'_, PyBytes>) -> ::pyo3::PyResult<()> {
    self.try_put_slice(data.as_ref()).map_err(Into::into)
  }

  /// Write a byte value to the buffer multiple times.
  ///
  /// Args:
  ///    val: The byte value to write.
  ///    cnt: Number of times to write the byte.
  ///
  /// Raises:
  ///    BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_bytes")]
  fn __python_put_bytes(&mut self, val: u8, cnt: usize) -> ::pyo3::PyResult<()> {
    self.try_put_bytes(val, cnt).map_err(Into::into)
  }

  /// Write an unsigned 8-bit integer to the buffer.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_u8")]
  fn __python_put_u8(&mut self, value: u8) -> ::pyo3::PyResult<()> {
    self.try_put_u8(value).map_err(Into::into)
  }

  /// Write a signed 8-bit integer to the buffer.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_i8")]
  fn __python_put_i8(&mut self, val: i8) -> ::pyo3::PyResult<()> {
    self.try_put_i8(val).map_err(Into::into)
  }

  /// Write an unsigned 16-bit integer in big-endian order.
  ///
  /// Args:
  ///    val: The value to write.
  ///
  /// Raises:
  ///    BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_u16")]
  fn __python_put_u16(&mut self, value: u16) -> PyResult<()> {
    self.try_put_u16(value).map_err(Into::into)
  }

  /// Write an unsigned 16-bit integer in little-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_u16_le")]
  fn __python_put_u16_le(&mut self, value: u16) -> PyResult<()> {
    self.try_put_u16_le(value).map_err(Into::into)
  }

  /// Write a signed 16-bit integer in big-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_i16")]
  fn __python_put_i16(&mut self, value: i16) -> PyResult<()> {
    self.try_put_i16(value).map_err(Into::into)
  }

  /// Write a signed 16-bit integer in little-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_i16_le")]
  fn __python_put_i16_le(&mut self, value: i16) -> PyResult<()> {
    self.try_put_i16_le(value).map_err(Into::into)
  }

  /// Write an unsigned 32-bit integer in big-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_u32")]
  fn __python_put_u32(&mut self, value: u32) -> PyResult<()> {
    self.try_put_u32(value).map_err(Into::into)
  }

  /// Write an unsigned 32-bit integer in little-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_u32_le")]
  fn __python_put_u32_le(&mut self, value: u32) -> PyResult<()> {
    self.try_put_u32_le(value).map_err(Into::into)
  }

  /// Write a signed 32-bit integer in big-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_i32")]
  fn __python_put_i32(&mut self, value: i32) -> PyResult<()> {
    self.try_put_i32(value).map_err(Into::into)
  }

  /// Write a signed 32-bit integer in little-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_i32_le")]
  fn __python_put_i32_le(&mut self, value: i32) -> PyResult<()> {
    self.try_put_i32_le(value).map_err(Into::into)
  }

  /// Write a 64-bit floating point number in big-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_u64")]
  fn __python_put_u64(&mut self, value: u64) -> PyResult<()> {
    self.try_put_u64(value).map_err(Into::into)
  }

  /// Write a 64-bit floating point number in little-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_u64_le")]
  fn __python_put_u64_le(&mut self, value: u64) -> PyResult<()> {
    self.try_put_u64_le(value).map_err(Into::into)
  }

  /// Write a signed 64-bit integer in big-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_i64")]
  fn __python_put_i64(&mut self, value: i64) -> PyResult<()> {
    self.try_put_i64(value).map_err(Into::into)
  }

  /// Write a signed 64-bit integer in little-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_i64_le")]
  fn __python_put_i64_le(&mut self, value: i64) -> PyResult<()> {
    self.try_put_i64_le(value).map_err(Into::into)
  }

  /// Write a 32-bit floating point number in big-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_f32")]
  fn __python_put_f32(&mut self, val: f32) -> ::pyo3::PyResult<()> {
    self.try_put_f32(val).map_err(Into::into)
  }

  /// Write a 32-bit floating point number in little-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_f32_le")]
  fn __python_put_f32_le(&mut self, val: f32) -> ::pyo3::PyResult<()> {
    self.try_put_f32_le(val).map_err(Into::into)
  }

  /// Write a 64-bit floating point number in big-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_f64")]
  fn __python_put_f64(&mut self, val: f64) -> ::pyo3::PyResult<()> {
    self.try_put_f64(val).map_err(Into::into)
  }

  /// Write a 64-bit floating point number in little-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_f64_le")]
  fn __python_put_f64_le(&mut self, val: f64) -> ::pyo3::PyResult<()> {
    self.try_put_f64_le(val).map_err(Into::into)
  }

  /// Write an unsigned 128-bit integer in big-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_u128")]
  fn __python_put_u128(&mut self, value: u128) -> PyResult<()> {
    self.try_put_u128(value).map_err(Into::into)
  }

  /// Write an unsigned 128-bit integer in little-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_u128_le")]
  fn __python_put_u128_le(&mut self, value: u128) -> PyResult<()> {
    self.try_put_u128_le(value).map_err(Into::into)
  }

  /// Write a signed 128-bit integer in big-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_i128")]
  fn __python_put_i128(&mut self, value: i128) -> PyResult<()> {
    self.try_put_i128(value).map_err(Into::into)
  }

  /// Write a signed 128-bit integer in little-endian order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_i128_le")]
  fn __python_put_i128_le(&mut self, value: i128) -> PyResult<()> {
    self.try_put_i128_le(value).map_err(Into::into)
  }

  /// Write an unsigned n-byte integer in big-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///     nbytes: Number of bytes to write (<= 8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  ///     ValueError: If nbytes is not <= 8.
  #[pyo3(name = "put_uint")]
  fn __python_put_uint(&mut self, val: u64, nbytes: usize) -> ::pyo3::PyResult<()> {
    self.try_put_uint(val, nbytes).map_err(Into::into)
  }

  /// Write an unsigned n-byte integer in little-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///     nbytes: Number of bytes to write (<= 8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  ///     ValueError: If nbytes is not <= 8.
  #[pyo3(name = "put_uint_le")]
  fn __python_put_uint_le(&mut self, val: u64, nbytes: usize) -> ::pyo3::PyResult<()> {
    self.try_put_uint_le(val, nbytes).map_err(Into::into)
  }

  /// Write a signed n-byte integer in big-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///     nbytes: Number of bytes to write (<= 8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  ///     ValueError: If nbytes is not <= 8.
  #[pyo3(name = "put_int")]
  fn __python_put_int(&mut self, val: i64, nbytes: usize) -> ::pyo3::PyResult<()> {
    self.try_put_int(val, nbytes).map_err(Into::into)
  }

  /// Write a signed n-byte integer in little-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///     nbytes: Number of bytes to write (<= 8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  ///     ValueError: If nbytes is not <= 8.
  #[pyo3(name = "put_int_le")]
  fn __python_put_int_le(&mut self, val: i64, nbytes: usize) -> ::pyo3::PyResult<()> {
    self.try_put_int_le(val, nbytes).map_err(Into::into)
  }

  // ==================== Buffer management methods ====================

  /// Clear the buffer, removing all data.
  ///
  /// The capacity is preserved.
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
  fn __python_truncate(&mut self, new_len: usize) {
    self.truncate(new_len);
  }

  /// Get the capacity of the buffer.
  ///
  /// Returns:
  ///     int: The number of bytes that can be written without reallocation.
  #[pyo3(name = "capacity")]
  fn __python_capacity(&self) -> usize {
    self.capacity()
  }

  /// Resize the buffer to the specified length, filling with zeros if expanding.
  ///
  /// Args:
  ///     new_len: The new length of the buffer.
  ///
  /// Raises:
  ///     BufferError: If the new length exceeds capacity.
  #[pyo3(name = "resize")]
  fn __python_resize(&mut self, new_len: usize) -> ::pyo3::PyResult<()> {
    self.try_resize(new_len).map_err(|e| {
      pyo3::exceptions::PyBufferError::new_err(format!(
        "resize exceeds capacity: requested {} but capacity is {}",
        e.requested, e.available
      ))
    })
  }

  // ==================== Split methods ====================

  /// Split the buffer at the given position, returning the head.
  ///
  /// After this operation, `self` will contain bytes from `[at..]`,
  /// and the returned buffer will contain bytes from `[..at]`.
  ///
  /// Args:
  ///     at: The position at which to split the buffer.
  ///
  /// Returns:
  ///     Buffer: A new buffer containing bytes from `[..at]`.
  ///
  /// Raises:
  ///     IndexError: If `at` is greater than the buffer length.
  ///
  /// Example:
  ///     >>> buf = Buffer.from_bytes(b"hello world")
  ///     >>> head = buf.split_to(5)
  ///     >>> bytes(head)
  ///     b'hello'
  ///     >>> bytes(buf)
  ///     b' world'
  #[pyo3(name = "split_to")]
  fn __python_split_to(&mut self, at: usize) -> ::pyo3::PyResult<Buffer> {
    self.py_split_to(at)
  }

  /// Split the buffer at the given position, returning the tail.
  ///
  /// After this operation, `self` will contain bytes from `[..at]`,
  /// and the returned buffer will contain bytes from `[at..]`.
  ///
  /// Args:
  ///     at: The position at which to split the buffer.
  ///
  /// Returns:
  ///     Buffer: A new buffer containing bytes from `[at..]`.
  ///
  /// Raises:
  ///     IndexError: If `at` is greater than the buffer length.
  ///
  /// Example:
  ///     >>> buf = Buffer.from_bytes(b"hello world")
  ///     >>> tail = buf.split_off(6)
  ///     >>> bytes(buf)
  ///     b'hello '
  ///     >>> bytes(tail)
  ///     b'world'
  #[pyo3(name = "split_off")]
  fn __python_split_off(&mut self, at: usize) -> ::pyo3::PyResult<Buffer> {
    self.py_split_off(at)
  }

  /// Get a slice of the buffer as a new Buffer.
  ///
  /// This creates a new buffer containing a copy of the specified range.
  ///
  /// Args:
  ///     start: The start position (inclusive).
  ///     end: The end position (exclusive).
  ///
  /// Returns:
  ///     Buffer: A new buffer containing the specified range.
  ///
  /// Raises:
  ///     IndexError: If the range is invalid.
  ///
  /// Example:
  ///     >>> buf = Buffer.from_bytes(b"hello world")
  ///     >>> slice = buf.slice(0, 5)
  ///     >>> bytes(slice)
  ///     b'hello'
  #[pyo3(name = "slice")]
  fn __python_slice(&self, start: usize, end: usize) -> ::pyo3::PyResult<Buffer> {
    self.py_slice(start, end)
  }

  /// Support pickling via `pickle.dumps` / `pickle.loads`.
  fn __reduce__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<(Py<PyAny>, (Py<PyBytes>,))> {
    let cls = py.get_type::<Self>();
    let from_bytes = cls.getattr("from_bytes")?;
    let data = PyBytes::new(py, slf.as_ref());
    Ok((from_bytes.unbind(), (data.unbind(),)))
  }
}
