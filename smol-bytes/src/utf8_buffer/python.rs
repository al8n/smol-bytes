use crate::python::{py_str_contains, py_str_getitem, py_str_richcmp};

use super::*;
use pyo3::{
  basic::CompareOp,
  exceptions::PyValueError,
  prelude::{Bound, *},
  types::PyBytes,
};

#[derive(Debug)]
#[pyclass]
struct Utf8CharIter {
  chars: Vec<char>,
  index: usize,
}

#[pymethods]
impl Utf8CharIter {
  fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
    slf
  }

  fn __next__(&mut self) -> Option<String> {
    if self.index < self.chars.len() {
      let ch = self.chars[self.index];
      self.index += 1;
      Some(ch.to_string())
    } else {
      None
    }
  }
}

#[pymethods]
impl Utf8Buffer {
  #[new]
  fn new_python() -> Self {
    Self::new()
  }

  /// Create a new UTF-8 buffer from a string.
  ///
  /// Args:
  ///     s: A string to copy into the buffer.
  ///
  /// Returns:
  ///     Utf8Buffer: A new buffer containing the string.
  ///
  /// Raises:
  ///     ValueError: If the string exceeds the 62-byte inline capacity.
  #[staticmethod]
  #[pyo3(name = "from_str")]
  fn __python_from_str(s: &str) -> PyResult<Self> {
    Self::try_from_str(s).map_err(|e| PyValueError::new_err(format!("string too large: {}", e)))
  }

  /// Return the buffer contents as a Python string.
  fn __str__(&self) -> &str {
    self.as_str()
  }

  /// Return a debug representation.
  fn __repr__(&self) -> String {
    format!("{:?}", self)
  }

  /// Return the number of Unicode scalar values in the buffer.
  fn __len__(&self) -> usize {
    self.as_str().chars().count()
  }

  /// Return whether the buffer is non-empty.
  fn __bool__(&self) -> bool {
    !self.is_empty()
  }

  /// Check if a substring is contained in the buffer.
  fn __contains__(&self, item: &Bound<'_, PyAny>) -> PyResult<bool> {
    py_str_contains(self.as_str(), item)
  }

  /// Support Unicode-character indexing and slicing.
  fn __getitem__(&self, index: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    py_str_getitem(self.as_str(), index)
  }

  /// Perform rich comparisons with native string semantics.
  fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<Py<PyAny>> {
    py_str_richcmp(self.as_str(), other, op)
  }

  #[allow(non_upper_case_globals)]
  const __hash__: Option<Py<PyAny>> = None;

  /// Append a character to the buffer.
  ///
  /// Args:
  ///     ch: The character to append.
  ///
  /// Raises:
  ///     ValueError: If there is not enough capacity.
  #[pyo3(name = "push")]
  fn __python_push(&mut self, ch: char) -> PyResult<()> {
    self
      .try_push(ch)
      .map_err(|e| PyValueError::new_err(e.to_string()))
  }

  /// Append a string to the buffer.
  ///
  /// Args:
  ///     s: The string to append.
  ///
  /// Raises:
  ///     ValueError: If there is not enough capacity.
  #[pyo3(name = "push_str")]
  fn __python_push_str(&mut self, s: &str) -> PyResult<()> {
    self
      .try_push_str(s)
      .map_err(|e| PyValueError::new_err(e.to_string()))
  }

  /// Clear the buffer.
  #[pyo3(name = "clear")]
  fn __python_clear(&mut self) {
    self.clear();
  }

  /// Truncate at a UTF-8 character boundary.
  #[pyo3(name = "truncate")]
  fn __python_truncate(&mut self, new_len: usize) -> PyResult<()> {
    self
      .try_truncate(new_len)
      .map_err(|error| PyValueError::new_err(error.to_string()))
  }

  /// Split the buffer at the given index, returning the head.
  ///
  /// Args:
  ///     at: The split index (must be on a character boundary).
  ///
  /// Returns:
  ///     Utf8Buffer: The content before the split point.
  ///
  /// Raises:
  ///     ValueError: If `at` is not on a character boundary or is out of bounds.
  #[pyo3(name = "split_to")]
  fn __python_split_to(&mut self, at: usize) -> PyResult<Self> {
    self
      .try_split_to(at)
      .map_err(|e| PyValueError::new_err(e.to_string()))
  }

