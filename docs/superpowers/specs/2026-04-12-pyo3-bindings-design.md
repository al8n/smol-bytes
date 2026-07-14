# pyo3 Python Bindings — Production-Ready Design

**Date**: 2026-04-12
**Status**: Approved
**Scope**: Polish existing pyo3 bindings, add missing features, create publishable Python package

---

## 1. Module Structure

The Python package mirrors the Rust module hierarchy with submodules for the two Bytes strategies.

### Python package layout

```
python/
└── smol_bytes/
    ├── __init__.py          # re-exports Buffer, BytesMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut
    ├── __init__.pyi         # type stubs for root types
    ├── shared/
    │   ├── __init__.py      # re-exports Bytes
    │   └── __init__.pyi     # type stubs for shared.Bytes
    └── compact/
        ├── __init__.py      # re-exports Bytes
        └── __init__.pyi     # type stubs for compact.Bytes
```

### Native module

The compiled Rust extension is `smol_bytes._smol_bytes` (underscore-prefixed, not imported directly). Pure-Python `__init__.py` files re-export from it:

```python
# smol_bytes/__init__.py
from ._smol_bytes import Buffer, BytesMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut

# smol_bytes/shared/__init__.py
from .._smol_bytes.shared import Bytes

# smol_bytes/compact/__init__.py
from .._smol_bytes.compact import Bytes
```

### Rust-side `#[pymodule]`

In `crates/smol-bytes/src/python.rs`, register the root module with two submodules:

```rust
#[pymodule]
fn _smol_bytes(m: &Bound<'_, PyModule>) -> PyResult<()> {
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

### Type mapping

| Python path | Rust type | pyclass wrapper |
|---|---|---|
| `smol_bytes.Buffer` | `Buffer` | direct (existing) |
| `smol_bytes.BytesMut` | `BytesMut` | direct (existing) |
| `smol_bytes.Utf8Buffer` | `Utf8Buffer` | direct (existing) |
| `smol_bytes.Utf8Bytes` | `Utf8Bytes` | direct (existing) |
| `smol_bytes.Utf8BytesMut` | `Utf8BytesMut` | direct (existing) |
| `smol_bytes.shared.Bytes` | `RawBytes<Shared>` | **new** `PySharedBytes` (like `PyCompactBytes`) |
| `smol_bytes.compact.Bytes` | `RawBytes<Compact>` | `PyCompactBytes` (existing) |

---

## 2. New Type: PySharedBytes

Create `crates/smol-bytes/src/bytes/strategy/shared/python.rs` modeled on the existing `compact/python.rs`. The wrapper is:

```rust
#[pyclass(name = "Bytes", skip_from_py_object)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PySharedBytes {
    inner: super::Bytes,
}
```

Exposes the same surface as `PyCompactBytes`: constructors (`__new__`, `from_bytes`, `from_str`), all dunder methods, Buf getters, split/slice/advance/truncate, storage checks (`is_inline`, `is_heap`). Code can be shared via the `PyBufExt` / `PyBufCmp` traits already in `python.rs`.

---

## 3. Gap-Filling Per Type

### 3a. Pickle support — all types

Every type gets `__reduce__`, returning a `(constructor, args)` tuple:

- **Byte types** (Buffer, BytesMut, shared.Bytes, compact.Bytes):
  `__reduce__` returns `(type(self).from_bytes, (bytes(self),))`
- **UTF-8 types** (Utf8Buffer, Utf8Bytes, Utf8BytesMut):
  `__reduce__` returns `(type(self).from_str, (str(self),))`

This enables `pickle.dumps()` / `pickle.loads()`, multiprocessing, and frameworks like Celery/Ray.

### 3b. Buffer protocol — byte types only

`Buffer`, `BytesMut`, `shared.Bytes`, `compact.Bytes` implement pyo3's `__buffer__` protocol, exposing a **read-only** contiguous `memoryview` of the underlying bytes.

UTF-8 types do NOT implement buffer protocol (matching Python `str` semantics — use `bytes(utf8_val)` or `str(utf8_val).encode()` instead).

### 3c. `__hash__` — immutable types only

Follow Python convention: only immutable types are hashable.

| Type | Mutable? | `__hash__` |
|---|---|---|
| Buffer | yes | skip |
| BytesMut | yes | skip |
| shared.Bytes | no | **add** |
| compact.Bytes | no | has already |
| Utf8Buffer | yes | skip |
| Utf8Bytes | no | **add** |
| Utf8BytesMut | yes | skip |

### 3d. `__iter__` for UTF-8 types

`Utf8Buffer`, `Utf8Bytes`, `Utf8BytesMut` gain `__iter__` that yields single-character Python `str` values, matching `str.__iter__` semantics.

### 3e. `__bytes__` for UTF-8 types

`Utf8Buffer`, `Utf8Bytes`, `Utf8BytesMut` gain `__bytes__` returning the raw UTF-8 encoded bytes as Python `bytes`, matching `str.encode('utf-8')` semantics.

### 3f. Complete gap matrix

| Method | Buffer | BytesMut | shared.Bytes | compact.Bytes | Utf8Buffer | Utf8Bytes | Utf8BytesMut |
|---|---|---|---|---|---|---|---|
| `__hash__` | skip | skip | **add** | has | skip | **add** | skip |
| `__buffer__` | **add** | **add** | **add** | **add** | skip | skip | skip |
| `__reduce__` | **add** | **add** | **add** | **add** | **add** | **add** | **add** |
| `__iter__` | has | has | **add** | has | **add** | **add** | **add** |
| `__bytes__` | has | has | **add** | has | **add** | **add** | **add** |
| `__copy__` | skip | has | **add** | has | has | has | has |
| `__deepcopy__` | skip | has | **add** | has | has | has | has |

---

## 4. Package Setup

### pyproject.toml

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
]

[tool.maturin]
features = ["pyo3/extension-module"]
module-name = "smol_bytes._smol_bytes"
python-source = "python"
```

