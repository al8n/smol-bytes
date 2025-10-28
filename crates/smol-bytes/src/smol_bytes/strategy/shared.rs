//! The **Shared** strategy for `SmolBytes`.
//!
//! This module provides the [`SmolBytes`] type alias configured with the [`Shared`] strategy,
//! which prioritizes **fast conversions** and **allocation sharing** with [`bytes::Bytes`].
//!
//! # Key Characteristics
//!
//! - **Zero-copy conversions**: Converting to/from `Bytes` is O(1) for heap-allocated data
//! - **Preserves heap allocations**: Once heap-allocated, stays on heap even when data shrinks
//! - **Reference-counted sharing**: Heap allocations use `Arc` for cheap clones
//! - **Recommended default**: Best for most use cases, especially I/O and networking
//!
//! # When to Use
//!
//! Choose this strategy when:
//!
//! - **Frequent `Bytes` conversions**: You often convert between `SmolBytes` and `bytes::Bytes`
//! - **Network protocols**: Building HTTP servers, WebSocket handlers, or other I/O-heavy applications
//! - **Performance-critical paths**: Speed is more important than memory overhead
//! - **Shared buffers**: You frequently clone buffers and want cheap reference counting
//!
//! # Basic Usage
//!
//! ```rust
//! use smol_bytes::strategy::shared::SmolBytes;
//!
//! // Small data (≤62 bytes) is stored inline
//! let small = SmolBytes::from_static(b"hello world");
//! assert!(!small.is_heap_allocated());
//!
//! // Large data is heap-allocated
//! let large = SmolBytes::from(vec![1u8; 100]);
//! assert!(large.is_heap_allocated());
//!
//! // Cheap clone (reference counting)
//! let clone = large.clone();
//! ```
//!
//! # Behavior Details
//!
//! ## Memory Layout
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │  SmolBytes (64 bytes on stack)          │
//! ├─────────────────────────────────────────┤
//! │  Variant: Inline (≤62 bytes)            │
//! │  ┌────────────────────────────────────┐ │
//! │  │ [u8; 62] data                      │ │
//! │  │ u8 length                          │ │
//! │  │ u8 current_offset                  │ │
//! │  └────────────────────────────────────┘ │
//! │                                           │
//! │  Variant: Heap (>62 bytes or shrunk)    │
//! │  ┌────────────────────────────────────┐ │
//! │  │ bytes::Bytes (Arc<[u8]>)           │ │
//! │  └────────────────────────────────────┘ │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Operations and Allocation Behavior
//!
//! ```rust
//! use smol_bytes::strategy::shared::SmolBytes;
//! use bytes::Buf;
//!
//! // Start with large heap allocation
//! let mut data = SmolBytes::from(vec![1u8; 100]);
//! assert!(data.is_heap_allocated());
//!
//! // After advance, still heap-allocated (Shared strategy)
//! data.advance(70); // 30 bytes remain
//! assert!(data.is_heap_allocated()); // ✓ Still on heap!
//!
//! // Zero-copy conversion to Bytes
//! let bytes: bytes::Bytes = data.into();
//! assert_eq!(bytes.len(), 30);
//! ```
//!
//! ## Comparison: Operations That Keep vs Convert to Heap
//!
//! | Operation | Starting State | Result State | Notes |
//! |-----------|---------------|--------------|-------|
//! | `advance()` | Heap (100 bytes) | Heap (30 bytes) | Stays heap |
//! | `truncate()` | Heap (100 bytes) | Heap (30 bytes) | Stays heap |
//! | `split_to()` | Heap (100 bytes) | Heap (70 bytes) | Both parts may be heap |
//! | `split_off()` | Heap (100 bytes) | Heap (30 bytes) | Both parts may be heap |
//! | `slice()` | Heap | Inline or Heap | Result inline if ≤62 bytes |
//!
//! # Performance Characteristics
//!
//! ## Fast Operations (O(1))
//!
//! - `clone()` - Reference count increment
//! - `advance()` - Pointer adjustment
//! - `truncate()` - Length update
//! - `into::<Bytes>()` - Zero-copy when heap-allocated
//!
//! ## Linear Operations (O(62) - copies up to 62 bytes)
//!
//! - Creating inline from slice
//! - `slice()` when result fits inline
//! - Operations producing inline results
//!
//! # Examples
//!
//! ## Network Protocol Buffer
//!
//! ```rust
//! use smol_bytes::strategy::shared::SmolBytes;
//! use bytes::Buf;
//!
//! // Receive data from network
//! let mut buffer = SmolBytes::from(vec![0u8; 1024]);
//!
//! // Process header (advance past it)
//! buffer.advance(16);
//!
//! // Buffer stays on heap for efficient passing to bytes::Bytes
//! assert!(buffer.is_heap_allocated());
//!
//! // Zero-copy conversion for writing
//! let bytes: bytes::Bytes = buffer.into();
//! // ... write bytes to socket
//! ```
//!
//! ## Parsing with Zero-Copy Slicing
//!
//! ```rust
//! use smol_bytes::strategy::shared::SmolBytes;
//!
//! let data = SmolBytes::from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
//!
//! // Extract different segments
//! let header = data.slice(0..2);
//! let payload = data.slice(2..8);
//! let checksum = data.slice(8..10);
//!
//! // All share the same underlying allocation!
//! ```
//!
//! ## Efficient Cloning
//!
//! ```rust
//! use smol_bytes::strategy::shared::SmolBytes;
//!
//! let original = SmolBytes::from(vec![1u8; 100]);
//!
//! // Cheap clones (just Arc reference count)
//! let clone1 = original.clone();
//! let clone2 = original.clone();
//! let clone3 = original.clone();
//!
//! // All share the same heap allocation
//! assert!(original.is_heap_allocated());
//! assert!(clone1.is_heap_allocated());
//! ```

