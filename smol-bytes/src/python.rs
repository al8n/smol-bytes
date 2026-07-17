use super::*;
use pyo3::{
  basic::CompareOp,
  exceptions::{PyBufferError, PyIndexError, PyOverflowError, PyUnicodeDecodeError, PyValueError},
  prelude::*,
  types::{
    PyAny, PyByteArray, PyByteArrayMethods, PyBytes, PyInt, PyMemoryView, PyModule, PyRange,
    PySlice, PySliceIndices, PyString,
  },
};

use crate::{
  Buf, BytesMut, OutOfBounds, RangeOutOfBounds, TryGetError, utf8_buf::char_offset_to_byte,
};

#[cfg(any(feature = "std", feature = "alloc"))]
use crate::bytes::{self, RawBytes};

/// Verifies that consuming `cnt` bytes from a UTF-8 wrapper preserves its
/// validated-string invariant.
pub(crate) fn validate_utf8_advance(value: &impl AsRef<str>, cnt: usize) -> PyResult<()> {
  let value = value.as_ref();

  if cnt > value.len() {
    return Err(PyBufferError::new_err(format!(
      "cannot advance past remaining: {} > {}",
      cnt,
      value.len()
    )));
  }

  if !value.is_char_boundary(cnt) {
    return Err(PyBufferError::new_err(format!(
      "cannot advance to byte {}: not a UTF-8 character boundary",
      cnt
    )));
  }

  Ok(())
}

/// Preflight an allocation of `additional` bytes so absurd or hostile sizes
/// raise `MemoryError` like CPython containers instead of aborting the
/// process inside the infallible Rust allocator path.
///
/// Only Rust-side allocations proportional to caller-controlled sizes need this
/// preflight: allocations made through CPython's allocator (`PyBytes::new`,
/// `PyString`) already raise `MemoryError` natively.
///
/// This is a best-effort guard: it probes exactly `additional` bytes, so
/// reserve-style callers (whose underlying reservation may allocate
/// `len + additional` plus growth slack) are protected against absurd or
/// hostile sizes, not byte-exact OOM boundaries.
pub(crate) fn py_check_alloc(additional: usize) -> PyResult<()> {
  let mut probe: Vec<u8> = Vec::new();
  probe
    .try_reserve_exact(additional)
    .map_err(|_| pyo3::exceptions::PyMemoryError::new_err("allocation failed"))
}

impl From<TryPutError> for PyErr {
  fn from(err: TryPutError) -> PyErr {
    PyBufferError::new_err(format!(
      "cannot put {} bytes, only {} bytes available",
      err.requested, err.available
    ))
  }
}

impl From<InvalidIntegerLength> for PyErr {
  fn from(err: InvalidIntegerLength) -> PyErr {
    PyValueError::new_err(err.to_string())
  }
}

