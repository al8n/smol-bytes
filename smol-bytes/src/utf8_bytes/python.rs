use pyo3::{
  basic::CompareOp,
  exceptions::{PyIndexError, PyTypeError, PyValueError},
  prelude::{Bound, *},
  types::{PyBytes, PyString},
};

use crate::bytes::strategy::compact::Compact;
use crate::bytes::strategy::shared::Shared;

/// Concrete shared Utf8Bytes type for Python bindings.
type SharedUtf8Bytes = super::Utf8Bytes<Shared>;

/// Concrete compact Utf8Bytes type for Python bindings.
type CompactUtf8Bytes = super::Utf8Bytes<Compact>;

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

/// Python bindings for the [`Utf8Bytes`](crate::Utf8Bytes) type (shared variant).
#[pyclass(name = "Utf8Bytes", skip_from_py_object)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PySharedUtf8Bytes {
  inner: SharedUtf8Bytes,
}

impl From<SharedUtf8Bytes> for PySharedUtf8Bytes {
  fn from(inner: SharedUtf8Bytes) -> Self {
    Self { inner }
  }
}

impl AsRef<str> for PySharedUtf8Bytes {
  fn as_ref(&self) -> &str {
    self.inner.as_str()
  }
}

impl AsRef<[u8]> for PySharedUtf8Bytes {
  fn as_ref(&self) -> &[u8] {
    self.inner.as_str().as_bytes()
  }
}

#[pymethods]
impl PySharedUtf8Bytes {
  #[new]
  fn new_python() -> Self {
    Self {
      inner: SharedUtf8Bytes::new(),
    }
  }

  /// Create from a static string.
  ///
  /// Args:
  ///     s: A string to create the bytes from.
  ///
  /// Returns:
  ///     Utf8Bytes: A new immutable UTF-8 bytes object.
  #[staticmethod]
  #[pyo3(name = "from_str")]
  fn __python_from_str(s: &str) -> Self {
    Self {
      inner: SharedUtf8Bytes::from(s),
    }
  }

  /// Return the contents as a Python string.
  fn __str__(&self) -> &str {
    self.inner.as_str()
  }

  /// Return a debug representation.
  fn __repr__(&self) -> String {
    format!("{:?}", self.inner)
  }

  /// Return the number of bytes.
  fn __len__(&self) -> usize {
    self.inner.len()
  }

  /// Return whether the bytes are non-empty.
  fn __bool__(&self) -> bool {
    !self.inner.is_empty()
  }

  /// Compute a hash from the underlying UTF-8 string content.
  fn __hash__(&self) -> u64 {
    use core::hash::{Hash, Hasher};
    let mut hasher = crate::DefaultHasher::new();
    self.inner.as_str().hash(&mut hasher);
    hasher.finish()
  }

  /// Check if a substring is contained.
  fn __contains__(&self, item: &Bound<'_, PyAny>) -> PyResult<bool> {
    if let Ok(s) = item.extract::<String>() {
      return Ok(self.inner.as_str().contains(s.as_str()));
    }
    Err(PyTypeError::new_err("argument must be a string"))
  }

