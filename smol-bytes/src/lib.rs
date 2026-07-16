#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(unsafe_op_in_unsafe_fn)]
#![doc = include_str!("../README.md")]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate std;

#[cfg(any(feature = "std", feature = "alloc"))]
pub use ::bytes::{Buf, BufMut};

#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub use ::bytes::buf;

#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub use bytes::strategy::{
  compact,
  shared::{self, Bytes},
};

#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub(crate) use bytes::strategy;

#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub use bytes_mut::BytesMut;

#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub use bytes::strategy::shared::Utf8Bytes;

#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub use utf8_bytes_mut::Utf8BytesMut;

pub use buffer::{Buffer, INLINE_CAP};
pub use error::*;
pub use utf8_buf::{Utf8Buf, Utf8BufMut};
pub use utf8_buffer::Utf8Buffer;

#[cfg(any(feature = "std", feature = "alloc"))]
mod bytes;
#[cfg(any(feature = "std", feature = "alloc"))]
mod bytes_mut;

#[cfg(feature = "pyo3")]
#[cfg_attr(docsrs, doc(cfg(feature = "pyo3")))]
mod python;

#[cfg(feature = "wasm")]
mod wasm;
#[cfg(feature = "wasm")]
mod wasm_iter;

#[cfg(feature = "pyo3")]
#[cfg_attr(docsrs, doc(cfg(feature = "pyo3")))]
pub use python::{register_classes, register_compact, register_shared};

mod buffer;

mod utf8_buf;
mod utf8_buffer;

#[cfg(any(feature = "std", feature = "alloc"))]
mod utf8_bytes;
#[cfg(any(feature = "std", feature = "alloc"))]
mod utf8_bytes_mut;

#[cfg(any(feature = "std", feature = "alloc"))]
mod macros;

/// Error types for byte buffer operations.
pub mod error;
