use core::{
  borrow::Borrow,
  cmp::Ordering,
  fmt,
  hash::{Hash, Hasher},
  marker::PhantomData,
  mem,
  ops::{self, Bound, RangeBounds},
};

use bytes::{Buf, Bytes};

use std::{borrow::Cow, boxed::Box, sync::Arc, vec::Vec};

pub use strategy::{ConversionFriendly, Inline};

use strategy::Strategy;

#[cfg(feature = "borsh")]
mod borsh;
#[cfg(feature = "serde")]
mod serde;

#[cfg(test)]
mod tests;

mod strategy;

/// Number of bytes that can be stored inline inside a [`SmolBytes`].
pub const INLINE_CAP: usize = InlineSize::_V38 as usize;

/// A compact, clone-efficient byte buffer.
#[derive(Clone)]
#[repr(transparent)]
pub struct SmolBytes<S = ConversionFriendly> {
  repr: Repr,
  _strategy: PhantomData<S>,
}

impl<S> SmolBytes<S>
where
  Self: Strategy,
{
  /// Creates a new empty Bytes.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use smol_bytes::SmolBytes;
  ///
  /// let empty = SmolBytes::new();
  /// assert_eq!(&empty[..], b"");
  /// ```
  #[inline]
  pub const fn new() -> Self {
    Self::inline([0; INLINE_CAP], 0, 0)
  }

  /// Creates an inline [`SmolBytes`] without allocating.
  ///
  /// # Panics
  ///
  /// Panics if `bytes.len() > INLINE_CAP`.
  ///
  /// ```rust
  /// use smol_bytes::SmolBytes;
  ///
  /// let b = SmolBytes::new_inline(b"hello");
  /// assert_eq!(&b[..], b"hello");
  /// ```
  #[inline]
  pub const fn new_inline(bytes: &[u8]) -> Self {
    let blen = bytes.len();
    assert!(blen <= INLINE_CAP);

    let mut buf = [0u8; INLINE_CAP];

    unsafe {
      core::ptr::copy(bytes.as_ptr(), buf.as_mut_ptr(), blen);
    }
    Self::inline(buf, 0, blen)
  }

  /// Creates a [`SmolBytes`] from a statically allocated byte slice.
  ///
  /// ```rust
  /// use smol_bytes::SmolBytes;
  ///
  /// let b = SmolBytes::from_static(b"hello");
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
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::SmolBytes;
  ///
  /// let a = SmolBytes::from(&b"hello world"[..]);
  /// let b = a.slice(2..5);
  ///
  /// assert_eq!(&b[..], b"llo");
  ///
  /// let a = SmolBytes::from(vec![1; 100]);
  /// let b = a.slice(10..90);
  /// assert_eq!(b.len(), 80);
  /// assert_eq!(&b[..], &[1; 80]);
  ///
  /// let c = a.slice(1..4);
  /// assert_eq!(&c[..], &[1, 1, 1]);
  /// ```
  ///
  /// # Panics
  ///
  /// Requires that `begin <= end` and `end <= self.len()`, otherwise slicing
  /// will panic.
  #[inline]
  pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
    Strategy::slice(self, range)
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
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::SmolBytes;
  ///
  /// let mut a = SmolBytes::from(&b"hello world"[..]);
  /// let b = a.split_off(5);
  ///
  /// assert_eq!(&a[..], b"hello");
  /// assert_eq!(&b[..], b" world");
  /// ```
  ///
  /// # Panics
  ///
  /// Panics if `at > len`.
  #[must_use = "consider SmolBytes::truncate if you don't need the other half"]
  pub fn split_off(&mut self, at: usize) -> Self {
    Strategy::split_off(self, at)
  }

  /// Splits the bytes into two at the given index.
  ///
  /// Afterwards `self` contains elements `[at, len)`, and the returned
  /// `SmolBytes` contains elements `[0, at)`.
  ///
  /// This is an `O(1)` operation that just increases the reference count and
  /// sets a few indices.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::SmolBytes;
  ///
  /// let mut a = SmolBytes::from(&b"hello world"[..]);
  /// let b = a.split_to(5);
  ///
  /// assert_eq!(&a[..], b" world");
  /// assert_eq!(&b[..], b"hello");
  /// ```
  ///
  /// # Panics
  ///
  /// Panics if `at > len`.
  #[must_use = "consider SmolBytes::advance if you don't need the other half"]
  pub fn split_to(&mut self, at: usize) -> Self {
    Strategy::split_to(self, at)
  }

  /// Truncates this [`SmolBytes`] to the specified length.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use smol_bytes::SmolBytes;
  ///
  /// let mut b = SmolBytes::from_static(b"hello world");
  /// b.truncate(5);
  /// assert_eq!(&b[..], b"hello");
  ///
  /// let mut b2 = SmolBytes::from(vec![1u8; 100]);
  /// b2.truncate(10);
  /// assert_eq!(b2.len(), 10);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn truncate(&mut self, new_len: usize) {
    Strategy::truncate(self, new_len);
  }

  /// Clears the contents of this [`SmolBytes`].
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use smol_bytes::SmolBytes;
  ///
  /// let mut b = SmolBytes::from_static(b"hello");
  /// b.clear();
  /// assert!(b.is_empty());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn clear(&mut self) {
    Strategy::clear(self)
  }
}

