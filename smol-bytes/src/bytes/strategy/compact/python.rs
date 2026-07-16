use crate::{
  Buf, OutOfBounds, RangeOutOfBounds,
  python::{PyBufCmp, PyBufCommon, PyBufExt, py_check_alloc},
};
use pyo3::{
  basic::CompareOp,
  exceptions::PyUnicodeDecodeError,
  prelude::{Bound, *},
  types::{PyAny, PyBytes, PyString},
};
use std::sync::Once;

type IntoIter = ::bytes::buf::IntoIter<super::Bytes>;

const DOC: &str = r#"Bytes

Immutable compact byte buffer. Mirrors `smol_bytes.compact::Bytes`, storing up to 62 bytes inline
before falling back to heap storage. All `Buf` getters (e.g. `get_u16_le`) are available, along
with rich comparisons, slicing, and zero-copy split helpers."#;

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

/// Python bindings for the memory-efficient [`compact::Bytes`](crate::compact::Bytes).
///
/// Instances behave like immutable Python `bytes` objects backed by the same zero-copy storage as
/// the Rust type. All `get_*` accessors from the `Buf` trait are available, along with slicing,
/// splitting, and comparison semantics that mirror `compact::Bytes`.
#[pyclass(name = "Bytes", skip_from_py_object)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PyCompactBytes {
  inner: super::Bytes,
}

impl From<super::Bytes> for PyCompactBytes {
  fn from(inner: super::Bytes) -> Self {
    Self { inner }
  }
}

impl AsRef<[u8]> for PyCompactBytes {
  #[cfg_attr(not(coverage), inline(always))]
  fn as_ref(&self) -> &[u8] {
    self.inner.as_ref()
  }
}

impl Buf for PyCompactBytes {
  #[cfg_attr(not(coverage), inline(always))]
  fn remaining(&self) -> usize {
    self.inner.remaining()
  }

  #[cfg_attr(not(coverage), inline(always))]
  fn chunk(&self) -> &[u8] {
    self.inner.chunk()
  }

  #[cfg_attr(not(coverage), inline(always))]
  fn advance(&mut self, cnt: usize) {
    Buf::advance(&mut self.inner, cnt);
  }
}

impl PyBufCommon for PyCompactBytes {
  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_split_to(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    self.inner.try_split_to(at).map(Into::into)
  }

  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_split_off(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    self.inner.try_split_off(at).map(Into::into)
  }

  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_slice(&self, start: usize, end: usize) -> Result<Self, RangeOutOfBounds> {
    self.inner.try_slice(start..end).map(Into::into)
  }

  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_advance(&mut self, cnt: usize) -> Result<(), OutOfBounds> {
    self.inner.try_advance(cnt)
  }
}

#[pymethods]
impl PyCompactBytes {
  #[new]
  fn new_python(py: Python<'_>) -> Self {
    DOC_ONCE.call_once(|| {
      let ty = py.get_type::<Self>();
      let _ = ty.setattr("__doc__", DOC);
    });
    Self {
      inner: super::Bytes::new(),
    }
  }

  /// Create a new immutable bytes object by copying from a bytes-like object.
  ///
  /// Args:
  ///     data: A bytes-like object (bytes, bytearray, etc.) to copy from.
  ///
  /// Returns:
  ///     Bytes: A new immutable bytes object containing a copy of the data.
  ///
  /// Raises:
  ///     MemoryError: If the requested allocation cannot be satisfied.
  ///
  /// Example:
  ///     >>> b = Bytes.from_bytes(b"Hello")
  ///     >>> bytes(b)
  ///     b'Hello'
  #[staticmethod]
  #[pyo3(name = "from_bytes")]
  fn __python_from_bytes(py_bytes: &[u8]) -> PyResult<Self> {
    py_check_alloc(py_bytes.len())?;
    Ok(Self {
      inner: super::Bytes::copy_from_slice(py_bytes),
    })
  }

