use core::{
  borrow::Borrow,
  ops::RangeBounds,
  str,
};

use super::{buffer::Buffer, error::*, utf8_buf::{Utf8Buf, Utf8BufMut}};

mod cmp;
mod fmt;
mod from;
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

/// A UTF-8 validated wrapper around [`Buffer`] that provides a String-like interface.
///
/// `Utf8Buffer` guarantees that its contents are always valid UTF-8, making it safe
/// to use string operations without additional validation. It provides methods for
/// splitting on character boundaries and ensures all operations maintain UTF-8 validity.
///
/// # Examples
///
/// ```
/// use smol_bytes::Utf8Buffer;
///
/// let mut buf = Utf8Buffer::new();
/// buf.push_str("Hello");
/// buf.push(' ');
/// buf.push_str("world!");
///
/// assert_eq!(buf.as_str(), "Hello world!");
/// ```
#[derive(Clone, Copy)]
#[cfg_attr(feature = "pyo3", ::pyo3::prelude::pyclass)]
pub struct Utf8Buffer {
  inner: Buffer,
}

impl Default for Utf8Buffer {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::new()
  }
}

impl Utf8Buffer {
  /// Creates a new, empty `Utf8Buffer`.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Buffer;
  ///
  /// let buf = Utf8Buffer::new();
  /// assert!(buf.is_empty());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self {
      inner: Buffer::new(),
    }
  }

  /// Returns a reference to the inner `Buffer`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_buffer(&self) -> &Buffer {
    &self.inner
  }

  /// Consumes `self` and returns the inner `Buffer`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn into_buffer(self) -> Buffer {
    self.inner
  }

  /// Returns the contents as a string slice.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Buffer;
  ///
  /// let buf = Utf8Buffer::from("hello");
  /// assert_eq!(buf.as_str(), "hello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_str(&self) -> &str {
    // SAFETY: Utf8Buffer guarantees valid UTF-8
    unsafe { str::from_utf8_unchecked(self.inner.as_slice()) }
  }

  /// Returns the length of the buffer in bytes.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Buffer;
  ///
  /// let buf = Utf8Buffer::from("hello");
  /// assert_eq!(buf.len(), 5);
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn len(&self) -> usize {
    self.inner.len()
  }

  /// Returns `true` if the buffer has a length of 0.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Buffer;
  ///
  /// let buf = Utf8Buffer::new();
  /// assert!(buf.is_empty());
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  /// Returns the number of remaining bytes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn remaining(&self) -> usize {
    self.inner.remaining()
  }

  /// Returns the capacity of the buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn capacity(&self) -> usize {
    self.inner.capacity()
  }

  /// Appends a character to the buffer.
  ///
  /// # Panics
  ///
  /// Panics if there is not enough capacity to hold the character.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Buffer;
  ///
  /// let mut buf = Utf8Buffer::new();
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
  /// # Panics
  ///
  /// Panics if there is not enough capacity to hold the string.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Buffer;
  ///
  /// let mut buf = Utf8Buffer::new();
  /// buf.push_str("hello");
  /// assert_eq!(buf.as_str(), "hello");
  /// ```
  pub fn push_str(&mut self, s: &str) {
    self.inner.put_slice(s.as_bytes());
  }

  /// Tries to append a character to the buffer.
  ///
  /// # Errors
  ///
  /// Returns an error if there is not enough capacity.
  pub fn try_push(&mut self, ch: char) -> Result<(), TryPutError> {
    let mut buf = [0u8; 4];
    let s = ch.encode_utf8(&mut buf);
    self.inner.try_put_slice(s.as_bytes())
  }

  /// Tries to append a string slice to the buffer.
  ///
  /// # Errors
  ///
  /// Returns an error if there is not enough capacity.
  pub fn try_push_str(&mut self, s: &str) -> Result<(), TryPutError> {
    self.inner.try_put_slice(s.as_bytes())
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

}

impl AsRef<str> for Utf8Buffer {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl AsRef<[u8]> for Utf8Buffer {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_ref(&self) -> &[u8] {
    self.as_str().as_bytes()
  }
}

impl Borrow<str> for Utf8Buffer {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

impl core::ops::Deref for Utf8Buffer {
  type Target = str;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    self.as_str()
  }
}

impl Utf8Buf for Utf8Buffer {
  fn try_split_to(&mut self, at: usize) -> Result<Self, Utf8Error> {
    self.validate_char_boundary(at)?;

    Ok(Self {
      inner: self.inner.try_split_to(at).expect("already checked bounds"),
    })
  }

  fn try_split_off(&mut self, at: usize) -> Result<Self, Utf8Error> {
    self.validate_char_boundary(at)?;

    Ok(Self {
      inner: self.inner.try_split_off(at).expect("already checked bounds"),
    })
  }

  fn try_slice(&self, range: impl RangeBounds<usize>) -> Result<Self, Utf8Error> {
    let (start, end) = self.range_bounds_to_start_end(range)?;

    Ok(Self {
      inner: self.inner.try_slice(start..end).expect("already checked bounds"),
    })
  }
}

impl Utf8BufMut for Utf8Buffer {
  fn push(&mut self, ch: char) {
    let mut buf = [0u8; 4];
    let s = ch.encode_utf8(&mut buf);
    self.inner.put_slice(s.as_bytes());
  }

  fn push_str(&mut self, s: &str) {
    self.inner.put_slice(s.as_bytes());
  }

  fn clear(&mut self) {
    self.inner.clear();
  }
}