  /// Support indexing and slicing.
  fn __getitem__(&self, index: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let py = index.py();

    if let Ok(i) = index.extract::<isize>() {
      let chars: Vec<char> = self.inner.as_str().chars().collect();
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
      let s = self.inner.as_str();
      let indices = slice.indices(s.len() as isize)?;

      let start = indices.start.max(0) as usize;
      let stop = indices.stop.max(0).min(s.len() as isize) as usize;

      if indices.step == 1 {
        return Ok(PyString::new(py, &s[start..stop]).into());
      }

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

  /// Perform rich comparisons.
  fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<bool> {
    if let Ok(other_self) = other.extract::<PyRef<'_, Self>>() {
      return Ok(match op {
        CompareOp::Lt => self.inner < other_self.inner,
        CompareOp::Le => self.inner <= other_self.inner,
        CompareOp::Eq => self.inner == other_self.inner,
        CompareOp::Ne => self.inner != other_self.inner,
        CompareOp::Gt => self.inner > other_self.inner,
        CompareOp::Ge => self.inner >= other_self.inner,
      });
    }

    if let Ok(s) = other.extract::<String>() {
      let s = s.as_str();
      return Ok(match op {
        CompareOp::Lt => self.inner.as_str() < s,
        CompareOp::Le => self.inner.as_str() <= s,
        CompareOp::Eq => self.inner.as_str() == s,
        CompareOp::Ne => self.inner.as_str() != s,
        CompareOp::Gt => self.inner.as_str() > s,
        CompareOp::Ge => self.inner.as_str() >= s,
      });
    }

    match op {
      CompareOp::Eq => Ok(false),
      CompareOp::Ne => Ok(true),
      _ => Err(PyTypeError::new_err(format!(
        "'>=' not supported between instances of 'Utf8Bytes' and '{}'",
        other.get_type().name()?
      ))),
    }
  }

  /// Split at the given index.
  ///
  /// Args:
  ///     at: The split index (must be on a character boundary).
  ///
  /// Returns:
  ///     Utf8Bytes: The content before the split point.
  ///
  /// Raises:
  ///     ValueError: If `at` is not on a character boundary or is out of bounds.
  #[pyo3(name = "split_to")]
  fn __python_split_to(&mut self, at: usize) -> PyResult<Self> {
    self
      .inner
      .try_split_to(at)
      .map(Into::into)
      .map_err(|e| PyValueError::new_err(e.to_string()))
  }

  /// Split at the given index, returning the tail.
  ///
  /// Args:
  ///     at: The split index (must be on a character boundary).
  ///
  /// Returns:
  ///     Utf8Bytes: The content after the split point.
  ///
  /// Raises:
  ///     ValueError: If `at` is not on a character boundary or is out of bounds.
  #[pyo3(name = "split_off")]
  fn __python_split_off(&mut self, at: usize) -> PyResult<Self> {
    self
      .inner
      .try_split_off(at)
      .map(Into::into)
      .map_err(|e| PyValueError::new_err(e.to_string()))
  }

  /// Return a sub-slice of the bytes.
  ///
  /// Args:
  ///     start: The start index (inclusive).
  ///     end: The end index (exclusive).
  ///
  /// Returns:
  ///     Utf8Bytes: A new bytes object containing the specified range.
  ///
  /// Raises:
  ///     ValueError: If the range is not on character boundaries or is out of bounds.
  #[pyo3(name = "slice")]
  fn __python_slice(&self, start: usize, end: usize) -> PyResult<Self> {
    self
      .inner
      .try_slice(start..end)
      .map(Into::into)
      .map_err(|e| PyValueError::new_err(e.to_string()))
  }

  /// Iterate over the characters of the bytes.
  fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<Utf8CharIter>> {
    let chars: Vec<char> = slf.inner.as_str().chars().collect();
    Py::new(slf.py(), Utf8CharIter { chars, index: 0 })
  }

  /// Return the UTF-8 bytes as a Python `bytes` object.
  fn __bytes__<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    PyBytes::new(py, self.inner.as_str().as_bytes())
  }

  /// Return whether the bytes are stored inline.
  #[pyo3(name = "is_inline")]
  fn __python_is_inline(&self) -> bool {
    self.inner.is_inline()
  }

  /// Return whether the bytes are stored on the heap.
  #[pyo3(name = "is_heap")]
  fn __python_is_heap(&self) -> bool {
    self.inner.is_heap()
  }

  /// Support copy.copy.
  #[pyo3(name = "__copy__")]
  fn __python_copy(&self) -> Self {
    self.clone()
  }

  /// Support copy.deepcopy.
  #[pyo3(name = "__deepcopy__")]
  fn __python_deepcopy(&self, _memo: &Bound<'_, PyAny>) -> Self {
    self.clone()
  }