impl<S> SmolBytes<S>
where
  Self: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn new_in(repr: Repr) -> Self {
    Self {
      repr,
      _strategy: PhantomData,
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn inline(buf: [u8; INLINE_CAP], cur: u8, blen: usize) -> Self {
    Self::new_in(Repr::inline(buf, cur, blen))
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn heap(bytes: Bytes) -> Self {
    Self::new_in(Repr::Heap(bytes))
  }

  /// Returns `true` if this is the only reference to the data and Into<BytesMut> would avoid cloning the underlying buffer.
  ///
  /// Always returns `false` if the data is backed by a static slice, or inlined.
  ///
  /// The result of this method may be invalidated immediately if another thread clones this value while this is being called. Ensure you have unique access to this value (&mut SmolBytes) first if you need to be certain the result is valid (i.e. for safety reasons).
  ///
  /// ```rust
  /// use smol_bytes::SmolBytes;
  ///
  /// let b = SmolBytes::from_static(b"hello");
  /// assert!(!b.is_unique());
  ///
  /// let b2 = SmolBytes::from(vec![1; 100]);
  /// assert!(b2.is_unique());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn is_unique(&self) -> bool {
    self.repr.is_unique()
  }

  /// Returns the length in bytes of this [`SmolBytes`].
  ///
  /// ```rust
  /// use smol_bytes::SmolBytes;
  ///
  /// let b = SmolBytes::from_static(b"hello");
  /// assert_eq!(b.len(), 5);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn len(&self) -> usize {
    self.repr.len()
  }

  /// Returns `true` if this [`SmolBytes`] contains no bytes.
  ///
  /// ```rust
  /// use smol_bytes::SmolBytes;
  ///
  /// let b = SmolBytes::new();
  /// assert!(b.is_empty());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_empty(&self) -> bool {
    self.repr.is_empty()
  }

  /// Creates a [`SmolBytes`] from any byte slice, allocating if needed.
  ///
  /// ```rust
  /// use smol_bytes::SmolBytes;
  ///
  /// let data = vec![1, 2, 3, 4, 5];
  /// let b = SmolBytes::copy_from_slice(&data);
  /// assert_eq!(&b[..], &data[..]);
  /// ```
  #[inline]
  pub fn copy_from_slice(bytes: impl AsRef<[u8]>) -> Self {
    Self::new_in(Repr::new(bytes.as_ref()))
  }

  /// Returns the byte slice underlying this [`SmolBytes`].
  ///
  /// ```rust
  /// use smol_bytes::SmolBytes;
  ///
  /// let b = SmolBytes::from_static(b"hello");
  /// assert_eq!(b.as_slice(), b"hello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_slice(&self) -> &[u8] {
    self.repr.as_slice()
  }

  /// Returns `true` if this [`SmolBytes`] is backed by a heap allocation.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_heap_allocated(&self) -> bool {
    matches!(self.repr, Repr::Heap(..))
  }

  /// Returns the inline capacity in bytes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn inline_capacity() -> usize {
    INLINE_CAP
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

impl<S> Default for SmolBytes<S>
where
  Self: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new()
  }
}

