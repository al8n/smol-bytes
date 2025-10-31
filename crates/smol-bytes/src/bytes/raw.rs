use core::{
  hash::{Hash, Hasher},
  marker::PhantomData,
  ops::RangeBounds,
};

use bytes::{Buf, Bytes};

use std::{borrow::Cow, boxed::Box, string::String, sync::Arc, vec::Vec};

use crate::{
  buffer::{Buffer, INLINE_CAP},
  BytesMut, OutOfBounds, RangeOutOfBounds,
};

use super::strategy::ImmutableStorage;

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

/// A compact, clone-efficient byte buffer.
#[derive(Clone)]
#[repr(transparent)]
pub struct RawBytes<S> {
  pub(crate) repr: Repr,
  _strategy: PhantomData<S>,
}

impl<S> RawBytes<S>
where
  Self: ImmutableStorage,
{
  /// Creates a new empty Bytes.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use smol_bytes::Bytes;
  ///
  /// let empty = Bytes::new();
  /// assert_eq!(&empty[..], b"");
  /// ```
  #[inline]
  pub const fn new() -> Self {
    Self::inline(Buffer::new())
  }

  /// Creates an inline `Bytes` without allocating.
  ///
  /// ## Panics
  ///
  /// Panics if `bytes.len() > INLINE_CAP`.
  ///
  /// ```rust
  /// use smol_bytes::Bytes;
  ///
  /// let b = Bytes::new_inline(b"hello");
  /// assert_eq!(&b[..], b"hello");
  /// ```
  #[inline]
  pub const fn new_inline(bytes: &[u8]) -> Self {
    let blen = bytes.len();
    assert!(blen <= INLINE_CAP);

    // SAFETY: We checked that blen <= INLINE_CAP
    Self::inline(unsafe { Buffer::copy_from_slice(bytes) })
  }

  /// Creates a `Bytes` from a statically allocated byte slice.
  ///
  /// ```rust
  /// use smol_bytes::Bytes;
  ///
  /// let b = Bytes::from_static(b"hello");
  /// assert_eq!(&b[..], b"hello");
  /// ```
  #[inline]
  pub const fn from_static(bytes: &'static [u8]) -> Self {
    if bytes.len() <= INLINE_CAP {
      return Self::new_inline(bytes);
    }
    Self::heap(Bytes::from_static(bytes))
  }

  /// Returns a slice of self for the provided range.
  ///
  /// This will increment the reference count for the underlying memory and
  /// return a new `Bytes` handle set to the slice.
  ///
  /// This operation is `O(1)`.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::Bytes;
  ///
  /// let a = Bytes::from(&b"hello world"[..]);
  /// let b = a.slice(2..5);
  ///
  /// assert_eq!(&b[..], b"llo");
  ///
  /// let a = Bytes::from(vec![1; 100]);
  /// let b = a.slice(10..90);
  /// assert_eq!(b.len(), 80);
  /// assert_eq!(&b[..], &[1; 80]);
  ///
  /// let c = a.slice(1..4);
  /// assert_eq!(&c[..], &[1, 1, 1]);
  /// ```
  ///
  /// ## Panics
  ///
  /// Requires that `begin <= end` and `end <= self.len()`, otherwise slicing
  /// will panic.
  #[inline]
  pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
    ImmutableStorage::slice(self, range)
  }

  /// Tries to create a slice of self for the provided range.
  ///
  /// Returns `Err(RangeOutOfBounds)` if the range is invalid.
  pub fn try_slice(&self, range: impl RangeBounds<usize>) -> Result<Self, RangeOutOfBounds> {
    ImmutableStorage::try_slice(self, range)
  }

  /// Splits the bytes into two at the given index.
  ///
  /// Afterwards `self` contains elements `[0, at)`, and the returned `Bytes`
  /// contains elements `[at, len)`. It's guaranteed that the memory does not
  /// move, that is, the address of `self` does not change, and the address of
  /// the returned slice is `at` bytes after that.
  ///
  /// This is an `O(1)` operation that just increases the reference count and
  /// sets a few indices.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::Bytes;
  ///
  /// let mut a = Bytes::from(&b"hello world"[..]);
  /// let b = a.split_off(5);
  ///
  /// assert_eq!(&a[..], b"hello");
  /// assert_eq!(&b[..], b" world");
  /// ```
  ///
  /// ## Panics
  ///
  /// Panics if `at > len`.
  #[must_use = "consider RawBytes::truncate if you don't need the other half"]
  pub fn split_off(&mut self, at: usize) -> Self {
    ImmutableStorage::split_off(self, at)
  }

  /// Tries to split the bytes into two at the given index, returning the tail.
  ///
  /// Returns `Err(OutOfBounds)` if `at > len`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn try_split_off(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    let len = self.len();
    if at > len {
      return Err(OutOfBounds::new(at, len));
    }
    Ok(self.split_off(at))
  }

  /// Splits the bytes into two at the given index.
  ///
  /// Afterwards `self` contains elements `[at, len)`, and the returned
  /// `Bytes` contains elements `[0, at)`.
  ///
  /// This is an `O(1)` operation that just increases the reference count and
  /// sets a few indices.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::Bytes;
  ///
  /// let mut a = Bytes::from(&b"hello world"[..]);
  /// let b = a.split_to(5);
  ///
  /// assert_eq!(&a[..], b" world");
  /// assert_eq!(&b[..], b"hello");
  /// ```
  ///
  /// ## Panics
  ///
  /// Panics if `at > len`.
  #[must_use = "consider RawBytes::advance if you don't need the other half"]
  pub fn split_to(&mut self, at: usize) -> Self {
    ImmutableStorage::split_to(self, at)
  }

  /// Tries to split the bytes into two at the given index, returning the head.
  ///
  /// Returns `Err(OutOfBounds)` if `at > len`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn try_split_to(&mut self, at: usize) -> Result<Self, OutOfBounds> {
    let len = self.len();
    if at > len {
      return Err(OutOfBounds::new(at, len));
    }
    Ok(self.split_to(at))
  }

  /// Truncates this `Bytes` to the specified length.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use smol_bytes::Bytes;
  ///
  /// let mut b = Bytes::from_static(b"hello world");
  /// b.truncate(5);
  /// assert_eq!(&b[..], b"hello");
  ///
  /// let mut b2 = Bytes::from(vec![1u8; 100]);
  /// b2.truncate(10);
  /// assert_eq!(b2.len(), 10);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn truncate(&mut self, new_len: usize) {
    ImmutableStorage::truncate(self, new_len);
  }

  /// Clears the contents of this `Bytes`.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use smol_bytes::Bytes;
  ///
  /// let mut b = Bytes::from_static(b"hello");
  /// b.clear();
  /// assert!(b.is_empty());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn clear(&mut self) {
    ImmutableStorage::clear(self)
  }

  /// Attempts to advance the buffer by `cnt` bytes, returning an error if out of bounds.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn try_advance(&mut self, cnt: usize) -> Result<(), OutOfBounds> {
    let len = self.len();
    if cnt > len {
      return Err(OutOfBounds::new(cnt, len));
    }
    ImmutableStorage::advance(self, cnt);
    Ok(())
  }
}

