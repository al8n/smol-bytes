use core::{
  borrow::Borrow,
  mem::{transmute, MaybeUninit},
  ops::{Bound, RangeBounds},
  ptr::{copy_nonoverlapping, write_bytes},
  slice::{from_raw_parts, from_raw_parts_mut},
};

use super::error::*;

mod cmp;
mod fmt;
mod from;
mod io;
mod iter;
mod ops;

#[cfg(feature = "arbitrary")]
mod arbitrary;
#[cfg(feature = "borsh")]
mod borsh;
#[cfg(all(feature = "quickcheck", any(feature = "std", feature = "alloc")))]
mod quickcheck;
#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "pyo3")]
mod python;

#[cfg(any(feature = "alloc", feature = "std"))]
pub use bytes::TryGetError;

/// Number of bytes that can be stored inline.
pub const INLINE_CAP: usize = InlineSize::MAX as usize;

/// A type used internally to encode inline lengths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum InlineSize {
  _V0 = 0,
  _V1,
  _V2,
  _V3,
  _V4,
  _V5,
  _V6,
  _V7,
  _V8,
  _V9,
  _V10,
  _V11,
  _V12,
  _V13,
  _V14,
  _V15,
  _V16,
  _V17,
  _V18,
  _V19,
  _V20,
  _V21,
  _V22,
  _V23,
  _V24,
  _V25,
  _V26,
  _V27,
  _V28,
  _V29,
  _V30,
  _V31,
  _V32,
  _V33,
  _V34,
  _V35,
  _V36,
  _V37,
  _V38,
  _V39,
  _V40,
  _V41,
  _V42,
  _V43,
  _V44,
  _V45,
  _V46,
  _V47,
  _V48,
  _V49,
  _V50,
  _V51,
  _V52,
  _V53,
  _V54,
  _V55,
  _V56,
  _V57,
  _V58,
  _V59,
  _V60,
  _V61,
  _V62,
}

impl core::ops::Sub for InlineSize {
  type Output = Self;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn sub(self, rhs: Self) -> Self::Output {
    // Safety: subtraction result is guaranteed to be less than or equal to INLINE_CAP
    unsafe { InlineSize::from_u8(self.to_u8() - rhs.to_u8()) }
  }
}

impl InlineSize {
  const MAX: u8 = InlineSize::_V62 as u8;

  /// ## Safety
  ///
  /// `value` must be less than or equal to [`INLINE_CAP`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const unsafe fn from_u8(value: u8) -> Self {
    debug_assert!(value <= InlineSize::MAX);
    transmute::<u8, InlineSize>(value)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn to_u8(self) -> u8 {
    self as u8
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn to_usize(self) -> usize {
    self as usize
  }
}

/// A fixed-size buffer for inline storage.
///
/// This type can hold at most `62` bytes on stack without heap allocation.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "pyo3", ::pyo3::prelude::pyclass)]
pub struct Buffer {
  // The write cursor
  len: InlineSize,
  // The read cursor
  cur: InlineSize,
  buf: [MaybeUninit<u8>; INLINE_CAP],
}

impl Default for Buffer {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new()
  }
}