use super::Strategy;
use crate::smol_bytes::raw::{InlineSize, RawSmolBytes, Repr, INLINE_CAP};
use bytes::Buf;
use core::mem;
use core::ops::{Bound, RangeBounds};

/// A strategy that preserves heap allocations for fast, zero-copy conversions with [`bytes::Bytes`].
///
/// # Overview
///
/// The `Shared` strategy prioritizes **fast conversions** and **allocation sharing** with
/// [`bytes::Bytes`]. When data is heap-allocated, it remains on the heap even after operations
/// like `advance()`, `truncate()`, or `split_to/off()` that reduce the size below the inline
/// capacity.
///
/// This makes `Shared` ideal when:
/// - You frequently convert between `SmolBytes` and `Bytes`
/// - You want to share heap allocations (cheap clones via reference counting)
/// - Performance is more important than memory overhead
///
/// # Behavior
///
/// - **Inline → Inline**: Small data stays inline (≤62 bytes)
/// - **Heap → Heap**: Large data stays on heap, **even if it shrinks** below inline capacity
/// - **Conversions**: Zero-cost `From`/`Into` with `Bytes` when heap-allocated
///
/// # Example
///
/// ```rust
/// use smol_bytes::strategy::shared::SmolBytes;
/// use bytes::Buf;
///
/// // Create heap-allocated bytes (>62 bytes)
/// let mut data = SmolBytes::from(vec![1u8; 100]);
/// assert!(data.is_heap_allocated());
///
/// // Advance past most data
/// data.advance(70); // Only 30 bytes remain
///
/// // Still heap-allocated! (Shared strategy preserves heap)
/// assert!(data.is_heap_allocated());
///
/// // Fast, zero-copy conversion to Bytes
/// let bytes: bytes::Bytes = data.into();
/// assert_eq!(bytes.len(), 30);
/// ```
///
/// # Comparison with Compact
///
/// | Operation | Shared | Compact |
/// |-----------|--------|---------|
/// | Heap→Inline on shrink | ❌ No | ✅ Yes |
/// | Bytes conversion | ⚡ Zero-copy | 📋 May copy |
/// | Memory usage | 💾 Higher | 💾 Lower |
/// | Best for | Speed, Bytes interop | Memory efficiency |
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Shared(());

impl Strategy for RawSmolBytes<Shared> {
  fn slice(&self, range: impl RangeBounds<usize>) -> Self {
    let len = self.len();

    let begin = match range.start_bound() {
      Bound::Included(&n) => n,
      Bound::Excluded(&n) => n.checked_add(1).expect("out of range"),
      Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
      Bound::Included(&n) => n.checked_add(1).expect("out of range"),
      Bound::Excluded(&n) => n,
      Bound::Unbounded => len,
    };

    let Some(slen) = end.checked_sub(begin) else {
      panic!(
        "range start must not be greater than end: {:?} <= {:?}",
        begin, end,
      );
    };

    assert!(
      end <= len,
      "range end out of bounds: {:?} <= {:?}",
      end,
      len,
    );

    match &self.repr {
      Repr::Inline { .. } => {
        let mut new_buf = [0u8; INLINE_CAP];
        new_buf[..slen].copy_from_slice(&self.as_slice()[begin..end]);
        Self::inline(new_buf, 0, slen)
      }
      Repr::Heap(bytes) => Self::heap(bytes.slice(begin..end)),
    }
  }

