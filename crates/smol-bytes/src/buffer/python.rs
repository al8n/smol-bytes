use crate::python::PyGetError;
use pyo3::{
  basic::CompareOp,
  exceptions::PyBufferError,
  prelude::{Bound, *},
  types::{PyBytes, PyString},
};

use super::*;

#[cfg(not(any(feature = "std", feature = "alloc")))]
type IntoIter = super::iter::IntoIter<Buffer>;

#[cfg(any(feature = "std", feature = "alloc"))]
type IntoIter = ::bytes::buf::IntoIter<Buffer>;

/// Iterator over the bytes contained by the buffer.
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

#[pymethods]
impl Buffer {
  #[new]
  fn new_python() -> Self {
    Self::new()
  }

  #[staticmethod]
  #[pyo3(name = "from_bytes")]
  fn __python_from_bytes(py_bytes: &[u8]) -> PyResult<Self> {
    Self::try_from(py_bytes).map_err(|e| {
      PyBufferError::new_err(format!(
        "overflow the buffer capacity: requested {} but only {} available",
        e.requested, e.available
      ))
    })
  }

  #[staticmethod]
  #[pyo3(name = "from_str")]
  fn __python_from_str(py_str: &str) -> PyResult<Self> {
    Self::try_from(py_str.as_bytes()).map_err(|e| {
      PyBufferError::new_err(format!(
        "overflow the buffer capacity: requested {} but only {} available",
        e.requested, e.available
      ))
    })
  }