Placed at repository root (not inside `crates/smol-bytes/`).

### maturin config

The `module-name` setting tells maturin to compile the native extension as `smol_bytes._smol_bytes`. The `python-source = "python"` tells it to include the `python/` directory as the pure-Python portion of the package.

### Type stubs (`.pyi` files)

Full type stubs for each module with:
- All method signatures with correct argument types and return types
- `@overload` for `__getitem__` (int → single element, slice → new buffer)
- Docstrings matching the Rust rustdoc

### Test file

`tests/python/test_smol_bytes.py` — a pytest suite covering:
- Import paths (`from smol_bytes import Buffer`, `from smol_bytes.shared import Bytes`, etc.)
- Construct → mutate → read round-trips for each type
- Pickle round-trips for each type
- Buffer protocol (`memoryview`) for byte types
- `__hash__` on immutable types, `TypeError` on mutable types
- `__iter__` yields expected values
- Cross-type operations (construct shared.Bytes from Buffer, etc.)

### CI workflow

`.github/workflows/python.yml`:
1. Build wheels with `maturin build` for Linux (manylinux), macOS (x86_64 + arm64), Windows
2. Install wheel and run `pytest tests/python/`
3. On git tag: publish to PyPI via `maturin publish`

---

## 5. What's NOT in scope

- wasm-bindgen bindings (separate future spec)
- Python async/await integration
- numpy dtype registration
- Custom Python exceptions (use standard `TypeError`, `ValueError`, `IndexError`)
- Python-side benchmarks (Rust benchmarks are sufficient)

---

## 6. Deliverables

1. `PySharedBytes` wrapper type in `src/bytes/strategy/shared/python.rs`
2. `#[pymodule]` function in `src/python.rs` with `shared` + `compact` submodules
3. Gap-filling methods on all 7 existing types (pickle, buffer protocol, hash, iter, bytes)
4. `pyproject.toml` at repo root
5. `python/smol_bytes/` package with `__init__.py` re-exports and `.pyi` type stubs
6. `tests/python/test_smol_bytes.py` pytest suite
7. `.github/workflows/python.yml` CI workflow