  /// Support pickling via `pickle.dumps` / `pickle.loads`.
  fn __reduce__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<(Py<PyAny>, (String,))> {
    let cls = py.get_type::<Self>();
    let from_str = cls.getattr("from_str")?;
    Ok((from_str.unbind(), (slf.inner.as_str().to_string(),)))
  }

  /// Return the number of bytes remaining for reading.
  ///
  /// Returns:
  ///     int: Number of bytes available for reading.
  #[pyo3(name = "remaining")]
  fn __python_remaining(&self) -> usize {
    use bytes::Buf;
    self.inner.as_inner().remaining()
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
    use bytes::Buf;
    if cnt > self.inner.as_inner().remaining() {
      return Err(pyo3::exceptions::PyBufferError::new_err(format!(
        "cannot advance past remaining: {} > {}",
        cnt,
        self.inner.as_inner().remaining()
      )));
    }
    crate::python::validate_utf8_advance(self, cnt)?;
    self.inner.inner.advance(cnt);
    Ok(())
  }

  /// Read an unsigned 8-bit integer from the underlying bytes.
  ///
  /// Raises:
  ///     BufferError: If fewer than 1 byte remains or consuming it would end inside a UTF-8 character.
  #[pyo3(name = "get_u8")]
  fn __python_get_u8(&mut self) -> PyResult<u8> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 1 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u8",
      ));
    }
    crate::python::validate_utf8_advance(self, 1)?;
    Ok(self.inner.inner.get_u8())
  }

  /// Read a signed 8-bit integer from the underlying bytes.
  ///
  /// Raises:
  ///     BufferError: If fewer than 1 byte remains or consuming it would end inside a UTF-8 character.
  #[pyo3(name = "get_i8")]
  fn __python_get_i8(&mut self) -> PyResult<i8> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 1 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i8",
      ));
    }
    crate::python::validate_utf8_advance(self, 1)?;
    Ok(self.inner.inner.get_i8())
  }

  /// Read an unsigned 16-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 2 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u16")]
  fn __python_get_u16(&mut self) -> PyResult<u16> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u16",
      ));
    }
    crate::python::validate_utf8_advance(self, 2)?;
    Ok(self.inner.inner.get_u16())
  }

  /// Read an unsigned 16-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 2 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u16_le")]
  fn __python_get_u16_le(&mut self) -> PyResult<u16> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u16",
      ));
    }
    crate::python::validate_utf8_advance(self, 2)?;
    Ok(self.inner.inner.get_u16_le())
  }

  /// Read a signed 16-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 2 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i16")]
  fn __python_get_i16(&mut self) -> PyResult<i16> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i16",
      ));
    }
    crate::python::validate_utf8_advance(self, 2)?;
    Ok(self.inner.inner.get_i16())
  }

  /// Read a signed 16-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 2 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i16_le")]
  fn __python_get_i16_le(&mut self) -> PyResult<i16> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i16",
      ));
    }
    crate::python::validate_utf8_advance(self, 2)?;
    Ok(self.inner.inner.get_i16_le())
  }

  /// Read an unsigned 32-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u32")]
  fn __python_get_u32(&mut self) -> PyResult<u32> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.inner.get_u32())
  }

  /// Read an unsigned 32-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u32_le")]
  fn __python_get_u32_le(&mut self) -> PyResult<u32> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.inner.get_u32_le())
  }

  /// Read a signed 32-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i32")]
  fn __python_get_i32(&mut self) -> PyResult<i32> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.inner.get_i32())
  }

  /// Read a signed 32-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i32_le")]
  fn __python_get_i32_le(&mut self) -> PyResult<i32> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.inner.get_i32_le())
  }

  /// Read an unsigned 64-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u64")]
  fn __python_get_u64(&mut self) -> PyResult<u64> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.inner.get_u64())
  }

  /// Read an unsigned 64-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u64_le")]
  fn __python_get_u64_le(&mut self) -> PyResult<u64> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.inner.get_u64_le())
  }

  /// Read a signed 64-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i64")]
  fn __python_get_i64(&mut self) -> PyResult<i64> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.inner.get_i64())
  }

  /// Read a signed 64-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i64_le")]
  fn __python_get_i64_le(&mut self) -> PyResult<i64> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.inner.get_i64_le())
  }

  /// Read an unsigned 128-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 16 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u128")]
  fn __python_get_u128(&mut self) -> PyResult<u128> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u128",
      ));
    }
    crate::python::validate_utf8_advance(self, 16)?;
    Ok(self.inner.inner.get_u128())
  }

  /// Read an unsigned 128-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 16 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u128_le")]
  fn __python_get_u128_le(&mut self) -> PyResult<u128> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u128",
      ));
    }
    crate::python::validate_utf8_advance(self, 16)?;
    Ok(self.inner.inner.get_u128_le())
  }

  /// Read a signed 128-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 16 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i128")]
  fn __python_get_i128(&mut self) -> PyResult<i128> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i128",
      ));
    }
    crate::python::validate_utf8_advance(self, 16)?;
    Ok(self.inner.inner.get_i128())
  }

  /// Read a signed 128-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 16 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i128_le")]
  fn __python_get_i128_le(&mut self) -> PyResult<i128> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i128",
      ));
    }
    crate::python::validate_utf8_advance(self, 16)?;
    Ok(self.inner.inner.get_i128_le())
  }

  /// Read a 32-bit floating point number in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f32")]
  fn __python_get_f32(&mut self) -> PyResult<f32> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.inner.get_f32())
  }

  /// Read a 32-bit floating point number in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f32_le")]
  fn __python_get_f32_le(&mut self) -> PyResult<f32> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.inner.get_f32_le())
  }

  /// Read a 64-bit floating point number in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f64")]
  fn __python_get_f64(&mut self) -> PyResult<f64> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.inner.get_f64())
  }

  /// Read a 64-bit floating point number in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f64_le")]
  fn __python_get_f64_le(&mut self) -> PyResult<f64> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.inner.get_f64_le())
  }

  /// Read an unsigned integer spanning `nbytes` in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than `nbytes` bytes remain, `nbytes` > 8, or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_uint")]
  fn __python_get_uint(&mut self, nbytes: usize) -> PyResult<u64> {
    use bytes::Buf;
    if nbytes > 8 || self.inner.as_inner().remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data or nbytes > 8",
      ));
    }
    crate::python::validate_utf8_advance(self, nbytes)?;
    Ok(self.inner.inner.get_uint(nbytes))
  }

  /// Read an unsigned integer spanning `nbytes` in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than `nbytes` bytes remain, `nbytes` > 8, or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_uint_le")]
  fn __python_get_uint_le(&mut self, nbytes: usize) -> PyResult<u64> {
    use bytes::Buf;
    if nbytes > 8 || self.inner.as_inner().remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data or nbytes > 8",
      ));
    }
    crate::python::validate_utf8_advance(self, nbytes)?;
    Ok(self.inner.inner.get_uint_le(nbytes))
  }

  /// Read a signed integer spanning `nbytes` in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than `nbytes` bytes remain, `nbytes` > 8, or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_int")]
  fn __python_get_int(&mut self, nbytes: usize) -> PyResult<i64> {
    use bytes::Buf;
    if nbytes > 8 || self.inner.as_inner().remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data or nbytes > 8",
      ));
    }
    crate::python::validate_utf8_advance(self, nbytes)?;
    Ok(self.inner.inner.get_int(nbytes))
  }

  /// Read a signed integer spanning `nbytes` in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than `nbytes` bytes remain, `nbytes` > 8, or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_int_le")]
  fn __python_get_int_le(&mut self, nbytes: usize) -> PyResult<i64> {
    use bytes::Buf;
    if nbytes > 8 || self.inner.as_inner().remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data or nbytes > 8",
      ));
    }
    crate::python::validate_utf8_advance(self, nbytes)?;
    Ok(self.inner.inner.get_int_le(nbytes))
  }
}

