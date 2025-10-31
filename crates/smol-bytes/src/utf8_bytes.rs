use core::{
  borrow::Borrow,
  ops::RangeBounds,
  str,
};

use super::{error::*, shared::Bytes, utf8_buf::Utf8Buf};

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

/// A UTF-8 validated wrapper around [`Bytes`] that provides a String-like interface.
///
/// `Utf8Bytes` guarantees that its contents are always valid UTF-8. It supports
/// zero-copy cloning and efficient slicing while maintaining UTF-8 validity.
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
#[derive(Clone)]
#[cfg_attr(feature = "pyo3", ::pyo3::prelude::pyclass)]
pub struct Utf8Bytes {
  pub(crate) inner: Bytes,
}

impl Default for Utf8Bytes {
  fn default() -> Self {
    Self::new()
  }
}

impl Utf8Bytes {
  /// Creates a new, empty `Utf8Bytes`.
  pub const fn new() -> Self {
    Self {
      inner: Bytes::new(),
    }
  }

  /// Creates a `Utf8Bytes` from a static string slice.
  pub const fn from_static(s: &'static str) -> Self {
    Self {
      inner: Bytes::from_static(s.as_bytes()),
    }
  }

  /// Returns a reference to the inner `Bytes`.
  pub const fn as_bytes(&self) -> &Bytes {
    &self.inner
  }

  /// Consumes `self` and returns the inner `Bytes`.
  pub fn into_bytes(self) -> Bytes {
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
    self.validate_char_boundary(at)?;

    Ok(Self {
      inner: self.inner.try_split_to(at).expect("already checked bounds"),
    })
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
    self.validate_char_boundary(at)?;

    Ok(Self {
      inner: self.inner.try_split_off(at).expect("already checked bounds"),
    })
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
    let (start, end) = self.range_bounds_to_start_end(range)?;

    Ok(Self {
      inner: self.inner.try_slice(start..end).expect("already checked bounds"),
    })
  }
}

impl AsRef<str> for Utf8Bytes {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl AsRef<[u8]> for Utf8Bytes {
  fn as_ref(&self) -> &[u8] {
    self.as_str().as_bytes()
  }
}

impl Borrow<str> for Utf8Bytes {
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

impl core::ops::Deref for Utf8Bytes {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    self.as_str()
  }
}

impl Utf8Buf for Utf8Bytes {
  // Default implementations from trait are used for:
  // - as_str() (via AsRef<str>)
  // - len(), is_empty()
  // - split_to(), split_off(), slice()
  // - range_bounds_to_start_end(), validate_char_boundary()
  //
  // Custom implementations provided above for:
  // - try_split_to(), try_split_off(), try_slice()
}
