use super::*;
use pyo3::{
  exceptions::{PyBufferError, PyIndexError, PyTypeError, PyUnicodeDecodeError, PyValueError},
  prelude::*,
  types::{PyAny, PyBytes, PySlice, PyString},
};

use crate::{Buf, OutOfBounds, RangeOutOfBounds};

#[cfg(any(feature = "std", feature = "alloc"))]
use crate::bytes::{self, RawBytes};

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
    PyIndexError::new_err(format!(
      "index out of bounds: requested {} but only {} available",
      err.requested, err.available
    ))
  }
}

impl From<RangeOutOfBounds> for PyErr {
  fn from(err: RangeOutOfBounds) -> PyErr {
    PyIndexError::new_err(format!(
      "range out of bounds: requested {}..{} but only {} available",
      err.start, err.end, err.available,
    ))
  }
}

pub trait PyBufCommon: Sized {
  fn py_try_split_to(&mut self, at: usize) -> Result<Self, OutOfBounds>;

  fn py_try_split_off(&mut self, at: usize) -> Result<Self, OutOfBounds>;

  fn py_try_slice(&self, start: usize, end: usize) -> Result<Self, RangeOutOfBounds>;

  fn py_try_advance(&mut self, cnt: usize) -> Result<(), OutOfBounds>;
}

