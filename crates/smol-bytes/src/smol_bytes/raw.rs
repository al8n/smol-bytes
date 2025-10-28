use core::{
  borrow::Borrow,
  cmp::Ordering,
  fmt,
  hash::{Hash, Hasher},
  marker::PhantomData,
  ops::{self, RangeBounds},
};

use bytes::{Buf, Bytes};

use std::{borrow::Cow, boxed::Box, sync::Arc, vec::Vec};

use crate::utils::{InlineStorage, INLINE_CAP};

use super::strategy::Strategy;

#[cfg(feature = "borsh")]
mod borsh;
#[cfg(feature = "serde")]
mod serde;

/// A compact, clone-efficient byte buffer.
#[derive(Clone)]
#[repr(transparent)]
pub struct RawSmolBytes<S> {
  pub(crate) repr: Repr,
  _strategy: PhantomData<S>,
}

impl<S> RawSmolBytes<S>
where
  Self: Strategy,
{
  /// Creates a new empty Bytes.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use smol_bytes::strategy::shared::SmolBytes;
  ///
  /// let empty: RawSmolBytes::new();
  /// assert_eq!(&empty[..], b"");
  /// ```
  #[inline]
  pub const fn new() -> Self {
    Self::inline(InlineStorage::new())
  }

  /// Creates an inline [`RawSmolBytes`] without allocating.
  ///
  /// # Panics
  ///
  /// Panics if `bytes.len() > INLINE_CAP`.
  ///
  /// ```rust
  /// use smol_bytes::strategy::shared::SmolBytes;
  ///
  /// let b: RawSmolBytes::new_inline(b"hello");
  /// assert_eq!(&b[..], b"hello");
  /// ```
  #[inline]
  pub const fn new_inline(bytes: &[u8]) -> Self {
    let blen = bytes.len();
    assert!(blen <= INLINE_CAP);

    // SAFETY: We checked that blen <= INLINE_CAP
    Self::inline(unsafe { InlineStorage::copy_from_slice(bytes) })
  }

  /// Creates a [`RawSmolBytes`] from a statically allocated byte slice.
  ///
  /// ```rust
  /// use smol_bytes::strategy::shared::SmolBytes;
  ///
  /// let b: RawSmolBytes::from_static(b"hello");
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
  /// use smol_bytes::strategy::shared::SmolBytes;
  ///
  /// let a: RawSmolBytes::from(&b"hello world"[..]);
  /// let b = a.slice(2..5);
  ///
  /// assert_eq!(&b[..], b"llo");
  ///
  /// let a: RawSmolBytes::from(vec![1; 100]);
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
  /// use smol_bytes::strategy::shared::SmolBytes;
  ///
  /// let mut a: RawSmolBytes::from(&b"hello world"[..]);
  /// let b = a.split_off(5);
  ///
  /// assert_eq!(&a[..], b"hello");
  /// assert_eq!(&b[..], b" world");
  /// ```
  ///
  /// # Panics
  ///
  /// Panics if `at > len`.
  #[must_use = "consider RawSmolBytes::truncate if you don't need the other half"]
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
  /// use smol_bytes::strategy::shared::SmolBytes;
  ///
  /// let mut a: RawSmolBytes::from(&b"hello world"[..]);
  /// let b = a.split_to(5);
  ///
  /// assert_eq!(&a[..], b" world");
  /// assert_eq!(&b[..], b"hello");
  /// ```
  ///
  /// # Panics
  ///
  /// Panics if `at > len`.
  #[must_use = "consider RawSmolBytes::advance if you don't need the other half"]
  pub fn split_to(&mut self, at: usize) -> Self {
    Strategy::split_to(self, at)
  }

  /// Truncates this [`RawSmolBytes`] to the specified length.
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use smol_bytes::strategy::shared::SmolBytes;
  ///
  /// let mut b: RawSmolBytes::from_static(b"hello world");
  /// b.truncate(5);
  /// assert_eq!(&b[..], b"hello");
  ///
  /// let mut b2: RawSmolBytes::from(vec![1u8; 100]);
  /// b2.truncate(10);
  /// assert_eq!(b2.len(), 10);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn truncate(&mut self, new_len: usize) {
    Strategy::truncate(self, new_len);
  }

  /// Clears the contents of this [`RawSmolBytes`].
  ///
  /// ## Examples
  ///
  /// ```rust
  /// use smol_bytes::strategy::shared::SmolBytes;
  ///
  /// let mut b: RawSmolBytes::from_static(b"hello");
  /// b.clear();
  /// assert!(b.is_empty());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn clear(&mut self) {
    Strategy::clear(self)
  }
}

