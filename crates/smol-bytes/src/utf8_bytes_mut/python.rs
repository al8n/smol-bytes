use super::*;
use pyo3::{
  basic::CompareOp,
  exceptions::{PyIndexError, PyTypeError, PyValueError},
  prelude::*,
  types::{PyBytes, PyString},
};

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
  #[staticmethod]
  #[pyo3(name = "with_capacity")]
  fn __python_with_capacity(capacity: usize) -> Self {
    Self::with_capacity(capacity)
  }

  /// Create from a string.
  ///
  /// Args:
  ///     s: A string to create the buffer from.
  ///
  /// Returns:
  ///     Utf8BytesMut: A new mutable UTF-8 buffer.
  #[staticmethod]
  #[pyo3(name = "from_str")]
  fn __python_from_str(s: &str) -> Self {
    Self::from(s)
  }

  /// Return the contents as a Python string.
  fn __str__(&self) -> &str {
    self.as_str()
  }

  /// Return a debug representation.
  fn __repr__(&self) -> String {
    format!("{:?}", self)
  }

  /// Return the number of bytes.
  fn __len__(&self) -> usize {
    self.len()
  }

  /// Return whether the buffer is non-empty.
  fn __bool__(&self) -> bool {
    !self.is_empty()
  }

  /// Check if a substring is contained.
  fn __contains__(&self, item: &Bound<'_, PyAny>) -> PyResult<bool> {
    if let Ok(s) = item.extract::<&str>() {
      return Ok(self.as_str().contains(s));
    }
    Err(PyTypeError::new_err("argument must be a string"))
  }

  /// Support indexing and slicing.
  fn __getitem__(&self, index: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let py = index.py();

    if let Ok(i) = index.extract::<isize>() {
      let chars: Vec<char> = self.as_str().chars().collect();
      let len = chars.len() as isize;
      let idx = if i < 0 { len + i } else { i };

      if idx < 0 || idx >= len {
        return Err(PyIndexError::new_err(format!("string index out of range: {} (len={})", i, len)));
      }

      return Ok(chars[idx as usize].to_string().into_pyobject(py)?.into_any().unbind());
    }

    if let Ok(slice) = index.downcast::<pyo3::types::PySlice>() {
      let s = self.as_str();
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
        CompareOp::Lt => self < &*other_self,
        CompareOp::Le => self <= &*other_self,
        CompareOp::Eq => self == &*other_self,
        CompareOp::Ne => self != &*other_self,
        CompareOp::Gt => self > &*other_self,
        CompareOp::Ge => self >= &*other_self,
      });
    }

    if let Ok(s) = other.extract::<&str>() {
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
        "'>=' not supported between instances of 'Utf8BytesMut' and '{}'",
        other.get_type().name()?
      ))),
    }
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
  #[pyo3(name = "push_str")]
  fn __python_push_str(&mut self, s: &str) {
    self.push_str(s);
  }

  /// Clear the buffer.
  #[pyo3(name = "clear")]
  fn __python_clear(&mut self) {
    self.clear();
  }

  /// Split at the given index.
  ///
  /// Args:
  ///     at: The split index (must be on a character boundary).
  ///
  /// Returns:
  ///     Utf8BytesMut: The content before the split point.
  ///
  /// Raises:
  ///     ValueError: If `at` is not on a character boundary or is out of bounds.
  #[pyo3(name = "split_to")]
  fn __python_split_to(&mut self, at: usize) -> PyResult<Self> {
    self.try_split_to(at).map_err(|e| PyValueError::new_err(e.to_string()))
  }

  /// Split at the given index, returning the tail.
  ///
  /// Args:
  ///     at: The split index (must be on a character boundary).
  ///
  /// Returns:
  ///     Utf8BytesMut: The content after the split point.
  ///
  /// Raises:
  ///     ValueError: If `at` is not on a character boundary or is out of bounds.
  #[pyo3(name = "split_off")]
  fn __python_split_off(&mut self, at: usize) -> PyResult<Self> {
    self.try_split_off(at).map_err(|e| PyValueError::new_err(e.to_string()))
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
  /// Args:
  ///     other: The buffer to merge.
  ///
  /// Returns:
  ///     Utf8BytesMut or None: Returns the other buffer if merge failed, None on success.
  #[pyo3(name = "unsplit")]
  fn __python_unsplit(&mut self, other: Self) -> Option<Self> {
    self.unsplit(other)
  }

  /// Reserve capacity.
  ///
  /// Args:
  ///     additional: Additional capacity to reserve.
  #[pyo3(name = "reserve")]
  fn __python_reserve(&mut self, additional: usize) {
    self.reserve(additional);
  }

  /// Return the capacity.
  #[pyo3(name = "capacity")]
  fn __python_capacity(&self) -> usize {
    self.capacity()
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
}
