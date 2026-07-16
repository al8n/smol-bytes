#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(unsafe_op_in_unsafe_fn)]

//! High-performance, clone-efficient byte buffers optimized for small data.
//!
//! `smol-bytes` provides byte buffers that store up to 62 bytes inline on the stack,
//! with automatic fallback to reference-counted heap allocation for larger data.
//! Cloning inline buffers is a simple `memcpy`, making it extremely fast.
//!
//! # Design Purpose
//!
//! This crate is optimized for scenarios where:
//! - Most data is small (≤62 bytes)
//! - Frequent cloning is required (e.g., AST construction)
//! - Minimal allocations are critical (e.g., FFI boundaries)
//!
//! ## Perfect for Lexers & Parsers
//!
//! Most programming language tokens fit inline:
//! - Keywords: `let`, `fn`, `const`, `return`
//! - Identifiers: Most variable/function names < 62 chars
//! - Literals: Numbers, operators, punctuation
//!
//! **Result**: Near-zero allocation AST building with cheap token cloning for
//! concurrent compilation.
//!
//! # Strategies
//!
//! Two optimization strategies are available:
//!
//! - **[`shared::Bytes`]** - Fast conversions with `bytes::Bytes`, preserves heap allocations
//! - **[`compact::Bytes`]** - Minimizes memory usage, aggressively inlines data
//!
//! # Quick Start
//!
//! ```rust
//! use smol_bytes::shared::Bytes;
//!
//! // Small data stored inline - no allocation!
//! let token = Bytes::from_static(b"identifier");
//! assert!(token.is_inline());
//!
//! // Cloning is extremely fast (just a memcpy)
//! let cloned = token.clone();
//! assert_eq!(token, cloned);
//! ```
//!
//! # Lexer/Parser Example
//!
//! ```rust
//! use smol_bytes::shared::Bytes;
//!
//! #[derive(Clone)]
//! enum Token {
//!     Identifier(Bytes),
//!     Keyword(Bytes),
//! }
//!
//! // Tokens are cloned frequently during AST construction
//! // With smol-bytes, this is extremely efficient
//! let id = Bytes::copy_from_slice(b"variable_name");
//! let token1 = Token::Identifier(id.clone()); // Fast inline clone
//! let token2 = Token::Identifier(id.clone()); // Fast inline clone
//! ```
//!
//! # UTF-8 String Wrappers
//!
//! Three string-flavored wrappers guarantee their contents are valid
//! UTF-8 and expose a `String`-like interface with char-boundary checked
//! splits and slices. Pick based on the mutability and storage model you
//! need:
//!
//! | Wrapper | Backing | Mutable? | Heap? | Use when |
//! |---|---|---|---|---|
//! | [`Utf8Buffer`] | [`Buffer`] | yes (push) | **no** — 62-byte cap | Tokens / small identifiers; push panics/errors on overflow |
//! | [`Utf8Bytes`] | [`shared::Bytes`] | no | yes | Shared, cloneable string slices; cheap refcount clones |
//! | [`Utf8BytesMut`] | [`BytesMut`] | yes | yes | Building up strings dynamically, splitting, unsplitting |
//!
//! All three implement [`Deref<Target = str>`](core::ops::Deref), so
//! standard `&str` methods are available directly. They also implement
//! the [`Utf8Buf`] / [`Utf8BufMut`] traits, which provide
//! char-boundary-checked split and slice operations:
//!
//! ```rust
//! use smol_bytes::{Utf8Buf, Utf8Bytes};
//!
//! let mut s = Utf8Bytes::from("café €uro");
//! let head = s.split_to(5);          // "café" (2-byte 'é' is on a boundary)
//! assert_eq!(head.as_str(), "café");
//! assert_eq!(s.as_str(), " €uro");
//! ```
//!
//! Splitting in the middle of a multi-byte character panics (or returns
//! `Utf8Error::InvalidCharBoundary` for the `try_*` variants).
//!
//! # Bindings and no-std
//!
//! Source-built Python bindings are available through the `pyo3` feature and
//! WebAssembly bindings through `wasm`. The fixed buffer types remain usable
//! in `no_std`; `alloc` enables the heap-backed types.
//!
//! See the [`shared`] and [`compact`] modules for detailed comparison and usage
//! examples.

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

#[cfg(all(feature = "pyo3", feature = "std"))]
pub use std::hash::DefaultHasher;
#[cfg(all(feature = "pyo3", not(feature = "std")))]
pub type DefaultHasher = ::core::hash::BuildHasherDefault<::core::hash::SipHasher>;
