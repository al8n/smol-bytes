use super::*;
use crate::python::{
  py_char_offset, py_char_range, py_check_alloc, py_offset, py_readable_buffer_snapshot,
  py_str_contains, py_str_getitem, py_str_richcmp,
};
use crate::utf8_buf::char_offset_to_byte;
use pyo3::{basic::CompareOp, exceptions::PyValueError, prelude::*, types::PyBytes};

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
impl Utf8BytesMut {
  #[new]
  fn new_python() -> Self {
    Self::new()
  }

  /// Create from a string with pre-allocated capacity.
  ///
  /// Args:
  ///     capacity: The capacity to pre-allocate.
  ///
  /// Returns:
  ///     Utf8BytesMut: A new mutable UTF-8 buffer.
  ///
  /// Raises:
  ///     MemoryError: If the requested allocation cannot be satisfied.
  #[staticmethod]
  #[pyo3(name = "with_capacity")]
  fn __python_with_capacity(capacity: usize) -> PyResult<Self> {
    py_check_alloc(capacity)?;
    Ok(Self::with_capacity(capacity))
  }

  /// Create from a string.
  ///
  /// Args:
  ///     s: A string to create the buffer from.
  ///
  /// Returns:
  ///     Utf8BytesMut: A new mutable UTF-8 buffer.
  ///
  /// Raises:
  ///     MemoryError: If the requested allocation cannot be satisfied.
  #[staticmethod]
  #[pyo3(name = "from_str")]
  fn __python_from_str(s: &str) -> PyResult<Self> {
    py_check_alloc(s.len())?;
    Ok(Self::from(s))
  }

  /// Decode bytes as UTF-8, replacing malformed sequences with `U+FFFD`.
  #[staticmethod]
  #[pyo3(name = "from_bytes_lossy")]
  fn __python_from_bytes_lossy(data: &Bound<'_, PyAny>) -> PyResult<Self> {
    let data = py_readable_buffer_snapshot(data)?;
    Ok(Self::from_utf8_lossy(crate::BytesMut::from(data)))
  }

  /// Return the contents as a Python string.
  fn __str__(&self) -> &str {
    self.as_str()
  }

  /// Return a debug representation.
  fn __repr__(&self) -> PyResult<String> {
    py_check_alloc(self.len().saturating_mul(6).saturating_add(64))?;
    Ok(format!("{:?}", self))
  }

  /// Return the number of Unicode scalar values.
  fn __len__(&self) -> usize {
    self.as_str().chars().count()
  }

  /// Return whether the buffer is non-empty.
  fn __bool__(&self) -> bool {
    !self.is_empty()
  }

  /// Check if a substring is contained.
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

  /// Iterate over the characters of the buffer.
  fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<Utf8CharIter>> {
    let char_count = slf.as_str().chars().count();
    py_check_alloc(char_count.saturating_mul(core::mem::size_of::<char>()))?;
    let chars: Vec<char> = slf.as_str().chars().collect();
    Py::new(slf.py(), Utf8CharIter { chars, index: 0 })
  }

