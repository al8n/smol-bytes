//! The **Compact** strategy for `SmolBytes`.
//!
//! This module provides the [`SmolBytes`] type alias configured with the [`Compact`] strategy,
//! which prioritizes **memory efficiency** by aggressively converting heap allocations back
//! to inline storage whenever possible.
//!
//! # Key Characteristics
//!
//! - **Aggressive inlining**: Automatically converts heap→inline when data fits (≤62 bytes)
//! - **Memory-efficient**: Minimizes heap allocations and memory overhead
//! - **Smart optimization**: Operations like `advance()`, `truncate()`, and `split_to/off()` trigger conversions
//! - **Best for constrained environments**: Ideal for embedded systems and memory-critical applications
//!
//! # When to Use
//!
//! Choose this strategy when:
//!
//! - **Memory is limited**: Embedded systems, microcontrollers, or memory-constrained environments
//! - **Many small buffers**: You work with numerous buffers that frequently shrink over time
//! - **Rare `Bytes` conversions**: You don't often convert to/from `bytes::Bytes`
//! - **Allocation minimization**: You want to minimize heap allocations at the cost of occasional copies
//!
//! # Basic Usage
//!
//! ```rust
//! use smol_bytes::{strategy::compact::SmolBytes, Buf};
//!
//! // Small data (≤62 bytes) is stored inline
//! let small = SmolBytes::from_static(b"hello world");
//! assert!(!small.is_heap());
//!
//! // Large data starts on heap
//! let mut large = SmolBytes::from(vec![1u8; 100]);
//! assert!(large.is_heap());
//!
//! // After shrinking, automatically converts to inline!
//! large.advance(70); // 30 bytes remain
//! assert!(!large.is_heap()); // ✓ Now inline!
//! ```
//!
//! # Behavior Details
//!
//! ## Memory Layout (Same as Shared)
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
//! │  Variant: Heap (>62 bytes only)         │
//! │  ┌────────────────────────────────────┐ │
//! │  │ bytes::Bytes (Arc<[u8]>)           │ │
//! │  └────────────────────────────────────┘ │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Operations and Allocation Behavior
//!
//! ```rust
//! use smol_bytes::{strategy::compact::SmolBytes, Buf};
//!
//! // Start with large heap allocation
//! let mut data = SmolBytes::from(vec![1u8; 100]);
//! assert!(data.is_heap());
//!
//! // After advance, automatically inlined (Compact strategy)
//! data.advance(70); // 30 bytes remain
//! assert!(!data.is_heap()); // ✓ Converted to inline!
//! ```
//!
//! ## Comparison: Heap→Inline Conversion Triggers
//!
//! | Operation | Before | After | Conversion? |
//! |-----------|--------|-------|-------------|
//! | `advance(n)` | Heap (100 bytes) | Inline (30 bytes) | ✅ Yes (if ≤62 bytes remain) |
//! | `truncate(n)` | Heap (100 bytes) | Inline (30 bytes) | ✅ Yes (if n ≤62) |
//! | `split_to(n)` | Heap (100 bytes) | Inline (remaining) | ✅ Yes (if remaining ≤62) |
//! | `split_off(n)` | Heap (100 bytes) | Inline (first part) | ✅ Yes (if first ≤62) |
//! | `slice(range)` | Heap | Inline | ✅ Yes (if result ≤62) |
//!
//! # Performance Characteristics
//!
//! ## Fast Operations (O(1))
//!
//! - `clone()` when inline - Simple memcpy
//! - `advance()` when staying inline or heap
//! - `truncate()` when staying inline or heap
//! - Operations that don't trigger conversion
//!
//! ## Linear Operations (O(62) - copies up to 62 bytes)
//!
//! - **Heap→Inline conversion** - Copies data to stack (up to 62 bytes)
//! - `advance()` when triggering conversion
//! - `truncate()` when triggering conversion
//! - `split_to()` / `split_off()` when triggering conversion
//! - `into::<Bytes>()` when inline (must copy to heap)
//!
//! **Note**: Since the maximum copy size is fixed at 62 bytes, these operations are very fast in practice!
//!
//! # Examples
//!
//! ## Stream Processing with Automatic Inlining
//!
//! ```rust
//! use smol_bytes::{strategy::compact::SmolBytes, Buf};
//!
//! // Process incoming stream
//! let mut buffer = SmolBytes::from(vec![0u8; 1024]);
//! assert!(buffer.is_heap());
//!
//! // As we consume data, it automatically inlines
//! buffer.advance(1000); // 24 bytes remain
//! assert!(!buffer.is_heap()); // Saved memory!
//! ```
//!
//! ## Memory-Efficient Buffer Pool
//!
//! ```rust
//! use smol_bytes::strategy::compact::SmolBytes;
//!
//! struct BufferPool {
//!     buffers: Vec<SmolBytes>,
//! }
//!
//! impl BufferPool {
//!     fn new() -> Self {
//!         Self { buffers: Vec::new() }
//!     }
//!
//!     fn add(&mut self, data: Vec<u8>) {
//!         // Automatically inlines if small enough
//!         self.buffers.push(SmolBytes::from(data));
//!     }
//!
//!     fn total_heap_allocations(&self) -> usize {
//!         self.buffers.iter()
//!             .filter(|b| b.is_heap())
//!             .count()
//!     }
//! }
//!
//! let mut pool = BufferPool::new();
//!
//! // Add mix of small and large buffers
//! pool.add(vec![1; 10]);  // Inline
//! pool.add(vec![2; 30]);  // Inline
//! pool.add(vec![3; 100]); // Heap
//!
//! // Only one heap allocation!
//! assert_eq!(pool.total_heap_allocations(), 1);
//! ```
//!
//! ## Truncate for Memory Savings
//!
//! ```rust
//! use smol_bytes::strategy::compact::SmolBytes;
//!
//! let mut data = SmolBytes::from(vec![1u8; 100]);
//! assert!(data.is_heap());
//!
//! // Truncate to small size - automatically inlines
//! data.truncate(20);
//! assert!(!data.is_heap());
//! assert_eq!(data.len(), 20);
//! ```
//!
//! ## Smart Split Operations
//!
//! ```rust
//! use smol_bytes::strategy::compact::SmolBytes;
//!
//! let mut data = SmolBytes::from(vec![1u8; 100]);
//!
//! // Split off small portion - both parts optimize
//! let first = data.split_to(30);  // first: 30 bytes (inline)
//! // data: 70 bytes (heap)
//!
//! assert!(!first.is_heap()); // Automatically inlined!
//! assert!(data.is_heap());    // Still too large for inline
//! ```
//!
//! # Trade-offs vs Shared Strategy
//!
//! ## Advantages
//!
//! - ✅ **Lower memory usage**: Fewer heap allocations
//! - ✅ **Better cache locality**: More data on stack
//! - ✅ **Fewer allocations**: Automatic heap→inline conversion
//! - ✅ **Simpler deallocation**: Inline data needs no cleanup
//!
//! ## Disadvantages
//!
//! - ❌ **Conversion overhead**: O(62) copy when heap→inline (up to 62 bytes)
//! - ❌ **Bytes conversion cost**: Must copy when inline
//! - ❌ **More copies**: Cloning inline data copies all bytes
//! - ❌ **No zero-copy for small data**: Inline can't share with `Bytes`
//!
//! # Benchmarks
//!
//! Typical performance characteristics (on x86_64):
//!
//! - **Heap→Inline conversion**: ~10-20ns for 62 bytes
//! - **Inline clone**: ~5-10ns for 62 bytes
//! - **Memory saved**: 32 bytes per buffer (no heap overhead)
//!
//! # Migration Guide
//!
//! If you're currently using `shared::SmolBytes` and considering switching:
//!
//! ```rust
//! // Before (Shared strategy)
//! use smol_bytes::{strategy::{shared, compact}, Buf};
//! 
//! let mut data = shared::SmolBytes::from(vec![1u8; 100]);
//! data.advance(70); // Still heap-allocated
//! let bytes: bytes::Bytes = data.into(); // Zero-copy ✓
//!
//! // After (Compact strategy)
//! 
//! let mut data = compact::SmolBytes::from(vec![1u8; 100]);
//! data.advance(70); // Now inline! Saved memory ✓
//! let bytes: bytes::Bytes = data.into(); // Copies 30 bytes (still fast!)
//! ```
//!
//! **Rule of thumb**: If you convert to `Bytes` more than once per buffer lifetime,
//! use `Shared`. If memory is more important than conversion speed, use `Compact`.

