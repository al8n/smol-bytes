//! WebAssembly bindings entry point for smol-bytes (thin cdylib;
//! `#[wasm_bindgen]` items live in `smol-bytes`).
#![allow(unused_imports)]
#[cfg(feature = "wasm")]
pub use smol_bytes::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn __smol_bytes_start() {
  use core::hint::black_box;
  black_box(smol_bytes::Buffer::new());
  black_box(smol_bytes::BytesMut::new());
  black_box(smol_bytes::Utf8Buffer::new());
  black_box(smol_bytes::Utf8BytesMut::new());
}
