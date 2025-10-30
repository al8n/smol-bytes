use crate::{python::PyBufExt, Buf, DefaultHasher};
use pyo3::{
  basic::CompareOp,
  exceptions::{PyBufferError, PyUnicodeDecodeError},
  prelude::{Bound, *},
  types::{PyAny, PyBytes, PyString},
};

use crate::TryGetError;

type IntoIter = ::bytes::buf::IntoIter<super::Bytes>;

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

#[pyclass(name = "Bytes")]
#[derive(Clone)]
pub struct PyCompactBytes {
  inner: super::Bytes,
}

impl From<super::Bytes> for PyCompactBytes {
  fn from(inner: super::Bytes) -> Self {
    Self { inner }
  }
}

impl PyCompactBytes {
  fn map_try_get_err(err: TryGetError) -> PyErr {
    #[cfg(any(feature = "std", feature = "alloc"))]
    {
      return PyBufferError::new_err(format!(
        "cannot get {} bytes, only {} bytes available",
        err.requested, err.available
      ));
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    {
      return PyBufferError::new_err(format!(
        "cannot get {} bytes, only {} bytes available",
        err.requested, err.available
      ));
    }
  }
}

#[pymethods]
impl PyCompactBytes {
  #[new]
  fn new_python() -> Self {
    Self {
      inner: super::Bytes::new(),
    }
  }

  #[staticmethod]
  #[pyo3(name = "from_bytes")]
  fn __python_from_bytes(py_bytes: &[u8]) -> Self {
    Self {
      inner: super::Bytes::copy_from_slice(py_bytes),
    }
  }

  #[staticmethod]
  #[pyo3(name = "from_str")]
  fn __python_from_str(py_str: &str) -> Self {
    Self {
      inner: super::Bytes::from(py_str),
    }
  }

  fn __bytes__<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    self.inner.py_bytes(py)
  }

  fn __bool__(&self) -> bool {
    !self.inner.is_empty()
  }

  fn __hash__(&self) -> u64 {
    use ::core::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    self.inner.hash(&mut hasher);
    hasher.finish()
  }

  fn __len__(&self) -> usize {
    self.inner.py_len()
  }

  fn __contains__(&self, item: &Bound<'_, PyAny>) -> PyResult<bool> {
    self.inner.py_contains(item)
  }

  fn __getitem__(&self, index: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    self.inner.py_getitem(index)
  }

  fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<Iter>> {
    let iter = Iter {
      inner: slf.inner.clone().into_iter(),
    };
    Py::new(slf.py(), iter)
  }

  fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<bool> {
    use core::cmp::Ordering;

    if let Ok(other_bytes) = other.extract::<PyRef<'_, Self>>() {
      let ordering = self.inner.cmp(&other_bytes.inner);
      return Ok(match op {
        CompareOp::Lt => ordering == Ordering::Less,
        CompareOp::Le => ordering != Ordering::Greater,
        CompareOp::Eq => ordering == Ordering::Equal,
        CompareOp::Ne => ordering != Ordering::Equal,
        CompareOp::Gt => ordering == Ordering::Greater,
        CompareOp::Ge => ordering != Ordering::Less,
      });
    }