impl<S> ops::Deref for SmolBytes<S>
where
  Self: Strategy,
{
  type Target = [u8];

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    self.as_slice()
  }
}

impl<S> Borrow<[u8]> for SmolBytes<S>
where
  Self: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn borrow(&self) -> &[u8] {
    self.as_slice()
  }
}

impl<S> AsRef<[u8]> for SmolBytes<S>
where
  Self: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_ref(&self) -> &[u8] {
    self.as_slice()
  }
}

impl<S> PartialEq for SmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.repr.ptr_eq(&other.repr) || self.as_slice() == other.as_slice()
  }
}

impl<S> Eq for SmolBytes<S> where Self: Strategy {}

impl<S> PartialEq<[u8]> for SmolBytes<S>
where
  Self: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &[u8]) -> bool {
    self.as_slice() == other
  }
}

impl<S> PartialEq<SmolBytes<S>> for [u8]
where
  SmolBytes<S>: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &SmolBytes<S>) -> bool {
    other == self
  }
}

impl<S> PartialOrd for SmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl<S> Ord for SmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_slice().cmp(other.as_slice())
  }
}

impl<S> Hash for SmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.as_slice().hash(state)
  }
}

impl<S> fmt::Debug for SmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Debug::fmt(self.as_slice(), f)
  }
}

impl<S> From<&[u8]> for SmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(slice: &[u8]) -> Self {
    Self::copy_from_slice(slice)
  }
}

impl<S> From<&mut [u8]> for SmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(slice: &mut [u8]) -> Self {
    Self::copy_from_slice(slice)
  }
}

impl<S> From<Vec<u8>> for SmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(vec: Vec<u8>) -> Self {
    Self::new_in(Repr::from_vec(vec))
  }
}

impl<S> From<Box<[u8]>> for SmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(slice: Box<[u8]>) -> Self {
    Self::new_in(Repr::from_box(slice))
  }
}

impl<S> From<Arc<[u8]>> for SmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(arc: Arc<[u8]>) -> Self {
    Self::new_in(Repr::from_arc(arc))
  }
}

impl<'a, S> From<Cow<'a, [u8]>> for SmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(cow: Cow<'a, [u8]>) -> Self {
    match cow {
      Cow::Borrowed(slice) => SmolBytes::copy_from_slice(slice),
      Cow::Owned(vec) => SmolBytes::from(vec),
    }
  }
}

impl<S> From<SmolBytes<S>> for Vec<u8>
where
  SmolBytes<S>: Strategy,
{
  #[inline]
  fn from(bytes: SmolBytes<S>) -> Self {
    bytes.into_vec()
  }
}

impl<S> From<SmolBytes<S>> for Arc<[u8]>
where
  SmolBytes<S>: Strategy,
{
  #[inline]
  fn from(bytes: SmolBytes<S>) -> Self {
    bytes.into_arc()
  }
}

impl<S> From<SmolBytes<S>> for Bytes
where
  SmolBytes<S>: Strategy,
{
  #[inline]
  fn from(bytes: SmolBytes<S>) -> Self {
    bytes.into_bytes()
  }
}

impl<'a, S> core::iter::FromIterator<&'a [u8]> for SmolBytes<S>
where
  Self: Strategy,
{
  fn from_iter<T: IntoIterator<Item = &'a [u8]>>(iter: T) -> Self {
    build_from_chunks(iter.into_iter())
  }
}

impl<S> core::iter::FromIterator<u8> for SmolBytes<S>
where
  Self: Strategy,
{
  fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
    build_from_iter(iter.into_iter())
  }
}

impl<S> Buf for SmolBytes<S>
where
  Self: Strategy,
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
    Strategy::advance(self, cnt);
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn copy_to_bytes(&mut self, len: usize) -> Bytes {
    Strategy::copy_to_bytes(self, len)
  }
}

