use core::{
  borrow::Borrow,
  ops::RangeBounds,
  str,
};

use bytes::BufMut;
use super::{error::*, BytesMut, utf8_buf::{Utf8Buf, Utf8BufMut}};

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

/// A UTF-8 validated wrapper around [`BytesMut`] that provides a mutable String-like interface.
///
/// `Utf8BytesMut` guarantees that its contents are always valid UTF-8, making it safe
/// to use string operations without additional validation. It provides methods for
/// splitting on character boundaries and ensures all operations maintain UTF-8 validity.
///
/// # Examples
///
/// ```
/// use smol_bytes::Utf8BytesMut;
///
/// let mut buf = Utf8BytesMut::new();
/// buf.push_str("Hello");
/// buf.push(' ');
/// buf.push_str("world!");
///
/// assert_eq!(buf.as_str(), "Hello world!");
/// ```
#[derive(Clone)]
#[cfg_attr(feature = "pyo3", ::pyo3::prelude::pyclass)]
pub struct Utf8BytesMut {
  pub(crate) inner: BytesMut,
}

impl Default for Utf8BytesMut {
  fn default() -> Self {
    Self::new()
  }
}

impl Utf8BytesMut {
  /// Creates a new, empty `Utf8BytesMut`.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let buf = Utf8BytesMut::new();
  /// assert!(buf.is_empty());
  /// ```
  pub fn new() -> Self {
    Self {
      inner: BytesMut::new(),
    }
  }