impl From<TryPutIntegerError> for PyErr {
  fn from(err: TryPutIntegerError) -> PyErr {
    match err {
      TryPutIntegerError::NotEnoughSpace(e) => e.into(),
      TryPutIntegerError::InvalidLength(e) => e.into(),
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

/// A fully normalized Python sequence subscript.
pub(crate) enum NormalizedSubscript {
  /// A valid single item index.
  Index(usize),
  /// A Python slice normalized for the supplied logical sequence length.
  Slice(PySliceIndices),
}

/// Returns every selected normalized slice position exactly once, already
/// converted to a valid Rust index.
///
/// `PySlice::indices` guarantees that every selected position lies in the
/// logical sequence, so each converted position is a valid index into it.
/// The final selected position is never advanced, which matters for a
/// maximal Python slice step in debug builds.
pub(crate) fn normalized_slice_positions(indices: &PySliceIndices) -> PyResult<Vec<usize>> {
  // Guard the positions `Vec` here so every caller is covered at one choke point.
  py_check_alloc(
    indices
      .slicelength
      .saturating_mul(core::mem::size_of::<usize>()),
  )?;
  let mut positions = Vec::with_capacity(indices.slicelength);
  let mut position = indices.start;
  let mut remaining = indices.slicelength;

  while remaining != 0 {
    positions.push(normalized_slice_position(position)?);
    remaining -= 1;
    if remaining != 0 {
      position = position
        .checked_add(indices.step)
        .ok_or_else(|| PyIndexError::new_err("slice index out of range"))?;
    }
  }

  Ok(positions)
}

fn normalized_slice_position(position: isize) -> PyResult<usize> {
  usize::try_from(position).map_err(|_| PyIndexError::new_err("slice index out of range"))
}

/// Evaluates Python's index protocol, avoiding a module lookup for builtin ints.
fn py_index_int<'py>(value: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyInt>> {
  if let Ok(value) = value.cast::<PyInt>() {
    return Ok(value.clone());
  }

  Ok(
    value
      .py()
      .get_type::<PyRange>()
      .call1((value,))?
      .getattr("stop")?
      .cast_into::<PyInt>()?,
  )
}

/// Normalizes a Python integer/index or slice without exposing Rust indexing.
pub(crate) fn normalize_subscript(
  index: &Bound<'_, PyAny>,
  logical_len: usize,
  index_error: &'static str,
) -> PyResult<NormalizedSubscript> {
  let py = index.py();
  let logical_len = isize::try_from(logical_len).map_err(|_| PyIndexError::new_err(index_error))?;

  if let Ok(slice) = index.cast::<PySlice>() {
    return slice.indices(logical_len).map(NormalizedSubscript::Slice);
  }

  let index = py_index_int(index)?.extract::<isize>().map_err(|err| {
    if err.is_instance_of::<PyOverflowError>(py) {
      PyIndexError::new_err(index_error)
    } else {
      err
    }
  })?;
  let normalized = if index < 0 {
    logical_len.checked_add(index)
  } else {
    Some(index)
  };
  match normalized.filter(|&index| index >= 0 && index < logical_len) {
    Some(index) => Ok(NormalizedSubscript::Index(index as usize)),
    None => Err(PyIndexError::new_err(index_error)),
  }
}

/// Implements Python bytes indexing and slicing for every raw wrapper.
pub(crate) fn py_bytes_getitem(data: &[u8], index: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
  let py = index.py();
  match normalize_subscript(index, data.len(), "buffer index out of range")? {
    NormalizedSubscript::Index(index) => Ok(data[index].into_pyobject(py)?.into_any().unbind()),
    NormalizedSubscript::Slice(indices) => {
      if indices.slicelength == 0 {
        return Ok(PyBytes::new(py, &[]).into());
      }

      if indices.step == 1 {
        let start = normalized_slice_position(indices.start)?;
        let stop = normalized_slice_position(indices.stop)?;
        return Ok(PyBytes::new(py, &data[start..stop]).into());
      }

      py_check_alloc(indices.slicelength)?;
      let mut result = std::vec::Vec::with_capacity(indices.slicelength);
      for position in normalized_slice_positions(&indices)? {
        result.push(data[position]);
      }
      Ok(PyBytes::new(py, &result).into())
    }
  }
}

/// Implements Python str indexing and slicing for every UTF-8 wrapper.
pub(crate) fn py_str_getitem(value: &str, index: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
  let py = index.py();
  let char_len = value.chars().count();
  match normalize_subscript(index, char_len, "string index out of range")? {
    NormalizedSubscript::Index(index) => Ok(
      value
        .chars()
        .nth(index)
        .ok_or_else(|| PyIndexError::new_err("string index out of range"))?
        .to_string()
        .into_pyobject(py)?
        .into_any()
        .unbind(),
    ),
    NormalizedSubscript::Slice(indices) if indices.step == 1 => {
      if indices.slicelength == 0 {
        return Ok(PyString::new(py, "").into());
      }

      let start = char_offset_to_byte(value, normalized_slice_position(indices.start)?)
        .ok_or_else(|| PyIndexError::new_err("string index out of range"))?;
      let stop = char_offset_to_byte(value, normalized_slice_position(indices.stop)?)
        .ok_or_else(|| PyIndexError::new_err("string index out of range"))?;
      Ok(PyString::new(py, &value[start..stop]).into())
    }
    NormalizedSubscript::Slice(indices) => {
      // Guards both the `chars` Vec built below and the `result` String it
      // feeds: each selected char can cost up to 4 UTF-8 bytes in `result`.
      py_check_alloc(
        char_len
          .saturating_mul(core::mem::size_of::<char>())
          .saturating_add(indices.slicelength.saturating_mul(4)),
      )?;
      let chars: std::vec::Vec<char> = value.chars().collect();
      let mut result = std::string::String::new();
      for position in normalized_slice_positions(&indices)? {
        let ch = chars
          .get(position)
          .ok_or_else(|| PyIndexError::new_err("string index out of range"))?;
        result.push(*ch);
      }
      Ok(PyString::new(py, &result).into())
    }
  }
}

/// Compares raw wrappers with one another, otherwise preserving native bytes semantics.
pub(crate) fn py_bytes_richcmp(
  value: &[u8],
  other: &Bound<'_, PyAny>,
  op: CompareOp,
) -> PyResult<Py<PyAny>> {
  macro_rules! compare_raw {
    ($type:ty) => {
      if let Ok(other) = other.extract::<PyRef<'_, $type>>() {
        return Ok(
          richcmp_ordering_to_bool(value.cmp(other.as_ref()), op)
            .into_pyobject(other.py())?
            .to_owned()
            .into_any()
            .unbind(),
        );
      }
    };
  }

  compare_raw!(crate::buffer::Buffer);
  compare_raw!(crate::bytes_mut::BytesMut);
  compare_raw!(crate::bytes::strategy::shared::PySharedBytes);
  compare_raw!(crate::bytes::strategy::compact::PyCompactBytes);

  let surrogate = PyBytes::new(other.py(), value);
  let method = match op {
    CompareOp::Lt => "__lt__",
    CompareOp::Le => "__le__",
    CompareOp::Eq => "__eq__",
    CompareOp::Ne => "__ne__",
    CompareOp::Gt => "__gt__",
    CompareOp::Ge => "__ge__",
  };

  if other.is_exact_instance_of::<PyByteArray>() || other.is_exact_instance_of::<PyMemoryView>() {
    return Ok(surrogate.rich_compare(other, op)?.unbind());
  }

  Ok(surrogate.call_method1(method, (other,))?.unbind())
}

/// Compares UTF-8 wrappers with one another, otherwise preserving native str semantics.
pub(crate) fn py_str_richcmp(
  value: &str,
  other: &Bound<'_, PyAny>,
  op: CompareOp,
) -> PyResult<Py<PyAny>> {
  macro_rules! compare_utf8 {
    ($type:ty) => {
      if let Ok(other) = other.extract::<PyRef<'_, $type>>() {
        return Ok(
          richcmp_ordering_to_bool(value.cmp(other.as_ref()), op)
            .into_pyobject(other.py())?
            .to_owned()
            .into_any()
            .unbind(),
        );
      }
    };
  }

