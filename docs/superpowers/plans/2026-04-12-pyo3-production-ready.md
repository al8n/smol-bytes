# pyo3 Production-Ready Bindings — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the pyo3 Python bindings production-ready with module registration, pickle, buffer protocol, missing dunder methods, package setup, type stubs, tests, and CI.

**Architecture:** The native extension compiles as `smol_bytes._smol_bytes` via maturin. Pure-Python `__init__.py` files in `python/smol_bytes/` re-export types. Submodules `shared` and `compact` mirror the Rust strategy pattern. A `PySharedBytes` wrapper (modeled on existing `PyCompactBytes`) bridges the generic `RawBytes<Shared>`.

**Tech Stack:** pyo3 0.28, maturin ≥1.0, pytest, GitHub Actions

**Spec:** `docs/superpowers/specs/2026-04-12-pyo3-bindings-design.md`

---

## File Map

### New files
| File | Purpose |
|---|---|
| `crates/smol-bytes/src/bytes/strategy/shared/python.rs` | `PySharedBytes` wrapper + `#[pymethods]` |
| `pyproject.toml` | maturin build config |
| `python/smol_bytes/__init__.py` | root re-exports |
| `python/smol_bytes/__init__.pyi` | type stubs for root types |
| `python/smol_bytes/shared/__init__.py` | shared submodule re-export |
| `python/smol_bytes/shared/__init__.pyi` | type stubs for shared.Bytes |
| `python/smol_bytes/compact/__init__.py` | compact submodule re-export |
| `python/smol_bytes/compact/__init__.pyi` | type stubs for compact.Bytes |
| `tests/python/test_smol_bytes.py` | pytest suite |
| `.github/workflows/python.yml` | CI workflow |

### Modified files
| File | Changes |
|---|---|
| `crates/smol-bytes/src/bytes/strategy/shared.rs` | Add `#[cfg(feature = "pyo3")] mod python;` |
| `crates/smol-bytes/src/python.rs` | Add `#[pymodule]` function with submodule registration |
| `crates/smol-bytes/src/buffer/python.rs` | Add `__reduce__`, `__buffer__` |
| `crates/smol-bytes/src/bytes_mut/python.rs` | Add `__reduce__`, `__buffer__` |
| `crates/smol-bytes/src/bytes/strategy/compact/python.rs` | Add `__reduce__`, `__buffer__`, `__copy__`, `__deepcopy__` |
| `crates/smol-bytes/src/utf8_buffer/python.rs` | Add `__reduce__`, `__iter__`, `__bytes__` |
| `crates/smol-bytes/src/utf8_bytes/python.rs` | Add `__hash__`, `__reduce__`, `__iter__`, `__bytes__` |
| `crates/smol-bytes/src/utf8_bytes_mut/python.rs` | Add `__reduce__`, `__iter__`, `__bytes__` |
| `crates/smol-bytes/Cargo.toml` | Ensure pyo3 `abi3` feature is correct |

---

## Task 1: Create PySharedBytes wrapper

**Files:**
- Create: `crates/smol-bytes/src/bytes/strategy/shared/python.rs`
- Modify: `crates/smol-bytes/src/bytes/strategy/shared.rs` (add `mod python`)

This mirrors the existing `PyCompactBytes` in `compact/python.rs`. The wrapper delegates to the shared helper traits (`PyBufExt`, `PyBufCmp`, `PyBufCommon`) already defined in `src/python.rs`.

- [ ] **Step 1: Create the shared/ directory and python.rs**

Create `crates/smol-bytes/src/bytes/strategy/shared/python.rs`:

