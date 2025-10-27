#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

//! A compact, clone-efficient byte buffer similar to `SmolStr` but for raw bytes.
//!
//! `SmolBytes` stores up to 39 bytes inline on the stack and falls back to a
//! reference-counted [`bytes::Bytes`] allocation for longer sequences. Cloning an
//! inline value is a simple copy, while heap-backed values share the allocation.

// #[cfg(not(any(feature = "std", feature = "alloc")))]
// compile_error!("smol-bytes requires either the \"std\" or \"alloc\" feature.");

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate std;

#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub use smol_bytes::{ConversionFriendly, Inline, SmolBytes, INLINE_CAP};

// #[cfg(any(feature = "std", feature = "alloc"))]
// #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
// pub use smol_bytes_mut::SmolBytesMut;

#[cfg(any(feature = "std", feature = "alloc"))]
mod smol_bytes;

#[cfg(any(feature = "std", feature = "alloc"))]
mod smol_bytes_mut;