/// Python bindings for the [`Utf8Bytes`](crate::Utf8Bytes) type (compact variant).
#[pyclass(name = "Utf8Bytes", skip_from_py_object)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PyCompactUtf8Bytes {
  inner: CompactUtf8Bytes,
}

impl From<CompactUtf8Bytes> for PyCompactUtf8Bytes {
  fn from(inner: CompactUtf8Bytes) -> Self {
    Self { inner }
  }
}

impl AsRef<str> for PyCompactUtf8Bytes {
  fn as_ref(&self) -> &str {
    self.inner.as_str()
  }
}

impl AsRef<[u8]> for PyCompactUtf8Bytes {
  fn as_ref(&self) -> &[u8] {
    self.inner.as_str().as_bytes()
  }
}

#[pymethods]
impl PyCompactUtf8Bytes {
  #[new]
  fn new_python() -> Self {
    Self {
      inner: CompactUtf8Bytes::new(),
    }
  }

  /// Create from a static string.
  ///
  /// Args:
  ///     s: A string to create the bytes from.
  ///
  /// Returns:
  ///     Utf8Bytes: A new immutable UTF-8 bytes object.
  #[staticmethod]
  #[pyo3(name = "from_str")]
  fn __python_from_str(s: &str) -> Self {
    Self {
      inner: CompactUtf8Bytes::from(s),
    }
  }

