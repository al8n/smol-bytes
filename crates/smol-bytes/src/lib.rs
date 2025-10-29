#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

//! A compact, clone-efficient byte buffer with inline storage optimization.
//!
//! `smol-bytes` provides space-efficient byte buffers that store up to 62 bytes inline
//! on the stack and fall back to reference-counted heap allocation for longer sequences.
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
//! // Use the Shared strategy (recommended for most use cases)
//! use smol_bytes::shared::Bytes;
//!
//! let data = Bytes::from_static(b"hello world");
//! assert_eq!(data.as_slice(), b"hello world");
//! ```
//!
//! See the [`strategy`] module for detailed comparison and usage examples.

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate std;

#[cfg(any(feature = "std", feature = "alloc"))]
pub use ::bytes::{Buf, BufMut};

pub use buffer::{Buffer, TryGetError, TryPutError, INLINE_CAP};

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
mod bytes;

#[cfg(any(feature = "std", feature = "alloc"))]
mod bytes_mut;

mod buffer;
