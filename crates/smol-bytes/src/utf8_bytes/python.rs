use super::*;
use pyo3::{
  basic::CompareOp,
  exceptions::{PyIndexError, PyTypeError, PyValueError},
  prelude::{*, Bound},
  types::{PyBytes, PyString},
};

#[pymethods]
impl Utf8Bytes {
  #[new]
  fn new_python() -> Self {
    Self::new()
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

  /// Return whether the bytes are non-empty.
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
    self.try_split_to(at).map_err(|e| PyValueError::new_err(e.to_string()))
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
    self.try_split_off(at).map_err(|e| PyValueError::new_err(e.to_string()))
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
    self.try_slice(start..end).map_err(|e| PyValueError::new_err(e.to_string()))
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