  /// Return the contents as a Python string.
  fn __str__(&self) -> &str {
    self.inner.as_str()
  }

  /// Return a debug representation.
  fn __repr__(&self) -> String {
    format!("{:?}", self.inner)
  }

  /// Return the number of bytes.
  fn __len__(&self) -> usize {
    self.inner.len()
  }

  /// Return whether the bytes are non-empty.
  fn __bool__(&self) -> bool {
    !self.inner.is_empty()
  }

  /// Compute a hash from the underlying UTF-8 string content.
  fn __hash__(&self) -> u64 {
    use core::hash::{Hash, Hasher};
    let mut hasher = crate::DefaultHasher::new();
    self.inner.as_str().hash(&mut hasher);
    hasher.finish()
  }

  /// Check if a substring is contained.
  fn __contains__(&self, item: &Bound<'_, PyAny>) -> PyResult<bool> {
    if let Ok(s) = item.extract::<String>() {
      return Ok(self.inner.as_str().contains(s.as_str()));
    }
    Err(PyTypeError::new_err("argument must be a string"))
  }

  /// Support indexing and slicing.
  fn __getitem__(&self, index: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let py = index.py();

    if let Ok(i) = index.extract::<isize>() {
      let chars: Vec<char> = self.inner.as_str().chars().collect();
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
      let s = self.inner.as_str();
      let indices = slice.indices(s.len() as isize)?;

      let start = indices.start.max(0) as usize;
      let stop = indices.stop.max(0).min(s.len() as isize) as usize;

      if indices.step == 1 {
        return Ok(PyString::new(py, &s[start..stop]).into());
      }

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

  /// Perform rich comparisons.
  fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<bool> {
    if let Ok(other_self) = other.extract::<PyRef<'_, Self>>() {
      return Ok(match op {
        CompareOp::Lt => self.inner < other_self.inner,
        CompareOp::Le => self.inner <= other_self.inner,
        CompareOp::Eq => self.inner == other_self.inner,
        CompareOp::Ne => self.inner != other_self.inner,
        CompareOp::Gt => self.inner > other_self.inner,
        CompareOp::Ge => self.inner >= other_self.inner,
      });
    }

    if let Ok(s) = other.extract::<String>() {
      let s = s.as_str();
      return Ok(match op {
        CompareOp::Lt => self.inner.as_str() < s,
        CompareOp::Le => self.inner.as_str() <= s,
        CompareOp::Eq => self.inner.as_str() == s,
        CompareOp::Ne => self.inner.as_str() != s,
        CompareOp::Gt => self.inner.as_str() > s,
        CompareOp::Ge => self.inner.as_str() >= s,
      });
    }

    match op {
      CompareOp::Eq => Ok(false),
      CompareOp::Ne => Ok(true),
      _ => Err(PyTypeError::new_err(format!(
        "'>=' not supported between instances of 'Utf8Bytes' and '{}'",
        other.get_type().name()?
      ))),
    }
  }

  /// Split at the given index.
  ///
  /// Args:
  ///     at: The split index (must be on a character boundary).
  ///
  /// Returns:
  ///     Utf8Bytes: The content before the split point.
  ///
  /// Raises:
  ///     ValueError: If `at` is not on a character boundary or is out of bounds.
  #[pyo3(name = "split_to")]
  fn __python_split_to(&mut self, at: usize) -> PyResult<Self> {
    self
      .inner
      .try_split_to(at)
      .map(Into::into)
      .map_err(|e| PyValueError::new_err(e.to_string()))
  }

  /// Split at the given index, returning the tail.
  ///
  /// Args:
  ///     at: The split index (must be on a character boundary).
  ///
  /// Returns:
  ///     Utf8Bytes: The content after the split point.
  ///
  /// Raises:
  ///     ValueError: If `at` is not on a character boundary or is out of bounds.
  #[pyo3(name = "split_off")]
  fn __python_split_off(&mut self, at: usize) -> PyResult<Self> {
    self
      .inner
      .try_split_off(at)
      .map(Into::into)
      .map_err(|e| PyValueError::new_err(e.to_string()))
  }

  /// Return a sub-slice of the bytes.
  ///
  /// Args:
  ///     start: The start index (inclusive).
  ///     end: The end index (exclusive).
  ///
  /// Returns:
  ///     Utf8Bytes: A new bytes object containing the specified range.
  ///
  /// Raises:
  ///     ValueError: If the range is not on character boundaries or is out of bounds.
  #[pyo3(name = "slice")]
  fn __python_slice(&self, start: usize, end: usize) -> PyResult<Self> {
    self
      .inner
      .try_slice(start..end)
      .map(Into::into)
      .map_err(|e| PyValueError::new_err(e.to_string()))
  }

  /// Iterate over the characters of the bytes.
  fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<Utf8CharIter>> {
    let chars: Vec<char> = slf.inner.as_str().chars().collect();
    Py::new(slf.py(), Utf8CharIter { chars, index: 0 })
  }

  /// Return the UTF-8 bytes as a Python `bytes` object.
  fn __bytes__<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    PyBytes::new(py, self.inner.as_str().as_bytes())
  }