impl<S> RawSmolBytes<S>
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
  pub(crate) const fn inline(storage: InlineStorage) -> Self {
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
  /// The result of this method may be invalidated immediately if another thread clones this value while this is being called. Ensure you have unique access to this value (&mut SmolBytes) first if you need to be certain the result is valid (i.e. for safety reasons).
  ///
  /// ```rust
  /// use smol_bytes::strategy::shared::SmolBytes;
  ///
  /// let b: RawSmolBytes::from_static(b"hello");
  /// assert!(!b.is_unique());
  ///
  /// let b2: RawSmolBytes::from(vec![1; 100]);
  /// assert!(b2.is_unique());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn is_unique(&self) -> bool {
    self.repr.is_unique()
  }

  /// Returns the length in bytes of this [`RawSmolBytes`].
  ///
  /// ```rust
  /// use smol_bytes::strategy::shared::SmolBytes;
  ///
  /// let b: RawSmolBytes::from_static(b"hello");
  /// assert_eq!(b.len(), 5);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn len(&self) -> usize {
    self.repr.len()
  }

  /// Returns `true` if this [`RawSmolBytes`] contains no bytes.
  ///
  /// ```rust
  /// use smol_bytes::strategy::shared::SmolBytes;
  ///
  /// let b: RawSmolBytes::new();
  /// assert!(b.is_empty());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_empty(&self) -> bool {
    self.repr.is_empty()
  }

  /// Creates a [`RawSmolBytes`] from any byte slice, allocating if needed.
  ///
  /// ```rust
  /// use smol_bytes::strategy::shared::SmolBytes;
  ///
  /// let data: Vec<u8> = vec![1, 2, 3, 4, 5];
  /// let b: RawSmolBytes::copy_from_slice(&data);
  /// assert_eq!(&b[..], &data[..]);
  /// ```
  #[inline]
  pub fn copy_from_slice(bytes: impl AsRef<[u8]>) -> Self {
    Self::new_in(Repr::new(bytes.as_ref()))
  }

  /// Returns the byte slice underlying this [`RawSmolBytes`].
  ///
  /// ```rust
  /// use smol_bytes::strategy::shared::SmolBytes;
  ///
  /// let b: RawSmolBytes::from_static(b"hello");
  /// assert_eq!(b.as_slice(), b"hello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_slice(&self) -> &[u8] {
    self.repr.as_slice()
  }

  /// Returns `true` if this [`RawSmolBytes`] is backed by a heap allocation.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_heap(&self) -> bool {
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

impl<S> Default for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new()
  }
}

impl<S> ops::Deref for RawSmolBytes<S>
where
  Self: Strategy,
{
  type Target = [u8];

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    self.as_slice()
  }
}

impl<S> Borrow<[u8]> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn borrow(&self) -> &[u8] {
    self.as_slice()
  }
}

impl<S> AsRef<[u8]> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_ref(&self) -> &[u8] {
    self.as_slice()
  }
}

impl<S> PartialEq for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.repr.ptr_eq(&other.repr) || self.as_slice() == other.as_slice()
  }
}

impl<S> Eq for RawSmolBytes<S> where Self: Strategy {}

impl<S> PartialEq<[u8]> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &[u8]) -> bool {
    self.as_slice() == other
  }
}

impl<S> PartialEq<RawSmolBytes<S>> for [u8]
where
  RawSmolBytes<S>: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &RawSmolBytes<S>) -> bool {
    other == self
  }
}

impl<S> PartialOrd for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl<S> Ord for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_slice().cmp(other.as_slice())
  }
}

impl<S> Hash for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.as_slice().hash(state)
  }
}

impl<S> fmt::Debug for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &self.repr {
      Repr::Inline(b) => b.fmt(f),
      Repr::Heap(b) => b.fmt(f),
    }
  }
}

impl<S> From<&[u8]> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(slice: &[u8]) -> Self {
    Self::copy_from_slice(slice)
  }
}

impl<S> From<&mut [u8]> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(slice: &mut [u8]) -> Self {
    Self::copy_from_slice(slice)
  }
}

impl<S> From<Vec<u8>> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(vec: Vec<u8>) -> Self {
    Self::new_in(Repr::from_vec(vec))
  }
}

impl<S> From<Box<[u8]>> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(slice: Box<[u8]>) -> Self {
    Self::new_in(Repr::from_box(slice))
  }
}

impl<S> From<Arc<[u8]>> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(arc: Arc<[u8]>) -> Self {
    Self::new_in(Repr::from_arc(arc))
  }
}

