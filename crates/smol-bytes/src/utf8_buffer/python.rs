use crate::python::PyBufExt as _;

use super::*;
use pyo3::{
  basic::CompareOp,
  exceptions::{PyIndexError, PyTypeError, PyValueError},
  prelude::{Bound, *},
  types::{PyBytes, PyString},
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

  /// Return the number of bytes in the buffer.
  fn __len__(&self) -> usize {
    self.len()
  }

  /// Return whether the buffer is non-empty.
  fn __bool__(&self) -> bool {
    !self.is_empty()
  }

  /// Check if a substring is contained in the buffer.
  fn __contains__(&self, item: &Bound<'_, PyAny>) -> bool {
    self.as_inner().py_contains(item)
  }

  /// Support indexing and slicing.
  fn __getitem__(&self, index: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let py = index.py();

    if let Ok(i) = index.extract::<isize>() {
      let chars: Vec<char> = self.as_str().chars().collect();
      let len = chars.len() as isize;
      let idx = if i < 0 { len + i } else { i };

      if idx < 0 || idx >= len {
        return Err(PyIndexError::new_err(format!(
          "string index out of range: {} (len={})",
          i, len
        )));
      }

      return Ok(
        chars[idx as usize]
          .to_string()
          .into_pyobject(py)?
          .into_any()
          .unbind(),
      );
    }

    if let Ok(slice) = index.cast::<pyo3::types::PySlice>() {
      let s = self.as_str();
      let indices = slice.indices(s.len() as isize)?;

      let start = indices.start.max(0) as usize;
      let stop = indices.stop.max(0).min(s.len() as isize) as usize;

      if indices.step == 1 {
        return Ok(PyString::new(py, &s[start..stop]).into());
      }

      // Handle step != 1
      let chars: Vec<char> = s.chars().collect();
      let mut result = String::new();

      if indices.step > 0 {
        let mut i = start;
        while i < stop && i < chars.len() {
          result.push(chars[i]);
          i += indices.step as usize;
        }
      } else if indices.step < 0 {
        let mut i = start.min(chars.len().saturating_sub(1));
        loop {
          result.push(chars[i]);
          if i == 0 || i < stop {
            break;
          }
          i = i.saturating_sub((-indices.step) as usize);
        }
      }

      return Ok(PyString::new(py, &result).into());
    }

    Err(PyTypeError::new_err("indices must be integers or slices"))
  }

  /// Perform rich comparisons with other strings.
  fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<bool> {
    if let Ok(other_self) = other.extract::<PyRef<'_, Self>>() {
      return Ok(match op {
        CompareOp::Lt => self < &*other_self,
        CompareOp::Le => self <= &*other_self,
        CompareOp::Eq => self == &*other_self,
        CompareOp::Ne => self != &*other_self,
        CompareOp::Gt => self > &*other_self,
        CompareOp::Ge => self >= &*other_self,
      });
    }

    if let Ok(s) = other.extract::<String>() {
      let s = s.as_str();
      return Ok(match op {
        CompareOp::Lt => self.as_str() < s,
        CompareOp::Le => self.as_str() <= s,
        CompareOp::Eq => self.as_str() == s,
        CompareOp::Ne => self.as_str() != s,
        CompareOp::Gt => self.as_str() > s,
        CompareOp::Ge => self.as_str() >= s,
      });
    }

    match op {
      CompareOp::Eq => Ok(false),
      CompareOp::Ne => Ok(true),
      _ => Err(PyTypeError::new_err(format!(
        "'>=' not supported between instances of 'Utf8Buffer' and '{}'",
        other.get_type().name()?
      ))),
    }
  }

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
  ///     BufferError: If trying to advance beyond available data.
  #[pyo3(name = "advance")]
  fn __python_advance(&mut self, cnt: usize) -> PyResult<()> {
    if cnt > self.inner.remaining() {
      return Err(pyo3::exceptions::PyBufferError::new_err(format!(
        "cannot advance past remaining: {} > {}",
        cnt,
        self.inner.remaining()
      )));
    }
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
  ///     BufferError: If fewer than 1 byte remains.
  #[pyo3(name = "get_u8")]
  fn __python_get_u8(&mut self) -> PyResult<u8> {
    if self.inner.remaining() < 1 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u8",
      ));
    }
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
  ///     BufferError: If fewer than 1 byte remains.
  #[pyo3(name = "get_i8")]
  fn __python_get_i8(&mut self) -> PyResult<i8> {
    if self.inner.remaining() < 1 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i8",
      ));
    }
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
  ///     BufferError: If fewer than 2 bytes remain.
  #[pyo3(name = "get_u16")]
  fn __python_get_u16(&mut self) -> PyResult<u16> {
    if self.inner.remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u16",
      ));
    }
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
  ///     BufferError: If fewer than 2 bytes remain.
  #[pyo3(name = "get_u16_le")]
  fn __python_get_u16_le(&mut self) -> PyResult<u16> {
    if self.inner.remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u16",
      ));
    }
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
  ///     BufferError: If fewer than 2 bytes remain.
  #[pyo3(name = "get_i16")]
  fn __python_get_i16(&mut self) -> PyResult<i16> {
    if self.inner.remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i16",
      ));
    }
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
  ///     BufferError: If fewer than 2 bytes remain.
  #[pyo3(name = "get_i16_le")]
  fn __python_get_i16_le(&mut self) -> PyResult<i16> {
    if self.inner.remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i16",
      ));
    }
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
  ///     BufferError: If fewer than 4 bytes remain.
  #[pyo3(name = "get_u32")]
  fn __python_get_u32(&mut self) -> PyResult<u32> {
    if self.inner.remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u32",
      ));
    }
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
  ///     BufferError: If fewer than 4 bytes remain.
  #[pyo3(name = "get_u32_le")]
  fn __python_get_u32_le(&mut self) -> PyResult<u32> {
    if self.inner.remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u32",
      ));
    }
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
  ///     BufferError: If fewer than 4 bytes remain.
  #[pyo3(name = "get_i32")]
  fn __python_get_i32(&mut self) -> PyResult<i32> {
    if self.inner.remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i32",
      ));
    }
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
  ///     BufferError: If fewer than 4 bytes remain.
  #[pyo3(name = "get_i32_le")]
  fn __python_get_i32_le(&mut self) -> PyResult<i32> {
    if self.inner.remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i32",
      ));
    }
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
  ///     BufferError: If fewer than 8 bytes remain.
  #[pyo3(name = "get_u64")]
  fn __python_get_u64(&mut self) -> PyResult<u64> {
    if self.inner.remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u64",
      ));
    }
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
  ///     BufferError: If fewer than 8 bytes remain.
  #[pyo3(name = "get_u64_le")]
  fn __python_get_u64_le(&mut self) -> PyResult<u64> {
    if self.inner.remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u64",
      ));
    }
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
  ///     BufferError: If fewer than 8 bytes remain.
  #[pyo3(name = "get_i64")]
  fn __python_get_i64(&mut self) -> PyResult<i64> {
    if self.inner.remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i64",
      ));
    }
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
  ///     BufferError: If fewer than 8 bytes remain.
  #[pyo3(name = "get_i64_le")]
  fn __python_get_i64_le(&mut self) -> PyResult<i64> {
    if self.inner.remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i64",
      ));
    }
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
  ///     BufferError: If fewer than 16 bytes remain.
  #[pyo3(name = "get_u128")]
  fn __python_get_u128(&mut self) -> PyResult<u128> {
    if self.inner.remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u128",
      ));
    }
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
  ///     BufferError: If fewer than 16 bytes remain.
  #[pyo3(name = "get_u128_le")]
  fn __python_get_u128_le(&mut self) -> PyResult<u128> {
    if self.inner.remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u128",
      ));
    }
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
  ///     BufferError: If fewer than 16 bytes remain.
  #[pyo3(name = "get_i128")]
  fn __python_get_i128(&mut self) -> PyResult<i128> {
    if self.inner.remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i128",
      ));
    }
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
  ///     BufferError: If fewer than 16 bytes remain.
  #[pyo3(name = "get_i128_le")]
  fn __python_get_i128_le(&mut self) -> PyResult<i128> {
    if self.inner.remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i128",
      ));
    }
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
  ///     BufferError: If fewer than 4 bytes remain.
  #[pyo3(name = "get_f32")]
  fn __python_get_f32(&mut self) -> PyResult<f32> {
    if self.inner.remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f32",
      ));
    }
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
  ///     BufferError: If fewer than 4 bytes remain.
  #[pyo3(name = "get_f32_le")]
  fn __python_get_f32_le(&mut self) -> PyResult<f32> {
    if self.inner.remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f32",
      ));
    }
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
  ///     BufferError: If fewer than 8 bytes remain.
  #[pyo3(name = "get_f64")]
  fn __python_get_f64(&mut self) -> PyResult<f64> {
    if self.inner.remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f64",
      ));
    }
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
  ///     BufferError: If fewer than 8 bytes remain.
  #[pyo3(name = "get_f64_le")]
  fn __python_get_f64_le(&mut self) -> PyResult<f64> {
    if self.inner.remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f64",
      ));
    }
    Ok(self.inner.get_f64_le())
  }

  /// Read an unsigned integer spanning `nbytes` in big-endian byte order.
  ///
  /// Advances the read cursor by `nbytes` bytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (1-8).
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than `nbytes` bytes remain or `nbytes` > 8.
  #[pyo3(name = "get_uint")]
  fn __python_get_uint(&mut self, nbytes: usize) -> PyResult<u64> {
    if nbytes > 8 || self.inner.remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data or nbytes > 8",
      ));
    }
    Ok(self.inner.get_uint(nbytes))
  }

  /// Read an unsigned integer spanning `nbytes` in little-endian byte order.
  ///
  /// Advances the read cursor by `nbytes` bytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (1-8).
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than `nbytes` bytes remain or `nbytes` > 8.
  #[pyo3(name = "get_uint_le")]
  fn __python_get_uint_le(&mut self, nbytes: usize) -> PyResult<u64> {
    if nbytes > 8 || self.inner.remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data or nbytes > 8",
      ));
    }
    Ok(self.inner.get_uint_le(nbytes))
  }

  /// Read a signed integer spanning `nbytes` in big-endian byte order.
  ///
  /// Advances the read cursor by `nbytes` bytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (1-8).
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than `nbytes` bytes remain or `nbytes` > 8.
  #[pyo3(name = "get_int")]
  fn __python_get_int(&mut self, nbytes: usize) -> PyResult<i64> {
    if nbytes > 8 || self.inner.remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data or nbytes > 8",
      ));
    }
    Ok(self.inner.get_int(nbytes))
  }

  /// Read a signed integer spanning `nbytes` in little-endian byte order.
  ///
  /// Advances the read cursor by `nbytes` bytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (1-8).
  ///
  /// Returns:
  ///     int: The decoded value.
  ///
  /// Raises:
  ///     BufferError: If fewer than `nbytes` bytes remain or `nbytes` > 8.
  #[pyo3(name = "get_int_le")]
  fn __python_get_int_le(&mut self, nbytes: usize) -> PyResult<i64> {
    if nbytes > 8 || self.inner.remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data or nbytes > 8",
      ));
    }
    Ok(self.inner.get_int_le(nbytes))
  }
}