impl Buffer {
  /// Creates a new, empty `Buffer`.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::Buffer;
  ///
  /// let buf = Buffer::new();
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self {
      len: InlineSize::_V0,
      cur: InlineSize::_V0,
      buf: [const { MaybeUninit::uninit() }; INLINE_CAP],
    }
  }

  /// Creates a new `Buffer` with the specified length, filled with zeroes.
  ///
  /// ## Safety
  /// - `len` must be less than or equal to [`INLINE_CAP`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const unsafe fn zeroed(len: usize) -> Self {
    let mut storage = [const { MaybeUninit::uninit() }; INLINE_CAP];
    core::ptr::write_bytes(storage.as_mut_ptr(), 0, len);
    Self {
      cur: InlineSize::_V0,
      // SAFETY: len is guaranteed to be less than or equal to INLINE_CAP
      len: unsafe { InlineSize::from_u8(len as u8) },
      buf: storage,
    }
  }

  /// Creates a new `Buffer` from the given array and length.
  ///
  /// ## Safety
  /// - `len` must be less than or equal to [`INLINE_CAP`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(unused)]
  pub(crate) const unsafe fn from_array(buf: [u8; INLINE_CAP], len: usize) -> Self {
    Self {
      cur: InlineSize::_V0,
      // SAFETY: len is guaranteed to be less than or equal to INLINE_CAP
      len: unsafe { InlineSize::from_u8(len as u8) },
      // SAFETY: all bytes are initialized
      buf: unsafe { transmute::<[u8; INLINE_CAP], [MaybeUninit<u8>; INLINE_CAP]>(buf) },
    }
  }

  /// Creates a new `Buffer` by copying from the given slice.
  ///
  /// ## Safety
  /// - the length of `src` must be less than or equal to [`INLINE_CAP`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const unsafe fn copy_from_slice(src: &[u8]) -> Self {
    let len = src.len();
    let mut storage = [const { MaybeUninit::uninit() }; INLINE_CAP];

    // SAFETY: caller guarantees that `len` is less than or equal to `INLINE_CAP`.
    copy_nonoverlapping(src.as_ptr(), storage.as_mut_ptr() as _, len);

    Self {
      // SAFETY: caller guarantees that `len` is less than or equal to `INLINE_CAP`.
      len: InlineSize::from_u8(len as u8),
      cur: InlineSize::_V0,
      buf: storage,
    }
  }

  /// Returns the number of bytes between the current position and the end of
  /// the buffer.
  ///
  /// This value is equal to the length of the slice returned
  /// by [`as_slice()`](Self::as_slice).
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Buffer;
  ///
  /// let mut buf = Buffer::try_from(*b"hello world").unwrap();
  ///
  /// assert_eq!(buf.remaining(), 11);
  ///
  /// buf.get_u8();
  ///
  /// assert_eq!(buf.remaining(), 10);
  /// ```
  ///
  /// # Implementer notes
  ///
  /// Implementations of `remaining` should ensure that the return value does
  /// not change unless a call is made to `advance` or any other function that
  /// is documented to change the `Buf`'s current position.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn remaining(&self) -> usize {
    self.len.to_usize() - self.cur as usize
  }

  /// Returns the number of bytes contained in this `BytesMut`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::Buffer;
  ///
  /// let bytes = Buffer::new();
  /// assert_eq!(bytes.len(), 0);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn len(&self) -> usize {
    self.len.to_usize()
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
  pub const fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Returns the number of bytes that can be written from the current
  /// position until the end of the buffer is reached.
  ///
  /// This value is equal to the length of the slice returned
  /// by `chunk_mut()`.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::{Buffer, BufMut};
  ///
  /// let mut dst = Buffer::new();
  ///
  /// let original_remaining = dst.remaining_mut();
  /// dst.put(&b"hello"[..]);
  ///
  /// assert_eq!(original_remaining - 5, dst.remaining_mut());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn remaining_mut(&self) -> usize {
    INLINE_CAP - (self.len.to_usize())
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
  /// use smol_bytes::Buffer;
  ///
  /// let mut b = Buffer::try_from(&b"hello world"[..]).unwrap();
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
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const unsafe fn set_len(&mut self, len: usize) {
    debug_assert!(len <= INLINE_CAP, "set_len out of bounds");
    self.len = unsafe { InlineSize::from_u8(len as u8) };
  }

  /// Advance the internal cursor of the `Buffer`
  ///
  /// The next call to `as_slice()` will return a slice starting `cnt` bytes
  /// further into the underlying buffer.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::{Buffer, INLINE_CAP};
  ///
  /// let mut buf = Buffer::try_from(&b"hello world"[..]).unwrap();
  ///
  /// assert_eq!(buf.as_slice(), &b"hello world"[..]);
  ///
  /// buf.advance(6);
  ///
  /// assert_eq!(buf.as_slice(), &b"world"[..]);
  ///
  /// // advancing will also reduce capacity
  /// assert_eq!(buf.capacity(), INLINE_CAP - 6);
  /// ```
  ///
  /// ## Panics
  ///
  /// This function panics if `cnt > self.remaining()`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn advance(&mut self, requested: usize) {
    if let Err(err) = self.try_advance(requested) {
      panic_advance(err.available, err.requested)
    }
  }

  /// Tries to advance the internal cursor of the `Buffer`.
  ///
  /// Returns `Err(OutOfBounds)` if `requested` exceeds the remaining length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn try_advance(&mut self, requested: usize) -> Result<(), OutOfBounds> {
    if requested == 0 {
      return Ok(());
    }

    let available = (self.len - self.cur).to_usize();
    if available < requested {
      return Err(OutOfBounds::new(requested, available));
    }
    self.cur = unsafe { InlineSize::from_u8(self.cur.to_u8() + requested as u8) };
    Ok(())
  }

  /// Advance the internal write cursor of the `Buffer`
  ///
  /// The next call to [`spare_capacity_mut`](Self::spare_capacity_mut) will return a slice starting `cnt` bytes
  /// further into the underlying buffer.
  ///
  /// ## Safety
  ///
  /// The caller must ensure that the next `cnt` bytes of `chunk` are
  /// initialized.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::Buffer;
  ///
  /// let mut buf = Buffer::new();
  ///
  /// // Write some data
  /// unsafe {
  ///   let tmp = buf.spare_capacity_mut();
  ///   core::ptr::copy(b"he".as_ptr(), tmp.as_mut_ptr() as _, 2);
  ///   buf.advance_mut(2);
  /// }
  ///
  /// // write more bytes
  /// unsafe {
  ///   let tmp = buf.spare_capacity_mut();
  ///   core::ptr::copy(b"llo".as_ptr(), tmp.as_mut_ptr() as _, 3);
  ///   buf.advance_mut(3);
  /// }
  ///
  /// assert_eq!(5, buf.len());
  /// assert_eq!(buf, "hello".as_bytes());
  /// ```
  ///
  /// ## Panics
  ///
  /// This function panic if `requested > self.remaining_mut()`.
  pub unsafe fn advance_mut(&mut self, requested: usize) {
    let available = self.remaining_mut();
    if requested > available {
      panic_advance(available, requested)
    }

    self.len = unsafe { InlineSize::from_u8(self.len.to_u8() + requested as u8) };
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
  /// use smol_bytes::Buffer;
  ///
  /// let mut bytes = Buffer::try_from(&b"hello world"[..]).unwrap();
  ///
  /// bytes.truncate(5);
  /// assert_eq!(bytes.as_mut_slice(), b"hello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn truncate(&mut self, new_len: usize) {
    if new_len == 0 {
      return self.clear();
    }

    let remaining = self.remaining();
    if new_len >= remaining {
      return;
    }

    let cur = self.cur as usize;
    if cur == 0 {
      self.len = unsafe { InlineSize::from_u8(new_len as u8) };
      return;
    }

    let (head, tail) = self.buf.split_at_mut(cur);
    unsafe {
      // SAFETY: new_len < remaining, so tail[new_len..] is valid
      copy_nonoverlapping(tail.as_ptr(), head.as_mut_ptr(), new_len);
    }
    self.len = unsafe { InlineSize::from_u8(new_len as u8) };
    self.cur = InlineSize::_V0;
  }

  /// Splits the buffer into two at the given index.
  ///
  /// Afterwards `self` contains elements `[0, at)`, and the returned `Buffer`
  /// contains elements `[at, len)`.
  ///
  /// This operation copies the tail into a new `Buffer`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::Buffer;
  ///
  /// let mut a = Buffer::try_from(&b"hello world"[..]).unwrap();
  /// let b = a.split_off(5);
  /// assert_eq!(a.as_slice(), b"hello");
  /// assert_eq!(b.as_slice(), b" world");
  /// ```
  ///
  /// ## Panics
  ///
  /// Panics if `at > len`.
  #[must_use = "consider Buffer::truncate if you don't need the other half"]
  pub fn split_off(&mut self, at: usize) -> Self {
    self
      .try_split_off(at)
      .unwrap_or_else(|_| panic!("split_off out of bounds: {} > {}", at, self.remaining()))
  }

  /// Splits the buffer into two at the given index.
  ///
  /// Afterwards `self` contains elements `[at, len)`, and the returned `Buffer`
  /// contains elements `[0, at)`.
  ///
  /// This operation copies the head into a new `Buffer`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::Buffer;
  ///
  /// let mut a = Buffer::try_from(&b"hello world"[..]).unwrap();
  /// let b = a.split_to(5);
  /// assert_eq!(b.as_slice(), b"hello");
  /// assert_eq!(a.as_slice(), b" world");
  /// ```
  ///
  /// ## Panics
  ///
  /// Panics if `at > len`.
  #[must_use = "consider Buffer::advance if you don't need the other half"]
  pub fn split_to(&mut self, at: usize) -> Self {
    self
      .try_split_to(at)
      .unwrap_or_else(|_| panic!("split_to out of bounds: {} > {}", at, self.remaining()))
  }

  /// Tries to split the buffer into two at the given index.
  ///
  /// Afterwards `self` contains elements `[0, at)`, and the returned `Buffer`
  /// contains elements `[at, len)`.
  ///
  /// Returns `Err(OutOfBounds)` if `at > remaining()`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::Buffer;
  ///
  /// let mut a = Buffer::try_from(&b"hello world"[..]).unwrap();
  /// let b = a.try_split_off(5).unwrap();
  /// assert_eq!(a.as_slice(), b"hello");
  /// assert_eq!(b.as_slice(), b" world");
  /// ```
  #[must_use = "consider Buffer::truncate if you don't need the other half"]
  pub const fn try_split_off(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    let len = self.remaining();
    if at > len {
      return Err(OutOfBounds::new(at, len));
    }

    let tail_len = len - at;
    let tail = unsafe {
      let mut new_buf = Self::new();
      if tail_len > 0 {
        let src = self.as_slice().as_ptr().add(at);
        copy_nonoverlapping(src, new_buf.buf.as_mut_ptr() as *mut u8, tail_len);
        new_buf.len = InlineSize::from_u8(tail_len as u8);
      }
      new_buf
    };

    self.truncate(at);
    Ok(tail)
  }

  /// Tries to split the buffer into two at the given index.
  ///
  /// Afterwards `self` contains elements `[at, len)`, and the returned `Buffer`
  /// contains elements `[0, at)`.
  ///
  /// Returns `Err(OutOfBounds)` if `at > remaining()`.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::Buffer;
  ///
  /// let mut a = Buffer::try_from(&b"hello world"[..]).unwrap();
  /// let b = a.try_split_to(5).unwrap();
  /// assert_eq!(b.as_slice(), b"hello");
  /// assert_eq!(a.as_slice(), b" world");
  /// ```
  #[must_use = "consider Buffer::advance if you don't need the other half"]
  pub const fn try_split_to(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    let len = self.remaining();
    if at > len {
      return Err(OutOfBounds::new(at, len));
    }

    let head = unsafe {
      let mut new_buf = Self::new();
      if at > 0 {
        let src = self.as_slice().as_ptr();
        copy_nonoverlapping(src, new_buf.buf.as_mut_ptr() as *mut u8, at);
        new_buf.len = InlineSize::from_u8(at as u8);
      }
      new_buf
    };

    self.cur = unsafe { InlineSize::from_u8(self.cur.to_u8() + at as u8) };
    Ok(head)
  }

  /// Creates a new buffer containing a copy of the specified range.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::Buffer;
  ///
  /// let buf = Buffer::try_from(&b"hello world"[..]).unwrap();
  /// let slice = buf.slice(0..5);
  /// assert_eq!(slice.as_slice(), b"hello");
  /// ```
  ///
  /// ## Panics
  ///
  /// Panics if the range is out of bounds.
  pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
    self.try_slice(range).unwrap_or_else(|e| panic!("{e}"))
  }

  /// Tries to create a new buffer containing a copy of the specified range.
  ///
  /// Returns `Err(OutOfBounds)` if the range is out of bounds or the slice is too large.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::Buffer;
  ///
  /// let buf = Buffer::try_from(&b"hello world"[..]).unwrap();
  /// let slice = buf.try_slice(0..5).unwrap();
  /// assert_eq!(slice.as_slice(), b"hello");
  /// ```
  pub fn try_slice(&self, range: impl RangeBounds<usize>) -> Result<Self, RangeOutOfBounds> {
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

    if begin > len || end > len {
      return Err(RangeOutOfBounds::new(begin, end, len));
    }

    if begin == end {
      return Ok(Self::new());
    }

    Ok(unsafe {
      let ptr = self.as_slice().as_ptr();
      let slice = from_raw_parts(ptr.add(begin), end - begin);
      Self::copy_from_slice(slice)
    })
  }

  /// Resizes the buffer to the specified length, filling with zeros if expanding.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::Buffer;
  ///
  /// let mut buf = Buffer::try_from(&b"hello"[..]).unwrap();
  /// buf.resize(8);
  /// assert_eq!(buf.as_slice(), b"hello\0\0\0");
  ///
  /// buf.resize(3);
  /// assert_eq!(buf.as_slice(), b"hel");
  /// ```
  ///
  /// ## Panics
  ///
  /// Panics if the new length exceeds capacity.
  pub fn resize(&mut self, new_len: usize) {
    let current_len = self.remaining();

    if new_len == current_len {
      return;
    }

    if new_len < current_len {
      self.truncate(new_len);
      return;
    }

    // Expanding
    let additional = new_len - current_len;
    assert!(
      self.remaining_mut() >= additional,
      "resize exceeds capacity: {} + {} > {}",
      current_len,
      additional,
      self.capacity()
    );

    // Fill with zeros
    unsafe {
      let ptr = self.buf.as_mut_ptr() as *mut u8;
      let start = self.len.to_usize();
      core::ptr::write_bytes(ptr.add(start), 0, additional);
      self.len = InlineSize::from_u8(new_len as u8);
    }
  }

  /// Tries to resize the buffer to the specified length, filling with zeros if expanding.
  ///
  /// Returns `Err(OutOfBounds)` if the new length exceeds capacity.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::Buffer;
  ///
  /// let mut buf = Buffer::try_from(&b"hello"[..]).unwrap();
  /// assert!(buf.try_resize(8).is_ok());
  /// assert_eq!(buf.as_slice(), b"hello\0\0\0");
  ///
  /// assert!(buf.try_resize(3).is_ok());
  /// assert_eq!(buf.as_slice(), b"hel");
  /// ```
  pub const fn try_resize(&mut self, new_len: usize) -> Result<(), OutOfBounds> {
    let current_len = self.remaining();

    if new_len == current_len {
      return Ok(());
    }

    if new_len < current_len {
      self.truncate(new_len);
      return Ok(());
    }

    // Expanding
    let additional = new_len - current_len;
    if self.remaining_mut() < additional {
      return Err(OutOfBounds::new(new_len, self.capacity()));
    }

    // Fill with zeros
    unsafe {
      let ptr = self.buf.as_mut_ptr() as *mut u8;
      let start = self.len.to_usize();
      core::ptr::write_bytes(ptr.add(start), 0, additional);
      self.len = InlineSize::from_u8(new_len as u8);
    }
    Ok(())
  }

  /// Clears the buffer, removing all data. Existing capacity is preserved.
  ///
  /// ## Example
  ///
  /// ```rust
  /// use smol_bytes::Buffer;
  ///
  /// let mut bytes = Buffer::try_from(&b"hello world"[..]).unwrap();
  /// bytes.clear();
  /// assert_eq!(bytes.len(), 0);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear(&mut self) {
    self.len = InlineSize::_V0;
    self.cur = InlineSize::_V0;
  }

  /// Returns the remaining spare capacity of the buffer as a slice of [`MaybeUninit<u8>`].
  ///
  /// The returned slice can be used to fill the buffer with data (e.g. by reading from a file) before marking the data as initialized using the [`set_len`](Self::set_len) method.
  ///
  /// ## Example
  ///
  /// ```
  /// use smol_bytes::{Buffer, INLINE_CAP};
  ///
  /// let mut buf = Buffer::new();
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
  /// assert_eq!(buf.as_slice(), &[0, 1, 2]);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
    let len = self.len.to_usize();
    unsafe { from_raw_parts_mut(self.buf.as_mut_ptr().add(len), INLINE_CAP - len) }
  }

  /// Returns the capacity of the buffer.
  ///
  /// The capacity is not always equal to [`INLINE_CAP`], as the [`advance`](Self::advance) method
  /// may move the underlying cursor forward, reducing the available capacity.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn capacity(&self) -> usize {
    INLINE_CAP - self.cur.to_usize()
  }

  /// Returns the initialized portion of the buffer as a slice.
  ///
  /// This will include all bytes from the start of the buffer up to the current length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_slice(&self) -> &[u8] {
    let ptr = self.buf.as_ptr() as *const u8;
    let remaining = self.remaining();
    unsafe { core::slice::from_raw_parts(ptr.add(self.cur.to_usize()), remaining) }
  }

  /// Returns the mutable initialized portion of the buffer as a mutable slice.
  ///
  /// This will include all bytes from the start of the buffer up to the current length.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut_slice(&mut self) -> &mut [u8] {
    let ptr = self.buf.as_mut_ptr() as *mut u8;
    let remaining = Self::remaining(self);
    unsafe { core::slice::from_raw_parts_mut(ptr.add(self.cur.to_usize()), remaining) }
  }

  /// Transfer bytes into `self` from `src` and advance the cursor by the
  /// number of bytes written.
  ///
  /// `self` must have enough remaining capacity to contain all of `src`.
  ///
  /// ```
  /// use smol_bytes::{Buffer, INLINE_CAP};
  ///
  /// let mut dst = Buffer::new();
  ///
  /// {
  ///     dst.put_slice(b"hello");
  ///     assert_eq!(INLINE_CAP - 5, dst.remaining_mut());
  /// }
  ///
  /// assert_eq!(dst, "hello");
  /// ```
  #[inline]
  pub fn put_slice(&mut self, src: &[u8]) {
    self
      .try_put_slice(src)
      .unwrap_or_else(|e| panic_advance(e.available, e.requested))
  }

  /// Try to transfer bytes into `self` from `src` and advance the cursor by the
  /// number of bytes written.
  ///
  /// `self` must have enough remaining capacity to contain all of `src`.
  ///
  /// ```
  /// use smol_bytes::{Buffer, INLINE_CAP};
  ///
  /// let mut dst = Buffer::new();
  ///
  /// {
  ///     dst.try_put_slice(b"hello").unwrap();
  ///     assert_eq!(INLINE_CAP - 5, dst.remaining_mut());
  /// }
  ///
  /// assert_eq!("hello", &dst);
  /// ```
  #[inline]
  pub const fn try_put_slice(&mut self, src: &[u8]) -> Result<(), TryPutError> {
    let available = self.remaining_mut();
    let requested = src.len();

    if requested > available {
      return Err(TryPutError {
        requested,
        available,
      });
    }

    let slen = self.len.to_usize();
    unsafe {
      copy_nonoverlapping(
        src.as_ptr(),
        self.buf.as_mut_ptr().add(slen) as _,
        requested,
      );
    }
    self.len = unsafe { InlineSize::from_u8(slen as u8 + requested as u8) };
    Ok(())
  }

  /// Put `cnt` bytes `val` into `self`.
  ///
  /// Logically equivalent to calling `self.put_u8(val)` `cnt` times, but may work faster.
  ///
  /// `self` must have at least `cnt` remaining capacity.
  ///
  /// ```
  /// use smol_bytes::{Buffer, INLINE_CAP};
  ///
  /// let mut dst = Buffer::new();
  ///
  /// {
  ///     dst.put_bytes(b'a', 4);
  ///     assert_eq!(INLINE_CAP - 4, dst.remaining_mut());
  /// }
  ///
  /// assert_eq!("aaaa", &dst);
  /// ```
  ///
  /// ## Panics
  ///
  /// This function panics if there is not enough remaining capacity in
  /// `self`.
  #[inline]
  pub fn put_bytes(&mut self, val: u8, cnt: usize) {
    self
      .try_put_bytes(val, cnt)
      .unwrap_or_else(|e| panic_advance(e.available, e.requested))
  }

  /// Try to put `cnt` bytes `val` into `self`.
  ///
  /// Logically equivalent to calling `self.put_u8(val)` `cnt` times, but may work faster.
  ///
  /// `self` must have at least `cnt` remaining capacity.
  ///
  /// ```
  /// use smol_bytes::{Buffer, INLINE_CAP};
  ///
  /// let mut dst = Buffer::new();
  ///
  /// {
  ///     dst.try_put_bytes(b'a', 4).unwrap();
  ///     assert_eq!(INLINE_CAP - 4, dst.remaining_mut());
  /// }
  ///
  /// assert_eq!("aaaa".as_bytes(), &dst);
  /// ```
  ///
  /// ## Panics
  ///
  /// This function panics if there is not enough remaining capacity in
  /// `self`.
  #[inline]
  pub const fn try_put_bytes(&mut self, val: u8, cnt: usize) -> Result<(), TryPutError> {
    if cnt == 0 {
      return Ok(());
    }

    let available = self.remaining_mut();
    if available < cnt {
      return Err(TryPutError {
        requested: cnt,
        available,
      });
    }

    // Safety: we have already checked that there is enough capacity.
    unsafe {
      write_bytes(self.buf.as_mut_ptr().add(self.len.to_usize()), val, cnt);
    }
    self.len = unsafe { InlineSize::from_u8(self.len.to_u8() + cnt as u8) };
    Ok(())
  }

  /// Put `cnt` bytes `val` into `self`.
  ///
  /// Logically equivalent to calling `self.put_u8(val)` `cnt` times, but may work faster.
  ///
  /// `self` must have at least `cnt` remaining capacity.
  ///
  /// ```
  /// use smol_bytes::{Buffer, INLINE_CAP};
  ///
  /// let mut dst = Buffer::new();
  ///
  /// {
  ///     dst.put_u8(b'a');
  ///     assert_eq!(INLINE_CAP - 1, dst.remaining_mut());
  /// }
  ///
  /// assert_eq!("a", &dst);
  /// ```
  ///
  /// ## Panics
  ///
  /// This function panics if there is not enough remaining capacity in
  /// `self`.
  #[inline]
  pub fn put_u8(&mut self, val: u8) {
    self
      .try_put_u8(val)
      .unwrap_or_else(|e| panic_advance(e.available, e.requested))
  }

  /// Try to put `cnt` bytes `val` into `self`.
  ///
  /// Logically equivalent to calling `self.put_u8(val)` `cnt` times, but may work faster.
  ///
  /// `self` must have at least `cnt` remaining capacity.
  ///
  /// ```
  /// use smol_bytes::{Buffer, INLINE_CAP};
  ///
  /// let mut dst = Buffer::new();
  ///
  /// {
  ///     dst.try_put_u8(b'a').unwrap();
  ///     assert_eq!(INLINE_CAP - 1, dst.remaining_mut());
  /// }
  ///
  /// assert_eq!("a".as_bytes(), &dst);
  /// ```
  ///
  /// ## Panics
  ///
  /// This function panics if there is not enough remaining capacity in
  /// `self`.
  #[inline]
  pub const fn try_put_u8(&mut self, val: u8) -> Result<(), TryPutError> {
    // let available = self.remaining_mut();
    // if available < 1 {
    //   return Err(TryPutError {
    //     requested: 1,
    //     available,
    //   });
    // }

    // self.buf[self.len.to_usize()].write(val);
    // self.len = unsafe { InlineSize::from_u8(self.len.to_u8() + 1) };
    // Ok(())
    Self::try_put_bytes(self, val, 1)
  }
}