  /// Creates a new `Utf8BytesMut` with the specified capacity.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let buf = Utf8BytesMut::with_capacity(100);
  /// assert!(buf.capacity() >= 100);
  /// ```
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      inner: BytesMut::with_capacity(capacity),
    }
  }

  /// Returns a reference to the inner `BytesMut`.
  pub const fn as_bytes_mut(&self) -> &BytesMut {
    &self.inner
  }

  /// Consumes `self` and returns the inner `BytesMut`.
  pub fn into_bytes_mut(self) -> BytesMut {
    self.inner
  }

  /// Returns the contents as a string slice.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let buf = Utf8BytesMut::from("hello");
  /// assert_eq!(buf.as_str(), "hello");
  /// ```
  #[inline]
  pub fn as_str(&self) -> &str {
    // SAFETY: Utf8BytesMut guarantees valid UTF-8
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

  /// Returns the capacity of the buffer.
  #[inline]
  pub fn capacity(&self) -> usize {
    self.inner.capacity()
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

  /// Appends a character to the buffer.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let mut buf = Utf8BytesMut::new();
  /// buf.push('a');
  /// buf.push('b');
  /// assert_eq!(buf.as_str(), "ab");
  /// ```
  pub fn push(&mut self, ch: char) {
    let mut buf = [0u8; 4];
    let s = ch.encode_utf8(&mut buf);
    self.inner.put_slice(s.as_bytes());
  }

  /// Appends a string slice to the buffer.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let mut buf = Utf8BytesMut::new();
  /// buf.push_str("hello");
  /// assert_eq!(buf.as_str(), "hello");
  /// ```
  pub fn push_str(&mut self, s: &str) {
    self.inner.put_slice(s.as_bytes());
  }

  /// Clears the buffer, removing all contents.
  pub fn clear(&mut self) {
    self.inner.clear();
  }

  /// Truncates the buffer to the specified length.
  ///
  /// # Panics
  ///
  /// Panics if `new_len` does not lie on a UTF-8 character boundary.
  pub fn truncate(&mut self, new_len: usize) {
    if new_len < self.len() {
      assert!(self.as_str().is_char_boundary(new_len), "new_len must lie on a UTF-8 character boundary");
    }
    self.inner.truncate(new_len);
  }

  /// Reserves capacity for at least `additional` more bytes.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let mut buf = Utf8BytesMut::new();
  /// buf.reserve(100);
  /// assert!(buf.capacity() >= 100);
  /// ```
  pub fn reserve(&mut self, additional: usize) {
    self.inner.reserve(additional);
  }

  /// Splits the buffer at the given index.
  ///
  /// # Panics
  ///
  /// Panics if `at` is not on a character boundary or is out of bounds.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let mut buf = Utf8BytesMut::from("hello world");
  /// let head = buf.split_to(5);
  ///
  /// assert_eq!(head.as_str(), "hello");
  /// assert_eq!(buf.as_str(), " world");
  /// ```
  pub fn split_to(&mut self, at: usize) -> Self {
    self.try_split_to(at).expect("split_to failed")
  }

  /// Tries to split the buffer at the given index.
  ///
  /// # Errors
  ///
  /// Returns an error if `at` is out of bounds or not on a character boundary.
  pub fn try_split_to(&mut self, at: usize) -> Result<Self, Utf8Error> {
    self.validate_char_boundary(at)?;

    Ok(Self {
      inner: self.inner.try_split_to(at).expect("already checked bounds").expect("BytesMut split succeeded"),
    })
  }

  /// Splits the buffer at the given index, returning the tail.
  ///
  /// # Panics
  ///
  /// Panics if `at` is not on a character boundary or is out of bounds.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let mut buf = Utf8BytesMut::from("hello world");
  /// let tail = buf.split_off(6);
  ///
  /// assert_eq!(buf.as_str(), "hello ");
  /// assert_eq!(tail.as_str(), "world");
  /// ```
  pub fn split_off(&mut self, at: usize) -> Self {
    self.try_split_off(at).expect("split_off failed")
  }

  /// Tries to split the buffer at the given index, returning the tail.
  ///
  /// # Errors
  ///
  /// Returns an error if `at` is out of bounds or not on a character boundary.
  pub fn try_split_off(&mut self, at: usize) -> Result<Self, Utf8Error> {
    self.validate_char_boundary(at)?;

    Ok(Self {
      inner: self.inner.try_split_off(at).expect("already checked bounds").expect("BytesMut split succeeded"),
    })
  }

  /// Removes all bytes from the buffer and returns them in a new buffer.
  ///
  /// Afterwards, `self` will be empty but will retain its capacity.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let mut buf = Utf8BytesMut::from("hello");
  /// let cap = buf.capacity();
  /// let data = buf.split();
  ///
  /// assert_eq!(data.as_str(), "hello");
  /// assert_eq!(buf.len(), 0);
  /// assert!(buf.capacity() >= cap);
  /// ```
  pub fn split(&mut self) -> Self {
    Self {
      inner: self.inner.try_split().expect("split always succeeds").expect("BytesMut split succeeded"),
    }
  }

  /// Attempts to merge another buffer back into this one.
  ///
  /// Returns `Some(other)` if the buffers cannot be efficiently merged.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let mut buf = Utf8BytesMut::from("hello");
  /// let tail = buf.split_off(5);
  /// let result = buf.unsplit(tail);
  /// assert!(result.is_none());
  /// assert_eq!(buf.as_str(), "hello");
  /// ```
  pub fn unsplit(&mut self, other: Self) -> Option<Self> {
    self.inner.unsplit(other.inner).map(|inner| Self { inner })
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
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let buf = Utf8BytesMut::from("hello world");
  /// let slice = buf.slice(0..5);
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
    let (start, end) = self.range_bounds_to_start_end(range)?;

    Ok(Self {
      inner: self.inner.try_slice(start..end).expect("already checked bounds").expect("BytesMut slice succeeded"),
    })
  }
}

impl AsRef<str> for Utf8BytesMut {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl AsRef<[u8]> for Utf8BytesMut {
  fn as_ref(&self) -> &[u8] {
    self.as_str().as_bytes()
  }
}

impl Borrow<str> for Utf8BytesMut {
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

impl core::ops::Deref for Utf8BytesMut {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    self.as_str()
  }
}

impl Utf8Buf for Utf8BytesMut {
  // Default implementations from trait are used for:
  // - as_str() (via AsRef<str>)
  // - len(), is_empty()
  // - split_to(), split_off(), slice()
  // - range_bounds_to_start_end(), validate_char_boundary()
  //
  // Custom implementations provided above for:
  // - try_split_to(), try_split_off(), try_slice()
}

impl Utf8BufMut for Utf8BytesMut {
  // Custom implementations provided above for:
  // - push(), push_str(), clear()
}
