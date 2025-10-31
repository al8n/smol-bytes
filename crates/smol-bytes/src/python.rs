use super::*;
use pyo3::type_object::PyTypeInfo;
use pyo3::PyClass;
use pyo3::{
  basic::CompareOp,
  exceptions::{PyBufferError, PyIndexError, PyTypeError, PyUnicodeDecodeError, PyValueError},
  prelude::*,
  types::{PyAny, PyBytes, PySlice, PyString},
};

use crate::{Buf, BytesMut, OutOfBounds, RangeOutOfBounds, TryGetError};

#[cfg(any(feature = "std", feature = "alloc"))]
use crate::bytes::{self, RawBytes};

impl From<TryPutError> for PyErr {
  fn from(err: TryPutError) -> PyErr {
    PyBufferError::new_err(format!(
      "cannot put {} bytes, only {} bytes available",
      err.requested, err.available
    ))
  }
}

impl From<TryPutIntegerError> for PyErr {
  fn from(err: TryPutIntegerError) -> PyErr {
    match err {
      TryPutIntegerError::NotEnoughSpace(e) => e.into(),
      TryPutIntegerError::InvalidLength { requested } => PyValueError::new_err(format!(
        "number of bytes must be less or equal to 8, got {}",
        requested
      )),
    }
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

fn try_get_error_parts(err: &TryGetError) -> (usize, usize) {
  (err.requested, err.available)
}

fn richcmp_ordering_to_bool(ordering: core::cmp::Ordering, op: CompareOp) -> bool {
  use core::cmp::Ordering;

  match op {
    CompareOp::Lt => ordering == Ordering::Less,
    CompareOp::Le => ordering != Ordering::Greater,
    CompareOp::Eq => ordering == Ordering::Equal,
    CompareOp::Ne => ordering != Ordering::Equal,
    CompareOp::Gt => ordering == Ordering::Greater,
    CompareOp::Ge => ordering != Ordering::Less,
  }
}

fn py_richcmp_bytes_like(
  self_bytes: &[u8],
  other: &Bound<'_, PyAny>,
  op: CompareOp,
) -> PyResult<Option<bool>> {
  if let Ok(py_bytes) = other.cast::<PyBytes>() {
    if let Some(ordering) = self_bytes.partial_cmp(py_bytes.as_bytes()) {
      return Ok(Some(richcmp_ordering_to_bool(ordering, op)));
    }
  }

  if let Ok(py_str) = other.cast::<PyString>() {
    if let Ok(s) = py_str.to_cow() {
      if let Some(ordering) = self_bytes.partial_cmp(s.as_ref().as_bytes()) {
        return Ok(Some(richcmp_ordering_to_bool(ordering, op)));
      }
    }
  }

  #[cfg(any(feature = "std", feature = "alloc"))]
  if let Ok(s) = other.extract::<std::string::String>() {
    if let Some(ordering) = self_bytes.partial_cmp(s.as_bytes()) {
      return Ok(Some(richcmp_ordering_to_bool(ordering, op)));
    }
  }

  #[cfg(any(feature = "std", feature = "alloc"))]
  if let Ok(byte_vec) = other.extract::<std::vec::Vec<u8>>() {
    if let Some(ordering) = self_bytes.partial_cmp(byte_vec.as_slice()) {
      return Ok(Some(richcmp_ordering_to_bool(ordering, op)));
    }
  }

  Ok(None)
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

impl PyBufCommon for BytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_split_to(&mut self, at: usize) -> Result<BytesMut, OutOfBounds> {
    match self.try_split_to(at)? {
      Ok(bytes) => Ok(bytes),
      Err(buffer) => Ok(BytesMut::from_inline(buffer)),
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_split_off(&mut self, at: usize) -> Result<BytesMut, OutOfBounds> {
    match self.try_split_off(at)? {
      Ok(bytes) => Ok(bytes),
      Err(buffer) => Ok(BytesMut::from_inline(buffer)),
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_slice(&self, _: usize, _: usize) -> Result<BytesMut, RangeOutOfBounds> {
    unreachable!("BytesMut does not support slicing");
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_advance(&mut self, cnt: usize) -> Result<(), OutOfBounds> {
    self.try_advance(cnt)
  }
}

pub trait PyBufCmp: PyClass + PartialEq + Ord + AsRef<[u8]> {
  fn py_type_name() -> &'static str {
    <Self as PyTypeInfo>::NAME
  }

  fn py_richcmp(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<bool> {
    if let Ok(other_self) = other.extract::<PyRef<'_, Self>>() {
      let ordering = self.cmp(&*other_self);
      return Ok(richcmp_ordering_to_bool(ordering, op));
    }

    if let Some(result) = py_richcmp_bytes_like(self.as_ref(), other, op)? {
      return Ok(result);
    }

    match op {
      CompareOp::Eq => Ok(false),
      CompareOp::Ne => Ok(true),
      _ => Err(PyTypeError::new_err(format!(
        "'<' not supported between instances of '{}' and '{}'",
        Self::py_type_name(),
        other.get_type().name()?
      ))),
    }
  }
}

impl<T> PyBufCmp for T where T: PyClass + PartialEq + Ord + AsRef<[u8]> {}

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

  fn py_map_try_get_error(err: TryGetError) -> PyErr {
    let (requested, available) = try_get_error_parts(&err);
    PyBufferError::new_err(format!(
      "cannot get {} bytes, only {} bytes available",
      requested, available
    ))
  }

  fn py_try_get<T, F>(&mut self, f: F) -> PyResult<T>
  where
    F: FnOnce(&mut Self) -> Result<T, TryGetError>,
  {
    f(self).map_err(Self::py_map_try_get_error)
  }

  fn py_get_u8(&mut self) -> PyResult<u8> {
    self.py_try_get(Self::try_get_u8)
  }

  fn py_get_i8(&mut self) -> PyResult<i8> {
    self.py_try_get(Self::try_get_i8)
  }

  fn py_get_u16(&mut self) -> PyResult<u16> {
    self.py_try_get(Self::try_get_u16)
  }

  fn py_get_u16_le(&mut self) -> PyResult<u16> {
    self.py_try_get(Self::try_get_u16_le)
  }

  fn py_get_i16(&mut self) -> PyResult<i16> {
    self.py_try_get(Self::try_get_i16)
  }

  fn py_get_i16_le(&mut self) -> PyResult<i16> {
    self.py_try_get(Self::try_get_i16_le)
  }

  fn py_get_u32(&mut self) -> PyResult<u32> {
    self.py_try_get(Self::try_get_u32)
  }

  fn py_get_u32_le(&mut self) -> PyResult<u32> {
    self.py_try_get(Self::try_get_u32_le)
  }

  fn py_get_i32(&mut self) -> PyResult<i32> {
    self.py_try_get(Self::try_get_i32)
  }

  fn py_get_i32_le(&mut self) -> PyResult<i32> {
    self.py_try_get(Self::try_get_i32_le)
  }

  fn py_get_f32(&mut self) -> PyResult<f32> {
    self.py_try_get(Self::try_get_f32)
  }

  fn py_get_f32_le(&mut self) -> PyResult<f32> {
    self.py_try_get(Self::try_get_f32_le)
  }

  fn py_get_u64(&mut self) -> PyResult<u64> {
    self.py_try_get(Self::try_get_u64)
  }

  fn py_get_u64_le(&mut self) -> PyResult<u64> {
    self.py_try_get(Self::try_get_u64_le)
  }

  fn py_get_i64(&mut self) -> PyResult<i64> {
    self.py_try_get(Self::try_get_i64)
  }

  fn py_get_i64_le(&mut self) -> PyResult<i64> {
    self.py_try_get(Self::try_get_i64_le)
  }

  fn py_get_f64(&mut self) -> PyResult<f64> {
    self.py_try_get(Self::try_get_f64)
  }

  fn py_get_f64_le(&mut self) -> PyResult<f64> {
    self.py_try_get(Self::try_get_f64_le)
  }

  fn py_get_u128(&mut self) -> PyResult<u128> {
    self.py_try_get(Self::try_get_u128)
  }

  fn py_get_u128_le(&mut self) -> PyResult<u128> {
    self.py_try_get(Self::try_get_u128_le)
  }

  fn py_get_i128(&mut self) -> PyResult<i128> {
    self.py_try_get(Self::try_get_i128)
  }

  fn py_get_i128_le(&mut self) -> PyResult<i128> {
    self.py_try_get(Self::try_get_i128_le)
  }

  fn py_get_uint(&mut self, nbytes: usize) -> PyResult<u64> {
    self
      .try_get_uint(nbytes)
      .map_err(Self::py_map_try_get_error)
  }

  fn py_get_uint_le(&mut self, nbytes: usize) -> PyResult<u64> {
    self
      .try_get_uint_le(nbytes)
      .map_err(Self::py_map_try_get_error)
  }

  fn py_get_int(&mut self, nbytes: usize) -> PyResult<i64> {
    self.try_get_int(nbytes).map_err(Self::py_map_try_get_error)
  }

  fn py_get_int_le(&mut self, nbytes: usize) -> PyResult<i64> {
    self
      .try_get_int_le(nbytes)
      .map_err(Self::py_map_try_get_error)
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

pub trait PyBufMutExt: PyBufExt + AsMut<[u8]> + BufMut {
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

impl<T> PyBufMutExt for T where T: PyBufExt + AsMut<[u8]> + BufMut {}