```rust
use super::*;
use crate::python::{PyBufCommon, PyBufCmp, PyBufExt};
use crate::{Buf, DefaultHasher, OutOfBounds, RangeOutOfBounds};
use pyo3::{
  basic::CompareOp,
  exceptions::{PyIndexError, PyTypeError, PyUnicodeDecodeError},
  prelude::*,
  types::{PyBytes, PyString},
};

use bytes::buf::IntoIter;

type IntoIter_ = IntoIter<PySharedBytes>;

const DOC: &str = r#"Bytes

Immutable byte buffer using the Shared strategy. Stores up to 62 bytes inline
before falling back to heap storage with zero-copy cloning via reference counting.
All `get_*` accessors from the `Buf` trait are available, along with rich comparisons,
slicing, and split helpers."#;

static DOC_ONCE: std::sync::Once = std::sync::Once::new();

#[derive(Debug)]
#[pyclass]
struct Iter {
  inner: IntoIter_,
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

/// Python wrapper for `shared::Bytes` (`RawBytes<Shared>`).
#[pyclass(name = "Bytes", skip_from_py_object)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PySharedBytes {
  inner: super::Bytes,
}

impl From<super::Bytes> for PySharedBytes {
  fn from(inner: super::Bytes) -> Self {
    Self { inner }
  }
}

impl AsRef<[u8]> for PySharedBytes {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_ref(&self) -> &[u8] {
    self.inner.as_ref()
  }
}

impl Buf for PySharedBytes {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn remaining(&self) -> usize {
    self.inner.remaining()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn chunk(&self) -> &[u8] {
    self.inner.chunk()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn advance(&mut self, cnt: usize) {
    Buf::advance(&mut self.inner, cnt);
  }
}

impl PyBufCommon for PySharedBytes {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_split_to(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    self.inner.try_split_to(at).map(Into::into)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_split_off(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    self.inner.try_split_off(at).map(Into::into)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_slice(&self, start: usize, end: usize) -> Result<Self, RangeOutOfBounds> {
    self.inner.try_slice(start..end).map(Into::into)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn py_try_advance(&mut self, cnt: usize) -> Result<(), OutOfBounds> {
    self.inner.try_advance(cnt)
  }
}

#[pymethods]
impl PySharedBytes {
  #[new]
  fn new_python(py: Python<'_>) -> Self {
    DOC_ONCE.call_once(|| {
      let ty = py.get_type::<Self>();
      let _ = ty.setattr("__doc__", DOC);
    });
    Self { inner: super::Bytes::new() }
  }

  #[staticmethod]
  #[pyo3(name = "from_bytes")]
  fn __python_from_bytes(py_bytes: &[u8]) -> Self {
    Self { inner: super::Bytes::copy_from_slice(py_bytes) }
  }

  #[staticmethod]
  #[pyo3(name = "from_str")]
  fn __python_from_str(py_str: &str) -> Self {
    Self { inner: super::Bytes::from(py_str) }
  }

  fn __bytes__<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    self.py_bytes(py)
  }

  fn __bool__(&self) -> bool { !self.inner.is_empty() }

  fn __hash__(&self) -> u64 {
    use ::core::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    self.inner.hash(&mut hasher);
    hasher.finish()
  }

  fn __len__(&self) -> usize { self.py_len() }

  fn __contains__(&self, item: &Bound<'_, PyAny>) -> bool { self.py_contains(item) }

  fn __getitem__(&self, index: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> { self.py_getitem(index) }

  fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<Iter>> {
    let iter = Iter { inner: slf.inner.clone().into_iter() };
    Py::new(slf.py(), iter)
  }

  fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<bool> {
    self.py_richcmp(other, op)
  }

  fn __str__(&self) -> PyResult<&str> {
    ::core::str::from_utf8(self.inner.as_ref()).map_err(|e| {
      PyUnicodeDecodeError::new_err(format!("invalid utf-8 at byte {}: {}", e.valid_up_to(), e))
    })
  }

  fn __repr__(&self) -> String { format!("{:?}", self.inner) }

  fn __copy__(&self) -> Self { self.clone() }
  fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> Self { self.clone() }

  fn __reduce__(&self, py: Python<'_>) -> PyResult<(PyObject, (Py<PyBytes>,))> {
    let cls = py.get_type::<Self>();
    let from_bytes = cls.getattr("from_bytes")?;
    let data = PyBytes::new(py, self.inner.as_ref());
    Ok((from_bytes.into(), (data.into(),)))
  }

  #[pyo3(name = "as_bytes")]
  fn __python_as_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> { self.py_bytes(py) }

  #[pyo3(name = "to_string")]
  fn __python_to_string<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyString>> { self.py_to_string(py) }

  #[pyo3(name = "is_inline")]
  fn __python_is_inline(&self) -> bool { self.inner.is_inline() }

  #[pyo3(name = "is_heap")]
  fn __python_is_heap(&self) -> bool { self.inner.is_heap() }

  #[pyo3(name = "remaining")]
  fn __python_remaining(&self) -> usize { self.py_remaining() }

  #[pyo3(name = "advance")]
  fn __python_advance(&mut self, cnt: usize) -> PyResult<()> { self.py_advance(cnt) }

  #[pyo3(name = "truncate")]
  fn __python_truncate(&mut self, new_len: usize) { self.inner.truncate(new_len); }

  #[pyo3(name = "clear")]
  fn __python_clear(&mut self) { self.inner.clear(); }

  #[pyo3(name = "split_to")]
  fn __python_split_to(&mut self, at: usize) -> PyResult<Self> { self.py_split_to(at) }

  #[pyo3(name = "split_off")]
  fn __python_split_off(&mut self, at: usize) -> PyResult<Self> { self.py_split_off(at) }

  #[pyo3(name = "slice")]
  fn __python_slice(&self, start: usize, end: usize) -> PyResult<Self> { self.py_slice(start, end) }

  // --- Buf getters (delegated via macro in PyBufExt) ---
  // These are inherited from the trait impl. For pyo3 we need explicit methods.

  crate::python::forward_py_buf_getters!();
}
```

