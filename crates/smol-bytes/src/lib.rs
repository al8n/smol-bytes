#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

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
//! # FFI-Friendly
//!
//! Minimal allocations make this crate ideal for FFI boundaries:
//! - Python bindings via `pyo3` (coming soon)
//! - WebAssembly support via `wasm-bindgen` (coming soon)
//! - `no_std` compatible with optional `alloc`
//!
//! See the [`strategy`] module for detailed comparison and usage examples.

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
pub use utf8_bytes::Utf8Bytes;

#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub use utf8_bytes_mut::Utf8BytesMut;

pub use buffer::{Buffer, INLINE_CAP};
pub use utf8_buffer::Utf8Buffer;
pub use error::*;
pub use utf8_buf::{Utf8Buf, Utf8BufMut};

#[cfg(any(feature = "std", feature = "alloc"))]
mod bytes;
#[cfg(any(feature = "std", feature = "alloc"))]
mod bytes_mut;

#[cfg(feature = "pyo3")]
#[cfg_attr(docsrs, doc(cfg(feature = "pyo3")))]
mod python;

mod buffer;

mod utf8_buf;
mod utf8_buffer;

#[cfg(any(feature = "std", feature = "alloc"))]
mod utf8_bytes;
#[cfg(any(feature = "std", feature = "alloc"))]
mod utf8_bytes_mut;

mod macros;

/// Error types for byte buffer operations.
pub mod error;

#[cfg(all(feature = "pyo3", feature = "std"))]
pub use std::hash::DefaultHasher;
#[cfg(all(feature = "pyo3", not(feature = "std")))]
pub type DefaultHasher = ::core::hash::BuildHasherDefault<::core::hash::SipHasher>;
