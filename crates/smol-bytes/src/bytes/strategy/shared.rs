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
//! use smol_bytes::shared::SmolBytes;
//!
//! // Small data (≤62 bytes) is stored inline
//! let small = SmolBytes::from_static(b"hello world");
//! assert!(!small.is_heap());
//!
//! // Large data is heap-allocated
//! let large = SmolBytes::from(vec![1u8; 100]);
//! assert!(large.is_heap());
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
//! use smol_bytes::shared::SmolBytes;
//! use bytes::Buf;
//!
//! // Start with large heap allocation
//! let mut data = SmolBytes::from(vec![1u8; 100]);
//! assert!(data.is_heap());
//!
//! // After advance, still heap-allocated (Shared strategy)
//! data.advance(70); // 30 bytes remain
//! assert!(data.is_heap()); // ✓ Still on heap!
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
//! use smol_bytes::shared::SmolBytes;
//! use bytes::Buf;
//!
//! // Receive data from network
//! let mut buffer = SmolBytes::from(vec![0u8; 1024]);
//!
//! // Process header (advance past it)
//! buffer.advance(16);
//!
//! // Buffer stays on heap for efficient passing to bytes::Bytes
//! assert!(buffer.is_heap());
//!
//! // Zero-copy conversion for writing
//! let bytes: bytes::Bytes = buffer.into();
//! // ... write bytes to socket
//! ```
//!
//! ## Parsing with Zero-Copy Slicing
//!
//! ```rust
//! use smol_bytes::shared::SmolBytes;
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
//! use smol_bytes::shared::SmolBytes;
//!
//! let original = SmolBytes::from(vec![1u8; 100]);
//!
//! // Cheap clones (just Arc reference count)
//! let clone1 = original.clone();
//! let clone2 = original.clone();
//! let clone3 = original.clone();
//!
//! // All share the same heap allocation
//! assert!(original.is_heap());
//! assert!(clone1.is_heap());
//! ```

use super::Strategy;
use crate::{
  buffer::{Buffer, INLINE_CAP},
  bytes::raw::{RawBytes, Repr},
};
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
/// use smol_bytes::shared::SmolBytes;
/// use bytes::Buf;
///
/// // Create heap-allocated bytes (>62 bytes)
/// let mut data = SmolBytes::from(vec![1u8; 100]);
/// assert!(data.is_heap());
///
/// // Advance past most data
/// data.advance(70); // Only 30 bytes remain
///
/// // Still heap-allocated! (Shared strategy preserves heap)
/// assert!(data.is_heap());
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

impl RawBytes<Shared> {
  /// Create [`SmolBytes`] with a buffer whose lifetime is controlled
  /// via an explicit owner.
  ///
  /// A common use case is to zero-copy construct from mapped memory.
  ///
  /// ```
  /// # struct File;
  /// #
  /// # impl File {
  /// #     pub fn open(_: &str) -> Result<Self, ()> {
  /// #         Ok(Self)
  /// #     }
  /// # }
  /// #
  /// # mod memmap2 {
  /// #     pub struct Mmap;
  /// #
  /// #     impl Mmap {
  /// #         pub unsafe fn map(_file: &super::File) -> Result<Self, ()> {
  /// #             Ok(Self)
  /// #         }
  /// #     }
  /// #
  /// #     impl AsRef<[u8]> for Mmap {
  /// #         fn as_ref(&self) -> &[u8] {
  /// #             b"buf"
  /// #         }
  /// #     }
  /// # }
  /// use smol_bytes::shared::SmolBytes;
  /// use memmap2::Mmap;
  ///
  /// # fn main() -> Result<(), ()> {
  /// let file = File::open("upload_bundle.tar.gz")?;
  /// let mmap = unsafe { Mmap::map(&file) }?;
  /// let b = SmolBytes::from_owner(mmap);
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// The `owner` will be transferred to the constructed [`SmolBytes`] object, which
  /// will ensure it is dropped once all remaining clones of the constructed
  /// object are dropped. The owner will then be responsible for dropping the
  /// specified region of memory as part of its [Drop] implementation.
  ///
  /// Note that converting [`SmolBytes`] constructed from an owner into a [`BytesMut`]
  /// will always create a deep copy of the buffer into newly allocated memory.
  pub fn from_owner<T>(owner: T) -> Self
  where
    T: AsRef<[u8]> + Send + 'static,
  {
    Self::heap(bytes::Bytes::from_owner(owner))
  }
}

impl Strategy for RawBytes<Shared> {
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

    assert!(
      begin <= len,
      "range start out of bounds: {:?} <= {:?}",
      begin,
      len,
    );
    assert!(
      end <= len,
      "range end out of bounds: {:?} <= {:?}",
      end,
      len,
    );

    match &self.repr {
      Repr::Inline(storage) => {
        // SAFETY: bounds checked above, and we are slicing within inline storage.
        Self::inline(unsafe { Buffer::copy_from_slice(&storage[begin..end]) })
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
      Self::inline(unsafe { Buffer::copy_from_slice(&src[..at]) })
    } else {
      // output cannot be inline, which means it must be heap allocated
      let mut bytes = self.repr.unwrap_heap_mut().clone();
      bytes.truncate(at);
      Self::heap(bytes)
    };

    match &mut self.repr {
      Repr::Inline(storage) => {
        storage.advance(at);
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
      let src = self.as_slice();
      // SAFETY: bounds checked above, and we are slicing within inline capacity.
      Self::inline(unsafe { Buffer::copy_from_slice(&src[at..len]) })
    } else {
      // output cannot be inline, which means it must be heap allocated
      let mut bytes = self.repr.unwrap_heap_mut().clone();
      bytes.advance(at);
      Self::heap(bytes)
    };

    // second, check if self can be made inline

    match &mut self.repr {
      Repr::Inline(storage) => {
        storage.truncate(at);
      }
      Repr::Heap(bytes) => {
        bytes.truncate(at);
      }
    }

    ret
  }

  fn truncate(&mut self, new_len: usize) {
    match &mut self.repr {
      Repr::Inline(storage) => storage.truncate(new_len),
      Repr::Heap(bytes) => bytes.truncate(new_len),
    }
  }

  fn advance(&mut self, cnt: usize) {
    match &mut self.repr {
      Repr::Inline(storage) => storage.advance(cnt),
      Repr::Heap(bytes) => bytes.advance(cnt),
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

/// A space-efficient byte buffer that shares heap allocations with [`bytes::Bytes`].
///
/// This is a type alias for [`RawBytes<Shared>`](crate::smol_bytes::RawBytes) using the [`Shared`] strategy.
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
/// use smol_bytes::shared::SmolBytes;
///
/// let data = SmolBytes::from_static(b"hello world");
/// assert_eq!(data.as_slice(), b"hello world");
///
/// // Efficient conversion to Bytes
/// let bytes: bytes::Bytes = data.into();
/// ```
pub type SmolBytes = RawBytes<Shared>;