#[allow(single_use_lifetimes)]
fn build_from_chunks<'a, S>(mut iter: impl Iterator<Item = &'a [u8]>) -> SmolBytes<S>
where
  SmolBytes<S>: Strategy,
{
  let mut buf = [0u8; INLINE_CAP];
  let mut len = 0usize;
  while let Some(chunk) = iter.next() {
    let slice = chunk;
    if len + slice.len() > INLINE_CAP {
      let (lower, _) = iter.size_hint();
      let mut vec = Vec::with_capacity(len + slice.len() + lower);
      vec.extend_from_slice(&buf[..len]);
      vec.extend_from_slice(slice);
      for rest in iter {
        vec.extend_from_slice(rest);
      }
      return SmolBytes::heap(Bytes::from(vec));
    }
    let end = len + slice.len();
    buf[len..end].copy_from_slice(slice);
    len = end;
  }
  SmolBytes::inline(buf, 0, len)
}

fn build_from_iter<S>(mut iter: impl Iterator<Item = u8>) -> SmolBytes<S>
where
  SmolBytes<S>: Strategy,
{
  let mut buf = [0u8; INLINE_CAP];
  let mut len = 0usize;
  while let Some(byte) = iter.next() {
    if len == INLINE_CAP {
      {
        let (lower, _) = iter.size_hint();
        let mut vec = Vec::with_capacity(len + 1 + lower);
        vec.extend_from_slice(&buf[..len]);
        vec.push(byte);
        vec.extend(iter);
        return SmolBytes::heap(Bytes::from(vec));
      }
      #[cfg(not(any(feature = "alloc", feature = "std")))]
      {
        unreachable!("alloc feature required for heap allocation");
      }
    }
    buf[len] = byte;
    len += 1;
  }
  SmolBytes::inline(buf, 0, len)
}

#[derive(Clone, Debug)]
enum Repr {
  Inline {
    len: InlineSize,
    cur: u8,
    buf: [u8; INLINE_CAP],
  },
  Heap(Bytes),
}

impl Repr {
  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn inline(buf: [u8; INLINE_CAP], cur: u8, len: usize) -> Self {
    Self::Inline {
      len: unsafe { InlineSize::from_u8(len as u8) },
      buf,
      cur,
    }
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
      let mut buf = [0u8; INLINE_CAP];
      let mut idx = 0usize;
      while idx < bytes.len() {
        buf[idx] = bytes[idx];
        idx += 1;
      }
      Some(Self::Inline {
        len: unsafe { InlineSize::from_u8(bytes.len() as u8) },
        buf,
        cur: 0,
      })
    } else {
      None
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn len(&self) -> usize {
    match self {
      Self::Inline { len, cur, .. } => len.to_usize() - (*cur as usize),
      Self::Heap(bytes) => bytes.len(),
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  const fn is_empty(&self) -> bool {
    match self {
      Self::Inline { len, cur, .. } => len.to_u8() - *cur == 0,
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
      Self::Inline { len, buf, cur } => &buf[*cur as usize..len.to_usize()],
      Self::Heap(bytes) => bytes.as_ref(),
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn unwrap_heap_mut(&mut self) -> &mut Bytes {
    match self {
      Self::Inline { .. } => panic!("cannot unwrap inline SmolBytes"),
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
      Self::Inline { len, buf, cur } => buf[cur as usize..len.to_usize()].to_vec(),
      Self::Heap(bytes) => bytes.into(),
    }
  }

  #[inline]
  fn into_bytes(self) -> Bytes {
    match self {
      Self::Inline { len, buf, cur } => Bytes::copy_from_slice(&buf[cur as usize..len.to_usize()]),
      Self::Heap(bytes) => bytes,
    }
  }

  #[inline]
  fn into_arc(self) -> Arc<[u8]> {
    match self {
      Self::Inline { len, buf, cur } => Arc::from(&buf[cur as usize..len.to_usize()]),
      Self::Heap(bytes) => Arc::from(Vec::<u8>::from(bytes)),
    }
  }
}

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
}

impl InlineSize {
  /// # Safety
  ///
  /// `value` must be less than or equal to [`INLINE_CAP`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const unsafe fn from_u8(value: u8) -> Self {
    debug_assert!(value <= InlineSize::_V38 as u8);
    core::mem::transmute::<u8, InlineSize>(value)
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

#[cfg(feature = "arbitrary")]
impl<'a, S> arbitrary::Arbitrary<'a> for SmolBytes<S>
where
  Self: Strategy,
{
  fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> Result<Self, arbitrary::Error> {
    let bytes = <&[u8]>::arbitrary(u)?;
    Ok(SmolBytes::copy_from_slice(bytes))
  }
}
