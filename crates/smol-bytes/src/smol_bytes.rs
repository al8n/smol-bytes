//! A compact, clone-efficient byte buffer with multiple optimization strategies.
//!
//! - [`strategy::shared::SmolBytes`] - Preserves heap allocations for fast [`bytes::Bytes`](bytes::Bytes) interop
//! - [`strategy::compact::SmolBytes`] - Aggressively inlines to minimize memory usage
//!
//! # Quick Start
//!
//! ```rust
//! // Use the Shared strategy (fast conversions, preserves heap)
//! use smol_bytes::strategy::shared::SmolBytes;
//!
//! let data = SmolBytes::from_static(b"hello world");
//! assert_eq!(data.as_slice(), b"hello world");
//! ```
//!
//! # Choosing a Strategy
//!
//! ## `Shared` Strategy (Recommended for most use cases)
//!
//! - **Fast**: Zero-copy conversions with `bytes::Bytes`
//! - **Shared allocations**: Cheap clones via reference counting
//! - **Trade-off**: May use more memory
//!
//! ```rust
//! use smol_bytes::strategy::shared::SmolBytes;
//! use bytes::Buf;
//!
//! let mut data = SmolBytes::from(vec![1u8; 100]);
//! data.advance(70); // Still heap-allocated for fast Bytes conversion
//! ```
//!
//! ## `Compact` Strategy (For memory-constrained applications)
//!
//! - **Memory-efficient**: Aggressively inlines when possible
//! - **Automatic optimization**: Converts heap→inline when data shrinks
//! - **Trade-off**: May copy data when converting to `Bytes`
//!
//! ```rust
//! use smol_bytes::strategy::compact::SmolBytes;
//! use bytes::Buf;
//!
//! let mut data = SmolBytes::from(vec![1u8; 100]);
//! data.advance(70); // Automatically converted to inline!
//! assert!(!data.is_heap_allocated());
//! ```

pub use raw::INLINE_CAP;

mod raw;

/// Strategy implementations and type aliases.
pub mod strategy;
