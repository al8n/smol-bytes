use core::mem::MaybeUninit;

use ::bytes::BufMut;
use bytes::Buf;

use crate::{buffer::Buffer, bytes::RawBytes, INLINE_CAP};

mod cmp;
mod fmt;
mod from;
mod iter;
mod ops;

#[cfg(feature = "arbitrary")]
mod arbitrary;
#[cfg(feature = "borsh")]
mod borsh;
#[cfg(feature = "quickcheck")]
mod quickcheck;
#[cfg(feature = "serde")]
mod serde;

/// A mutable byte buffer with inline storage optimization.
///
/// `BytesMut` is similar to [`BytesMut`] but optimized for small buffers.
/// It stores data inline (on the stack) for buffers up to 62 bytes, avoiding heap
/// allocations. For larger buffers, it automatically promotes to heap storage using
/// [`BytesMut`] internally.
///
/// # Inline vs Heap Storage
///
/// - **Inline**: Buffers ≤62 bytes are stored directly in the `BytesMut` struct
/// - **Heap**: Buffers >62 bytes are automatically promoted to heap allocation
/// - Once promoted to heap, the buffer stays on the heap (no automatic demotion)
///
/// # Split Operations
///
/// Split operations have different behavior based on storage type:
/// - [`split_off`](Self::split_off): Returns `Ok(BytesMut)` for heap, `Err(Buffer)` for inline
/// - [`split_to`](Self::split_to): Returns `Ok(BytesMut)` for heap, `Err(Buffer)` for inline
/// - [`split`](Self::split): Returns `Ok(BytesMut)` for heap, `Err(Buffer)` for inline
/// - [`unsplit`](Self::unsplit): Only works when both buffers are heap-allocated
///
/// `Buffer` is a mutable inline-only buffer (max 62 bytes), while `BytesMut` can grow.
/// Use [`make_heap`](Self::make_heap) to explicitly promote an inline buffer to heap storage.
///
/// ## Examples
///
/// ```
/// use smol_bytes::BytesMut;
///
/// // Small buffer uses inline storage
/// let mut buf = BytesMut::from(&b"hello"[..]);
/// assert!(buf.is_inline());
/// assert_eq!(&buf[..], b"hello");
///
/// // Extending beyond capacity promotes to heap
/// buf.extend_from_slice(b" world and more data that exceeds inline capacity................................");
/// assert!(buf.is_heap());
/// ```
///
/// # Freezing
///
/// Convert to an immutable buffer using:
/// - [`freeze_shared`](Self::freeze_shared): Convert to [`shared::Bytes`](crate::shared::Bytes)
/// - [`freeze_compact`](Self::freeze_compact): Convert to [`compact::Bytes`](crate::compact::Bytes)
#[derive(Clone)]
pub struct BytesMut(Repr);

impl Default for BytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new()
  }
}

impl BytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn from_bytes(bytes: bytes::Bytes) -> Self {
    if !bytes.is_unique() && bytes.len() <= INLINE_CAP {
      // SAFETY: bytes.len() is guaranteed to be less than or equal to INLINE_CAP
      return Self(Repr::Inline(unsafe { Buffer::copy_from_slice(&bytes) }));
    }

    Self(Repr::Heap(bytes.into()))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn from_bytes_mut(bytes: bytes::BytesMut) -> Self {
    Self(Repr::Heap(bytes))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn from_inline(bytes: Buffer) -> Self {
    Self(Repr::Inline(bytes))
  }
}

impl BytesMut {
  /// Creates a new `BytesMut` containing `len` zeros.
  ///
  /// The resulting object has a length of `len` and a capacity greater
  /// than or equal to `len`. The entire length of the object will be filled
  /// with zeros.
  ///
  /// On some platforms or allocators this function may be faster than
  /// a manual implementation.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::BytesMut;
  ///
  /// let zeros = BytesMut::zeroed(42);
  ///
  /// assert!(zeros.capacity() >= 42);
  /// assert_eq!(zeros.len(), 42);
  /// zeros.into_iter().for_each(|x| assert_eq!(x, 0));
  /// ```
  pub fn zeroed(len: usize) -> Self {
    if len <= INLINE_CAP {
      // SAFETY: len is guaranteed to be less than or equal to INLINE_CAP
      Self(Repr::Inline(unsafe { Buffer::zeroed(len) }))
    } else {
      Self(Repr::Heap(bytes::BytesMut::zeroed(len)))
    }
  }