  /// Split the buffer at the given index, returning the tail.
  ///
  /// Args:
  ///     at: The split index (must be on a character boundary).
  ///
  /// Returns:
  ///     Utf8Buffer: The content after the split point.
  ///
  /// Raises:
  ///     ValueError: If `at` is not on a character boundary or is out of bounds.
  #[pyo3(name = "split_off")]
  fn __python_split_off(&mut self, at: usize) -> PyResult<Self> {
    self
      .try_split_off(at)
      .map_err(|e| PyValueError::new_err(e.to_string()))
  }

  /// Return a sub-slice of the buffer.
  ///
  /// Args:
  ///     start: The start index (inclusive).
  ///     end: The end index (exclusive).
  ///
  /// Returns:
  ///     Utf8Buffer: A new buffer containing the specified range.
  ///
  /// Raises:
  ///     ValueError: If the range is not on character boundaries or is out of bounds.
  #[pyo3(name = "slice")]
  fn __python_slice(&self, start: usize, end: usize) -> PyResult<Self> {
    self
      .try_slice(start..end)
      .map_err(|e| PyValueError::new_err(e.to_string()))
  }

  /// Return the length of the buffer in bytes.
  #[pyo3(name = "len")]
  fn __python_len_method(&self) -> usize {
    self.len()
  }

  /// Return whether the buffer is empty.
  #[pyo3(name = "is_empty")]
  fn __python_is_empty(&self) -> bool {
    self.is_empty()
  }

  /// Return the capacity of the buffer.
  #[pyo3(name = "capacity")]
  fn __python_capacity(&self) -> usize {
    self.capacity()
  }

  /// Iterate over the characters of the buffer.
  fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<Utf8CharIter>> {
    let chars: Vec<char> = slf.as_str().chars().collect();
    Py::new(slf.py(), Utf8CharIter { chars, index: 0 })
  }