  /// Return whether the bytes are stored inline.
  #[pyo3(name = "is_inline")]
  fn __python_is_inline(&self) -> bool {
    self.inner.is_inline()
  }

  /// Return whether the bytes are stored on the heap.
  #[pyo3(name = "is_heap")]
  fn __python_is_heap(&self) -> bool {
    self.inner.is_heap()
  }

  /// Support copy.copy.
  #[pyo3(name = "__copy__")]
  fn __python_copy(&self) -> Self {
    self.clone()
  }

  /// Support copy.deepcopy.
  #[pyo3(name = "__deepcopy__")]
  fn __python_deepcopy(&self, _memo: &Bound<'_, PyAny>) -> Self {
    self.clone()
  }

  /// Support pickling via `pickle.dumps` / `pickle.loads`.
  fn __reduce__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<(Py<PyAny>, (String,))> {
    let cls = py.get_type::<Self>();
    let from_str = cls.getattr("from_str")?;
    Ok((from_str.unbind(), (slf.inner.as_str().to_string(),)))
  }

  /// Return the number of bytes remaining for reading.
  ///
  /// Returns:
  ///     int: Number of bytes available for reading.
  #[pyo3(name = "remaining")]
  fn __python_remaining(&self) -> usize {
    use bytes::Buf;
    self.inner.as_inner().remaining()
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
    use bytes::Buf;
    if cnt > self.inner.as_inner().remaining() {
      return Err(pyo3::exceptions::PyBufferError::new_err(format!(
        "cannot advance past remaining: {} > {}",
        cnt,
        self.inner.as_inner().remaining()
      )));
    }
    crate::python::validate_utf8_advance(self, cnt)?;
    self.inner.inner.advance(cnt);
    Ok(())
  }