    macro_rules! compare {
      ($other_val:expr) => {{
        if let Some(ordering) = self.inner.partial_cmp($other_val) {
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

    if let Ok(py_bytes) = other.cast::<PyBytes>() {
      compare!(py_bytes.as_bytes());
    }

    if let Ok(py_str) = other.cast::<PyString>() {
      if let Ok(s) = py_str.to_cow() {
        compare!(s.as_ref());
      }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    if let Ok(s) = other.extract::<std::string::String>() {
      compare!(&s);
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    if let Ok(byte_vec) = other.extract::<std::vec::Vec<u8>>() {
      compare!(&byte_vec);
    }

    match op {
      CompareOp::Eq => Ok(false),
      CompareOp::Ne => Ok(true),
      _ => Err(pyo3::exceptions::PyTypeError::new_err(format!(
        "'<' not supported between instances of 'Bytes' and '{}'",
        other.get_type().name()?
      ))),
    }
  }

  fn __str__(&self) -> PyResult<&str> {
    ::core::str::from_utf8(self.inner.as_ref()).map_err(|e| {
      PyUnicodeDecodeError::new_err(format!(
        "invalid utf-8 sequence at byte {}: {}",
        e.valid_up_to(),
        e
      ))
    })
  }

  fn __repr__(&self) -> String {
    format!("{:?}", self.inner)
  }

  #[pyo3(name = "as_bytes")]
  fn __python_as_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    self.inner.py_bytes(py)
  }

  #[pyo3(name = "to_string")]
  fn __python_to_string<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyString>> {
    self.inner.py_to_string(py)
  }

  #[pyo3(name = "is_inline")]
  fn __python_is_inline(&self) -> bool {
    self.inner.is_inline()
  }

  #[pyo3(name = "is_heap")]
  fn __python_is_heap(&self) -> bool {
    self.inner.is_heap()
  }

  #[pyo3(name = "remaining")]
  fn __python_remaining(&self) -> usize {
    self.inner.py_remaining()
  }

  #[pyo3(name = "advance")]
  fn __python_advance(&mut self, cnt: usize) -> PyResult<()> {
    self.inner.py_advance(cnt)
  }

  #[pyo3(name = "truncate")]
  fn __python_truncate(&mut self, new_len: usize) {
    self.inner.truncate(new_len);
  }

  #[pyo3(name = "clear")]
  fn __python_clear(&mut self) {
    self.inner.clear();
  }

  #[pyo3(name = "split_to")]
  fn __python_split_to(&mut self, at: usize) -> PyResult<Self> {
    self.inner.py_split_to(at).map(|inner| Self { inner })
  }

  #[pyo3(name = "split_off")]
  fn __python_split_off(&mut self, at: usize) -> PyResult<Self> {
    self.inner.py_split_off(at).map(|inner| Self { inner })
  }

  #[pyo3(name = "slice")]
  fn __python_slice(&self, start: usize, end: usize) -> PyResult<Self> {
    self.inner.py_slice(start, end).map(|inner| Self { inner })
  }

  #[pyo3(name = "__copy__")]
  fn __python_copy(&self) -> Self {
    self.clone()
  }

  #[pyo3(name = "__deepcopy__")]
  fn __python_deepcopy(&self, _memo: &Bound<'_, PyAny>) -> Self {
    self.clone()
  }

  #[pyo3(name = "get_u8")]
  fn __python_get_u8(&mut self) -> PyResult<u8> {
    self.inner.try_get_u8().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_i8")]
  fn __python_get_i8(&mut self) -> PyResult<i8> {
    self.inner.try_get_i8().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_u16")]
  fn __python_get_u16(&mut self) -> PyResult<u16> {
    self.inner.try_get_u16().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_u16_le")]
  fn __python_get_u16_le(&mut self) -> PyResult<u16> {
    self.inner.try_get_u16_le().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_i16")]
  fn __python_get_i16(&mut self) -> PyResult<i16> {
    self.inner.try_get_i16().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_i16_le")]
  fn __python_get_i16_le(&mut self) -> PyResult<i16> {
    self.inner.try_get_i16_le().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_u32")]
  fn __python_get_u32(&mut self) -> PyResult<u32> {
    self.inner.try_get_u32().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_u32_le")]
  fn __python_get_u32_le(&mut self) -> PyResult<u32> {
    self.inner.try_get_u32_le().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_i32")]
  fn __python_get_i32(&mut self) -> PyResult<i32> {
    self.inner.try_get_i32().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_i32_le")]
  fn __python_get_i32_le(&mut self) -> PyResult<i32> {
    self.inner.try_get_i32_le().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_f32")]
  fn __python_get_f32(&mut self) -> PyResult<f32> {
    self.inner.try_get_f32().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_f32_le")]
  fn __python_get_f32_le(&mut self) -> PyResult<f32> {
    self.inner.try_get_f32_le().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_u64")]
  fn __python_get_u64(&mut self) -> PyResult<u64> {
    self.inner.try_get_u64().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_u64_le")]
  fn __python_get_u64_le(&mut self) -> PyResult<u64> {
    self.inner.try_get_u64_le().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_i64")]
  fn __python_get_i64(&mut self) -> PyResult<i64> {
    self.inner.try_get_i64().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_i64_le")]
  fn __python_get_i64_le(&mut self) -> PyResult<i64> {
    self.inner.try_get_i64_le().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_f64")]
  fn __python_get_f64(&mut self) -> PyResult<f64> {
    self.inner.try_get_f64().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_f64_le")]
  fn __python_get_f64_le(&mut self) -> PyResult<f64> {
    self.inner.try_get_f64_le().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_u128")]
  fn __python_get_u128(&mut self) -> PyResult<u128> {
    self.inner.try_get_u128().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_u128_le")]
  fn __python_get_u128_le(&mut self) -> PyResult<u128> {
    self.inner.try_get_u128_le().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_i128")]
  fn __python_get_i128(&mut self) -> PyResult<i128> {
    self.inner.try_get_i128().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_i128_le")]
  fn __python_get_i128_le(&mut self) -> PyResult<i128> {
    self.inner.try_get_i128_le().map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_uint")]
  fn __python_get_uint(&mut self, nbytes: usize) -> PyResult<u64> {
    self
      .inner
      .try_get_uint(nbytes)
      .map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_uint_le")]
  fn __python_get_uint_le(&mut self, nbytes: usize) -> PyResult<u64> {
    self
      .inner
      .try_get_uint_le(nbytes)
      .map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_int")]
  fn __python_get_int(&mut self, nbytes: usize) -> PyResult<i64> {
    self
      .inner
      .try_get_int(nbytes)
      .map_err(Self::map_try_get_err)
  }

  #[pyo3(name = "get_int_le")]
  fn __python_get_int_le(&mut self, nbytes: usize) -> PyResult<i64> {
    self
      .inner
      .try_get_int_le(nbytes)
      .map_err(Self::map_try_get_err)
  }
}
