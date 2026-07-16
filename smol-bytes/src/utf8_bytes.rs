use core::{borrow::Borrow, ops::RangeBounds, str};

use super::{
  bytes::{RawBytes, strategy::ImmutableStorage},
  error::*,
  utf8_buf::Utf8Buf,
};

mod cmp;
mod fmt;
mod from;
mod ops;

#[cfg(feature = "arbitrary")]
mod arbitrary;
#[cfg(feature = "borsh")]
mod borsh;
#[cfg(feature = "quickcheck")]
mod quickcheck;
#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "pyo3")]
mod python;
#[cfg(feature = "pyo3")]
pub use python::PyCompactUtf8Bytes;
#[cfg(feature = "pyo3")]
pub use python::PySharedUtf8Bytes;

#[cfg(feature = "wasm")]
mod wasm;

/// Immutable UTF-8 string with inline/heap storage, generic over the
/// immutable storage strategy `S`.
///
/// Guarantees valid UTF-8. Supports zero-copy cloning via reference counting.
///
/// # Examples
///
/// ```
/// use smol_bytes::Utf8Bytes;
///
/// let bytes = Utf8Bytes::from("hello world");
/// let slice = bytes.slice(0..5);
/// assert_eq!(slice.as_str(), "hello");
/// ```
pub struct Utf8Bytes<S> {
  pub(crate) inner: RawBytes<S>,
}

impl<S> Clone for Utf8Bytes<S> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<S> Default for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn default() -> Self {
    Self::new()
  }
}

impl<S> Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  /// Creates a new, empty `Utf8Bytes`.
  pub fn new() -> Self {
    Self {
      inner: RawBytes::new(),
    }
  }

  /// Creates a `Utf8Bytes` from a static string slice.
  pub fn from_static(s: &'static str) -> Self {
    Self {
      inner: RawBytes::from_static(s.as_bytes()),
    }
  }

  /// Returns a reference to the inner `RawBytes`.
  ///
  /// Note: named `as_inner` (not `as_bytes`) to avoid shadowing
  /// [`str::as_bytes`], which is reachable via [`Deref`](core::ops::Deref)
  /// to `str`. Use `self.as_ref::<[u8]>()` or `(*self).as_bytes()` if you
  /// want the raw `&[u8]`.
  pub const fn as_inner(&self) -> &RawBytes<S> {
    &self.inner
  }

  /// Consumes `self` and returns the inner `RawBytes`.
  pub fn into_inner(self) -> RawBytes<S> {
    self.inner
  }

  /// Returns the contents as a string slice.
  #[inline]
  pub fn as_str(&self) -> &str {
    // SAFETY: Utf8Bytes guarantees valid UTF-8
    unsafe { str::from_utf8_unchecked(self.inner.as_ref()) }
  }

  /// Returns the length in bytes.
  #[inline]
  pub fn len(&self) -> usize {
    self.inner.len()
  }

  /// Returns `true` if the buffer has length 0.
  #[inline]
  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  /// Returns whether the bytes are stored inline.
  #[inline]
  pub fn is_inline(&self) -> bool {
    self.inner.is_inline()
  }

  /// Returns whether the bytes are stored on the heap.
  #[inline]
  pub fn is_heap(&self) -> bool {
    self.inner.is_heap()
  }

  /// Splits the bytes at the given index.
  ///
  /// # Panics
  ///
  /// Panics if `at` is not on a character boundary or is out of bounds.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Bytes;
  ///
  /// let mut bytes = Utf8Bytes::from("hello world");
  /// let head = bytes.split_to(5);
  ///
  /// assert_eq!(head.as_str(), "hello");
  /// assert_eq!(bytes.as_str(), " world");
  /// ```
  pub fn split_to(&mut self, at: usize) -> Self {
    self.try_split_to(at).expect("split_to failed")
  }

  /// Tries to split the bytes at the given index.
  ///
  /// # Errors
  ///
  /// Returns an error if `at` is out of bounds or not on a character boundary.
  pub fn try_split_to(&mut self, at: usize) -> Result<Self, Utf8Error> {
    <Self as Utf8Buf>::try_split_to(self, at)
  }

  /// Splits the bytes at the given index, returning the tail.
  ///
  /// # Panics
  ///
  /// Panics if `at` is not on a character boundary or is out of bounds.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Bytes;
  ///
  /// let mut bytes = Utf8Bytes::from("hello world");
  /// let tail = bytes.split_off(6);
  ///
  /// assert_eq!(bytes.as_str(), "hello ");
  /// assert_eq!(tail.as_str(), "world");
  /// ```
  pub fn split_off(&mut self, at: usize) -> Self {
    self.try_split_off(at).expect("split_off failed")
  }

  /// Tries to split the bytes at the given index, returning the tail.
  ///
  /// # Errors
  ///
  /// Returns an error if `at` is out of bounds or not on a character boundary.
  pub fn try_split_off(&mut self, at: usize) -> Result<Self, Utf8Error> {
    <Self as Utf8Buf>::try_split_off(self, at)
  }

  /// Returns a slice of this buffer.
  ///
  /// # Panics
  ///
  /// Panics if the range is not on character boundaries or is out of bounds.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Bytes;
  ///
  /// let bytes = Utf8Bytes::from("hello world");
  /// let slice = bytes.slice(0..5);
  ///
  /// assert_eq!(slice.as_str(), "hello");
  /// ```
  pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
    self.try_slice(range).expect("slice failed")
  }

  /// Tries to return a slice of this buffer.
  ///
  /// # Errors
  ///
  /// Returns an error if the range is out of bounds or not on character boundaries.
  pub fn try_slice(&self, range: impl RangeBounds<usize>) -> Result<Self, Utf8Error> {
    <Self as Utf8Buf>::try_slice(self, range)
  }
}

impl<S> AsRef<str> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl<S> AsRef<[u8]> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn as_ref(&self) -> &[u8] {
    self.as_str().as_bytes()
  }
}

impl<S> Borrow<str> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

impl<S> core::ops::Deref for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  type Target = str;

  fn deref(&self) -> &Self::Target {
    self.as_str()
  }
}

// The trait impl holds the real bodies. Inherent methods (above) forward
// here via explicit `<Self as Utf8Buf>::method` UFCS so there is no
// ambiguity about which impl is called and no fragile reliance on the
// inherent-over-trait method resolution rule.
impl<S> Utf8Buf for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn try_split_to(&mut self, at: usize) -> Result<Self, Utf8Error> {
    self.validate_char_boundary(at)?;
    Ok(Self {
      inner: self.inner.try_split_to(at).expect("already checked bounds"),
    })
  }

  fn try_split_off(&mut self, at: usize) -> Result<Self, Utf8Error> {
    self.validate_char_boundary(at)?;
    Ok(Self {
      inner: self
        .inner
        .try_split_off(at)
        .expect("already checked bounds"),
    })
  }

  fn try_slice(&self, range: impl RangeBounds<usize>) -> Result<Self, Utf8Error> {
    let (start, end) = self.range_bounds_to_start_end(range)?;
    Ok(Self {
      inner: self
        .inner
        .try_slice(start..end)
        .expect("already checked bounds"),
    })
  }
}
