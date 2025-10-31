use crate::python::PyBufExt as _;

use super::*;
use pyo3::{
  basic::CompareOp,
  exceptions::{PyIndexError, PyTypeError, PyValueError},
  prelude::{*, Bound},
  types::{PyBytes, PyString},
};

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
    self.as_buffer().py_contains(item)
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
    self.try_push(ch).map_err(|e| PyValueError::new_err(e.to_string()))
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
    self.try_push_str(s).map_err(|e| PyValueError::new_err(e.to_string()))
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
    self.try_split_to(at).map_err(|e| PyValueError::new_err(e.to_string()))
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
    self.try_split_off(at).map_err(|e| PyValueError::new_err(e.to_string()))
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
    self.try_slice(start..end).map_err(|e| PyValueError::new_err(e.to_string()))
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
}