impl<'a, S> From<Cow<'a, [u8]>> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(cow: Cow<'a, [u8]>) -> Self {
    match cow {
      Cow::Borrowed(slice) => RawSmolBytes::copy_from_slice(slice),
      Cow::Owned(vec) => RawSmolBytes::from(vec),
    }
  }
}

impl<S> From<RawSmolBytes<S>> for Vec<u8>
where
  RawSmolBytes<S>: Strategy,
{
  #[inline]
  fn from(bytes: RawSmolBytes<S>) -> Self {
    bytes.into_vec()
  }
}

impl<S> From<RawSmolBytes<S>> for Arc<[u8]>
where
  RawSmolBytes<S>: Strategy,
{
  #[inline]
  fn from(bytes: RawSmolBytes<S>) -> Self {
    bytes.into_arc()
  }
}

impl<S> From<RawSmolBytes<S>> for Bytes
where
  RawSmolBytes<S>: Strategy,
{
  #[inline]
  fn from(bytes: RawSmolBytes<S>) -> Self {
    bytes.into_bytes()
  }
}

impl<'a, S> core::iter::FromIterator<&'a [u8]> for RawSmolBytes<S>
where
  Self: Strategy,
{
  fn from_iter<T: IntoIterator<Item = &'a [u8]>>(iter: T) -> Self {
    build_from_chunks(iter.into_iter())
  }
}

impl<S> core::iter::FromIterator<u8> for RawSmolBytes<S>
where
  Self: Strategy,
{
  fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
    build_from_iter(iter.into_iter())
  }
}

impl<S> Buf for RawSmolBytes<S>
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
fn build_from_chunks<'a, S>(mut iter: impl Iterator<Item = &'a [u8]>) -> RawSmolBytes<S>
where
  RawSmolBytes<S>: Strategy,
{
  let mut buf = InlineStorage::new();
  let mut len = 0usize;
  while let Some(chunk) = iter.next() {
    let slice = chunk;
    if len + slice.len() > INLINE_CAP {
      let (lower, _) = iter.size_hint();
      let mut vec = Vec::with_capacity(len + slice.len() + lower);
      vec.extend_from_slice(buf.as_slice());
      vec.extend_from_slice(slice);
      for rest in iter {
        vec.extend_from_slice(rest);
      }
      return RawSmolBytes::heap(Bytes::from(vec));
    }
    let end = len + slice.len();
    buf.append_slice(slice);
    len = end;
  }
  RawSmolBytes::inline(buf)
}

fn build_from_iter<S>(mut iter: impl Iterator<Item = u8>) -> RawSmolBytes<S>
where
  RawSmolBytes<S>: Strategy,
{
  let mut buf = InlineStorage::new();
  let mut len = 0usize;
  while let Some(byte) = iter.next() {
    if len == INLINE_CAP {
      {
        let (lower, _) = iter.size_hint();
        let mut vec = Vec::with_capacity(len + 1 + lower);
        vec.extend_from_slice(buf.as_slice());
        vec.push(byte);
        vec.extend(iter);
        return RawSmolBytes::heap(Bytes::from(vec));
      }
      #[cfg(not(any(feature = "alloc", feature = "std")))]
      {
        unreachable!("alloc feature required for heap allocation");
      }
    }
    buf[len] = byte;
    len += 1;
  }
  RawSmolBytes::inline(buf)
}

#[derive(Clone, Debug)]
pub(crate) enum Repr {
  Inline(InlineStorage),
  Heap(Bytes),
}

impl Repr {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn inline(storage: InlineStorage) -> Self {
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
      Some(Self::Inline(unsafe {
        InlineStorage::copy_from_slice(bytes)
      }))
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

#[cfg(feature = "arbitrary")]
impl<'a, S> arbitrary::Arbitrary<'a> for RawSmolBytes<S>
where
  Self: Strategy,
{
  fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> Result<Self, arbitrary::Error> {
    let bytes = <&[u8]>::arbitrary(u)?;
    Ok(RawSmolBytes::copy_from_slice(bytes))
  }
}

#[cfg(test)]
mod tests {
  use super::INLINE_CAP;
  use crate::strategy::shared::SmolBytes;

  #[test]
  fn inline_capacity_matches_constant() {
    assert_eq!(SmolBytes::inline_capacity(), INLINE_CAP);
  }

  #[test]
  fn default_is_empty() {
    let smol: SmolBytes = SmolBytes::default();
    assert!(smol.is_empty());
    assert_eq!(smol.len(), 0);
  }
}