impl<S> RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn new_in(repr: Repr) -> Self {
    Self {
      repr,
      _strategy: PhantomData,
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn inline(storage: Buffer) -> Self {
    Self::new_in(Repr::inline(storage))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn heap(bytes: Bytes) -> Self {
    Self::new_in(Repr::Heap(bytes))
  }

  /// Returns `true` if this is the only reference to the data and Into<BytesMut> would avoid cloning the underlying buffer.
  ///
  /// Always returns `false` if the data is backed by a static slice, or inlined.
  ///
  /// The result of this method may be invalidated immediately if another thread clones this value while this is being called. Ensure you have unique access to this value (&mut Bytes) first if you need to be certain the result is valid (i.e. for safety reasons).
  ///
  /// ```rust
  /// use smol_bytes::Bytes;
  ///
  /// let b = Bytes::from_static(b"hello");
  /// assert!(!b.is_unique());
  ///
  /// let b2 = Bytes::from(vec![1; 100]);
  /// assert!(b2.is_unique());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn is_unique(&self) -> bool {
    self.repr.is_unique()
  }

  /// Returns the length in bytes of this `Bytes`.
  ///
  /// ```rust
  /// use smol_bytes::Bytes;
  ///
  /// let b = Bytes::from_static(b"hello");
  /// assert_eq!(b.len(), 5);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn len(&self) -> usize {
    self.repr.len()
  }