  /// Return the UTF-8 bytes as a Python `bytes` object.
  fn __bytes__<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    PyBytes::new(py, self.as_str().as_bytes())
  }

  /// Append a character.
  ///
  /// Args:
  ///     ch: The character to append.
  #[pyo3(name = "push")]
  fn __python_push(&mut self, ch: char) {
    self.push(ch);
  }

  /// Append a string.
  ///
  /// Args:
  ///     s: The string to append.
  ///
  /// Raises:
  ///     MemoryError: If the requested allocation cannot be satisfied.
  #[pyo3(name = "push_str")]
  fn __python_push_str(&mut self, s: &str) -> PyResult<()> {
    py_check_alloc(s.len())?;
    self.push_str(s);
    Ok(())
  }

  /// Insert one Unicode scalar value at a character offset.
  #[pyo3(name = "insert_chars")]
  fn __python_insert_chars(&mut self, index: &Bound<'_, PyAny>, ch: char) -> PyResult<()> {
    let index = py_char_offset(self.as_str(), index, "index")?;
    py_check_alloc(ch.len_utf8())?;
    self.try_insert(index, ch).map_err(Into::into)
  }

  /// Insert a string at a character offset.
  #[pyo3(name = "insert_str_chars")]
  fn __python_insert_str_chars(&mut self, index: &Bound<'_, PyAny>, s: &str) -> PyResult<()> {
    let index = py_char_offset(self.as_str(), index, "index")?;
    if !s.is_empty() {
      py_check_alloc(s.len())?;
    }
    self.try_insert_str(index, s).map_err(Into::into)
  }

  /// Replace the character range `[start, end)` with a string.
  #[pyo3(name = "replace_range_chars")]
  fn __python_replace_range_chars(
    &mut self,
    start: &Bound<'_, PyAny>,
    end: &Bound<'_, PyAny>,
    replacement: &str,
  ) -> PyResult<()> {
    let (start, end) = py_char_range(self.as_str(), start, end)?;
    let growth = replacement.len().saturating_sub(end - start);
    if growth != 0 {
      py_check_alloc(growth)?;
    }
    self
      .try_replace_range(start..end, replacement)
      .map_err(Into::into)
  }

  /// Remove and return the final Unicode scalar value, or `None` when empty.
  #[pyo3(name = "pop")]
  fn __python_pop(&mut self) -> Option<String> {
    self.pop().map(|ch| ch.to_string())
  }

  /// Clear the buffer.
  #[pyo3(name = "clear")]
  fn __python_clear(&mut self) {
    self.clear();
  }

  /// Truncate to `new_len` Unicode scalar values (characters).
  ///
  /// A length at or beyond the current character count leaves the buffer
  /// unchanged.
  #[pyo3(name = "truncate_chars")]
  fn __python_truncate(&mut self, new_len: &Bound<'_, PyAny>) -> PyResult<()> {
    let new_len = py_offset(new_len, "new_len")?;
    match char_offset_to_byte(self.as_str(), new_len) {
      Some(byte_len) => self
        .try_truncate(byte_len)
        .map_err(|error| PyValueError::new_err(error.to_string())),
      None => Ok(()),
    }
  }

  /// Split at the given Unicode scalar value (character) offset.
  ///
  /// Args:
  ///     at: The split offset in Unicode scalar values (characters).
  ///
  /// Returns:
  ///     Utf8BytesMut: The content before the split point.
  ///
  /// Raises:
  ///     IndexError: If `at` is out of range.
  #[pyo3(name = "split_to_chars")]
  fn __python_split_to(&mut self, at: &Bound<'_, PyAny>) -> PyResult<Self> {
    let byte_at = py_char_offset(self.as_str(), at, "split index")?;
    self.try_split_to(byte_at).map_err(Into::into)
  }

  /// Split at the given Unicode scalar value (character) offset, returning the tail.
  ///
  /// Args:
  ///     at: The split offset in Unicode scalar values (characters).
  ///
  /// Returns:
  ///     Utf8BytesMut: The content after the split point.
  ///
  /// Raises:
  ///     IndexError: If `at` is out of range.
  #[pyo3(name = "split_off_chars")]
  fn __python_split_off(&mut self, at: &Bound<'_, PyAny>) -> PyResult<Self> {
    let byte_at = py_char_offset(self.as_str(), at, "split index")?;
    self.try_split_off(byte_at).map_err(Into::into)
  }

  /// Return an independent mutable copy of the Unicode scalar range `[start, end)`.
  #[pyo3(name = "slice_chars")]
  fn __python_slice(&self, start: &Bound<'_, PyAny>, end: &Bound<'_, PyAny>) -> PyResult<Self> {
    let (start, end) = py_char_range(self.as_str(), start, end)?;
    let value = &self.as_str()[start..end];
    py_check_alloc(value.len())?;
    Ok(Self::from(value))
  }

  /// Split and return all bytes.
  ///
  /// Returns:
  ///     Utf8BytesMut: All the bytes from the buffer.
  #[pyo3(name = "split")]
  fn __python_split(&mut self) -> Self {
    self.split()
  }

  /// Merge another buffer back into this one.
  ///
  /// Python uses non-consuming snapshot semantics: the call-entry contents of
  /// `other` are appended, while `other` remains valid and unchanged. This is
  /// intentionally distinct from the consuming Rust method.
  /// Passing this same Python object as `other` is rejected with `RuntimeError`
  /// by Python's mutable-borrow rules.
  ///
  /// Args:
  ///     other: The buffer to merge.
  ///
  /// Returns:
  ///     None: `other` remains valid and unchanged.
  #[pyo3(name = "unsplit")]
  fn __python_unsplit(&mut self, other: PyRef<'_, Self>) -> PyResult<()> {
    py_check_alloc(other.len())?;
    self.unsplit((*other).clone());
    Ok(())
  }

  /// Freeze the current contents with the shared strategy, leaving this
  /// mutable UTF-8 buffer empty and reusable.
  #[pyo3(name = "freeze_shared")]
  fn __python_freeze_shared(&mut self) -> crate::utf8_bytes::PySharedUtf8Bytes {
    crate::utf8_bytes::PySharedUtf8Bytes::from(core::mem::take(self).freeze_shared())
  }

  /// Freeze the current contents with the compact strategy, leaving this
  /// mutable UTF-8 buffer empty and reusable.
  #[pyo3(name = "freeze_compact")]
  fn __python_freeze_compact(&mut self) -> crate::utf8_bytes::PyCompactUtf8Bytes {
    crate::utf8_bytes::PyCompactUtf8Bytes::from(core::mem::take(self).freeze_compact())
  }

  /// Reserve capacity.
  ///
  /// Args:
  ///     additional: Additional capacity to reserve.
  ///
  /// Raises:
  ///     MemoryError: If the requested allocation cannot be satisfied.
  #[pyo3(name = "reserve")]
  fn __python_reserve(&mut self, additional: usize) -> PyResult<()> {
    py_check_alloc(additional)?;
    self.reserve(additional);
    Ok(())
  }

  /// Return the capacity.
  #[pyo3(name = "capacity")]
  fn __python_capacity(&self) -> usize {
    self.capacity()
  }

  /// Return the number of Unicode scalar values.
  #[pyo3(name = "char_len")]
  fn __python_char_len(&self) -> usize {
    self.as_str().chars().count()
  }

  /// Return the length in bytes.
  #[pyo3(name = "byte_len")]
  fn __python_byte_len(&self) -> usize {
    self.len()
  }

  /// Return whether the bytes are stored inline.
  #[pyo3(name = "is_inline")]
  fn __python_is_inline(&self) -> bool {
    self.is_inline()
  }

  /// Return whether the bytes are stored on the heap.
  #[pyo3(name = "is_heap")]
  fn __python_is_heap(&self) -> bool {
    self.is_heap()
  }

  /// Copy a small heap value back into inline storage when possible.
  #[pyo3(name = "try_make_inline")]
  fn __python_try_make_inline(&mut self) -> bool {
    self.try_make_inline()
  }

  /// Support copy.copy.
  ///
  /// Raises:
  ///     MemoryError: If the requested allocation cannot be satisfied.
  #[pyo3(name = "__copy__")]
  fn __python_copy(&self) -> PyResult<Self> {
    py_check_alloc(self.len())?;
    Ok(self.clone())
  }

  /// Support copy.deepcopy.
  ///
  /// Raises:
  ///     MemoryError: If the requested allocation cannot be satisfied.
  #[pyo3(name = "__deepcopy__")]
  fn __python_deepcopy(&self, _memo: &Bound<'_, PyAny>) -> PyResult<Self> {
    py_check_alloc(self.len())?;
    Ok(self.clone())
  }

  /// Support pickling via `pickle.dumps` / `pickle.loads`.
  fn __reduce__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<(Py<PyAny>, (String,))> {
    let cls = py.get_type::<Self>();
    let from_str = cls.getattr("from_str")?;
    py_check_alloc(slf.as_str().len())?;
    Ok((from_str.unbind(), (slf.as_str().to_string(),)))
  }

  /// Return the number of bytes remaining for reading.
  ///
  /// Returns:
  ///     int: Number of bytes available for reading.
  #[pyo3(name = "remaining_bytes")]
  fn __python_remaining(&self) -> usize {
    use bytes::Buf;
    self.inner.remaining()
  }

  /// Advance the read cursor by the specified number of bytes.
  ///
  /// Args:
  ///     cnt: Number of bytes to advance.
  ///
  /// Raises:
  ///     BufferError: If `cnt` exceeds available data.
  ///     ValueError: If it would end inside a UTF-8 character.
  #[pyo3(name = "advance_bytes")]
  fn __python_advance(&mut self, cnt: &Bound<'_, PyAny>) -> PyResult<()> {
    use bytes::Buf;
    let cnt = py_offset(cnt, "cnt")?;
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
  ///     BufferError: If fewer than 1 byte remains.
  ///     ValueError: If consuming it would end inside a UTF-8 character.
  #[pyo3(name = "get_u8")]
  fn __python_get_u8(&mut self) -> PyResult<u8> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 1 byte remains.
  ///     ValueError: If consuming it would end inside a UTF-8 character.
  #[pyo3(name = "get_i8")]
  fn __python_get_i8(&mut self) -> PyResult<i8> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 2 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u16")]
  fn __python_get_u16(&mut self) -> PyResult<u16> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 2 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u16_le")]
  fn __python_get_u16_le(&mut self) -> PyResult<u16> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 2 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i16")]
  fn __python_get_i16(&mut self) -> PyResult<i16> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 2 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i16_le")]
  fn __python_get_i16_le(&mut self) -> PyResult<i16> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 4 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u32")]
  fn __python_get_u32(&mut self) -> PyResult<u32> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 4 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u32_le")]
  fn __python_get_u32_le(&mut self) -> PyResult<u32> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 4 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i32")]
  fn __python_get_i32(&mut self) -> PyResult<i32> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 4 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i32_le")]
  fn __python_get_i32_le(&mut self) -> PyResult<i32> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 8 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u64")]
  fn __python_get_u64(&mut self) -> PyResult<u64> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 8 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u64_le")]
  fn __python_get_u64_le(&mut self) -> PyResult<u64> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 8 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i64")]
  fn __python_get_i64(&mut self) -> PyResult<i64> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 8 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i64_le")]
  fn __python_get_i64_le(&mut self) -> PyResult<i64> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 16 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u128")]
  fn __python_get_u128(&mut self) -> PyResult<u128> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 16 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_u128_le")]
  fn __python_get_u128_le(&mut self) -> PyResult<u128> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 16 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i128")]
  fn __python_get_i128(&mut self) -> PyResult<i128> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 16 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_i128_le")]
  fn __python_get_i128_le(&mut self) -> PyResult<i128> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 4 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f32")]
  fn __python_get_f32(&mut self) -> PyResult<f32> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 4 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f32_le")]
  fn __python_get_f32_le(&mut self) -> PyResult<f32> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 8 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f64")]
  fn __python_get_f64(&mut self) -> PyResult<f64> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than 8 bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_f64_le")]
  fn __python_get_f64_le(&mut self) -> PyResult<f64> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than `nbytes` bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_uint")]
  fn __python_get_uint(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<u64> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than `nbytes` bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_uint_le")]
  fn __python_get_uint_le(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<u64> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than `nbytes` bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_int")]
  fn __python_get_int(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<i64> {
    use bytes::Buf;
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
  ///     BufferError: If fewer than `nbytes` bytes remain.
  ///     ValueError: If consuming them would end inside a UTF-8 character.
  #[pyo3(name = "get_int_le")]
  fn __python_get_int_le(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<i64> {
    use bytes::Buf;
    let nbytes = crate::python::py_integer_width(nbytes)?;
    if self.inner.remaining() < nbytes {
      return Err(pyo3::exceptions::PyBufferError::new_err("not enough data"));
    }
    crate::python::validate_utf8_advance(self, nbytes)?;
    Ok(self.inner.get_int_le(nbytes))
  }
}