  compare_utf8!(crate::utf8_buffer::Utf8Buffer);
  compare_utf8!(crate::utf8_bytes_mut::Utf8BytesMut);
  compare_utf8!(crate::utf8_bytes::PySharedUtf8Bytes);
  compare_utf8!(crate::utf8_bytes::PyCompactUtf8Bytes);

  let method = match op {
    CompareOp::Lt => "__lt__",
    CompareOp::Le => "__le__",
    CompareOp::Eq => "__eq__",
    CompareOp::Ne => "__ne__",
    CompareOp::Gt => "__gt__",
    CompareOp::Ge => "__ge__",
  };
  Ok(
    PyString::new(other.py(), value)
      .call_method1(method, (other,))?
      .unbind(),
  )
}

/// Preserves native Python bytes containment semantics for raw wrappers.
pub(crate) fn py_bytes_contains(value: &[u8], item: &Bound<'_, PyAny>) -> PyResult<bool> {
  PyBytes::new(item.py(), value).contains(item)
}

/// Preserves native Python string containment semantics for UTF-8 wrappers.
pub(crate) fn py_str_contains(value: &str, item: &Bound<'_, PyAny>) -> PyResult<bool> {
  PyString::new(item.py(), value).contains(item)
}

/// Parses the public Python variable-width integer argument.
pub(crate) fn py_integer_width(value: &Bound<'_, PyAny>) -> PyResult<usize> {
  let py = value.py();
  let width = py_index_int(value)?.extract::<usize>().map_err(|err| {
    if err.is_instance_of::<PyOverflowError>(py) {
      PyValueError::new_err("nbytes must be in the range 0..=8")
    } else {
      err
    }
  })?;

  if width <= 8 {
    Ok(width)
  } else {
    Err(PyValueError::new_err("nbytes must be in the range 0..=8"))
  }
}

fn py_assignment_byte(value: &Bound<'_, PyAny>) -> PyResult<u8> {
  let py = value.py();
  py_index_int(value)?.extract::<u8>().map_err(|err| {
    if err.is_instance_of::<PyOverflowError>(py) {
      PyValueError::new_err("byte must be in range(0, 256)")
    } else {
      err
    }
  })
}

fn py_slice_assignment_bytes(value: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
  let py = value.py();
  let bytes = PyByteArray::new(py, &[]);
  bytes.call_method1("__setitem__", (PySlice::full(py), value))?;
  py_check_alloc(bytes.len())?;
  Ok(bytes.to_vec())
}

