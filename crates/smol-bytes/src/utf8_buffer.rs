use core::{borrow::Borrow, ops::RangeBounds, str};

use super::{
  buffer::Buffer,
  error::*,
  utf8_buf::{Utf8Buf, Utf8BufMut},
};

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

#[cfg(feature = "wasm")]
mod wasm;

#[doc = "UTF-8 validated wrapper around `Buffer` with a String-like interface."]
#[doc = ""]
#[doc = "Guarantees valid UTF-8. Split/slice operations check char boundaries."]
#[doc = "Fixed inline capacity of 62 bytes."]
#[cfg_attr(not(feature = "wasm"), doc = "")]
#[cfg_attr(not(feature = "wasm"), doc = "# Examples")]
#[cfg_attr(not(feature = "wasm"), doc = "")]
#[cfg_attr(not(feature = "wasm"), doc = "```")]
#[cfg_attr(not(feature = "wasm"), doc = "use smol_bytes::Utf8Buffer;")]
#[cfg_attr(not(feature = "wasm"), doc = "")]
#[cfg_attr(not(feature = "wasm"), doc = "let mut buf = Utf8Buffer::new();")]
#[cfg_attr(not(feature = "wasm"), doc = "buf.push_str(\"Hello\");")]
#[cfg_attr(not(feature = "wasm"), doc = "buf.push(' ');")]
#[cfg_attr(not(feature = "wasm"), doc = "buf.push_str(\"world!\");")]
#[cfg_attr(
  not(feature = "wasm"),
  doc = "assert_eq!(buf.as_str(), \"Hello world!\");"
)]
#[cfg_attr(not(feature = "wasm"), doc = "```")]
#[cfg_attr(feature = "wasm", doc = "")]
#[cfg_attr(feature = "wasm", doc = "@example")]
#[cfg_attr(feature = "wasm", doc = "```typescript")]
#[cfg_attr(feature = "wasm", doc = "import { Utf8Buffer } from 'smol-bytes';")]
#[cfg_attr(feature = "wasm", doc = "const buf = new Utf8Buffer();")]
#[cfg_attr(feature = "wasm", doc = "buf.pushStr('hello world');")]
#[cfg_attr(
  feature = "wasm",
  doc = "console.log(buf.toString()); // 'hello world'"
)]
#[cfg_attr(feature = "wasm", doc = "```")]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "pyo3", ::pyo3::prelude::pyclass(skip_from_py_object))]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
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

  /// Creates a `Utf8Buffer` from a static string slice.
  ///
  /// Unlike [`Utf8Bytes::from_static`](crate::Utf8Bytes::from_static),
  /// this method **copies** the bytes into the inline buffer — `Utf8Buffer`
  /// has no heap variant. It exists as a const constructor for parity.
  ///
  /// # Panics
  ///
  /// Panics at compile time (via `const` assertion) if `s.len() > 62`.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Buffer;
  ///
  /// const GREETING: Utf8Buffer = Utf8Buffer::from_static("hello");
  /// assert_eq!(GREETING.as_str(), "hello");
  /// ```
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_static(s: &'static str) -> Self {
    let bytes = s.as_bytes();
    assert!(
      bytes.len() <= crate::INLINE_CAP,
      "string too large for Utf8Buffer inline capacity"
    );
    // SAFETY: length checked against INLINE_CAP above; input is already valid UTF-8.
    Self {
      inner: unsafe { Buffer::copy_from_slice(bytes) },
    }
  }

  /// Returns a reference to the inner `Buffer`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_inner(&self) -> &Buffer {
    &self.inner
  }

  /// Consumes `self` and returns the inner `Buffer`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn into_inner(self) -> Buffer {
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

  /// Returns the number of bytes stored in the buffer.
  ///
  /// This is equivalent to [`len`](Self::len).
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Buffer;
  ///
  /// let buf = Utf8Buffer::from("hi");
  /// assert_eq!(buf.remaining(), 2);
  /// ```
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
  /// `Utf8Buffer` is backed by a fixed 62-byte inline [`Buffer`]; this
  /// method panics when the character would not fit. Use [`try_push`]
  /// for a fallible variant.
  ///
  /// [`try_push`]: Self::try_push
  ///
  /// # Panics
  ///
  /// Panics if the buffer's remaining capacity is less than `ch.len_utf8()`.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Buffer;
  ///
  /// let mut buf = Utf8Buffer::new();
  /// buf.push('a');
  /// buf.push('é'); // 2-byte char
  /// assert_eq!(buf.as_str(), "aé");
  /// ```
  pub fn push(&mut self, ch: char) {
    let mut buf = [0u8; 4];
    let s = ch.encode_utf8(&mut buf);
    self.inner.put_slice(s.as_bytes());
  }

  /// Appends a string slice to the buffer.
  ///
  /// `Utf8Buffer` is backed by a fixed 62-byte inline [`Buffer`]; this
  /// method panics when the string would not fit. Use [`try_push_str`]
  /// for a fallible variant.
  ///
  /// [`try_push_str`]: Self::try_push_str
  ///
  /// # Panics
  ///
  /// Panics if the buffer's remaining capacity is less than `s.len()`.
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
  /// Returns [`TryPutError`] if the buffer's remaining capacity is less
  /// than `ch.len_utf8()`. On error the buffer is unchanged.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Buffer;
  ///
  /// let mut buf = Utf8Buffer::from("a".repeat(60).as_str());
  /// assert!(buf.try_push('a').is_ok());        // 61 bytes
  /// assert!(buf.try_push('a').is_ok());        // 62 bytes
  /// assert!(buf.try_push('a').is_err());       // full
  /// ```
  pub fn try_push(&mut self, ch: char) -> Result<(), TryPutError> {
    let mut buf = [0u8; 4];
    let s = ch.encode_utf8(&mut buf);
    self.inner.try_put_slice(s.as_bytes())
  }

  /// Tries to append a string slice to the buffer.
  ///
  /// # Errors
  ///
  /// Returns [`TryPutError`] if the buffer's remaining capacity is less
  /// than `s.len()`. On error the buffer is unchanged.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Buffer;
  ///
  /// let mut buf = Utf8Buffer::new();
  /// assert!(buf.try_push_str("hello").is_ok());
  /// assert_eq!(buf.as_str(), "hello");
  /// ```
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
      assert!(
        self.as_str().is_char_boundary(new_len),
        "new_len must lie on a UTF-8 character boundary"
      );
    }
    self.inner.truncate(new_len);
  }

  /// Splits the buffer at `at`, returning the head `[0, at)` and leaving
  /// `self` with `[at, len)`.
  ///
  /// # Panics
  ///
  /// Panics if `at` is out of bounds or not on a UTF-8 character boundary.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Buffer;
  ///
  /// let mut buf = Utf8Buffer::from("hello world");
  /// let head = buf.split_to(5);
  /// assert_eq!(head.as_str(), "hello");
  /// assert_eq!(buf.as_str(), " world");
  /// ```
  pub fn split_to(&mut self, at: usize) -> Self {
    <Self as Utf8Buf>::split_to(self, at)
  }

  /// Tries to split the buffer at `at`.
  ///
  /// # Errors
  ///
  /// Returns [`Utf8Error::InvalidCharBoundary`] if `at` is not on a
  /// character boundary, or [`Utf8Error::OutOfBounds`] if `at > len`.
  pub fn try_split_to(&mut self, at: usize) -> Result<Self, Utf8Error> {
    <Self as Utf8Buf>::try_split_to(self, at)
  }

  /// Splits the buffer at `at`, returning the tail `[at, len)` and leaving
  /// `self` with `[0, at)`.
  ///
  /// # Panics
  ///
  /// Panics if `at` is out of bounds or not on a UTF-8 character boundary.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Buffer;
  ///
  /// let mut buf = Utf8Buffer::from("hello world");
  /// let tail = buf.split_off(6);
  /// assert_eq!(buf.as_str(), "hello ");
  /// assert_eq!(tail.as_str(), "world");
  /// ```
  pub fn split_off(&mut self, at: usize) -> Self {
    <Self as Utf8Buf>::split_off(self, at)
  }

  /// Tries to split the buffer at `at`, returning the tail.
  ///
  /// # Errors
  ///
  /// Returns [`Utf8Error::InvalidCharBoundary`] if `at` is not on a
  /// character boundary, or [`Utf8Error::OutOfBounds`] if `at > len`.
  pub fn try_split_off(&mut self, at: usize) -> Result<Self, Utf8Error> {
    <Self as Utf8Buf>::try_split_off(self, at)
  }

  /// Returns a copy of a sub-range of the buffer.
  ///
  /// # Panics
  ///
  /// Panics if the range is out of bounds or not on UTF-8 character boundaries.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8Buffer;
  ///
  /// let buf = Utf8Buffer::from("hello world");
  /// let slice = buf.slice(0..5);
  /// assert_eq!(slice.as_str(), "hello");
  /// ```
  pub fn slice(&self, range: impl core::ops::RangeBounds<usize>) -> Self {
    <Self as Utf8Buf>::slice(self, range)
  }

  /// Tries to return a copy of a sub-range of the buffer.
  ///
  /// # Errors
  ///
  /// Returns [`Utf8Error`] if the range is out of bounds or not on
  /// character boundaries.
  pub fn try_slice(&self, range: impl core::ops::RangeBounds<usize>) -> Result<Self, Utf8Error> {
    <Self as Utf8Buf>::try_slice(self, range)
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
