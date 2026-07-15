//! Optimization strategies for `Bytes`.
//!
//! This module provides two different optimization strategies that control how `Bytes`
//! manages the transition between inline (stack) storage and heap-allocated storage:
//!
//! - **[`shared`]** - Preserves heap allocations for fast conversions with `bytes::Bytes`
//! - **[`compact`]** - Aggressively inlines data to minimize memory usage
//!
//! # Quick Comparison
//!
//! | Feature | [`shared::Bytes`] | [`compact::Bytes`] |
//! |---------|----------------------|------------------------|
//! | **Heap→Inline** | Never converts | Converts when possible |
//! | **Bytes conversion** | Zero-copy (fast) | May copy data |
//! | **Memory usage** | Higher | Lower |
//! | **Best for** | Bytes interop, speed | Memory-constrained apps |
//! | **Default?** | ✅ Recommended | ⚠️ Special cases |
//!
//! # When to Use Which ImmutableStorage
//!
//! ## Use `shared::Bytes` (Default) when:
//!
//! - You frequently convert between `Bytes` and `bytes::Bytes`
//! - Performance is more important than memory overhead
//! - You want cheap clones via reference counting
//! - You're building network protocols or I/O-heavy applications
//!
//! ```rust
//! use smol_bytes::{shared::Bytes, Buf};
//!
//! let mut data = Bytes::from(vec![1u8; 100]);
//! data.advance(70);
//! // Still heap-allocated for fast Bytes conversion
//! assert!(data.is_heap());
//! ```
//!
//! ## Use `compact::Bytes` when:
//!
//! - Memory footprint is critical (embedded systems, memory-constrained environments)
//! - You're working with many small buffers that shrink over time
//! - Conversions to `Bytes` are rare
//! - You want to minimize heap allocations
//!
//! ```rust
//! use smol_bytes::{compact::Bytes, Buf};
//!
//! let mut data = Bytes::from(vec![1u8; 100]);
//! data.advance(70);
//! // Automatically converted to inline storage!
//! assert!(!data.is_heap());
//! ```
//!
//! # Examples
//!
//! ## Shared ImmutableStorage Example
//!
//! ```rust
//! use smol_bytes::shared::Bytes;
//!
//! // Small data is inline
//! let small = Bytes::from_static(b"hello");
//! assert!(!small.is_heap());
//!
//! // Large data is heap-allocated
//! let large = Bytes::from(vec![1u8; 100]);
//! assert!(large.is_heap());
//!
//! // Zero-copy conversion to Bytes
//! let bytes: bytes::Bytes = large.into();
//! ```
//!
//! ## Compact ImmutableStorage Example
//!
//! ```rust
//! use smol_bytes::{compact::Bytes, Buf};
//!
//! // Start with large heap allocation
//! let mut data = Bytes::from(vec![1u8; 64]);
//! assert!(data.is_heap());
//!
//! // Operations automatically convert to inline when possible
//! data.truncate(30); // Now fits inline!
//! assert!(!data.is_heap());
//! ```
//!
//! # Performance Characteristics
//!
//! ## Shared ImmutableStorage
//!
//! | Operation | Complexity | Allocation Behavior |
//! |-----------|------------|---------------------|
//! | `advance()` | O(1) | Preserves heap |
//! | `truncate()` | O(1) | Preserves heap |
//! | `split_to()` | O(1) or O(62)* | Preserves heap |
//! | `split_off()` | O(1) or O(62)* | Preserves heap |
//! | `into::<Bytes>()` | O(1) | Zero-copy if heap |
//!
//! *O(62) when result needs to be inline (copies up to 62 bytes)
//!
//! ## Compact ImmutableStorage
//!
//! | Operation | Complexity | Allocation Behavior |
//! |-----------|------------|---------------------|
//! | `advance()` | O(1) or O(62)* | May convert heap→inline |
//! | `truncate()` | O(1) or O(62)* | May convert heap→inline |
//! | `split_to()` | O(1) or O(62)* | May convert heap→inline |
//! | `split_off()` | O(1) or O(62)* | May convert heap→inline |
//! | `into::<Bytes>()` | O(1) or O(62) | May need to copy |
//!
//! *O(62) when conversion happens (copies up to 62 bytes)
//!
//! # Implementation Details
//!
//! The strategy pattern is implemented using a sealed trait to prevent external implementations.
//! The strategy is selected at compile-time via a zero-sized type parameter, resulting in
//! **zero runtime overhead**.
//!
//! Both strategies store data in the same [`RawBytes<S>`](crate::smol_bytes::raw::RawBytes)
//! type, with only the behavior differing based on the strategy parameter.

pub(crate) use sealed::ImmutableStorage;

/// The **Compact** strategy: Aggressively inlines data to minimize memory usage.
///
/// This strategy automatically converts heap-allocated buffers to inline storage
/// whenever the data shrinks to fit within the inline capacity (62 bytes or less).
///
/// See [`compact::Bytes`] for usage examples and detailed documentation.
pub mod compact;

/// The **Shared** strategy: Preserves heap allocations for fast `Bytes` conversions.
///
/// This strategy keeps heap-allocated buffers on the heap even when they could fit
/// inline, enabling zero-copy conversions with `bytes::Bytes`.
///
/// See [`shared::Bytes`] for usage examples and detailed documentation.
pub mod shared;

mod sealed {
  use crate::error::*;
  use core::ops::RangeBounds;

  pub trait ImmutableStorage {
    fn slice(&self, range: impl RangeBounds<usize>) -> Self;

    fn try_slice(&self, range: impl RangeBounds<usize>) -> Result<Self, RangeOutOfBounds>
    where
      Self: Sized;

    fn split_to(&mut self, to: usize) -> Self;

    fn split_off(&mut self, at: usize) -> Self;

    fn truncate(&mut self, len: usize);

    fn advance(&mut self, cnt: usize);

    fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes;

    fn clear(&mut self);
  }
}