pub trait PyBufCommon: Sized {
  fn py_try_split_to(&mut self, at: usize) -> Result<Self, OutOfBounds>;

  fn py_try_split_off(&mut self, at: usize) -> Result<Self, OutOfBounds>;

  fn py_try_slice(&self, start: usize, end: usize) -> Result<Self, RangeOutOfBounds>;

  fn py_try_advance(&mut self, cnt: usize) -> Result<(), OutOfBounds>;
}

impl PyBufCommon for Buffer {
  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_split_to(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    self.try_split_to(at)
  }

  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_split_off(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    self.try_split_off(at)
  }

  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_slice(&self, start: usize, end: usize) -> Result<Self, RangeOutOfBounds> {
    self.try_slice(start..end)
  }

  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_advance(&mut self, cnt: usize) -> Result<(), OutOfBounds> {
    self.try_advance(cnt)
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<S> PyBufCommon for RawBytes<S>
where
  RawBytes<S>: bytes::strategy::ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_split_to(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    self.try_split_to(at)
  }

  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_split_off(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    self.try_split_off(at)
  }

  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_slice(&self, start: usize, end: usize) -> Result<Self, RangeOutOfBounds> {
    self.try_slice(start..end)
  }

  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_advance(&mut self, cnt: usize) -> Result<(), OutOfBounds> {
    self.try_advance(cnt)
  }
}

impl PyBufCommon for BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_split_to(&mut self, at: usize) -> Result<BytesMut, OutOfBounds> {
    match self.try_split_to(at)? {
      Ok(bytes) => Ok(bytes),
      Err(buffer) => Ok(BytesMut::from_inline(buffer)),
    }
  }

  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_split_off(&mut self, at: usize) -> Result<BytesMut, OutOfBounds> {
    match self.try_split_off(at)? {
      Ok(bytes) => Ok(bytes),
      Err(buffer) => Ok(BytesMut::from_inline(buffer)),
    }
  }

  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_slice(&self, start: usize, end: usize) -> Result<BytesMut, RangeOutOfBounds> {
    let len = self.len();
    if start > end || end > len {
      return Err(RangeOutOfBounds::new(start, end, len));
    }
    Ok(BytesMut::from(&self.as_slice()[start..end]))
  }

  #[cfg_attr(not(coverage), inline(always))]
  fn py_try_advance(&mut self, cnt: usize) -> Result<(), OutOfBounds> {
    self.try_advance(cnt)
  }
}