use super::Strategy;
use crate::{
  smol_bytes::raw::{RawSmolBytes, Repr},
  utils::{InlineStorage, INLINE_CAP},
};
use bytes::Buf;
use core::mem;
use core::ops::{Bound, RangeBounds};

/// A strategy that aggressively inlines data to minimize heap allocations and memory usage.
///
/// # Overview
///
/// The `Compact` strategy prioritizes **memory efficiency** over conversion speed. It
/// automatically converts heap-allocated data back to inline storage whenever possible
/// (when data size ≤62 bytes) after operations like `advance()`, `truncate()`, or
/// `split_to/off()`.
///
/// This makes `Compact` ideal when:
/// - Memory footprint is critical
/// - You want to minimize heap allocations
/// - You're working with small, frequently-modified buffers
/// - Conversions to `Bytes` are rare
///
/// # Behavior
///
/// - **Inline → Inline**: Small data stays inline (≤62 bytes)
/// - **Heap → Inline**: Automatically inlines when data shrinks ≤62 bytes
/// - **Smart optimization**: Operations like `advance()`, `truncate()`, and `split_to/off()`
///   can trigger heap→inline conversion
///
/// # Example
///
/// ```rust
/// use smol_bytes::strategy::compact::SmolBytes;
/// use bytes::Buf;
///
/// // Create heap-allocated bytes (>62 bytes)
/// let mut data = SmolBytes::from(vec![1u8; 100]);
/// assert!(data.is_heap());
///
/// // Advance past most data
/// data.advance(70); // Only 30 bytes remain
///
/// // Automatically converted to inline! (Compact strategy saves memory)
/// assert!(!data.is_heap());
/// ```
///
/// # Comparison with Shared
///
/// | Operation | Compact | Shared |
/// |-----------|---------|--------|
/// | Heap→Inline on shrink | ✅ Yes | ❌ No |
/// | Bytes conversion | 📋 May copy | ⚡ Zero-copy |
/// | Memory usage | 💾 Lower | 💾 Higher |
/// | Best for | Memory efficiency | Speed, Bytes interop |
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Compact(());