  /// Returns `true` if this `Bytes` contains no bytes.
  ///
  /// ```rust
  /// use smol_bytes::Bytes;
  ///
  /// let b = Bytes::new();
  /// assert!(b.is_empty());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_empty(&self) -> bool {
    self.repr.is_empty()
  }

  /// Creates a `Bytes` from any byte slice, allocating if needed.
  ///
  /// ```rust
  /// use smol_bytes::Bytes;
  ///
  /// let data: Vec<u8> = vec![1, 2, 3, 4, 5];
  /// let b = Bytes::copy_from_slice(&data);
  /// assert_eq!(&b[..], &data[..]);
  /// ```
  #[inline]
  pub fn copy_from_slice(bytes: impl AsRef<[u8]>) -> Self {
    Self::new_in(Repr::new(bytes.as_ref()))
  }

  /// Returns the byte slice underlying this `Bytes`.
  ///
  /// ```rust
  /// use smol_bytes::Bytes;
  ///
  /// let b = Bytes::from_static(b"hello");
  /// assert_eq!(b, b"hello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_slice(&self) -> &[u8] {
    self.repr.as_slice()
  }

  /// Returns `true` if this `Bytes` is backed by a inline buffer.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::Bytes;
  ///
  /// let inline_buf = Bytes::from(&b"hello"[..]);
  /// let heap_buf = Bytes::from("hello world and more data that exceeds inline capacity................................".as_bytes());
  ///
  /// assert!(inline_buf.is_inline());
  /// assert!(!heap_buf.is_inline());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_inline(&self) -> bool {
    matches!(self.repr, Repr::Inline(..))
  }

  /// Returns `true` if this `Bytes` is backed by a heap allocation.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::Bytes;
  ///
  /// let inline_buf = Bytes::from(&b"hello"[..]);
  /// let heap_buf = Bytes::from("hello world and more data that exceeds inline capacity........................".as_bytes());
  ///
  /// assert!(!inline_buf.is_heap());
  /// assert!(heap_buf.is_heap());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_heap(&self) -> bool {
    matches!(self.repr, Repr::Heap(..))
  }

  /// Unwraps the inline buffer, consuming `self`.
  ///
  /// ## Panics
  /// - Panics if the buffer is heap allocated.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::Bytes;
  ///
  /// let buf = Bytes::from(&b"hello"[..]);
  ///
  /// let inline_buffer = buf.unwrap_inline();
  /// assert_eq!(&inline_buffer[..], b"hello");
  /// ```
  #[inline]
  pub fn unwrap_inline(self) -> Buffer {
    match self.repr {
      Repr::Inline(b) => b,
      Repr::Heap(_) => panic!("called `Bytes::unwrap_inline()` on a heap allocated buffer"),
    }
  }

  /// Attempts to unwrap the inline buffer, consuming `self`.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::Bytes;
  ///
  /// let inline_buf = Bytes::from("hello".as_bytes());
  /// let heap_buf = Bytes::from("hello world and more data that exceeds inline capacity................................".as_bytes());
  ///
  /// assert!(inline_buf.try_unwrap_inline().is_ok());
  /// assert!(heap_buf.try_unwrap_inline().is_err());
  /// ```
  #[inline]
  pub fn try_unwrap_inline(self) -> Result<Buffer, Bytes> {
    match self.repr {
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
  /// use smol_bytes::Bytes;
  ///
  /// let mut buf = Bytes::from("hello world and more data that exceeds inline capacity................................".as_bytes());
  ///
  /// let heap_buffer = buf.unwrap_heap();
  /// assert_eq!(&heap_buffer[..], b"hello world and more data that exceeds inline capacity................................");
  /// ```
  #[inline]
  pub fn unwrap_heap(self) -> Bytes {
    match self.repr {
      Repr::Inline(_) => panic!("called `Bytes::unwrap_heap()` on an inline buffer"),
      Repr::Heap(b) => b,
    }
  }

  /// Attempts to unwrap the heap buffer, consuming `self`.
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::Bytes;
  ///
  /// let inline_buf = Bytes::from(&b"hello"[..]);
  /// let mut heap_buf = Bytes::from(b"hello world and more data that exceeds inline capacity................................");
  ///
  /// assert!(heap_buf.try_unwrap_heap().is_ok());
  /// assert!(inline_buf.try_unwrap_heap().is_err());
  /// ```
  #[inline]
  pub fn try_unwrap_heap(self) -> Result<Bytes, Buffer> {
    match self.repr {
      Repr::Inline(b) => Err(b),
      Repr::Heap(b) => Ok(b),
    }
  }

  /// Try to convert self into `BytesMut`.
  ///
  /// If `self` is unique for the entire original buffer, this will succeed
  /// and return a `BytesMut` with the contents of `self` without copying.
  /// If `self` is not unique for the entire original buffer, this will fail
  /// and return self.
  ///
  /// This will also always fail if the buffer was constructed via either
  /// [from_owner](Bytes::from_owner) or [from_static](Bytes::from_static).
  ///
  /// ## Examples
  ///
  /// ```
  /// use smol_bytes::{Bytes, BytesMut};
  ///
  /// let bytes = Bytes::from(b"hello".to_vec());
  /// assert_eq!(bytes.try_into_mut(), Ok(BytesMut::from(&b"hello"[..])));
  ///
  /// let bytes = Bytes::from("a".repeat(100).as_bytes());
  /// assert_eq!(bytes.try_into_mut(), Ok(BytesMut::from("a".repeat(100).as_bytes())));
  /// ```
  pub fn try_into_mut(self) -> Result<BytesMut, Bytes> {
    match self.repr {
      Repr::Inline(storage) => Ok(BytesMut::from_inline(storage)),
      Repr::Heap(bytes) => bytes.try_into_mut().map(BytesMut::from_bytes_mut),
    }
  }

  /// Converts `self` into a [`Vec<u8>`], reusing the allocation if possible.
  #[inline]
  pub fn into_vec(self) -> Vec<u8> {
    self.repr.into_vec()
  }

  /// Converts `self` into a [`Bytes`], reusing the allocation if possible.
  #[inline]
  pub fn into_bytes(self) -> Bytes {
    self.repr.into_bytes()
  }

  /// Converts `self` into an [`Arc<[u8]>`].
  #[inline]
  pub fn into_arc(self) -> Arc<[u8]> {
    self.repr.into_arc()
  }

  /// Returns a [`Vec<u8>`] containing a copy of the bytes.
  #[inline]
  pub fn to_vec(&self) -> Vec<u8> {
    self.as_slice().to_vec()
  }

  /// Returns a boxed slice containing a copy of the bytes.
  #[inline]
  pub fn to_boxed_slice(&self) -> Box<[u8]> {
    self.as_slice().into()
  }
}

impl<S> Default for RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new()
  }
}