  /// Create a new immutable bytes object from a UTF-8 string.
  ///
  /// Encodes the string as UTF-8 bytes and creates an immutable bytes object containing them.
  ///
  /// Args:
  ///     s: A string to encode as UTF-8.
  ///
  /// Returns:
  ///     Bytes: A new immutable bytes object containing the UTF-8 encoded string.
  ///
  /// Raises:
  ///     MemoryError: If the requested allocation cannot be satisfied.
  ///
  /// Example:
  ///     >>> b = Bytes.from_str("Hello")
  ///     >>> bytes(b)
  ///     b'Hello'
  #[staticmethod]
  #[pyo3(name = "from_str")]
  fn __python_from_str(py_str: &str) -> PyResult<Self> {
    py_check_alloc(py_str.len())?;
    Ok(Self {
      inner: super::Bytes::from(py_str),
    })
  }

  /// Return the contents as a Python `bytes` object.
  fn __bytes__<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    self.py_bytes(py)
  }

  /// Return `True` when any bytes remain readable.
  fn __bool__(&self) -> bool {
    !self.inner.is_empty()
  }

  #[allow(non_upper_case_globals)]
  const __hash__: Option<Py<PyAny>> = None;

  /// Return the number of remaining readable bytes.
  fn __len__(&self) -> usize {
    self.py_len()
  }

  /// Check membership of a byte or bytes-like object.
  fn __contains__(&self, item: &Bound<'_, PyAny>) -> PyResult<bool> {
    self.py_contains(item)
  }

  /// Support indexing and slicing of the bytes.
  fn __getitem__(&self, index: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    self.py_getitem(index)
  }

  /// Iterate over the readable bytes.
  fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<Iter>> {
    let iter = Iter {
      inner: slf.inner.clone().into_iter(),
    };
    Py::new(slf.py(), iter)
  }

  /// Perform rich comparisons (`==`, `<`, etc.) with other byte sequences.
  fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<Py<PyAny>> {
    self.py_richcmp(other, op)
  }

  /// Interpret the bytes as UTF-8, raising `UnicodeDecodeError` on failure.
  fn __str__(&self, py: Python<'_>) -> PyResult<&str> {
    ::core::str::from_utf8(self.inner.as_ref())
      .map_err(|err| PyUnicodeDecodeError::new_err_from_utf8(py, self.inner.as_ref(), err))
  }

  /// Debug representation mirroring Rust's `Debug` output.
  fn __repr__(&self) -> PyResult<String> {
    py_check_alloc(self.inner.len().saturating_mul(4).saturating_add(64))?;
    Ok(format!("{:?}", self.inner))
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
  fn __python_to_string<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyString>> {
    self.py_to_string(py)
  }

  /// Check if the bytes are using inline (stack) storage.
  ///
  /// Small byte sequences (≤62 bytes) are stored inline for better performance.
  /// Larger sequences are automatically stored on the heap.
  ///
  /// Returns:
  ///     bool: True if the bytes are stored inline, False if on the heap.
  ///
  /// Example:
  ///     >>> small = Bytes.from_bytes(b"small")
  ///     >>> small.is_inline()
  ///     True
  ///     >>> large = Bytes.from_bytes(b"x" * 100)
  ///     >>> large.is_inline()
  ///     False
  #[pyo3(name = "is_inline")]
  fn __python_is_inline(&self) -> bool {
    self.inner.is_inline()
  }

  /// Check if the bytes are using heap storage.
  ///
  /// Byte sequences larger than 62 bytes are stored on the heap.
  /// This is the opposite of `is_inline()`.
  ///
  /// Returns:
  ///     bool: True if the bytes are on the heap, False if inline.
  ///
  /// Example:
  ///     >>> large = Bytes.from_bytes(b"x" * 100)
  ///     >>> large.is_heap()
  ///     True
  #[pyo3(name = "is_heap")]
  fn __python_is_heap(&self) -> bool {
    self.inner.is_heap()
  }

  /// Returns the number of bytes remaining to be read from the buffer.
  ///
  /// Returns:
  ///     int: Number of bytes available for reading.
  #[pyo3(name = "remaining")]
  fn __python_remaining(&self) -> usize {
    self.py_remaining()
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

  /// Truncate the buffer to the specified length.
  ///
  /// If the new length is greater than the current length, this has no effect.
  ///
  /// Args:
  ///     new_len: The new length of the buffer.
  #[pyo3(name = "truncate")]
  fn __python_truncate(&mut self, new_len: usize) {
    self.inner.truncate(new_len);
  }

  /// Clear the buffer, removing all data.
  #[pyo3(name = "clear")]
  fn __python_clear(&mut self) {
    self.inner.clear();
  }

  /// Split the buffer at the given position, returning the head.
  ///
  /// After this operation, `self` will contain bytes from `[at..]`,
  /// and the returned buffer will contain bytes from `[..at]`.
  ///
  /// Args:
  ///     at: The position at which to split the buffer.
  ///
  /// Returns:
  ///     Bytes: A new buffer containing bytes from `[..at]`.
  ///
  /// Raises:
  ///     IndexError: If `at` is greater than the buffer length.
  ///
  /// Example:
  ///     >>> buf = Bytes.from_bytes(b"hello world")
  ///     >>> head = buf.split_to(5)
  ///     >>> bytes(head)
  ///     b'hello'
  ///     >>> bytes(buf)
  ///     b' world'
  #[pyo3(name = "split_to")]
  fn __python_split_to(&mut self, at: usize) -> PyResult<Self> {
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
  ///     Bytes: A new buffer containing bytes from `[at..]`.
  ///
  /// Raises:
  ///     IndexError: If `at` is greater than the buffer length.
  ///
  /// Example:
  ///     >>> buf = Bytes.from_bytes(b"hello world")
  ///     >>> tail = buf.split_off(6)
  ///     >>> bytes(buf)
  ///     b'hello '
  ///     >>> bytes(tail)
  ///     b'world'
  #[pyo3(name = "split_off")]
  fn __python_split_off(&mut self, at: usize) -> PyResult<Self> {
    self.py_split_off(at)
  }

  /// Get a slice of the buffer as a new Bytes.
  ///
  /// This creates a new buffer containing a copy of the specified range.
  ///
  /// Args:
  ///     start: The start position (inclusive).
  ///     end: The end position (exclusive).
  ///
  /// Returns:
  ///     Bytes: A new buffer containing the specified range.
  ///
  /// Raises:
  ///     IndexError: If the range is invalid.
  ///
  /// Example:
  ///     >>> buf = Bytes.from_bytes(b"hello world")
  ///     >>> slice = buf.slice(0, 5)
  ///     >>> bytes(slice)
  ///     b'hello'
  #[pyo3(name = "slice")]
  fn __python_slice(&self, start: usize, end: usize) -> PyResult<Self> {
    self.py_slice(start, end)
  }

  /// Support `copy.copy`, returning a shallow copy.
  #[pyo3(name = "__copy__")]
  fn __python_copy(&self) -> Self {
    self.clone()
  }

  /// Support `copy.deepcopy`, returning a new `Bytes` clone.
  #[pyo3(name = "__deepcopy__")]
  fn __python_deepcopy(&self, _memo: &Bound<'_, PyAny>) -> Self {
    self.clone()
  }

  /// Read an unsigned 8-bit integer from the buffer.
  ///
  /// The current position is advanced by 1 byte.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u8")]
  fn __python_get_u8(&mut self) -> PyResult<u8> {
    self.inner.py_get_u8()
  }

  /// Read a signed 8-bit integer from the buffer.
  ///
  /// The current position is advanced by 1 byte.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i8")]
  fn __python_get_i8(&mut self) -> PyResult<i8> {
    self.inner.py_get_i8()
  }

  /// Read an unsigned 16-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 2 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u16")]
  fn __python_get_u16(&mut self) -> PyResult<u16> {
    self.inner.py_get_u16()
  }

  /// Read an unsigned 16-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 2 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u16_le")]
  fn __python_get_u16_le(&mut self) -> PyResult<u16> {
    self.inner.py_get_u16_le()
  }

  /// Read a signed 16-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 2 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i16")]
  fn __python_get_i16(&mut self) -> PyResult<i16> {
    self.inner.py_get_i16()
  }

  /// Read a signed 16-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 2 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i16_le")]
  fn __python_get_i16_le(&mut self) -> PyResult<i16> {
    self.inner.py_get_i16_le()
  }

  /// Read an unsigned 32-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u32")]
  fn __python_get_u32(&mut self) -> PyResult<u32> {
    self.inner.py_get_u32()
  }

  /// Read an unsigned 32-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u32_le")]
  fn __python_get_u32_le(&mut self) -> PyResult<u32> {
    self.inner.py_get_u32_le()
  }

  /// Read a signed 32-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i32")]
  fn __python_get_i32(&mut self) -> PyResult<i32> {
    self.inner.py_get_i32()
  }

  /// Read a signed 32-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i32_le")]
  fn __python_get_i32_le(&mut self) -> PyResult<i32> {
    self.inner.py_get_i32_le()
  }

  /// Read a 32-bit floating point number in big-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_f32")]
  fn __python_get_f32(&mut self) -> PyResult<f32> {
    self.inner.py_get_f32()
  }

  /// Read a 32-bit floating point number in little-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_f32_le")]
  fn __python_get_f32_le(&mut self) -> PyResult<f32> {
    self.inner.py_get_f32_le()
  }

  /// Read an unsigned 64-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u64")]
  fn __python_get_u64(&mut self) -> PyResult<u64> {
    self.inner.py_get_u64()
  }

  /// Read an unsigned 64-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u64_le")]
  fn __python_get_u64_le(&mut self) -> PyResult<u64> {
    self.inner.py_get_u64_le()
  }

  /// Read a signed 64-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i64")]
  fn __python_get_i64(&mut self) -> PyResult<i64> {
    self.inner.py_get_i64()
  }

  /// Read a signed 64-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i64_le")]
  fn __python_get_i64_le(&mut self) -> PyResult<i64> {
    self.inner.py_get_i64_le()
  }

  /// Read a 64-bit floating point number in big-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_f64")]
  fn __python_get_f64(&mut self) -> PyResult<f64> {
    self.inner.py_get_f64()
  }

  /// Read a 64-bit floating point number in little-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_f64_le")]
  fn __python_get_f64_le(&mut self) -> PyResult<f64> {
    self.inner.py_get_f64_le()
  }

  /// Read an unsigned 128-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 16 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u128")]
  fn __python_get_u128(&mut self) -> PyResult<u128> {
    self.inner.py_get_u128()
  }

  /// Read an unsigned 128-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 16 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u128_le")]
  fn __python_get_u128_le(&mut self) -> PyResult<u128> {
    self.inner.py_get_u128_le()
  }

  /// Read a signed 128-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 16 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i128")]
  fn __python_get_i128(&mut self) -> PyResult<i128> {
    self.inner.py_get_i128()
  }

  /// Read a signed 128-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 16 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i128_le")]
  fn __python_get_i128_le(&mut self) -> PyResult<i128> {
    self.inner.py_get_i128_le()
  }

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
  fn __python_get_uint(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<u64> {
    self.inner.py_get_uint_object(nbytes)
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
  fn __python_get_uint_le(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<u64> {
    self.inner.py_get_uint_le_object(nbytes)
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
  fn __python_get_int(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<i64> {
    self.inner.py_get_int_object(nbytes)
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
  fn __python_get_int_le(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<i64> {
    self.inner.py_get_int_le_object(nbytes)
  }

  /// Support pickling via `pickle.dumps` / `pickle.loads`.
  fn __reduce__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<(Py<PyAny>, (Py<PyBytes>,))> {
    let cls = py.get_type::<Self>();
    let from_bytes = cls.getattr("from_bytes")?;
    let data = PyBytes::new(py, slf.as_ref());
    Ok((from_bytes.unbind(), (data.unbind(),)))
  }

  /// Expose a snapshot copy as a buffer (enables `memoryview`); later mutations
  /// of this object are not reflected in the view.
  unsafe fn __getbuffer__(
    slf: PyRef<'_, Self>,
    view: *mut pyo3::ffi::Py_buffer,
    flags: ::std::os::raw::c_int,
  ) -> PyResult<()> {
    let py = slf.py();
    let snapshot = PyBytes::new(py, slf.as_ref());
    if unsafe { pyo3::ffi::PyObject_GetBuffer(snapshot.as_ptr(), view, flags) } == 0 {
      Ok(())
    } else {
      Err(PyErr::fetch(py))
    }
  }
}