  fn split_to(&mut self, at: usize) -> Self {
    let len = self.len();
    if at == len {
      return mem::take(self);
    }

    if at == 0 {
      return Self::new();
    }

    assert!(at <= len, "split_to out of bounds: {:?} <= {:?}", at, len,);

    // first, check if output can be inline
    let ret = if at <= INLINE_CAP {
      let mut buf = [0u8; INLINE_CAP];
      let src = self.as_slice();
      buf[..at].copy_from_slice(&src[..at]);
      Self::inline(buf, 0, at)
    } else {
      // output cannot be inline, which means it must be heap allocated
      let mut bytes = self.repr.unwrap_heap_mut().clone();
      bytes.truncate(at);
      Self::heap(bytes)
    };

    let remaining_size = len - at;

    match &mut self.repr {
      Repr::Inline { len, cur, .. } => {
        *cur = len.to_u8() - (remaining_size as u8);
      }
      Repr::Heap(bytes) => {
        bytes.advance(at);
      }
    }

    ret
  }

  fn split_off(&mut self, at: usize) -> Self {
    let len = self.len();
    if at == len {
      return Self::new();
    }

    if at == 0 {
      return mem::take(self);
    }

    assert!(at <= len, "split_off out of bounds: {:?} <= {:?}", at, len,);

    // first, check if output would be inline
    let output_size = len - at;
    let ret = if output_size <= INLINE_CAP {
      let mut buf = [0u8; INLINE_CAP];
      let src = self.as_slice();
      buf[..output_size].copy_from_slice(&src[at..len]);
      Self::inline(buf, 0, output_size)
    } else {
      // output cannot be inline, which means it must be heap allocated
      let mut bytes = self.repr.unwrap_heap_mut().clone();
      bytes.advance(at);
      Self::heap(bytes)
    };

    // second, check if self can be made inline

    match &mut self.repr {
      Repr::Inline { len, cur, .. } => {
        // len represents the end position in buf, so we need cur + at
        *len = unsafe { InlineSize::from_u8((*cur as usize + at) as u8) };
      }
      Repr::Heap(bytes) => {
        bytes.truncate(at);
      }
    }

    ret
  }

  fn truncate(&mut self, new_len: usize) {
    match &mut self.repr {
      Repr::Inline { len, cur, .. } => {
        let current_len = len.to_usize() - (*cur as usize);
        if new_len <= current_len {
          // len represents the end position in buf, so we need cur + new_len
          *len = unsafe { InlineSize::from_u8((*cur as usize + new_len) as u8) };
        }
      }
      Repr::Heap(bytes) => bytes.truncate(new_len),
    }
  }

  fn advance(&mut self, cnt: usize) {
    match &mut self.repr {
      Repr::Inline { len, cur, .. } => {
        let remaining = len.to_u8() - *cur;
        assert!(
          cnt <= remaining as usize,
          "cannot advance past `remaining`: {:?} <= {:?}",
          cnt,
          remaining,
        );
        *cur += cnt as u8;
      }
      Repr::Heap(bytes) => bytes.advance(cnt),
    }
  }

  fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes {
    self.split_to(len).into()
  }

  fn clear(&mut self) {
    match &mut self.repr {
      Repr::Heap(bytes) => bytes.clear(),
      Repr::Inline { len, cur, .. } => {
        *len = InlineSize::_V0;
        *cur = 0;
      }
    }
  }
}

/// A space-efficient byte buffer that shares heap allocations with [`bytes::Bytes`].
///
/// This is a type alias for [`RawSmolBytes<Shared>`](crate::smol_bytes::RawSmolBytes) using the [`Shared`] strategy.
///
/// # When to use
///
/// Use `SmolBytes` (with `Shared` strategy) when:
/// - You frequently convert to/from `bytes::Bytes`
/// - You want fast, zero-copy operations
/// - Memory overhead is acceptable for performance gains
///
/// For memory-constrained applications, consider [`compact::SmolBytes`](super::compact::SmolBytes) instead.
///
/// # Example
///
/// ```rust
/// use smol_bytes::strategy::shared::SmolBytes;
///
/// let data = SmolBytes::from_static(b"hello world");
/// assert_eq!(data.as_slice(), b"hello world");
///
/// // Efficient conversion to Bytes
/// let bytes: bytes::Bytes = data.into();
/// ```
pub type SmolBytes = RawSmolBytes<Shared>;