impl<S> Buf for RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[inline]
  fn remaining(&self) -> usize {
    self.len()
  }

  #[inline]
  fn chunk(&self) -> &[u8] {
    self.as_slice()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn advance(&mut self, cnt: usize) {
    ImmutableStorage::advance(self, cnt);
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn copy_to_bytes(&mut self, len: usize) -> Bytes {
    ImmutableStorage::copy_to_bytes(self, len)
  }

  crate::macros::forward_buf! { repr {
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
  }}
}

#[derive(Clone, Debug)]
pub(crate) enum Repr {
  Inline(Buffer),
  Heap(Bytes),
}

impl Repr {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn inline(storage: Buffer) -> Self {
    Self::Inline(storage)
  }

  #[inline]
  fn new(bytes: &[u8]) -> Self {
    if let Some(inline) = Self::new_on_stack(bytes) {
      inline
    } else {
      Self::Heap(Bytes::copy_from_slice(bytes))
    }
  }

  #[inline]
  fn from_vec(vec: Vec<u8>) -> Self {
    if let Some(inline) = Self::new_on_stack(&vec) {
      inline
    } else {
      Self::Heap(Bytes::from(vec))
    }
  }

  #[inline]
  fn from_box(vec: Box<[u8]>) -> Self {
    if let Some(inline) = Self::new_on_stack(&vec) {
      inline
    } else {
      Self::Heap(Bytes::from(vec))
    }
  }

  #[inline]
  fn from_arc(vec: Arc<[u8]>) -> Self {
    if let Some(inline) = Self::new_on_stack(&vec) {
      inline
    } else {
      Self::Heap(Bytes::copy_from_slice(vec.as_ref()))
    }
  }

  #[inline]
  const fn new_on_stack(bytes: &[u8]) -> Option<Self> {
    if bytes.len() <= INLINE_CAP {
      // SAFETY: we checked that bytes.len() <= INLINE_CAP.
      Some(Self::Inline(unsafe { Buffer::copy_from_slice(bytes) }))
    } else {
      None
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn len(&self) -> usize {
    match self {
      Self::Inline(storage) => storage.remaining(),
      Self::Heap(bytes) => bytes.len(),
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn is_empty(&self) -> bool {
    match self {
      Self::Inline(storage) => storage.remaining() == 0,
      Self::Heap(bytes) => bytes.is_empty(),
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn is_unique(&self) -> bool {
    match self {
      Self::Inline { .. } => false,
      Self::Heap(bytes) => bytes.is_unique(),
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_slice(&self) -> &[u8] {
    match self {
      Self::Inline(storage) => storage.as_slice(),
      Self::Heap(bytes) => bytes.as_ref(),
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn unwrap_heap_mut(&mut self) -> &mut Bytes {
    match self {
      Self::Inline { .. } => panic!("cannot unwrap inline Bytes"),
      Self::Heap(bytes) => bytes,
    }
  }

  #[inline]
  fn ptr_eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Inline { .. }, Self::Inline { .. }) => false,
      (Self::Heap(a), Self::Heap(b)) => a.as_ptr() == b.as_ptr(),
      _ => false,
    }
  }

  #[inline]
  fn into_vec(self) -> Vec<u8> {
    match self {
      Self::Inline(storage) => storage.to_vec(),
      Self::Heap(bytes) => bytes.into(),
    }
  }

  #[inline]
  fn into_bytes(self) -> Bytes {
    match self {
      Self::Inline(storage) => Bytes::copy_from_slice(storage.as_slice()),
      Self::Heap(bytes) => bytes,
    }
  }

  #[inline]
  fn into_arc(self) -> Arc<[u8]> {
    match self {
      Self::Inline(storage) => Arc::from(storage.as_slice()),
      Self::Heap(bytes) => Arc::from(Vec::<u8>::from(bytes)),
    }
  }
}