impl PyBufCommon for Buffer {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_split_to(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    self.try_split_to(at)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_split_off(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    self.try_split_off(at)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_slice(&self, start: usize, end: usize) -> Result<Self, RangeOutOfBounds> {
    self.try_slice(start..end)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_advance(&mut self, cnt: usize) -> Result<(), OutOfBounds> {
    self.try_advance(cnt)
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<S> PyBufCommon for RawBytes<S>
where
  RawBytes<S>: bytes::strategy::Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_split_to(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    self.try_split_to(at)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_split_off(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    self.try_split_off(at)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_slice(&self, start: usize, end: usize) -> Result<Self, RangeOutOfBounds> {
    self.try_slice(start..end)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_advance(&mut self, cnt: usize) -> Result<(), OutOfBounds> {
    self.try_advance(cnt)
  }
}

pub trait PyBufExt: Buf + AsRef<[u8]> + PyBufCommon + Sized {
  fn py_len(&self) -> usize {
    self.remaining()
  }

  fn py_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    PyBytes::new(py, self.as_ref())
  }

  fn py_to_string<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyString>> {
    ::core::str::from_utf8(self.as_ref())
      .map(|s| PyString::new(py, s))
      .map_err(|e| {
        PyUnicodeDecodeError::new_err(format!(
          "invalid utf-8 sequence at byte {}: {}",
          e.valid_up_to(),
          e
        ))
      })
  }

  fn py_contains(&self, item: &Bound<'_, PyAny>) -> PyResult<bool> {
    let haystack = self.as_ref();

    if let Ok(byte) = item.extract::<u8>() {
      return Ok(haystack.contains(&byte));
    }

    if let Ok(bytes) = item.extract::<std::vec::Vec<u8>>() {
      if bytes.is_empty() {
        return Ok(true);
      }
      if bytes.len() > haystack.len() {
        return Ok(false);
      }
      return Ok(haystack.windows(bytes.len()).any(|w| w == bytes.as_slice()));
    }

    if let Ok(s) = item.extract::<std::string::String>() {
      let bytes = s.as_bytes();
      if bytes.is_empty() {
        return Ok(true);
      }
      if bytes.len() > haystack.len() {
        return Ok(false);
      }
      return Ok(haystack.windows(bytes.len()).any(|w| w == bytes));
    }

    Err(PyTypeError::new_err(
      "argument should be an integer or bytes-like object",
    ))
  }

  fn py_getitem(&self, index: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let py = index.py();
    let data = self.as_ref();

    if let Ok(i) = index.extract::<isize>() {
      let len = data.len() as isize;
      let idx = if i < 0 { len + i } else { i };

      if idx < 0 || idx >= len {
        return Err(PyIndexError::new_err(format!(
          "buffer index out of range: {} (len={})",
          i, len
        )));
      }

      return Ok(data[idx as usize].into_pyobject(py)?.into_any().unbind());
    }

    if let Ok(slice) = index.cast::<PySlice>() {
      let len = data.len();
      let indices = slice.indices(len as isize)?;

      let start = indices.start.max(0) as usize;
      let stop = indices.stop.max(0).min(len as isize) as usize;
      let step = indices.step;

      if step == 1 {
        if start <= stop && stop <= len {
          return Ok(PyBytes::new(py, &data[start..stop]).into());
        }
        return Err(PyIndexError::new_err("slice out of range"));
      } else if step > 1 {
        let mut result = std::vec::Vec::new();
        let mut i = start;
        while i < stop && i < len {
          result.push(data[i]);
          i += step as usize;
        }
        return Ok(PyBytes::new(py, &result).into());
      } else if step < 0 {
        let mut result = std::vec::Vec::new();
        if len == 0 {
          return Ok(PyBytes::new(py, &result).into());
        }
        let mut i = start.min(len.saturating_sub(1));
        loop {
          result.push(data[i]);
          if i == 0 || i < stop {
            break;
          }
          i = i.saturating_sub((-step) as usize);
        }
        return Ok(PyBytes::new(py, &result).into());
      }

      return Err(PyValueError::new_err("slice step cannot be zero"));
    }

    Err(PyTypeError::new_err(
      "buffer indices must be integers or slices",
    ))
  }

  fn py_remaining(&self) -> usize {
    self.remaining()
  }

  fn py_split_to(&mut self, at: usize) -> PyResult<Self>
  where
    Self: Sized,
  {
    self.py_try_split_to(at).map_err(Into::into)
  }

  fn py_split_off(&mut self, at: usize) -> PyResult<Self>
  where
    Self: Sized,
  {
    self.py_try_split_off(at).map_err(Into::into)
  }

  fn py_slice(&self, start: usize, end: usize) -> PyResult<Self>
  where
    Self: Sized,
  {
    self.py_try_slice(start, end).map_err(Into::into)
  }

  fn py_advance(&mut self, cnt: usize) -> PyResult<()>
  where
    Self: Sized,
  {
    self.py_try_advance(cnt).map_err(Into::into)
  }
}

impl<T> PyBufExt for T where T: Buf + AsRef<[u8]> + PyBufCommon {}

pub trait PyBufMutExt: PyBufExt + AsMut<[u8]> {
  fn py_setitem(&mut self, index: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>) -> PyResult<()> {
    let data = self.as_mut();
    let len = data.len();

    if let Ok(i) = index.extract::<isize>() {
      let len_isize = len as isize;
      let idx = if i < 0 { len_isize + i } else { i };

      if idx < 0 || idx >= len_isize {
        return Err(PyIndexError::new_err(format!(
          "buffer index out of range: {} (len={})",
          i, len_isize
        )));
      }

      let byte = value
        .extract::<u8>()
        .map_err(|_| PyTypeError::new_err("an integer is required"))?;

      data[idx as usize] = byte;
      return Ok(());
    }

    if let Ok(slice) = index.cast::<PySlice>() {
      let indices = slice.indices(len as isize)?;

      let start = indices.start.max(0) as usize;
      let stop = indices.stop.max(0).min(len as isize) as usize;
      let step = indices.step;

      let bytes = if let Ok(b) = value.extract::<std::vec::Vec<u8>>() {
        b
      } else if let Ok(s) = value.extract::<std::string::String>() {
        s.into_bytes()
      } else {
        return Err(PyTypeError::new_err("can only assign bytes-like objects"));
      };

      if step == 1 {
        let slice_len = stop.saturating_sub(start);
        if bytes.len() != slice_len {
          return Err(PyValueError::new_err(format!(
            "attempt to assign bytes of size {} to slice of size {}",
            bytes.len(),
            slice_len
          )));
        }
        data[start..stop].copy_from_slice(&bytes);
        return Ok(());
      } else if step != 0 {
        let mut positions = std::vec::Vec::new();

        if step > 0 {
          let mut i = start;
          while i < stop && i < len {
            positions.push(i);
            i += step as usize;
          }
        } else {
          if len == 0 {
            if bytes.is_empty() {
              return Ok(());
            }
            return Err(PyValueError::new_err(format!(
              "attempt to assign bytes of size {} to extended slice of size 0",
              bytes.len()
            )));
          }

          let mut i = start.min(len.saturating_sub(1));
          loop {
            positions.push(i);
            if i == 0 || i < stop {
              break;
            }
            i = i.saturating_sub((-step) as usize);
          }
        }

        if bytes.len() != positions.len() {
          return Err(PyValueError::new_err(format!(
            "attempt to assign bytes of size {} to extended slice of size {}",
            bytes.len(),
            positions.len()
          )));
        }

        for (pos, byte) in positions.into_iter().zip(bytes.into_iter()) {
          data[pos] = byte;
        }
        return Ok(());
      }

      return Err(PyValueError::new_err("slice step cannot be zero"));
    }

    Err(PyTypeError::new_err(
      "buffer indices must be integers or slices",
    ))
  }
}

impl<T> PyBufMutExt for T where T: PyBufExt + AsMut<[u8]> {}