  /// Creates a new, empty `BytesMut`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::BytesMut;
  ///
  /// let bytes = BytesMut::new();
  /// assert_eq!(bytes.len(), 0);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self(Repr::Inline(Buffer::new()))
  }

  /// Creates a new `BytesMut` with the specified capacity.
  ///
  /// The returned `BytesMut` will be able to hold at least capacity bytes without reallocating.
  ///
  /// It is important to note that this function does not specify the length of the returned BytesMut, but only the capacity.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_capacity(capacity: usize) -> Self {
    if capacity <= INLINE_CAP {
      Self(Repr::Inline(Buffer::new()))
    } else {
      Self(Repr::Heap(bytes::BytesMut::with_capacity(capacity)))
    }
  }

  /// Appends given bytes to this `BytesMut`.
  ///
  /// If this `BytesMut` object does not have enough capacity, it is resized
  /// first.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::BytesMut;
  ///
  /// let mut buf = BytesMut::with_capacity(0);
  /// buf.extend_from_slice(b"aaabbb");
  /// buf.extend_from_slice(b"cccddd");
  ///
  /// assert_eq!(b"aaabbbcccddd", &buf[..]);
  /// ```
  #[inline]
  pub fn extend_from_slice(&mut self, extend: &[u8]) {
    match &mut self.0 {
      Repr::Inline(b) => {
        let available = b.remaining_mut();
        let requested = extend.len();
        if available >= requested {
          b.put_slice(extend);
          return;
        }

        let mut new_buf = bytes::BytesMut::with_capacity(b.len() + requested);
        new_buf.put_slice(b.as_slice());
        new_buf.extend_from_slice(extend);
        self.0 = Repr::Heap(new_buf);
      }
      Repr::Heap(b) => b.extend_from_slice(extend),
    }
  }

  /// Clears the buffer, removing all data. Existing capacity is preserved.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::BytesMut;
  ///
  /// let mut bytes = BytesMut::from(&b"hello world"[..]);
  /// bytes.clear();
  /// assert_eq!(bytes.len(), 0);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn clear(&mut self) {
    match &mut self.0 {
      Repr::Inline(b) => b.clear(),
      Repr::Heap(b) => b.clear(),
    }
  }

  /// Shortens the buffer, keeping the first len bytes and dropping the rest.
  ///
  /// If len is greater than the buffer’s current length, this has no effect.
  ///
  /// Existing underlying capacity is preserved.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::BytesMut;
  ///
  /// let mut bytes = BytesMut::from(&b"hello world"[..]);
  ///
  /// bytes.truncate(5);
  /// assert_eq!(bytes.as_slice(), b"hello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn truncate(&mut self, len: usize) {
    match &mut self.0 {
      Repr::Inline(b) => b.truncate(len),
      Repr::Heap(b) => b.truncate(len),
    }
  }

  /// Returns the number of bytes the `BytesMut` can hold without reallocating.
  ///
  /// ## Example
  ///
  /// ```rust
  ///
  /// use smol_bytes::BytesMut;
  ///
  /// let bytes = BytesMut::with_capacity(100);
  ///
  /// assert_eq!(bytes.capacity(), 100);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn capacity(&self) -> usize {
    match &self.0 {
      Repr::Inline(b) => b.capacity(),
      Repr::Heap(b) => b.capacity(),
    }
  }

  /// Returns `true` if the `BytesMut` is using inline storage.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use smol_bytes::BytesMut;
  ///
  /// let inline_buf = BytesMut::with_capacity(10);
  /// assert!(inline_buf.is_inline());
  ///
  /// let heap_buf = BytesMut::with_capacity(100);
  /// assert!(!heap_buf.is_inline());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_inline(&self) -> bool {
    matches!(&self.0, Repr::Inline(_))
  }

  /// Returns `true` if the `BytesMut` is using heap storage.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::BytesMut;
  ///
  /// let inline_buf = BytesMut::with_capacity(10);
  /// assert!(!inline_buf.is_heap());
  ///
  /// let heap_buf = BytesMut::with_capacity(100);
  /// assert!(heap_buf.is_heap());
  /// ```
  pub const fn is_heap(&self) -> bool {
    matches!(&self.0, Repr::Heap(_))
  }

  /// Unwraps the inline buffer, consuming `self`.
  ///
  /// ## Panics
  /// - Panics if the buffer is heap allocated.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::BytesMut;
  ///
  /// let buf = BytesMut::from(&b"hello"[..]);
  ///
  /// let inline_buffer = buf.unwrap_inline();
  /// assert_eq!(&inline_buffer[..], b"hello");
  /// ```
  #[inline]
  pub fn unwrap_inline(self) -> Buffer {
    match self.0 {
      Repr::Inline(b) => b,
      Repr::Heap(_) => panic!("called `BytesMut::unwrap_inline()` on a heap allocated buffer"),
    }
  }

  /// Attempts to unwrap the inline buffer, consuming `self`.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::BytesMut;
  ///
  /// let inline_buf = BytesMut::from(&b"hello"[..]);
  /// let heap_buf = BytesMut::with_capacity(100);
  ///
  /// assert!(inline_buf.try_unwrap_inline().is_ok());
  /// assert!(heap_buf.try_unwrap_inline().is_err());
  /// ```
  #[inline]
  pub fn try_unwrap_inline(self) -> Result<Buffer, bytes::BytesMut> {
    match self.0 {
      Repr::Inline(b) => Ok(b),
      Repr::Heap(b) => Err(b),
    }
  }

  /// Unwraps the heap buffer, consuming `self`.
  ///
  /// ## Panics
  /// - Panics if the buffer is inline.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::BytesMut;
  ///
  /// let mut buf = BytesMut::with_capacity(100);
  /// buf.extend_from_slice(b"hello world and more data that exceeds inline capacity................................");
  ///
  /// let heap_buffer = buf.unwrap_heap();
  /// assert_eq!(&heap_buffer[..], b"hello world and more data that exceeds inline capacity................................");
  /// ```
  #[inline]
  pub fn unwrap_heap(self) -> bytes::BytesMut {
    match self.0 {
      Repr::Inline(_) => panic!("called `BytesMut::unwrap_heap()` on an inline buffer"),
      Repr::Heap(b) => b,
    }
  }

  /// Attempts to unwrap the heap buffer, consuming `self`.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::BytesMut;
  ///
  /// let inline_buf = BytesMut::from(&b"hello"[..]);
  /// let mut heap_buf = BytesMut::with_capacity(100);
  /// heap_buf.extend_from_slice(b"hello world and more data that exceeds inline capacity................................");
  ///
  /// assert!(heap_buf.try_unwrap_heap().is_ok());
  /// assert!(inline_buf.try_unwrap_heap().is_err());
  /// ```
  #[inline]
  pub fn try_unwrap_heap(self) -> Result<bytes::BytesMut, Buffer> {
    match self.0 {
      Repr::Inline(b) => Err(b),
      Repr::Heap(b) => Ok(b),
    }
  }

  /// Converts the `BytesMut` into a heap allocated buffer if it is currently inline.
  ///
  /// If the buffer is already heap allocated, this function does nothing.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::BytesMut;
  ///
  /// let mut buf = BytesMut::from(&b"hello"[..]);
  /// assert!(buf.is_inline());
  /// buf.make_heap();
  /// assert!(buf.is_heap());
  /// ```
  pub fn make_heap(&mut self) {
    match &mut self.0 {
      Repr::Inline(b) => {
        let mut new_buf = bytes::BytesMut::with_capacity(b.len());
        new_buf.put_slice(b.as_slice());
        self.0 = Repr::Heap(new_buf);
      }
      Repr::Heap(_) => {}
    }
  }

  /// Splits the bytes into two at the given index.
  ///
  /// Afterwards `self` contains elements `[0, at)`, and the returned value
  /// contains elements `[at, len)`.
  ///
  /// For heap-allocated buffers, this is an `O(1)` operation that increases the
  /// reference count and returns `Ok(BytesMut)`.
  ///
  /// For inline buffers, the tail is copied into a `Buffer` and returned as `Err(Buffer)`.
  /// Both `BytesMut` and `Buffer` are mutable, but `Buffer` is limited to 62 bytes inline storage.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::BytesMut;
  ///
  /// // Inline buffer
  /// let mut a = BytesMut::from(&b"hello world"[..]);
  /// match a.split_off(5) {
  ///     Ok(mut b) => {
  ///         // Heap: BytesMut can grow beyond 62 bytes
  ///         b[0] = b'!';
  ///         assert_eq!(&b[..], b"!world");
  ///     }
  ///     Err(mut b) => {
  ///         // Inline: Buffer is limited to 62 bytes but still mutable
  ///         assert_eq!(&b[..], b" world");
  ///     }
  /// }
  /// assert_eq!(&a[..], b"hello");
  ///
  /// // Heap buffer
  /// let mut a = BytesMut::with_capacity(64);
  /// a.extend_from_slice(b"hello world");
  /// let mut b = a.split_off(5).unwrap();
  /// a[0] = b'j';
  /// b[0] = b'!';
  /// assert_eq!(&a[..], b"jello");
  /// assert_eq!(&b[..], b"!world");
  /// ```
  ///
  /// ## Panics
  ///
  /// Panics if `at > len`.
  #[must_use = "consider BytesMut::truncate if you don't need the other half"]
  pub fn split_off(&mut self, at: usize) -> Result<Self, Buffer> {
    match &mut self.0 {
      Repr::Inline(b) => Err(b.split_off(at)),
      Repr::Heap(b) => Ok(Self(Repr::Heap(b.split_off(at)))),
    }
  }

  /// Removes the bytes from the current view, returning them in a new buffer.
  ///
  /// Afterwards, `self` will be empty, but will retain any additional
  /// capacity that it had before the operation. This is identical to
  /// `self.split_to(self.len())`.
  ///
  /// For heap buffers, this is an `O(1)` operation.
  /// For inline buffers, the data is copied into a `Buffer`.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::{BytesMut, BufMut};
  ///
  /// let mut buf = BytesMut::with_capacity(1024);
  /// buf.put(&b"hello world"[..]);
  /// let other = buf.split().unwrap();
  /// assert!(buf.is_empty());
  /// assert_eq!(other, b"hello world"[..]);
  /// ```
  #[must_use = "consider BytesMut::clear if you don't need the other half"]
  pub fn split(&mut self) -> Result<Self, Buffer> {
    let len = self.len();
    self.split_to(len)
  }

  /// Splits the buffer into two at the given index.
  ///
  /// Afterwards `self` contains elements `[at, len)`, and the returned value
  /// contains elements `[0, at)`.
  ///
  /// For heap-allocated buffers, this is an `O(1)` operation that increases the
  /// reference count and returns `Ok(BytesMut)`.
  ///
  /// For inline buffers, the head is copied into a `Buffer` and returned as `Err(Buffer)`.
  /// Both `BytesMut` and `Buffer` are mutable, but `Buffer` is limited to 62 bytes inline storage.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::BytesMut;
  ///
  /// // Inline buffer
  /// let mut a = BytesMut::from(&b"hello world"[..]);
  /// match a.split_to(5) {
  ///     Ok(mut b) => {
  ///         // Heap: BytesMut can grow beyond 62 bytes
  ///         b[0] = b'j';
  ///         assert_eq!(&b[..], b"jello");
  ///     }
  ///     Err(b) => {
  ///         // Inline: Buffer is limited to 62 bytes but still mutable
  ///         assert_eq!(&b[..], b"hello");
  ///     }
  /// }
  /// assert_eq!(&a[..], b" world");
  ///
  /// // Heap buffer
  /// let mut a = BytesMut::with_capacity(64);
  /// a.extend_from_slice(b"hello world");
  /// let mut b = a.split_to(5).unwrap();
  /// a[0] = b'!';  // Replaces the space with '!'
  /// b[0] = b'j';
  /// assert_eq!(&a[..], b"!world");
  /// assert_eq!(&b[..], b"jello");
  /// ```
  ///
  /// ## Panics
  ///
  /// Panics if `at > len`.
  #[must_use = "consider BytesMut::advance if you don't need the other half"]
  pub fn split_to(&mut self, at: usize) -> Result<Self, Buffer> {
    match &mut self.0 {
      Repr::Inline(b) => Err(b.split_to(at)),
      Repr::Heap(b) => Ok(Self(Repr::Heap(b.split_to(at)))),
    }
  }

  /// Absorbs a `BytesMut` that was previously split off.
  ///
  /// Both `BytesMut` objects must be heap allocated for this to succeed. If one of them
  /// is inline, the method returns `Some(other)`, leaving `self` unchanged.
  ///
  /// If the two `BytesMut` objects were previously contiguous and not mutated
  /// in a way that causes re-allocation i.e., if `other` was created by
  /// calling `split_off` on this `BytesMut`, then this is an `O(1)` operation
  /// that just decreases a reference count and sets a few indices.
  /// Otherwise this method degenerates to
  /// `self.extend_from_slice(other.as_ref())`.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::BytesMut;
  ///
  /// let mut buf = BytesMut::with_capacity(64);
  /// buf.extend_from_slice(b"aaabbbcccddd");
  ///
  /// let split = buf.split_off(6).unwrap();
  /// assert_eq!(b"aaabbb", &buf[..]);
  /// assert_eq!(b"cccddd", &split[..]);
  ///
  /// assert!(buf.unsplit(split).is_none());
  /// assert_eq!(b"aaabbbcccddd", &buf[..]);
  /// ```
  pub fn unsplit(&mut self, other: Self) -> Option<Self> {
    match (&mut self.0, other.0) {
      (Repr::Heap(b1), Repr::Heap(b2)) => {
        b1.unsplit(b2);
        None
      }
      (_, Repr::Inline(storage)) => Some(Self(Repr::Inline(storage))),
      (_, Repr::Heap(bytes_mut)) => Some(Self(Repr::Heap(bytes_mut))),
    }
  }

  #[inline]
  pub(crate) fn freeze<S>(self) -> RawBytes<S>
  where
    RawBytes<S>: crate::strategy::Strategy,
  {
    match self.0 {
      Repr::Inline(storage) => RawBytes::inline(storage),
      Repr::Heap(b) => RawBytes::heap(b.freeze()),
    }
  }

  /// Converts `self` into an immutable [`shared::Bytes`](crate::shared::Bytes).
  ///
  /// The conversion is zero cost and is used to indicate that the slice
  /// referenced by the handle will no longer be mutated. Once the conversion
  /// is done, the handle can be cloned and shared across threads.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::{BytesMut, BufMut};
  /// use std::thread;
  ///
  /// let mut b = BytesMut::with_capacity(64);
  /// b.put(&b"hello world"[..]);
  /// let b1 = b.freeze_shared();
  /// let b2 = b1.clone();
  ///
  /// let th = thread::spawn(move || {
  ///     assert_eq!(&b1[..], b"hello world");
  /// });
  ///
  /// assert_eq!(&b2[..], b"hello world");
  /// th.join().unwrap();
  /// ```
  pub fn freeze_shared(self) -> crate::shared::Bytes {
    self.freeze()
  }

  /// Converts `self` into an immutable [`compact::Bytes`](crate::compact::Bytes).
  ///
  /// The conversion is zero cost and is used to indicate that the slice
  /// referenced by the handle will no longer be mutated. Once the conversion
  /// is done, the handle can be cloned and shared across threads.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::{BytesMut, BufMut};
  /// use std::thread;
  ///
  /// let mut b = BytesMut::with_capacity(64);
  /// b.put(&b"hello world"[..]);
  /// let b1 = b.freeze_compact();
  /// let b2 = b1.clone();
  ///
  /// let th = thread::spawn(move || {
  ///     assert_eq!(&b1[..], b"hello world");
  /// });
  ///
  /// assert_eq!(&b2[..], b"hello world");
  /// th.join().unwrap();
  /// ```
  pub fn freeze_compact(self) -> crate::compact::Bytes {
    self.freeze()
  }

  /// Returns the remaining spare capacity of the buffer as a slice of [`MaybeUninit<u8>`].
  ///
  /// The returned slice can be used to fill the buffer with data (e.g. by reading from a file) before marking the data as initialized using the [`set_len`](Self::set_len) method.
  ///
  /// ## Example
  ///
  /// ```
  /// use smol_bytes::{BytesMut, INLINE_CAP};
  ///
  /// // Allocate buffer big enough for 10 bytes.
  /// let mut buf = BytesMut::with_capacity(10);
  ///
  /// // Fill in the first 3 elements.
  /// let uninit = buf.spare_capacity_mut();
  /// uninit[0].write(0);
  /// uninit[1].write(1);
  /// uninit[2].write(2);
  ///
  /// // Mark the first 3 bytes of the buffer as being initialized.
  /// unsafe {
  ///   buf.set_len(3);
  /// }
  ///
  /// assert_eq!(&buf[..], &[0, 1, 2]);
  /// ```
  pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
    match &mut self.0 {
      Repr::Inline(b) => b.spare_capacity_mut(),
      Repr::Heap(b) => b.spare_capacity_mut(),
    }
  }

  /// Attempts to cheaply reclaim already allocated capacity for at least `additional` more
  /// bytes to be inserted into the given `BytesMut` and returns `true` if it succeeded.
  ///
  /// `try_reclaim` behaves exactly like `reserve`, except that it never allocates new storage
  /// and returns a `bool` indicating whether it was successful in doing so:
  ///
  /// `try_reclaim` returns false under these conditions:
  ///  - The spare capacity left is less than `additional` bytes AND
  ///  - The existing allocation cannot be reclaimed cheaply or it was less than
  ///    `additional` bytes in size
  ///
  /// Reclaiming the allocation cheaply is possible if the `BytesMut` has no outstanding
  /// references through other `BytesMut`s or `Bytes` which point to the same underlying
  /// storage.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::BytesMut;
  ///
  /// let mut buf = BytesMut::with_capacity(64);
  /// assert_eq!(true, buf.try_reclaim(64));
  /// assert_eq!(64, buf.capacity());
  ///
  /// buf.extend_from_slice(b"abcd");
  /// let mut split = buf.split().unwrap();
  /// assert_eq!(60, buf.capacity());
  /// assert_eq!(4, split.len());
  /// assert_eq!(false, split.try_reclaim(64));
  /// assert_eq!(false, buf.try_reclaim(64));
  /// // The split buffer is filled with "abcd"
  /// assert_eq!(false, split.try_reclaim(4));
  /// // buf is empty and has capacity for 60 bytes
  /// assert_eq!(true, buf.try_reclaim(60));
  ///
  /// drop(buf);
  /// assert_eq!(false, split.try_reclaim(64));
  ///
  /// split.clear();
  /// assert_eq!(4, split.capacity());
  /// assert_eq!(true, split.try_reclaim(64));
  /// assert_eq!(64, split.capacity());
  /// ```
  #[inline]
  #[must_use = "consider BytesMut::reserve if you need an infallible reservation"]
  pub fn try_reclaim(&mut self, additional: usize) -> bool {
    match &mut self.0 {
      Repr::Inline(b) => {
        let rem = b.remaining_mut();

        if additional <= rem {
          // The handle can already store at least `additional` more bytes, so
          // there is no further work needed to be done.
          return true;
        }
        false
      }
      Repr::Heap(b) => b.try_reclaim(additional),
    }
  }

  /// Sets the length of the buffer.
  ///
  /// This will explicitly set the size of the buffer without actually
  /// modifying the data, so it is up to the caller to ensure that the data
  /// has been initialized.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::BytesMut;
  ///
  /// let mut b = BytesMut::from(&b"hello world"[..]);
  ///
  /// unsafe {
  ///     b.set_len(5);
  /// }
  ///
  /// assert_eq!(&b[..], b"hello");
  ///
  /// unsafe {
  ///     b.set_len(11);
  /// }
  ///
  /// assert_eq!(&b[..], b"hello world");
  /// ```
  #[allow(clippy::missing_safety_doc)]
  #[inline]
  pub unsafe fn set_len(&mut self, len: usize) {
    match &mut self.0 {
      Repr::Inline(b) => b.set_len(len),
      Repr::Heap(b) => b.set_len(len),
    }
  }

  /// Resizes the buffer so that `len` is equal to `new_len`.
  ///
  /// If `new_len` is greater than `len`, the buffer is extended by the
  /// difference with each additional byte set to `value`. If `new_len` is
  /// less than `len`, the buffer is simply truncated.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::BytesMut;
  ///
  /// let mut buf = BytesMut::new();
  ///
  /// buf.resize(3, 0x1);
  /// assert_eq!(&buf[..], &[0x1, 0x1, 0x1]);
  ///
  /// buf.resize(2, 0x2);
  /// assert_eq!(&buf[..], &[0x1, 0x1]);
  ///
  /// buf.resize(4, 0x3);
  /// assert_eq!(&buf[..], &[0x1, 0x1, 0x3, 0x3]);
  /// ```
  pub fn resize(&mut self, new_len: usize, value: u8) {
    let additional = if let Some(additional) = new_len.checked_sub(self.len()) {
      additional
    } else {
      self.truncate(new_len);
      return;
    };

    if additional == 0 {
      return;
    }

    match &mut self.0 {
      Repr::Inline(storage) => {
        let rem = storage.remaining_mut();

        if additional <= rem {
          // The handle can already store at least `additional` more bytes, so
          // fill them with the value.
          let dst = storage.spare_capacity_mut().as_mut_ptr();

          // SAFETY: `spare_capacity_mut` returns a valid, properly aligned pointer and we've
          // verified there's enough space to write `additional` bytes. There are at least
          // `new_len` initialized bytes in the buffer so no uninitialized bytes are being exposed.
          unsafe {
            core::ptr::write_bytes(dst, value, additional);
            storage.set_len(new_len);
          }
          return;
        }
        let mut new_buf = bytes::BytesMut::with_capacity(storage.len() + additional);
        new_buf.put_slice(storage.as_slice());
        let dst = new_buf.spare_capacity_mut().as_mut_ptr();

        // SAFETY: `spare_capacity_mut` returns a valid, properly aligned pointer and we've
        // reserved enough space to write `additional` bytes.
        unsafe { core::ptr::write_bytes(dst, value, additional) };

        // SAFETY: There are at least `new_len` initialized bytes in the buffer so no
        // uninitialized bytes are being exposed.
        unsafe { new_buf.set_len(new_len) };

        self.0 = Repr::Heap(new_buf);
      }
      Repr::Heap(b) => b.resize(new_len, value),
    }
  }

  /// Reserves capacity for at least `additional` more bytes to be inserted
  /// into the given `BytesMut`.
  ///
  /// More than `additional` bytes may be reserved in order to avoid frequent
  /// reallocations. A call to `reserve` may result in an allocation.
  ///
  /// Before allocating new buffer space, the function will attempt to reclaim
  /// space in the existing buffer. If the current handle references a view
  /// into a larger original buffer, and all other handles referencing part
  /// of the same original buffer have been dropped, then the current view
  /// can be copied/shifted to the front of the buffer and the handle can take
  /// ownership of the full buffer, provided that the full buffer is large
  /// enough to fit the requested additional capacity.
  ///
  /// This optimization will only happen if shifting the data from the current
  /// view to the front of the buffer is not too expensive in terms of the
  /// (amortized) time required. The precise condition is subject to change;
  /// as of now, the length of the data being shifted needs to be at least as
  /// large as the distance that it's shifted by. If the current view is empty
  /// and the original buffer is large enough to fit the requested additional
  /// capacity, then reallocations will never happen.
  ///
  /// ## Examples
  ///
  /// In the following example, a new buffer is allocated.
  ///
  /// ```
  /// use smol_bytes::BytesMut;
  ///
  /// let mut buf = BytesMut::from(&b"hello"[..]);
  /// buf.reserve(64);
  /// assert!(buf.capacity() >= 69);
  /// ```
  ///
  /// ## Panics
  ///
  /// Panics if the new capacity overflows `usize`.
  #[inline]
  pub fn reserve(&mut self, additional: usize) {
    match &mut self.0 {
      Repr::Inline(storage) => {
        let rem = storage.remaining_mut();

        if additional <= rem {
          // The handle can already store at least `additional` more bytes, so
          // there is no further work needed to be done.
          return;
        }
        let mut new_buf = bytes::BytesMut::with_capacity(storage.len() + additional);
        new_buf.extend_from_slice(storage.as_slice());
        self.0 = Repr::Heap(new_buf);
      }
      Repr::Heap(b) => b.reserve(additional),
    }
  }

  /// Returns the number of bytes contained in this `BytesMut`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::BytesMut;
  ///
  /// let bytes = BytesMut::new();
  /// assert_eq!(bytes.len(), 0);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn len(&self) -> usize {
    match &self.0 {
      Repr::Inline(b) => b.len(),
      Repr::Heap(b) => b.len(),
    }
  }

  /// Returns `true` if the BytesMut has a length of `0`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::BytesMut;
  ///
  /// let bytes = BytesMut::new();
  /// assert!(bytes.is_empty());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Returns a slice of the buffer's contents.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::BytesMut;
  ///
  /// let buf = BytesMut::from(&b"hello"[..]);
  /// assert_eq!(buf.as_slice(), b"hello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_slice(&self) -> &[u8] {
    self
  }

  /// Returns a mutable slice of the buffer's contents.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::BytesMut;
  ///
  /// let mut buf = BytesMut::from(&b"hello"[..]);
  /// buf.as_mut_slice()[0] = b'j';
  /// assert_eq!(buf.as_slice(), b"jello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_mut_slice(&mut self) -> &mut [u8] {
    self
  }
}