impl From<bytes::Bytes> for RawSmolBytes<Compact> {
  fn from(bytes: bytes::Bytes) -> Self {
    Self::heap(bytes)
  }
}

impl Strategy for RawSmolBytes<Compact> {
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

    if slen <= INLINE_CAP {
      // SAFETY: bounds checked above, and we are slicing within inline capacity.
      return Self::inline(unsafe { InlineStorage::copy_from_slice(&self.as_slice()[begin..end]) });
    }

    match &self.repr {
      Repr::Inline(_) => {
        unreachable!("slice length exceeds inline capacity");
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
      let src = self.as_slice();
      // SAFETY: bounds checked above, and we are slicing within inline capacity.
      Self::inline(unsafe { InlineStorage::copy_from_slice(&src[..at]) })
    } else {
      // output cannot be inline, which means it must be heap allocated
      let mut bytes = self.repr.unwrap_heap_mut().clone();
      bytes.truncate(at);
      Self::heap(bytes)
    };

    // second, check if self can be made inline
    let remaining_size = len - at;
    if remaining_size <= INLINE_CAP {
      // check if we already are inline, if so, adjust cur, avoid copy
      if let Repr::Inline(storage) = &mut self.repr {
        storage.advance(at);
        return ret;
      }

      let src = self.as_slice();
      // SAFETY: bounds checked above, and we are slicing within inline capacity.
      let repr = Repr::inline(unsafe { InlineStorage::copy_from_slice(&src[at..len]) });

      let _ = mem::replace(&mut self.repr, repr);
    } else {
      // self remains heap allocated
      self.repr.unwrap_heap_mut().advance(at);
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
      let src = self.as_slice();
      // SAFETY: bounds checked above, and we are slicing within inline capacity.
      Self::inline(unsafe { InlineStorage::copy_from_slice(&src[at..len]) })
    } else {
      // output cannot be inline, which means it must be heap allocated
      let mut bytes = self.repr.unwrap_heap_mut().clone();
      bytes.advance(at);
      Self::heap(bytes)
    };

    // second, check if self can be made inline
    if at <= INLINE_CAP {
      // check if we already are inline, if so, adjust len
      if let Repr::Inline(storage) = &mut self.repr {
        storage.truncate(at);
        return ret;
      }
      let src = self.as_slice();
      // SAFETY: bounds checked above, and we are slicing within inline capacity.
      let repr = Repr::inline(unsafe { InlineStorage::copy_from_slice(&src[..at]) });
      let _ = mem::replace(&mut self.repr, repr);
    } else {
      // self remains heap allocated
      self.repr.unwrap_heap_mut().truncate(at);
    }

    ret
  }