  fn __bytes__<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    PyBytes::new(py, self.as_slice())
  }

  fn __bool__(&self) -> bool {
    !self.is_empty()
  }

  fn __hash__(&self) -> u64 {
    use ::core::hash::{Hash, Hasher};

    let mut hasher = crate::DefaultHasher::new();
    self.hash(&mut hasher);
    hasher.finish()
  }

  fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<Iter>> {
    let iter = Iter {
      inner: slf.into_iter(),
    };
    Py::new(slf.py(), iter)
  }

  fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<bool> {
    use core::cmp::Ordering;

    // For Buffer vs Buffer, use Ord::cmp
    if let Ok(buffer) = other.extract::<PyRef<'_, Self>>() {
      let ordering = self.cmp(&*buffer);
      return Ok(match op {
        CompareOp::Lt => ordering == Ordering::Less,
        CompareOp::Le => ordering != Ordering::Greater,
        CompareOp::Eq => ordering == Ordering::Equal,
        CompareOp::Ne => ordering != Ordering::Equal,
        CompareOp::Gt => ordering == Ordering::Greater,
        CompareOp::Ge => ordering != Ordering::Less,
      });
    }

    // Helper macro for PartialOrd comparisons
    macro_rules! compare {
      ($other_val:expr) => {{
        if let Some(ordering) = self.partial_cmp($other_val) {
          return Ok(match op {
            CompareOp::Lt => ordering == Ordering::Less,
            CompareOp::Le => ordering != Ordering::Greater,
            CompareOp::Eq => ordering == Ordering::Equal,
            CompareOp::Ne => ordering != Ordering::Equal,
            CompareOp::Gt => ordering == Ordering::Greater,
            CompareOp::Ge => ordering != Ordering::Less,
          });
        }
      }};
    }

    // Try bytes (PyBytes)
    if let Ok(py_bytes) = other.cast::<PyBytes>() {
      let bytes_slice: &[u8] = py_bytes.as_bytes();
      compare!(bytes_slice);
    }

    // Try str (PyString)
    if let Ok(py_str) = other.cast::<PyString>() {
      if let Ok(s) = py_str.to_cow() {
        let str_ref: &str = s.as_ref();
        compare!(str_ref);
      }
    }

    // Try String
    #[cfg(any(feature = "std", feature = "alloc"))]
    if let Ok(s) = other.extract::<std::string::String>() {
      compare!(&s);
    }

    // Try Vec<u8>
    #[cfg(any(feature = "std", feature = "alloc"))]
    if let Ok(byte_vec) = other.extract::<std::vec::Vec<u8>>() {
      compare!(&byte_vec);
    }

    // Not comparable - for equality return false, for ordering raise TypeError
    match op {
      CompareOp::Eq => Ok(false),
      CompareOp::Ne => Ok(true),
      _ => Err(pyo3::exceptions::PyTypeError::new_err(format!(
        "'<' not supported between instances of 'Buffer' and '{}'",
        other.get_type().name()?
      ))),
    }
  }

  fn __str__(&self) -> String {
    if let Ok(s) = ::core::str::from_utf8(self.as_ref()) {
      s.to_string()
    } else {
      format!("<Buffer len={}>", self.len())
    }
  }

  fn __repr__(&self) -> String {
    if let Ok(s) = ::core::str::from_utf8(self.as_ref()) {
      format!("Buffer(b\"{}\")", s)
    } else {
      format!("Buffer(<{} bytes>)", self.len())
    }
  }

  fn __len__(&self) -> usize {
    self.len()
  }

  fn __contains__(&self, item: &Bound<'_, PyAny>) -> PyResult<bool> {
    // Check if item is a single byte (int)
    if let Ok(byte) = item.extract::<u8>() {
      return Ok(self.as_slice().contains(&byte));
    }

    // Check if item is bytes-like
    if let Ok(bytes) = item.extract::<Vec<u8>>() {
      if bytes.is_empty() {
        return Ok(true);
      }
      // Simple substring search
      let haystack = self.as_slice();
      if bytes.len() > haystack.len() {
        return Ok(false);
      }
      Ok(haystack.windows(bytes.len()).any(|w| w == bytes.as_slice()))
    } else if let Ok(s) = item.extract::<String>() {
      let bytes = s.as_bytes();
      if bytes.is_empty() {
        return Ok(true);
      }
      let haystack = self.as_slice();
      if bytes.len() > haystack.len() {
        return Ok(false);
      }
      Ok(haystack.windows(bytes.len()).any(|w| w == bytes))
    } else {
      Err(pyo3::exceptions::PyTypeError::new_err(
        "argument should be an integer or bytes-like object",
      ))
    }
  }

  fn __getitem__(&self, index: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let py = index.py();

    // Handle integer index
    if let Ok(i) = index.extract::<isize>() {
      let len = self.remaining() as isize;
      let idx = if i < 0 { len + i } else { i };

      if idx < 0 || idx >= len {
        return Err(pyo3::exceptions::PyIndexError::new_err(format!(
          "buffer index out of range: {} (len={})",
          i, len
        )));
      }

      return Ok(
        self.as_slice()[idx as usize]
          .into_pyobject(py)?
          .into_any()
          .unbind(),
      );
    }

    // Handle slice
    if let Ok(slice) = index.cast::<pyo3::types::PySlice>() {
      let len = self.remaining();
      let indices = slice.indices(len as isize)?;

      let start = indices.start.max(0) as usize;
      let stop = indices.stop.max(0).min(len as isize) as usize;
      let step = indices.step;

      if step == 1 {
        // Simple slice
        if start <= stop && stop <= len {
          return Ok(PyBytes::new(py, &self.as_slice()[start..stop]).into());
        }
        return Err(pyo3::exceptions::PyIndexError::new_err(
          "slice out of range",
        ));
      } else if step > 1 {
        // Stepped slice
        let mut result = Vec::new();
        let mut i = start;
        while i < stop && i < len {
          result.push(self.as_slice()[i]);
          i += step as usize;
        }
        return Ok(PyBytes::new(py, &result).into());
      } else if step < 0 {
        // Negative step (reverse)
        let mut result = Vec::new();
        let mut i = start.min(len - 1);
        loop {
          result.push(self.as_slice()[i]);
          if i == 0 || i < stop {
            break;
          }
          i = i.saturating_sub((-step) as usize);
        }
        return Ok(PyBytes::new(py, &result).into());
      }

      return Err(pyo3::exceptions::PyValueError::new_err(
        "slice step cannot be zero",
      ));
    }

    Err(pyo3::exceptions::PyTypeError::new_err(
      "buffer indices must be integers or slices",
    ))
  }

  fn __setitem__(&mut self, index: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>) -> PyResult<()> {
    // Handle integer index
    if let Ok(i) = index.extract::<isize>() {
      let len = self.remaining() as isize;
      let idx = if i < 0 { len + i } else { i };

      if idx < 0 || idx >= len {
        return Err(pyo3::exceptions::PyIndexError::new_err(format!(
          "buffer index out of range: {} (len={})",
          i, len
        )));
      }

      let byte = value
        .extract::<u8>()
        .map_err(|_| pyo3::exceptions::PyTypeError::new_err("an integer is required"))?;

      self.as_mut()[idx as usize] = byte;
      return Ok(());
    }

    // Handle slice assignment
    if let Ok(slice) = index.cast::<pyo3::types::PySlice>() {
      let len = self.remaining();
      let indices = slice.indices(len as isize)?;

      let start = indices.start.max(0) as usize;
      let stop = indices.stop.max(0).min(len as isize) as usize;
      let step = indices.step;

      // Extract the bytes to assign
      let bytes = if let Ok(b) = value.extract::<Vec<u8>>() {
        b
      } else if let Ok(s) = value.extract::<String>() {
        s.into_bytes()
      } else {
        return Err(pyo3::exceptions::PyTypeError::new_err(
          "can only assign bytes-like objects",
        ));
      };

      if step == 1 {
        // Simple slice assignment
        let slice_len = stop - start;
        if bytes.len() != slice_len {
          return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "attempt to assign bytes of size {} to slice of size {}",
            bytes.len(),
            slice_len
          )));
        }
        self.as_mut()[start..stop].copy_from_slice(&bytes);
        return Ok(());
      } else if step != 0 {
        // Extended slice assignment
        let mut positions = Vec::new();
        let mut i = start;
        if step > 0 {
          while i < stop && i < len {
            positions.push(i);
            i += step as usize;
          }
        } else {
          let mut i = start.min(len - 1);
          loop {
            positions.push(i);
            if i == 0 || i < stop {
              break;
            }
            i = i.saturating_sub((-step) as usize);
          }
        }

        if bytes.len() != positions.len() {
          return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "attempt to assign bytes of size {} to extended slice of size {}",
            bytes.len(),
            positions.len()
          )));
        }

        for (pos, &byte) in positions.iter().zip(bytes.iter()) {
          self.as_mut()[*pos] = byte;
        }
        return Ok(());
      }

      return Err(pyo3::exceptions::PyValueError::new_err(
        "slice step cannot be zero",
      ));
    }

    Err(pyo3::exceptions::PyTypeError::new_err(
      "buffer indices must be integers or slices",
    ))
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
    PyBytes::new(py, self.as_slice())
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
  fn __python_to_string<'py>(&self, py: Python<'py>) -> ::pyo3::PyResult<Bound<'py, PyString>> {
    ::core::str::from_utf8(self.as_ref())
      .map(|s| PyString::new(py, s))
      .map_err(|e| {
        pyo3::exceptions::PyUnicodeDecodeError::new_err(format!(
          "invalid utf-8 sequence at byte {}: {}",
          e.valid_up_to(),
          e
        ))
      })
  }

  /// Read an unsigned 8-bit integer from the buffer.
  ///
  /// The current position is advanced by 1 byte.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u8")]
  fn __python_get_u8(&mut self) -> ::pyo3::PyResult<u8> {
    self.try_get_u8().map_err(|e| PyGetError::from(e).into())
  }

  /// Read a signed 8-bit integer from the buffer.
  ///
  /// The current position is advanced by 1 byte.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i8")]
  fn __python_get_i8(&mut self) -> ::pyo3::PyResult<i8> {
    self.try_get_i8().map_err(|e| PyGetError::from(e).into())
  }

  // ==================== 16-bit methods ====================

  /// Read an unsigned 16-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 2 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u16")]
  fn __python_get_u16(&mut self) -> ::pyo3::PyResult<u16> {
    self.try_get_u16().map_err(|e| PyGetError::from(e).into())
  }

  /// Read an unsigned 16-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 2 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u16_le")]
  fn __python_get_u16_le(&mut self) -> ::pyo3::PyResult<u16> {
    self
      .try_get_u16_le()
      .map_err(|e| PyGetError::from(e).into())
  }

  /// Read a signed 16-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 2 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i16")]
  fn __python_get_i16(&mut self) -> ::pyo3::PyResult<i16> {
    self.try_get_i16().map_err(|e| PyGetError::from(e).into())
  }

  /// Read a signed 16-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 2 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i16_le")]
  fn __python_get_i16_le(&mut self) -> ::pyo3::PyResult<i16> {
    self
      .try_get_i16_le()
      .map_err(|e| PyGetError::from(e).into())
  }

  // ==================== 32-bit methods ====================

  /// Read an unsigned 32-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u32")]
  fn __python_get_u32(&mut self) -> ::pyo3::PyResult<u32> {
    self.try_get_u32().map_err(|e| PyGetError::from(e).into())
  }

  /// Read an unsigned 32-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u32_le")]
  fn __python_get_u32_le(&mut self) -> ::pyo3::PyResult<u32> {
    self
      .try_get_u32_le()
      .map_err(|e| PyGetError::from(e).into())
  }

  /// Read a signed 32-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i32")]
  fn __python_get_i32(&mut self) -> ::pyo3::PyResult<i32> {
    self.try_get_i32().map_err(|e| PyGetError::from(e).into())
  }

  /// Read a signed 32-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i32_le")]
  fn __python_get_i32_le(&mut self) -> ::pyo3::PyResult<i32> {
    self
      .try_get_i32_le()
      .map_err(|e| PyGetError::from(e).into())
  }

  /// Read a 32-bit floating point number in big-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_f32")]
  fn __python_get_f32(&mut self) -> ::pyo3::PyResult<f32> {
    self.try_get_f32().map_err(|e| PyGetError::from(e).into())
  }

  /// Read a 32-bit floating point number in little-endian byte order.
  ///
  /// The current position is advanced by 4 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_f32_le")]
  fn __python_get_f32_le(&mut self) -> ::pyo3::PyResult<f32> {
    self
      .try_get_f32_le()
      .map_err(|e| PyGetError::from(e).into())
  }

  // ==================== 64-bit methods ====================

  /// Read an unsigned 64-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u64")]
  fn __python_get_u64(&mut self) -> ::pyo3::PyResult<u64> {
    self.try_get_u64().map_err(|e| PyGetError::from(e).into())
  }

  /// Read an unsigned 64-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u64_le")]
  fn __python_get_u64_le(&mut self) -> ::pyo3::PyResult<u64> {
    self
      .try_get_u64_le()
      .map_err(|e| PyGetError::from(e).into())
  }

  /// Read a signed 64-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i64")]
  fn __python_get_i64(&mut self) -> ::pyo3::PyResult<i64> {
    self.try_get_i64().map_err(|e| PyGetError::from(e).into())
  }

  /// Read a signed 64-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i64_le")]
  fn __python_get_i64_le(&mut self) -> ::pyo3::PyResult<i64> {
    self
      .try_get_i64_le()
      .map_err(|e| PyGetError::from(e).into())
  }

  /// Read a 64-bit floating point number in big-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_f64")]
  fn __python_get_f64(&mut self) -> ::pyo3::PyResult<f64> {
    self.try_get_f64().map_err(|e| PyGetError::from(e).into())
  }

  /// Read a 64-bit floating point number in little-endian byte order.
  ///
  /// The current position is advanced by 8 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_f64_le")]
  fn __python_get_f64_le(&mut self) -> ::pyo3::PyResult<f64> {
    self
      .try_get_f64_le()
      .map_err(|e| PyGetError::from(e).into())
  }

  // ==================== 128-bit methods ====================

  /// Read an unsigned 128-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 16 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u128")]
  fn __python_get_u128(&mut self) -> ::pyo3::PyResult<u128> {
    self.try_get_u128().map_err(|e| PyGetError::from(e).into())
  }

  /// Read an unsigned 128-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 16 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_u128_le")]
  fn __python_get_u128_le(&mut self) -> ::pyo3::PyResult<u128> {
    self
      .try_get_u128_le()
      .map_err(|e| PyGetError::from(e).into())
  }

  /// Read a signed 128-bit integer in big-endian byte order.
  ///
  /// The current position is advanced by 16 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i128")]
  fn __python_get_i128(&mut self) -> ::pyo3::PyResult<i128> {
    self.try_get_i128().map_err(|e| PyGetError::from(e).into())
  }

  /// Read a signed 128-bit integer in little-endian byte order.
  ///
  /// The current position is advanced by 16 bytes.
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_i128_le")]
  fn __python_get_i128_le(&mut self) -> ::pyo3::PyResult<i128> {
    self
      .try_get_i128_le()
      .map_err(|e| PyGetError::from(e).into())
  }

  // ==================== Variable-length methods ====================

  /// Read an unsigned n-byte integer in big-endian byte order.
  ///
  /// The current position is advanced by nbytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (1-8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_uint")]
  fn __python_get_uint(&mut self, nbytes: usize) -> ::pyo3::PyResult<u64> {
    self
      .try_get_uint(nbytes)
      .map_err(|e| PyGetError::from(e).into())
  }

  /// Read an unsigned n-byte integer in little-endian byte order.
  ///
  /// The current position is advanced by nbytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (1-8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_uint_le")]
  fn __python_get_uint_le(&mut self, nbytes: usize) -> ::pyo3::PyResult<u64> {
    self
      .try_get_uint_le(nbytes)
      .map_err(|e| PyGetError::from(e).into())
  }

  /// Read a signed n-byte integer in big-endian byte order.
  ///
  /// The current position is advanced by nbytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (1-8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_int")]
  fn __python_get_int(&mut self, nbytes: usize) -> ::pyo3::PyResult<i64> {
    self
      .try_get_int(nbytes)
      .map_err(|e| PyGetError::from(e).into())
  }

  /// Read a signed n-byte integer in little-endian byte order.
  ///
  /// The current position is advanced by nbytes.
  ///
  /// Args:
  ///     nbytes: Number of bytes to read (1-8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough remaining data in the buffer.
  #[pyo3(name = "get_int_le")]
  fn __python_get_int_le(&mut self, nbytes: usize) -> ::pyo3::PyResult<i64> {
    self
      .try_get_int_le(nbytes)
      .map_err(|e| PyGetError::from(e).into())
  }

  // ==================== Buffer control methods ====================

  /// Returns the number of bytes remaining to be read from the buffer.
  ///
  /// Returns:
  ///     int: Number of bytes available for reading.
  #[pyo3(name = "remaining")]
  fn __python_remaining(&self) -> usize {
    self.remaining()
  }

  /// Returns the number of bytes that can be written to the buffer.
  ///
  /// Returns:
  ///     int: Number of bytes available for writing.
  #[pyo3(name = "remaining_mut")]
  fn __python_remaining_mut(&self) -> usize {
    self.remaining_mut()
  }

  /// Advance the read cursor by the specified number of bytes.
  ///
  /// Args:
  ///     cnt: Number of bytes to advance.
  ///
  /// Raises:
  ///     BufferError: If trying to advance beyond available data.
  #[pyo3(name = "advance")]
  fn __python_advance(&mut self, cnt: usize) -> ::pyo3::PyResult<()> {
    let available = self.remaining();
    if cnt > available {
      return Err(pyo3::exceptions::PyBufferError::new_err(format!(
        "cannot advance {} bytes, only {} bytes available",
        cnt, available
      )));
    }
    self.advance(cnt);
    Ok(())
  }

  // ==================== Put methods ====================

  /// Write a signed 8-bit integer to the buffer.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_i8")]
  fn __python_put_i8(&mut self, val: i8) -> ::pyo3::PyResult<()> {
    self.try_put_i8(val).map_err(Into::into)
  }

  /// Write an unsigned n-byte integer in big-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///     nbytes: Number of bytes to write (1-8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_uint")]
  fn __python_put_uint(&mut self, val: u64, nbytes: usize) -> ::pyo3::PyResult<()> {
    self.try_put_uint(val, nbytes).map_err(Into::into)
  }

  /// Write an unsigned n-byte integer in little-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///     nbytes: Number of bytes to write (1-8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_uint_le")]
  fn __python_put_uint_le(&mut self, val: u64, nbytes: usize) -> ::pyo3::PyResult<()> {
    self.try_put_uint_le(val, nbytes).map_err(Into::into)
  }

  /// Write a signed n-byte integer in big-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///     nbytes: Number of bytes to write (1-8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_int")]
  fn __python_put_int(&mut self, val: i64, nbytes: usize) -> ::pyo3::PyResult<()> {
    self.try_put_int(val, nbytes).map_err(Into::into)
  }

  /// Write a signed n-byte integer in little-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///     nbytes: Number of bytes to write (1-8).
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_int_le")]
  fn __python_put_int_le(&mut self, val: i64, nbytes: usize) -> ::pyo3::PyResult<()> {
    self.try_put_int_le(val, nbytes).map_err(Into::into)
  }

  /// Write a 32-bit floating point number in big-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_f32")]
  fn __python_put_f32(&mut self, val: f32) -> ::pyo3::PyResult<()> {
    self.try_put_f32(val).map_err(Into::into)
  }

  /// Write a 32-bit floating point number in little-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_f32_le")]
  fn __python_put_f32_le(&mut self, val: f32) -> ::pyo3::PyResult<()> {
    self.try_put_f32_le(val).map_err(Into::into)
  }

  /// Write a 64-bit floating point number in big-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_f64")]
  fn __python_put_f64(&mut self, val: f64) -> ::pyo3::PyResult<()> {
    self.try_put_f64(val).map_err(Into::into)
  }

  /// Write a 64-bit floating point number in little-endian byte order.
  ///
  /// Args:
  ///     val: The value to write.
  ///
  /// Raises:
  ///     BufferError: If there is not enough space in the buffer.
  #[pyo3(name = "put_f64_le")]
  fn __python_put_f64_le(&mut self, val: f64) -> ::pyo3::PyResult<()> {
    self.try_put_f64_le(val).map_err(Into::into)
  }

  // ==================== Buffer management methods ====================

  /// Clear the buffer, removing all data.
  ///
  /// The capacity is preserved.
  #[pyo3(name = "clear")]
  fn __python_clear(&mut self) {
    self.clear();
  }

  /// Truncate the buffer to the specified length.
  ///
  /// If the new length is greater than the current length, this has no effect.
  ///
  /// Args:
  ///     new_len: The new length of the buffer.
  #[pyo3(name = "truncate")]
  fn __python_truncate(&mut self, new_len: usize) {
    self.truncate(new_len);
  }

  /// Get the capacity of the buffer.
  ///
  /// Returns:
  ///     int: The number of bytes that can be written without reallocation.
  #[pyo3(name = "capacity")]
  fn __python_capacity(&self) -> usize {
    self.capacity()
  }

  /// Resize the buffer to the specified length, filling with zeros if expanding.
  ///
  /// Args:
  ///     new_len: The new length of the buffer.
  ///
  /// Raises:
  ///     BufferError: If the new length exceeds capacity.
  #[pyo3(name = "resize")]
  fn __python_resize(&mut self, new_len: usize) -> ::pyo3::PyResult<()> {
    self.try_resize(new_len).map_err(|e| {
      pyo3::exceptions::PyBufferError::new_err(format!(
        "resize exceeds capacity: requested {} but capacity is {}",
        e.requested, e.available
      ))
    })
  }

  // ==================== Split methods ====================

  /// Split the buffer at the given position, returning the head.
  ///
  /// After this operation, `self` will contain bytes from `[at..]`,
  /// and the returned buffer will contain bytes from `[..at]`.
  ///
  /// Args:
  ///     at: The position at which to split the buffer.
  ///
  /// Returns:
  ///     Buffer: A new buffer containing bytes from `[..at]`.
  ///
  /// Raises:
  ///     IndexError: If `at` is greater than the buffer length.
  ///
  /// Example:
  ///     >>> buf = Buffer(b"hello world")
  ///     >>> head = buf.split_to(5)
  ///     >>> bytes(head)
  ///     b'hello'
  ///     >>> bytes(buf)
  ///     b' world'
  #[pyo3(name = "split_to")]
  fn __python_split_to(&mut self, at: usize) -> ::pyo3::PyResult<Buffer> {
    self.try_split_to(at).map_err(Into::into)
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
  ///     Buffer: A new buffer containing bytes from `[at..]`.
  ///
  /// Raises:
  ///     IndexError: If `at` is greater than the buffer length.
  ///
  /// Example:
  ///     >>> buf = Buffer(b"hello world")
  ///     >>> tail = buf.split_off(6)
  ///     >>> bytes(buf)
  ///     b'hello '
  ///     >>> bytes(tail)
  ///     b'world'
  #[pyo3(name = "split_off")]
  fn __python_split_off(&mut self, at: usize) -> ::pyo3::PyResult<Buffer> {
    self.try_split_off(at).map_err(Into::into)
  }

  /// Get a slice of the buffer as a new Buffer.
  ///
  /// This creates a new buffer containing a copy of the specified range.
  ///
  /// Args:
  ///     start: The start position (inclusive).
  ///     end: The end position (exclusive).
  ///
  /// Returns:
  ///     Buffer: A new buffer containing the specified range.
  ///
  /// Raises:
  ///     IndexError: If the range is invalid.
  ///
  /// Example:
  ///     >>> buf = Buffer(b"hello world")
  ///     >>> slice = buf.slice(0, 5)
  ///     >>> bytes(slice)
  ///     b'hello'
  #[pyo3(name = "slice")]
  fn __python_slice(&self, start: usize, end: usize) -> ::pyo3::PyResult<Buffer> {
    self.try_slice(start..end).map_err(Into::into)
  }
}
