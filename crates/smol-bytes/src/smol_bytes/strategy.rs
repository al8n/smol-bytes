//! Optimization strategies for `SmolBytes`.
//!
//! This module provides two different optimization strategies that control how `SmolBytes`
//! manages the transition between inline (stack) storage and heap-allocated storage:
//!
//! - **[`shared`]** - Preserves heap allocations for fast conversions with `bytes::Bytes`
//! - **[`compact`]** - Aggressively inlines data to minimize memory usage
//!
//! # Quick Comparison
//!
//! | Feature | [`shared::SmolBytes`] | [`compact::SmolBytes`] |
//! |---------|----------------------|------------------------|
//! | **Heap→Inline** | Never converts | Converts when possible |
//! | **Bytes conversion** | Zero-copy (fast) | May copy data |
//! | **Memory usage** | Higher | Lower |
//! | **Best for** | Bytes interop, speed | Memory-constrained apps |
//! | **Default?** | ✅ Recommended | ⚠️ Special cases |
//!
//! # When to Use Which Strategy
//!
//! ## Use `shared::SmolBytes` (Default) when:
//!
//! - You frequently convert between `SmolBytes` and `bytes::Bytes`
//! - Performance is more important than memory overhead
//! - You want cheap clones via reference counting
//! - You're building network protocols or I/O-heavy applications
//!
//! ```rust
//! use smol_bytes::strategy::shared::SmolBytes;
//! use bytes::Buf;
//!
//! let mut data = SmolBytes::from(vec![1u8; 100]);
//! data.advance(70);
//! // Still heap-allocated for fast Bytes conversion
//! assert!(data.is_heap_allocated());
//! ```
//!
//! ## Use `compact::SmolBytes` when:
//!
//! - Memory footprint is critical (embedded systems, memory-constrained environments)
//! - You're working with many small buffers that shrink over time
//! - Conversions to `Bytes` are rare
//! - You want to minimize heap allocations
//!
//! ```rust
//! use smol_bytes::strategy::compact::SmolBytes;
//! use bytes::Buf;
//!
//! let mut data = SmolBytes::from(vec![1u8; 100]);
//! data.advance(70);
//! // Automatically converted to inline storage!
//! assert!(!data.is_heap_allocated());
//! ```
//!
//! # Examples
//!
//! ## Shared Strategy Example
//!
//! ```rust
//! use smol_bytes::strategy::shared::SmolBytes;
//!
//! // Small data is inline
//! let small = SmolBytes::from_static(b"hello");
//! assert!(!small.is_heap_allocated());
//!
//! // Large data is heap-allocated
//! let large = SmolBytes::from(vec![1u8; 100]);
//! assert!(large.is_heap_allocated());
//!
//! // Zero-copy conversion to Bytes
//! let bytes: bytes::Bytes = large.into();
//! ```
//!
//! ## Compact Strategy Example
//!
//! ```rust
//! use smol_bytes::strategy::compact::SmolBytes;
//! use bytes::Buf;
//!
//! // Start with large heap allocation
//! let mut data = SmolBytes::from(vec![1u8; 50]);
//! assert!(data.is_heap_allocated());
//!
//! // Operations automatically convert to inline when possible
//! data.truncate(30); // Now fits inline!
//! assert!(!data.is_heap_allocated());
//! ```
//!
//! # Performance Characteristics
//!
//! ## Shared Strategy
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
//! ## Compact Strategy
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
//! Both strategies store data in the same [`RawSmolBytes<S>`](crate::smol_bytes::raw::RawSmolBytes)
//! type, with only the behavior differing based on the strategy parameter.

pub(crate) use sealed::Strategy;

/// The **Compact** strategy: Aggressively inlines data to minimize memory usage.
///
/// This strategy automatically converts heap-allocated buffers to inline storage
/// whenever the data shrinks to fit within the inline capacity (62 bytes or less).
///
/// See [`compact::SmolBytes`] for usage examples and detailed documentation.
pub mod compact;

/// The **Shared** strategy: Preserves heap allocations for fast `Bytes` conversions.
///
/// This strategy keeps heap-allocated buffers on the heap even when they could fit
/// inline, enabling zero-copy conversions with `bytes::Bytes`.
///
/// See [`shared::SmolBytes`] for usage examples and detailed documentation.
pub mod shared;

mod sealed {
  use core::ops::RangeBounds;

  pub trait Strategy {
    fn slice(&self, range: impl RangeBounds<usize>) -> Self;

    fn split_to(&mut self, to: usize) -> Self;

    fn split_off(&mut self, at: usize) -> Self;

    fn truncate(&mut self, len: usize);

    fn advance(&mut self, cnt: usize);

    fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes;

    fn clear(&mut self);
  }
}