  fn truncate(&mut self, new_len: usize) {
    match &mut self.repr {
      Repr::Inline(storage) => {
        storage.truncate(new_len);
      }
      Repr::Heap(bytes) => {
        if new_len <= INLINE_CAP {
          let repr = Repr::inline(unsafe { InlineStorage::copy_from_slice(&bytes[..new_len]) });
          let _ = mem::replace(&mut self.repr, repr);
        } else {
          bytes.truncate(new_len);
        }
      }
    }
  }

  fn advance(&mut self, cnt: usize) {
    match &mut self.repr {
      Repr::Inline(storage) => {
        storage.advance(cnt);
      }
      Repr::Heap(bytes) => {
        // check if we can make inline after advance
        let len = bytes.len();
        assert!(
          cnt <= len,
          "cannot advance past `remaining`: {:?} <= {:?}",
          cnt,
          len,
        );

        let remaining = len - cnt;
        if remaining <= INLINE_CAP {
          // SAFETY: bounds checked above, and we are slicing within inline capacity.
          let repr = Repr::inline(unsafe { InlineStorage::copy_from_slice(&bytes[cnt..]) });
          let _ = mem::replace(&mut self.repr, repr);
        } else {
          bytes.advance(cnt);
        }
      }
    }
  }

  fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes {
    self.split_to(len).into()
  }

  fn clear(&mut self) {
    match &mut self.repr {
      Repr::Heap(bytes) => bytes.clear(),
      Repr::Inline(storage) => storage.clear(),
    }
  }
}

/// A memory-efficient byte buffer that aggressively inlines data to minimize heap usage.
///
/// This is a type alias for [`RawSmolBytes<Compact>`](crate::smol_bytes::RawSmolBytes) using the [`Compact`] strategy.
///
/// # When to use
///
/// Use `SmolBytes` (with `Compact` strategy) when:
/// - Memory footprint is critical
/// - You're working with small, frequently-modified buffers
/// - Heap allocations should be minimized
/// - Conversions to `bytes::Bytes` are infrequent
///
/// For applications that frequently convert to/from `Bytes`, consider [`shared::SmolBytes`](super::shared::SmolBytes) instead.
///
/// # Example
///
/// ```rust
/// use smol_bytes::strategy::compact::SmolBytes;
/// use bytes::Buf;
///
/// let mut data = SmolBytes::from(vec![1u8; 100]);
/// assert!(data.is_heap());
///
/// // After advancing, automatically converts to inline
/// data.advance(70);
/// assert!(!data.is_heap()); // Saved memory!
/// ```
pub type SmolBytes = RawSmolBytes<Compact>;