  /// Return the UTF-8 bytes as a Python `bytes` object.
  fn __bytes__<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    PyBytes::new(py, self.as_str().as_bytes())
  }

  /// Support copy.copy.
  #[pyo3(name = "__copy__")]
  fn __python_copy(&self) -> Self {
    *self
  }

  /// Support copy.deepcopy.
  #[pyo3(name = "__deepcopy__")]
  fn __python_deepcopy(&self, _memo: &Bound<'_, PyAny>) -> Self {
    *self
  }

  /// Support pickling via `pickle.dumps` / `pickle.loads`.
  fn __reduce__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<(Py<PyAny>, (String,))> {
    let cls = py.get_type::<Self>();
    let from_str = cls.getattr("from_str")?;
    Ok((from_str.unbind(), (slf.as_str().to_string(),)))
  }

  /// Return the number of bytes remaining for reading.
  ///
  /// Returns:
  ///     int: Number of bytes available for reading.
  #[pyo3(name = "remaining")]
  fn __python_remaining(&self) -> usize {
    self.inner.remaining()
  }

  /// Advance the read cursor by the specified number of bytes.
  ///
  /// Args:
  ///     cnt: Number of bytes to advance.
  ///
  /// Raises:
  ///     BufferError: If `cnt` exceeds available data or would end inside a UTF-8 character.
  #[pyo3(name = "advance")]
  fn __python_advance(&mut self, cnt: usize) -> PyResult<()> {
    if cnt > self.inner.remaining() {
      return Err(pyo3::exceptions::PyBufferError::new_err(format!(
        "cannot advance past remaining: {} > {}",
        cnt,
        self.inner.remaining()
      )));
    }
    crate::python::validate_utf8_advance(self, cnt)?;
    self.inner.advance(cnt);
    Ok(())
  }

  /// Read an unsigned 8-bit integer from the underlying bytes.
  ///
  /// Advances the read cursor by 1 byte.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 1 byte remains or consuming it would end inside a UTF-8 character.
  #[pyo3(name = "get_u8")]
  fn __python_get_u8(&mut self) -> PyResult<u8> {
    if self.inner.remaining() < 1 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u8",
      ));
    }
    crate::python::validate_utf8_advance(self, 1)?;
    Ok(self.inner.get_u8())
  }

  /// Read a signed 8-bit integer from the underlying bytes.
  ///
  /// Advances the read cursor by 1 byte.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 1 byte remains or consuming it would end inside a UTF-8 character.
  #[pyo3(name = "get_i8")]
  fn __python_get_i8(&mut self) -> PyResult<i8> {
    if self.inner.remaining() < 1 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i8",
      ));
    }
    crate::python::validate_utf8_advance(self, 1)?;
    Ok(self.inner.get_i8())
  }

  /// Read an unsigned 16-bit integer in big-endian byte order.
  ///
  /// Advances the read cursor by 2 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 2 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u16")]
  fn __python_get_u16(&mut self) -> PyResult<u16> {
    if self.inner.remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u16",
      ));
    }
    crate::python::validate_utf8_advance(self, 2)?;
    Ok(self.inner.get_u16())
  }

  /// Read an unsigned 16-bit integer in little-endian byte order.
  ///
  /// Advances the read cursor by 2 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 2 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u16_le")]
  fn __python_get_u16_le(&mut self) -> PyResult<u16> {
    if self.inner.remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u16",
      ));
    }
    crate::python::validate_utf8_advance(self, 2)?;
    Ok(self.inner.get_u16_le())
  }

  /// Read a signed 16-bit integer in big-endian byte order.
  ///
  /// Advances the read cursor by 2 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 2 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i16")]
  fn __python_get_i16(&mut self) -> PyResult<i16> {
    if self.inner.remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i16",
      ));
    }
    crate::python::validate_utf8_advance(self, 2)?;
    Ok(self.inner.get_i16())
  }

  /// Read a signed 16-bit integer in little-endian byte order.
  ///
  /// Advances the read cursor by 2 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 2 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i16_le")]
  fn __python_get_i16_le(&mut self) -> PyResult<i16> {
    if self.inner.remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i16",
      ));
    }
    crate::python::validate_utf8_advance(self, 2)?;
    Ok(self.inner.get_i16_le())
  }

  /// Read an unsigned 32-bit integer in big-endian byte order.
  ///
  /// Advances the read cursor by 4 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u32")]
  fn __python_get_u32(&mut self) -> PyResult<u32> {
    if self.inner.remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.get_u32())
  }

  /// Read an unsigned 32-bit integer in little-endian byte order.
  ///
  /// Advances the read cursor by 4 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u32_le")]
  fn __python_get_u32_le(&mut self) -> PyResult<u32> {
    if self.inner.remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.get_u32_le())
  }

  /// Read a signed 32-bit integer in big-endian byte order.
  ///
  /// Advances the read cursor by 4 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i32")]
  fn __python_get_i32(&mut self) -> PyResult<i32> {
    if self.inner.remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.get_i32())
  }

  /// Read a signed 32-bit integer in little-endian byte order.
  ///
  /// Advances the read cursor by 4 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i32_le")]
  fn __python_get_i32_le(&mut self) -> PyResult<i32> {
    if self.inner.remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.get_i32_le())
  }

  /// Read an unsigned 64-bit integer in big-endian byte order.
  ///
  /// Advances the read cursor by 8 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u64")]
  fn __python_get_u64(&mut self) -> PyResult<u64> {
    if self.inner.remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.get_u64())
  }

  /// Read an unsigned 64-bit integer in little-endian byte order.
  ///
  /// Advances the read cursor by 8 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u64_le")]
  fn __python_get_u64_le(&mut self) -> PyResult<u64> {
    if self.inner.remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.get_u64_le())
  }

  /// Read a signed 64-bit integer in big-endian byte order.
  ///
  /// Advances the read cursor by 8 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i64")]
  fn __python_get_i64(&mut self) -> PyResult<i64> {
    if self.inner.remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.get_i64())
  }

  /// Read a signed 64-bit integer in little-endian byte order.
  ///
  /// Advances the read cursor by 8 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i64_le")]
  fn __python_get_i64_le(&mut self) -> PyResult<i64> {
    if self.inner.remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.get_i64_le())
  }

  /// Read an unsigned 128-bit integer in big-endian byte order.
  ///
  /// Advances the read cursor by 16 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 16 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u128")]
  fn __python_get_u128(&mut self) -> PyResult<u128> {
    if self.inner.remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u128",
      ));
    }
    crate::python::validate_utf8_advance(self, 16)?;
    Ok(self.inner.get_u128())
  }

  /// Read an unsigned 128-bit integer in little-endian byte order.
  ///
  /// Advances the read cursor by 16 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 16 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u128_le")]
  fn __python_get_u128_le(&mut self) -> PyResult<u128> {
    if self.inner.remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u128",
      ));
    }
    crate::python::validate_utf8_advance(self, 16)?;
    Ok(self.inner.get_u128_le())
  }

  /// Read a signed 128-bit integer in big-endian byte order.
  ///
  /// Advances the read cursor by 16 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 16 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i128")]
  fn __python_get_i128(&mut self) -> PyResult<i128> {
    if self.inner.remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i128",
      ));
    }
    crate::python::validate_utf8_advance(self, 16)?;
    Ok(self.inner.get_i128())
  }

  /// Read a signed 128-bit integer in little-endian byte order.
  ///
  /// Advances the read cursor by 16 bytes.
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 16 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i128_le")]
  fn __python_get_i128_le(&mut self) -> PyResult<i128> {
    if self.inner.remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i128",
      ));
    }
    crate::python::validate_utf8_advance(self, 16)?;
    Ok(self.inner.get_i128_le())
  }

  /// Read a 32-bit floating point number in big-endian byte order.
  ///
  /// Advances the read cursor by 4 bytes.
  ///
  /// Returns:
  ///     float: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f32")]
  fn __python_get_f32(&mut self) -> PyResult<f32> {
    if self.inner.remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.get_f32())
  }

  /// Read a 32-bit floating point number in little-endian byte order.
  ///
  /// Advances the read cursor by 4 bytes.
  ///
  /// Returns:
  ///     float: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f32_le")]
  fn __python_get_f32_le(&mut self) -> PyResult<f32> {
    if self.inner.remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.get_f32_le())
  }

  /// Read a 64-bit floating point number in big-endian byte order.
  ///
  /// Advances the read cursor by 8 bytes.
  ///
  /// Returns:
  ///     float: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f64")]
  fn __python_get_f64(&mut self) -> PyResult<f64> {
    if self.inner.remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.get_f64())
  }

  /// Read a 64-bit floating point number in little-endian byte order.
  ///
  /// Advances the read cursor by 8 bytes.
  ///
  /// Returns:
  ///     float: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f64_le")]
  fn __python_get_f64_le(&mut self) -> PyResult<f64> {
    if self.inner.remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.get_f64_le())
  }

  /// Read an unsigned integer spanning `nbytes` in big-endian byte order.
  ///
  /// Advances the read cursor by `nbytes` bytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (0-8).
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     ValueError: If `nbytes` is outside 0-8.
  ///     BufferError: If fewer than `nbytes` bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_uint")]
  fn __python_get_uint(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<u64> {
    let nbytes = crate::python::py_integer_width(nbytes)?;
    if self.inner.remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err("not enough data"));
    }
    crate::python::validate_utf8_advance(self, nbytes)?;
    Ok(self.inner.get_uint(nbytes))
  }

  /// Read an unsigned integer spanning `nbytes` in little-endian byte order.
  ///
  /// Advances the read cursor by `nbytes` bytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (0-8).
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     ValueError: If `nbytes` is outside 0-8.
  ///     BufferError: If fewer than `nbytes` bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_uint_le")]
  fn __python_get_uint_le(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<u64> {
    let nbytes = crate::python::py_integer_width(nbytes)?;
    if self.inner.remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err("not enough data"));
    }
    crate::python::validate_utf8_advance(self, nbytes)?;
    Ok(self.inner.get_uint_le(nbytes))
  }

  /// Read a signed integer spanning `nbytes` in big-endian byte order.
  ///
  /// Advances the read cursor by `nbytes` bytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (0-8).
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     ValueError: If `nbytes` is outside 0-8.
  ///     BufferError: If fewer than `nbytes` bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_int")]
  fn __python_get_int(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<i64> {
    let nbytes = crate::python::py_integer_width(nbytes)?;
    if self.inner.remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err("not enough data"));
    }
    crate::python::validate_utf8_advance(self, nbytes)?;
    Ok(self.inner.get_int(nbytes))
  }

  /// Read a signed integer spanning `nbytes` in little-endian byte order.
  ///
  /// Advances the read cursor by `nbytes` bytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (0-8).
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     ValueError: If `nbytes` is outside 0-8.
  ///     BufferError: If fewer than `nbytes` bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_int_le")]
  fn __python_get_int_le(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<i64> {
    let nbytes = crate::python::py_integer_width(nbytes)?;
    if self.inner.remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err("not enough data"));
    }
    crate::python::validate_utf8_advance(self, nbytes)?;
    Ok(self.inner.get_int_le(nbytes))
  }
}