#[cfg(any(feature = "alloc", feature = "std"))]
const _: () = {
  use bytes::{buf::UninitSlice, Buf, BufMut};

  use crate::macros::{forward_buf, forward_buf_mut};

  impl Buf for Buffer {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn remaining(&self) -> usize {
      Self::remaining(self)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn chunk(&self) -> &[u8] {
      self.borrow()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn advance(&mut self, cnt: usize) {
      Self::advance(self, cnt);
    }

    forward_buf! {
      i16,
      i32,
      i64,
      i128,
      u16,
      u32,
      u64,
      u128,
      f32,
      f64,
    }
  }

  #[cfg(any(feature = "alloc", feature = "std"))]
  unsafe impl BufMut for Buffer {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn remaining_mut(&self) -> usize {
      Self::remaining_mut(self)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    unsafe fn advance_mut(&mut self, cnt: usize) {
      Self::advance_mut(self, cnt);
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn chunk_mut(&mut self) -> &mut UninitSlice {
      let len = self.len.to_usize();
      if len >= INLINE_CAP {
        return UninitSlice::new(&mut []);
      }
      UninitSlice::uninit(&mut self.buf[len..])
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn put_slice(&mut self, src: &[u8]) {
      Self::put_slice(self, src);
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn put_bytes(&mut self, val: u8, requested: usize) {
      Self::put_bytes(self, val, requested);
    }

    forward_buf_mut! {
      i16,
      i32,
      i64,
      i128,
      u16,
      u32,
      u64,
      u128,
      f32,
      f64,
    }
  }
};

/// Panic with a nice error message.
#[cold]
fn panic_advance(available: usize, requested: usize) -> ! {
  panic!("advance out of bounds: the len is {available} but advancing by {requested}",);
}

#[cold]
fn panic_does_not_fit(size: usize, nbytes: usize) -> ! {
  panic!(
    "size too large: the integer type can fit {} bytes, but nbytes is {}",
    size, nbytes
  );
}

const _: () = {
  const fn _assert<T: Send + Sync>() {}
  _assert::<Buffer>();
};