pub trait PyBufCmp: AsRef<[u8]> {
  fn py_richcmp(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<Py<PyAny>> {
    py_bytes_richcmp(self.as_ref(), other, op)
  }
}

impl<T> PyBufCmp for T where T: AsRef<[u8]> {}

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
      .map_err(|err| PyUnicodeDecodeError::new_err_from_utf8(py, self.as_ref(), err))
  }

  fn py_contains(&self, item: &Bound<'_, PyAny>) -> PyResult<bool> {
    py_bytes_contains(self.as_ref(), item)
  }

  fn py_getitem(&self, index: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    py_bytes_getitem(self.as_ref(), index)
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

  fn py_get_uint_object(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<u64> {
    self.py_get_uint(py_integer_width(nbytes)?)
  }

  fn py_get_uint_le(&mut self, nbytes: usize) -> PyResult<u64> {
    self
      .try_get_uint_le(nbytes)
      .map_err(Self::py_map_try_get_error)
  }

  fn py_get_uint_le_object(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<u64> {
    self.py_get_uint_le(py_integer_width(nbytes)?)
  }

  fn py_get_int(&mut self, nbytes: usize) -> PyResult<i64> {
    self.try_get_int(nbytes).map_err(Self::py_map_try_get_error)
  }

  fn py_get_int_object(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<i64> {
    self.py_get_int(py_integer_width(nbytes)?)
  }

  fn py_get_int_le(&mut self, nbytes: usize) -> PyResult<i64> {
    self
      .try_get_int_le(nbytes)
      .map_err(Self::py_map_try_get_error)
  }

  fn py_get_int_le_object(&mut self, nbytes: &Bound<'_, PyAny>) -> PyResult<i64> {
    self.py_get_int_le(py_integer_width(nbytes)?)
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
  fn py_setitem(
    &mut self,
    index: &Bound<'_, PyAny>,
    value: &Bound<'_, PyAny>,
    self_assignment: Option<Vec<u8>>,
  ) -> PyResult<()> {
    let normalized = normalize_subscript(index, self.as_ref().len(), "buffer index out of range")?;

    match normalized {
      NormalizedSubscript::Index(index) => {
        let byte = py_assignment_byte(value)?;
        self.as_mut()[index] = byte;
        Ok(())
      }
      NormalizedSubscript::Slice(indices) => {
        let bytes = match self_assignment {
          Some(bytes) => bytes,
          None => py_slice_assignment_bytes(value)?,
        };

        // Contiguous fast path: a unit-step slice targets one continuous range,
        // so copy straight into it without materializing per-position indices.
        if indices.step == 1 {
          let start = normalized_slice_position(indices.start)?;
          let len = indices.slicelength;
          if bytes.len() != len {
            return Err(PyValueError::new_err(format!(
              "attempt to assign bytes of size {} to slice of size {}",
              bytes.len(),
              len
            )));
          }
          self.as_mut()[start..start + len].copy_from_slice(&bytes);
          return Ok(());
        }

        // General (extended-step) path: `normalized_slice_positions` preflights
        // its own allocation and already returns valid Rust indices.
        let positions = normalized_slice_positions(&indices)?;

        if bytes.len() != positions.len() {
          return Err(PyValueError::new_err(format!(
            "attempt to assign bytes of size {} to slice of size {}",
            bytes.len(),
            positions.len()
          )));
        }

        // Every conversion and length check is complete before the first write.
        let data = self.as_mut();
        for (position, byte) in positions.into_iter().zip(bytes) {
          data[position] = byte;
        }
        Ok(())
      }
    }
  }
}

impl<T> PyBufMutExt for T where T: PyBufExt + AsMut<[u8]> + BufMut {}

/// Register the core smol-bytes types into a Python module.
///
/// `module_name` is the user-facing Python import path (e.g., `"smol_bytes"`
/// or `"my_crate"`). It is written to each class's `__module__` attribute
/// so that `repr()` and `pickle` resolve correctly.
///
/// # Example — downstream crate
///
/// ```rust,ignore
/// #[pyo3::prelude::pymodule]
/// fn my_crate(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
///     smol_bytes::register_classes(m, "my_crate")?;
///     Ok(())
/// }
/// ```
pub fn register_classes(m: &Bound<'_, PyModule>, module_name: &str) -> PyResult<()> {
  m.add_class::<crate::buffer::Buffer>()?;
  m.getattr("Buffer")?.setattr("__module__", module_name)?;

  m.add_class::<crate::bytes_mut::BytesMut>()?;
  m.getattr("BytesMut")?.setattr("__module__", module_name)?;

  m.add_class::<crate::utf8_buffer::Utf8Buffer>()?;
  m.getattr("Utf8Buffer")?
    .setattr("__module__", module_name)?;

  m.add_class::<crate::utf8_bytes::PySharedUtf8Bytes>()?;
  m.getattr("Utf8Bytes")?.setattr("__module__", module_name)?;

  m.add_class::<crate::utf8_bytes_mut::Utf8BytesMut>()?;
  m.getattr("Utf8BytesMut")?
    .setattr("__module__", module_name)?;

  Ok(())
}

/// Register `shared::Bytes` ([`PySharedBytes`](crate::shared::PySharedBytes))
/// into a Python module.
///
/// `module_name` is the user-facing import path (e.g., `"smol_bytes.shared"`).
pub fn register_shared(m: &Bound<'_, PyModule>, module_name: &str) -> PyResult<()> {
  m.add_class::<crate::bytes::strategy::shared::PySharedBytes>()?;
  m.getattr("Bytes")?.setattr("__module__", module_name)?;
  m.add_class::<crate::utf8_bytes::PySharedUtf8Bytes>()?;
  m.getattr("Utf8Bytes")?.setattr("__module__", module_name)?;
  Ok(())
}

/// Register `compact::Bytes` ([`PyCompactBytes`](crate::compact::PyCompactBytes))
/// into a Python module.
///
/// `module_name` is the user-facing import path (e.g., `"smol_bytes.compact"`).
pub fn register_compact(m: &Bound<'_, PyModule>, module_name: &str) -> PyResult<()> {
  m.add_class::<crate::bytes::strategy::compact::PyCompactBytes>()?;
  m.getattr("Bytes")?.setattr("__module__", module_name)?;
  m.add_class::<crate::utf8_bytes::PyCompactUtf8Bytes>()?;
  m.getattr("Utf8Bytes")?.setattr("__module__", module_name)?;
  Ok(())
}