  /// Read an unsigned 8-bit integer from the underlying bytes.
  ///
  /// Raises:
  ///     BufferError: If fewer than 1 byte remains or consuming it would end inside a UTF-8 character.
  #[pyo3(name = "get_u8")]
  fn __python_get_u8(&mut self) -> PyResult<u8> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 1 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u8",
      ));
    }
    crate::python::validate_utf8_advance(self, 1)?;
    Ok(self.inner.inner.get_u8())
  }

  /// Read a signed 8-bit integer from the underlying bytes.
  ///
  /// Raises:
  ///     BufferError: If fewer than 1 byte remains or consuming it would end inside a UTF-8 character.
  #[pyo3(name = "get_i8")]
  fn __python_get_i8(&mut self) -> PyResult<i8> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 1 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i8",
      ));
    }
    crate::python::validate_utf8_advance(self, 1)?;
    Ok(self.inner.inner.get_i8())
  }

  /// Read an unsigned 16-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 2 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u16")]
  fn __python_get_u16(&mut self) -> PyResult<u16> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u16",
      ));
    }
    crate::python::validate_utf8_advance(self, 2)?;
    Ok(self.inner.inner.get_u16())
  }

  /// Read an unsigned 16-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 2 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u16_le")]
  fn __python_get_u16_le(&mut self) -> PyResult<u16> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u16",
      ));
    }
    crate::python::validate_utf8_advance(self, 2)?;
    Ok(self.inner.inner.get_u16_le())
  }

  /// Read a signed 16-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 2 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i16")]
  fn __python_get_i16(&mut self) -> PyResult<i16> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i16",
      ));
    }
    crate::python::validate_utf8_advance(self, 2)?;
    Ok(self.inner.inner.get_i16())
  }

  /// Read a signed 16-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 2 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i16_le")]
  fn __python_get_i16_le(&mut self) -> PyResult<i16> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 2 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i16",
      ));
    }
    crate::python::validate_utf8_advance(self, 2)?;
    Ok(self.inner.inner.get_i16_le())
  }

  /// Read an unsigned 32-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u32")]
  fn __python_get_u32(&mut self) -> PyResult<u32> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.inner.get_u32())
  }

  /// Read an unsigned 32-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u32_le")]
  fn __python_get_u32_le(&mut self) -> PyResult<u32> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.inner.get_u32_le())
  }

  /// Read a signed 32-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i32")]
  fn __python_get_i32(&mut self) -> PyResult<i32> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.inner.get_i32())
  }

  /// Read a signed 32-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i32_le")]
  fn __python_get_i32_le(&mut self) -> PyResult<i32> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.inner.get_i32_le())
  }

  /// Read an unsigned 64-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u64")]
  fn __python_get_u64(&mut self) -> PyResult<u64> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.inner.get_u64())
  }

  /// Read an unsigned 64-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u64_le")]
  fn __python_get_u64_le(&mut self) -> PyResult<u64> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.inner.get_u64_le())
  }

  /// Read a signed 64-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i64")]
  fn __python_get_i64(&mut self) -> PyResult<i64> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.inner.get_i64())
  }

  /// Read a signed 64-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i64_le")]
  fn __python_get_i64_le(&mut self) -> PyResult<i64> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.inner.get_i64_le())
  }

  /// Read an unsigned 128-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 16 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u128")]
  fn __python_get_u128(&mut self) -> PyResult<u128> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u128",
      ));
    }
    crate::python::validate_utf8_advance(self, 16)?;
    Ok(self.inner.inner.get_u128())
  }

  /// Read an unsigned 128-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 16 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u128_le")]
  fn __python_get_u128_le(&mut self) -> PyResult<u128> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for u128",
      ));
    }
    crate::python::validate_utf8_advance(self, 16)?;
    Ok(self.inner.inner.get_u128_le())
  }

  /// Read a signed 128-bit integer in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 16 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i128")]
  fn __python_get_i128(&mut self) -> PyResult<i128> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i128",
      ));
    }
    crate::python::validate_utf8_advance(self, 16)?;
    Ok(self.inner.inner.get_i128())
  }

  /// Read a signed 128-bit integer in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 16 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i128_le")]
  fn __python_get_i128_le(&mut self) -> PyResult<i128> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 16 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for i128",
      ));
    }
    crate::python::validate_utf8_advance(self, 16)?;
    Ok(self.inner.inner.get_i128_le())
  }

  /// Read a 32-bit floating point number in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f32")]
  fn __python_get_f32(&mut self) -> PyResult<f32> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.inner.get_f32())
  }

  /// Read a 32-bit floating point number in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 4 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f32_le")]
  fn __python_get_f32_le(&mut self) -> PyResult<f32> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 4 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f32",
      ));
    }
    crate::python::validate_utf8_advance(self, 4)?;
    Ok(self.inner.inner.get_f32_le())
  }

  /// Read a 64-bit floating point number in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f64")]
  fn __python_get_f64(&mut self) -> PyResult<f64> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.inner.get_f64())
  }

  /// Read a 64-bit floating point number in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than 8 bytes remain or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f64_le")]
  fn __python_get_f64_le(&mut self) -> PyResult<f64> {
    use bytes::Buf;
    if self.inner.as_inner().remaining() < 8 {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data for f64",
      ));
    }
    crate::python::validate_utf8_advance(self, 8)?;
    Ok(self.inner.inner.get_f64_le())
  }

  /// Read an unsigned integer spanning `nbytes` in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than `nbytes` bytes remain, `nbytes` > 8, or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_uint")]
  fn __python_get_uint(&mut self, nbytes: usize) -> PyResult<u64> {
    use bytes::Buf;
    if nbytes > 8 || self.inner.as_inner().remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data or nbytes > 8",
      ));
    }
    crate::python::validate_utf8_advance(self, nbytes)?;
    Ok(self.inner.inner.get_uint(nbytes))
  }

  /// Read an unsigned integer spanning `nbytes` in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than `nbytes` bytes remain, `nbytes` > 8, or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_uint_le")]
  fn __python_get_uint_le(&mut self, nbytes: usize) -> PyResult<u64> {
    use bytes::Buf;
    if nbytes > 8 || self.inner.as_inner().remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data or nbytes > 8",
      ));
    }
    crate::python::validate_utf8_advance(self, nbytes)?;
    Ok(self.inner.inner.get_uint_le(nbytes))
  }

  /// Read a signed integer spanning `nbytes` in big-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than `nbytes` bytes remain, `nbytes` > 8, or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_int")]
  fn __python_get_int(&mut self, nbytes: usize) -> PyResult<i64> {
    use bytes::Buf;
    if nbytes > 8 || self.inner.as_inner().remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data or nbytes > 8",
      ));
    }
    crate::python::validate_utf8_advance(self, nbytes)?;
    Ok(self.inner.inner.get_int(nbytes))
  }

  /// Read a signed integer spanning `nbytes` in little-endian byte order.
  ///
  /// Raises:
  ///     BufferError: If fewer than `nbytes` bytes remain, `nbytes` > 8, or consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_int_le")]
  fn __python_get_int_le(&mut self, nbytes: usize) -> PyResult<i64> {
    use bytes::Buf;
    if nbytes > 8 || self.inner.as_inner().remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err(
        "not enough data or nbytes > 8",
      ));
    }
    crate::python::validate_utf8_advance(self, nbytes)?;
    Ok(self.inner.inner.get_int_le(nbytes))
  }
}
