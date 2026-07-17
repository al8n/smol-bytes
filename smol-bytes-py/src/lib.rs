//! Python extension module for smol-bytes (thin cdylib; all pyclasses live in
//! `smol-bytes`).
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::types::PyModule;

#[cfg(feature = "pyo3")]
#[pymodule]
fn _smol_bytes(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
  smol_bytes::register_classes(m, "smol_bytes")?;

  let shared = PyModule::new(m.py(), "shared")?;
  smol_bytes::register_shared(&shared, "smol_bytes.shared")?;
  m.add_submodule(&shared)?;
  py.import("sys")?
    .getattr("modules")?
    .set_item("smol_bytes._smol_bytes.shared", &shared)?;

  let compact = PyModule::new(m.py(), "compact")?;
  smol_bytes::register_compact(&compact, "smol_bytes.compact")?;
  m.add_submodule(&compact)?;
  py.import("sys")?
    .getattr("modules")?
    .set_item("smol_bytes._smol_bytes.compact", &compact)?;

  Ok(())
}