impl Buf for BytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn remaining(&self) -> usize {
    match &self.0 {
      Repr::Inline(b) => b.remaining(),
      Repr::Heap(b) => b.remaining(),
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn chunk(&self) -> &[u8] {
    match &self.0 {
      Repr::Inline(b) => b.as_slice(),
      Repr::Heap(b) => b.chunk(),
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn advance(&mut self, cnt: usize) {
    match &mut self.0 {
      Repr::Inline(b) => b.advance(cnt),
      Repr::Heap(b) => b.advance(cnt),
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes {
    match self.split_to(len) {
      Ok(a) => a.freeze_shared().into(),
      Err(b) => ::bytes::Bytes::copy_from_slice(b.as_slice()),
    }
  }
}

unsafe impl BufMut for BytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn remaining_mut(&self) -> usize {
    usize::MAX - self.len()
  }

  unsafe fn advance_mut(&mut self, cnt: usize) {
    match &mut self.0 {
      Repr::Inline(b) => b.advance_mut(cnt),
      Repr::Heap(b) => b.advance_mut(cnt),
    }
  }

  fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice {
    match &mut self.0 {
      Repr::Inline(b) => b.chunk_mut(),
      Repr::Heap(b) => b.chunk_mut(),
    }
  }

  fn put_slice(&mut self, src: &[u8]) {
    self.extend_from_slice(src);
  }

  fn put_bytes(&mut self, val: u8, cnt: usize) {
    self.reserve(cnt);

    unsafe {
      let dst = self.spare_capacity_mut();
      // Reserved above
      debug_assert!(dst.len() >= cnt);

      core::ptr::write_bytes(dst.as_mut_ptr(), val, cnt);

      self.advance_mut(cnt);
    }
  }
}

#[derive(Clone)]
enum Repr {
  Inline(Buffer),
  Heap(bytes::BytesMut),
}
