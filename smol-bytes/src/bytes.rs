//! High-performance byte buffers with multiple optimization strategies.
//!
//! - [`shared::Bytes`] - Preserves heap allocations for fast [`bytes::Bytes`](bytes::Bytes) interop
//! - [`compact::Bytes`] - Aggressively inlines to minimize memory usage
//!
//! Both strategies store ≤62 bytes inline with extremely fast cloning (simple `memcpy`).
//!
//! # Quick Start
//!
//! ```rust
//! use smol_bytes::shared::Bytes;
//!
//! // Inline storage - no allocation, fast cloning
//! let token = Bytes::from_static(b"identifier");
//! assert!(token.is_inline());
//! let cloned = token.clone(); // Just a memcpy!
//! ```
//!
//! # Choosing a ImmutableStorage
//!
//! ## `Shared` ImmutableStorage (Recommended)
//!
//! **Best for**: Lexers, parsers, protocol handlers
//!
//! - **Fast cloning**: Inline data = `memcpy`, heap data = refcount increment
//! - **Zero-copy**: Seamless conversions with `bytes::Bytes`
//! - **Stable layout**: Heap allocations stay on heap
//!
//! **Use case**: Language lexer/parser
//! ```rust
//! use smol_bytes::shared::Bytes;
//!
//! // Most tokens fit inline - extremely fast to clone
//! let keyword = Bytes::from_static(b"fn");
//! let identifier = Bytes::copy_from_slice(b"my_function");
//!
//! // Cloning tokens during AST construction is nearly free
//! let token_copy = keyword.clone(); // Inline: just memcpy
//! ```
//!
//! ## `Compact` ImmutableStorage (For memory efficiency)
//!
//! **Best for**: Memory-constrained environments, embedded systems
//!
//! - **Aggressive inlining**: Converts heap→inline when data shrinks
//! - **Memory-efficient**: Minimizes heap usage
//! - **Trade-off**: May copy data during conversions
//!
//! ```rust
//! use smol_bytes::{compact::Bytes, Buf};
//!
//! let mut data = Bytes::from(vec![1u8; 100]);
//! data.advance(70); // Automatically converted to inline!
//! assert!(data.is_inline());
//! ```
mod raw;

pub(crate) use raw::{RawBytes, Repr};

/// ImmutableStorage implementations and type aliases.
pub mod strategy;