Note: the `forward_py_buf_getters!()` macro needs to be created OR the getter methods need to be listed explicitly. Check how `compact/python.rs` and `buffer/python.rs` expose the get_u8/get_u16/etc. methods — they use `crate::python::PyBufExt` trait methods via explicit `#[pymethods]` wrappers. Copy that same block from `compact/python.rs` (lines after `__python_slice`).

- [ ] **Step 2: Add `mod python` to shared.rs**

At the end of `crates/smol-bytes/src/bytes/strategy/shared.rs`, before the `pub type Bytes` line, add:

```rust
#[cfg(feature = "pyo3")]
mod python;
#[cfg(feature = "pyo3")]
pub use python::PySharedBytes;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check --no-default-features --features "std pyo3"`
Expected: clean compilation

- [ ] **Step 4: Commit**

```bash
git add crates/smol-bytes/src/bytes/strategy/shared/python.rs crates/smol-bytes/src/bytes/strategy/shared.rs
git commit -m "feat(pyo3): add PySharedBytes wrapper for shared::Bytes"
```

---

## Task 2: Add `#[pymodule]` with submodules

**Files:**
- Modify: `crates/smol-bytes/src/python.rs`

- [ ] **Step 1: Add the module function**

At the end of `crates/smol-bytes/src/python.rs`, add:

```rust
#[pymodule]
fn _smol_bytes(m: &Bound<'_, PyModule>) -> PyResult<()> {
  use crate::{
    buffer::Buffer,
    bytes_mut::BytesMut,
    utf8_buffer::Utf8Buffer,
    utf8_bytes::Utf8Bytes,
    utf8_bytes_mut::Utf8BytesMut,
    bytes::strategy::shared::python::PySharedBytes,
    bytes::strategy::compact::python::PyCompactBytes,
  };

  m.add_class::<Buffer>()?;
  m.add_class::<BytesMut>()?;
  m.add_class::<Utf8Buffer>()?;
  m.add_class::<Utf8Bytes>()?;
  m.add_class::<Utf8BytesMut>()?;

  let shared = PyModule::new(m.py(), "shared")?;
  shared.add_class::<PySharedBytes>()?;
  m.add_submodule(&shared)?;

  let compact = PyModule::new(m.py(), "compact")?;
  compact.add_class::<PyCompactBytes>()?;
  m.add_submodule(&compact)?;

  Ok(())
}
```

Also ensure the necessary imports are at the top: `use pyo3::prelude::*;` and `use pyo3::types::PyModule;`.

