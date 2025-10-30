use super::*;
use pyo3::{exceptions::PyBufferError, prelude::*};

/// Python-specific wrapper for TryGetError
#[derive(Debug, Clone)]
pub struct PyGetError {
  pub requested: usize,
  pub available: usize,
}

impl From<buffer::TryGetError> for PyGetError {
  fn from(err: buffer::TryGetError) -> Self {
    Self {
      requested: err.requested,
      available: err.available,
    }
  }
}

impl From<PyGetError> for PyErr {
  fn from(err: PyGetError) -> PyErr {
    PyBufferError::new_err(format!(
      "cannot get {} bytes, only {} bytes available",
      err.requested, err.available
    ))
  }
}

impl From<TryPutError> for PyErr {
  fn from(err: TryPutError) -> PyErr {
    PyBufferError::new_err(format!(
      "cannot put {} bytes, only {} bytes available",
      err.requested, err.available
    ))
  }
}

impl From<OutOfBounds> for PyErr {
  fn from(err: OutOfBounds) -> PyErr {
    pyo3::exceptions::PyIndexError::new_err(format!(
      "index out of bounds: requested {} but only {} available",
      err.requested, err.available
    ))
  }
}

impl From<RangeOutOfBounds> for PyErr {
  fn from(err: RangeOutOfBounds) -> PyErr {
    pyo3::exceptions::PyIndexError::new_err(format!(
      "range out of bounds: requested {}..{} but only {} available",
      err.start, err.end, err.available,
    ))
  }
}