Note: `PyCompactBytes` may need to be made `pub` in `compact/python.rs` (it's currently just `pub struct`). Also check `compact.rs` has a `pub use python::PyCompactBytes;`.

- [ ] **Step 2: Ensure pub exports for PyCompactBytes**

In `crates/smol-bytes/src/bytes/strategy/compact.rs`, verify the pyo3 module re-exports:
```rust
#[cfg(feature = "pyo3")]
mod python;
#[cfg(feature = "pyo3")]
pub use python::PyCompactBytes;
```

If `pub use` is missing, add it.

- [ ] **Step 3: Verify it compiles**

Run: `cargo check --no-default-features --features "std pyo3"`
Expected: clean compilation

- [ ] **Step 4: Commit**

```bash
git add crates/smol-bytes/src/python.rs crates/smol-bytes/src/bytes/strategy/compact.rs
git commit -m "feat(pyo3): add #[pymodule] with shared/compact submodules"
```

---

## Task 3: Add pickle support to all types

**Files:**
- Modify: all 7 python.rs files (buffer, bytes_mut, compact, shared, utf8_buffer, utf8_bytes, utf8_bytes_mut)

The pattern for byte types:
```rust
fn __reduce__(&self, py: Python<'_>) -> PyResult<(PyObject, (Py<PyBytes>,))> {
    let cls = py.get_type::<Self>();
    let from_bytes = cls.getattr("from_bytes")?;
    let data = PyBytes::new(py, self.as_ref());
    Ok((from_bytes.into(), (data.into(),)))
}
```

The pattern for UTF-8 types:
```rust
fn __reduce__(&self, py: Python<'_>) -> PyResult<(PyObject, (String,))> {
    let cls = py.get_type::<Self>();
    let from_str = cls.getattr("from_str")?;
    Ok((from_str.into(), (self.as_str().to_string(),)))
}
```

- [ ] **Step 1: Add `__reduce__` to Buffer** in `buffer/python.rs` inside the `#[pymethods]` block
- [ ] **Step 2: Add `__reduce__` to BytesMut** in `bytes_mut/python.rs`
- [ ] **Step 3: Add `__reduce__` to PyCompactBytes** in `compact/python.rs` (PySharedBytes already has it from Task 1)
- [ ] **Step 4: Add `__reduce__` to Utf8Buffer** in `utf8_buffer/python.rs`
- [ ] **Step 5: Add `__reduce__` to Utf8Bytes** in `utf8_bytes/python.rs`
- [ ] **Step 6: Add `__reduce__` to Utf8BytesMut** in `utf8_bytes_mut/python.rs`
- [ ] **Step 7: Verify compilation**

Run: `cargo check --no-default-features --features "std pyo3"`

- [ ] **Step 8: Commit**

```bash
git add crates/smol-bytes/src/*/python.rs crates/smol-bytes/src/bytes/strategy/*/python.rs
git commit -m "feat(pyo3): add pickle support (__reduce__) to all types"
```

---

## Task 4: Add buffer protocol to byte types

**Files:**
- Modify: `buffer/python.rs`, `bytes_mut/python.rs`, `compact/python.rs`, `shared/python.rs`

pyo3 0.28 supports the buffer protocol via `__buffer__` / `__release_buffer__` or the `#[pyclass(buffer)]` approach. The simplest method for read-only access:

```rust
unsafe fn __getbuffer__(
    slf: PyRef<'_, Self>,
    view: *mut pyo3::ffi::Py_buffer,
    flags: std::os::raw::c_int,
) -> PyResult<()> {
    use pyo3::ffi;
    use std::ffi::c_void;

    if flags & ffi::PyBUF_WRITABLE != 0 {
        return Err(pyo3::exceptions::PyBufferError::new_err("buffer is read-only"));
    }

    let slice = slf.as_ref();
    unsafe {
        (*view).buf = slice.as_ptr() as *mut c_void;
        (*view).len = slice.len() as isize;
        (*view).itemsize = 1;
        (*view).ndim = 1;
        (*view).format = b"B\0".as_ptr() as *mut _;
        (*view).shape = &mut (*view).len;
        (*view).strides = &mut (*view).itemsize;
        (*view).suboffsets = std::ptr::null_mut();
        (*view).readonly = 1;
        (*view).obj = pyo3::ffi::_Py_NewRef(slf.as_ptr());
        (*view).internal = std::ptr::null_mut();
    }
    Ok(())
}
```

Note: check pyo3 0.28's recommended buffer protocol API — it may have changed from older versions. Look at how pyo3 docs recommend implementing `__getbuffer__`. Adapt the pattern above to the version in use.

- [ ] **Step 1: Add `__getbuffer__` to Buffer**
- [ ] **Step 2: Add `__getbuffer__` to BytesMut**
- [ ] **Step 3: Add `__getbuffer__` to PyCompactBytes**
- [ ] **Step 4: PySharedBytes already set up in Task 1 — add if not already present**
- [ ] **Step 5: Verify compilation**

Run: `cargo check --no-default-features --features "std pyo3"`

- [ ] **Step 6: Commit**

```bash
git add crates/smol-bytes/src/*/python.rs crates/smol-bytes/src/bytes/strategy/*/python.rs
git commit -m "feat(pyo3): add buffer protocol (__getbuffer__) to byte types"
```

---

## Task 5: Fill remaining dunder method gaps

**Files:**
- Modify: `utf8_buffer/python.rs`, `utf8_bytes/python.rs`, `utf8_bytes_mut/python.rs`

### 5a. `__hash__` for Utf8Bytes (immutable)

```rust
fn __hash__(&self) -> u64 {
    use ::core::hash::{Hash, Hasher};
    let mut hasher = crate::DefaultHasher::new();
    self.as_str().hash(&mut hasher);
    hasher.finish()
}
```

Add to `utf8_bytes/python.rs` only (Utf8Bytes is the only immutable UTF-8 type without `__hash__`).

### 5b. `__iter__` for all UTF-8 types

Yields single-character `str` values:

```rust
fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<Utf8Iter>> {
    let chars: Vec<char> = slf.as_str().chars().collect();
    Py::new(slf.py(), Utf8Iter { chars, index: 0 })
}
```

With the iterator helper struct (one per file, or shared in `src/python.rs`):

```rust
#[derive(Debug)]
#[pyclass]
struct Utf8Iter {
    chars: Vec<char>,
    index: usize,
}

#[pymethods]
impl Utf8Iter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> { slf }

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
```

Add to `utf8_buffer/python.rs`, `utf8_bytes/python.rs`, `utf8_bytes_mut/python.rs`.

### 5c. `__bytes__` for all UTF-8 types

Returns UTF-8 encoded bytes:

```rust
fn __bytes__<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    PyBytes::new(py, self.as_str().as_bytes())
}
```

Add to all three UTF-8 python.rs files.

- [ ] **Step 1: Add `__hash__` to Utf8Bytes**
- [ ] **Step 2: Add `Utf8Iter` struct + `__iter__` to Utf8Buffer**
- [ ] **Step 3: Add `Utf8Iter` struct + `__iter__` to Utf8Bytes**
- [ ] **Step 4: Add `Utf8Iter` struct + `__iter__` to Utf8BytesMut**
- [ ] **Step 5: Add `__bytes__` to all three UTF-8 types**
- [ ] **Step 6: Verify compilation**

Run: `cargo check --no-default-features --features "std pyo3"`

- [ ] **Step 7: Commit**

```bash
git add crates/smol-bytes/src/utf8_*/python.rs
git commit -m "feat(pyo3): add __hash__, __iter__, __bytes__ to UTF-8 types"
```

---

## Task 6: Package setup

**Files:**
- Create: `pyproject.toml`, `python/smol_bytes/__init__.py`, `python/smol_bytes/shared/__init__.py`, `python/smol_bytes/compact/__init__.py`

- [ ] **Step 1: Create pyproject.toml** at repo root

```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "smol-bytes"
requires-python = ">=3.8"
description = "High-performance, clone-efficient byte buffers optimized for small data"
license = { text = "MIT OR Apache-2.0" }
classifiers = [
    "Programming Language :: Rust",
    "Programming Language :: Python :: Implementation :: CPython",
    "Programming Language :: Python :: 3",
]

[tool.maturin]
features = ["pyo3/extension-module"]
module-name = "smol_bytes._smol_bytes"
python-source = "python"
manifest-path = "crates/smol-bytes/Cargo.toml"
```

- [ ] **Step 2: Create python/smol_bytes/__init__.py**

```python
"""smol-bytes: High-performance, clone-efficient byte buffers."""

from ._smol_bytes import Buffer, BytesMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut

__all__ = ["Buffer", "BytesMut", "Utf8Buffer", "Utf8Bytes", "Utf8BytesMut"]
```

- [ ] **Step 3: Create python/smol_bytes/shared/__init__.py**

```python
"""Shared strategy — preserves heap allocations for fast bytes::Bytes interop."""

from .._smol_bytes.shared import Bytes

__all__ = ["Bytes"]
```

- [ ] **Step 4: Create python/smol_bytes/compact/__init__.py**

```python
"""Compact strategy — aggressively inlines to minimize memory usage."""

from .._smol_bytes.compact import Bytes

__all__ = ["Bytes"]
```

- [ ] **Step 5: Verify maturin builds**

Run: `maturin develop --features pyo3` (requires maturin installed: `pip install maturin`)
Expected: builds and installs the package into the active virtualenv

- [ ] **Step 6: Quick smoke test**

Run:
```bash
python -c "from smol_bytes import Buffer; b = Buffer(); print(b)"
python -c "from smol_bytes.shared import Bytes; b = Bytes.from_bytes(b'hello'); print(bytes(b))"
python -c "from smol_bytes.compact import Bytes; b = Bytes.from_bytes(b'hello'); print(bytes(b))"
```

- [ ] **Step 7: Commit**

```bash
git add pyproject.toml python/
git commit -m "feat(pyo3): add maturin package setup and Python re-exports"
```

---

## Task 7: Type stubs

**Files:**
- Create: `python/smol_bytes/__init__.pyi`, `python/smol_bytes/shared/__init__.pyi`, `python/smol_bytes/compact/__init__.pyi`

Write `.pyi` files with full method signatures for IDE autocompletion and mypy. Each type gets all its `#[pymethods]` listed with Python type annotations.

Key typing patterns:
- `def __getitem__(self, index: int) -> int: ...` and `@overload def __getitem__(self, index: slice) -> Self: ...`
- `def __reduce__(self) -> tuple[object, tuple[bytes]]: ...`
- `def __buffer__(self, flags: int) -> memoryview: ...`

- [ ] **Step 1: Write root `__init__.pyi`** with Buffer, BytesMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut
- [ ] **Step 2: Write `shared/__init__.pyi`** with Bytes
- [ ] **Step 3: Write `compact/__init__.pyi`** with Bytes
- [ ] **Step 4: Commit**

```bash
git add python/smol_bytes/*.pyi python/smol_bytes/*/*.pyi
git commit -m "feat(pyo3): add type stubs (.pyi) for IDE support"
```

---

## Task 8: Python tests

**Files:**
- Create: `tests/python/test_smol_bytes.py`

- [ ] **Step 1: Write pytest suite**

Cover:
1. **Import paths**: all 7 types from correct modules
2. **Construct + read**: each type, verify `bytes()`, `str()`, `len()`, `bool()`
3. **Pickle round-trip**: `pickle.loads(pickle.dumps(x)) == x` for each type
4. **Buffer protocol**: `memoryview(buf)` for byte types, `TypeError` for UTF-8 types
5. **Hashing**: `hash()` works on immutable types, `TypeError` on mutable types
6. **Iteration**: `list(Utf8Bytes.from_str("café"))` yields `['c', 'a', 'f', 'é']`
7. **Slicing**: `buf[2:5]`, `buf[-1]`, `buf[::2]`
8. **Split operations**: `split_to`, `split_off`, `slice`
9. **Rich comparison**: `==`, `<`, `>` between same types and with `bytes`/`str`
10. **`__bytes__` on UTF-8 types**: `bytes(Utf8Bytes.from_str("café"))` returns UTF-8 bytes

- [ ] **Step 2: Run tests**

Run: `pytest tests/python/test_smol_bytes.py -v`
Expected: all tests pass

- [ ] **Step 3: Commit**

```bash
git add tests/python/
git commit -m "test(pyo3): add Python integration test suite"
```

---

## Task 9: CI workflow

**Files:**
- Create: `.github/workflows/python.yml`

- [ ] **Step 1: Write workflow**

```yaml
name: Python

on:
  push:
    branches: [main, "*.*.x"]
  pull_request:

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        python-version: ["3.8", "3.12"]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: ${{ matrix.python-version }}
      - uses: dtolnay/rust-toolchain@stable
      - run: pip install maturin pytest
      - run: maturin develop --features pyo3 --manifest-path crates/smol-bytes/Cargo.toml
      - run: pytest tests/python/ -v

  wheels:
    if: startsWith(github.ref, 'refs/tags/')
    needs: test
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: PyO3/maturin-action@v1
        with:
          command: build
          args: --release --features pyo3 --manifest-path crates/smol-bytes/Cargo.toml
      - uses: actions/upload-artifact@v4
        with:
          name: wheels-${{ matrix.os }}
          path: target/wheels/

  publish:
    if: startsWith(github.ref, 'refs/tags/')
    needs: wheels
    runs-on: ubuntu-latest
    permissions:
      id-token: write
    steps:
      - uses: actions/download-artifact@v4
        with:
          pattern: wheels-*
          merge-multiple: true
          path: dist/
      - uses: pypa/gh-action-pypi-publish@release/v1
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/python.yml
git commit -m "ci: add Python wheel build and publish workflow"
```
