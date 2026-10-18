use core::{borrow::Borrow, convert::Infallible, ops::RangeBounds, str};

use super::{
  BytesMut,
  error::*,
  utf8_buf::{Utf8Buf, Utf8BufMut, range_bounds_to_start_end, sealed, validate_char_boundary},
};
use bytes::BufMut;

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

#[cfg(feature = "pyo3")]
mod python;

#[cfg(feature = "wasm")]
mod wasm;

#[doc = "Growable mutable UTF-8 string with inline/heap storage."]
#[doc = ""]
#[doc = "Guarantees valid UTF-8. Split operations check char boundaries."]
#[cfg_attr(not(feature = "wasm"), doc = "")]
#[cfg_attr(not(feature = "wasm"), doc = "# Examples")]
#[cfg_attr(not(feature = "wasm"), doc = "")]
#[cfg_attr(not(feature = "wasm"), doc = "```")]
#[cfg_attr(not(feature = "wasm"), doc = "use smol_bytes::Utf8BytesMut;")]
#[cfg_attr(not(feature = "wasm"), doc = "")]
#[cfg_attr(not(feature = "wasm"), doc = "let mut buf = Utf8BytesMut::new();")]
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
#[cfg_attr(feature = "wasm", doc = "import { Utf8BytesMut } from 'smol-bytes';")]
#[cfg_attr(feature = "wasm", doc = "const buf = new Utf8BytesMut();")]
#[cfg_attr(feature = "wasm", doc = "buf.pushStr('hello world');")]
#[cfg_attr(
  feature = "wasm",
  doc = "console.log(buf.toString()); // 'hello world'"
)]
#[cfg_attr(feature = "wasm", doc = "```")]
#[derive(Clone)]
#[cfg_attr(feature = "pyo3", ::pyo3::prelude::pyclass(from_py_object))]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Utf8BytesMut {
  pub(crate) inner: BytesMut,
}

impl Default for Utf8BytesMut {
  fn default() -> Self {
    Self::new()
  }
}

impl sealed::Sealed for Utf8BytesMut {}

impl core::fmt::Write for Utf8BytesMut {
  #[inline(always)]
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    self.push_str(s);
    Ok(())
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
  /// const EMPTY: Utf8BytesMut = Utf8BytesMut::new();
  /// assert!(EMPTY.is_empty());
  /// ```
  #[inline(always)]
  pub const fn new() -> Self {
    Self {
      inner: BytesMut::new(),
    }
  }

  /// Validates owned mutable bytes as UTF-8 without discarding them on
  /// failure.
  pub fn from_utf8(bytes: BytesMut) -> Result<Self, FromUtf8Error<BytesMut>> {
    match core::str::from_utf8(bytes.as_ref()) {
      Ok(_) => Ok(Self { inner: bytes }),
      Err(error) => Err(FromUtf8Error::new(bytes, error)),
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
  ///
  /// Note: named `as_inner` to avoid confusion with `str::as_bytes`/
  /// `String::as_bytes_mut`, which users might expect to return `&[u8]`.
  #[inline(always)]
  pub const fn as_inner(&self) -> &BytesMut {
    &self.inner
  }

  /// Consumes `self` and returns the inner `BytesMut`.
  #[inline(always)]
  pub fn into_inner(self) -> BytesMut {
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
  #[inline(always)]
  pub fn as_str(&self) -> &str {
    // SAFETY: Utf8BytesMut guarantees valid UTF-8
    unsafe { str::from_utf8_unchecked(self.inner.as_ref()) }
  }

  /// Returns the contents as a mutable string slice.
  ///
  /// Mutations must preserve UTF-8 validity and the byte length of edited
  /// ranges, as required by [`prim@str`].
  pub fn as_mut_str(&mut self) -> &mut str {
    str::from_utf8_mut(self.inner.as_mut_slice()).expect("UTF-8 invariant")
  }

  /// Returns the length in bytes.
  #[inline(always)]
  pub fn len(&self) -> usize {
    self.inner.len()
  }

  /// Returns `true` if the buffer has length 0.
  #[cfg_attr(not(coverage), inline(always))]
  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  /// Returns the number of bytes the buffer can hold without reallocating.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let buf = Utf8BytesMut::with_capacity(128);
  /// assert!(buf.capacity() >= 128);
  /// ```
  #[inline]
  pub fn capacity(&self) -> usize {
    self.inner.capacity()
  }

  /// Returns `true` if the underlying bytes are stored inline (≤ 62 bytes).
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let small = Utf8BytesMut::from("hi");
  /// assert!(small.is_inline());
  /// ```
  #[inline]
  pub fn is_inline(&self) -> bool {
    self.inner.is_inline()
  }

  /// Returns `true` if the underlying bytes are heap-allocated.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let big = Utf8BytesMut::with_capacity(128);
  /// assert!(big.is_heap());
  /// ```
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
    <Self as Utf8BufMut>::push(self, ch)
  }

  /// Appends a character without allocation failure.
  pub fn try_push(&mut self, ch: char) -> Result<(), Infallible> {
    <Self as Utf8BufMut>::try_push(self, ch)
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
    <Self as Utf8BufMut>::push_str(self, s)
  }

  /// Appends a string slice without allocation failure.
  pub fn try_push_str(&mut self, s: &str) -> Result<(), Infallible> {
    <Self as Utf8BufMut>::try_push_str(self, s)
  }

  /// Clears the buffer, removing all contents.
  pub fn clear(&mut self) {
    <Self as Utf8BufMut>::clear(self)
  }

  /// Truncates the buffer to the specified length.
  ///
  /// # Panics
  ///
  /// Panics if `new_len` does not lie on a UTF-8 character boundary.
  pub fn truncate(&mut self, new_len: usize) {
    self
      .try_truncate(new_len)
      .expect("new_len must lie on a UTF-8 character boundary");
  }

  /// Truncates the buffer at a UTF-8 character boundary.
  pub fn try_truncate(&mut self, new_len: usize) -> Result<(), Utf8IndexError> {
    <Self as Utf8BufMut>::try_truncate(self, new_len)
  }

  /// Freezes this mutable value using the default shared immutable strategy.
  ///
  /// Heap-backed values retain their allocation. [`freeze_compact`](Self::freeze_compact)
  /// also transfers existing heap storage without copying, including a short current view;
  /// its compact strategy applies to later shortening or view operations.
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let text = Utf8BytesMut::from("hello").freeze();
  /// assert_eq!(text.as_str(), "hello");
  /// ```
  pub fn freeze(self) -> crate::Utf8Bytes {
    crate::utf8_bytes::Utf8Bytes {
      inner: self.inner.freeze(),
    }
  }

  /// Freezes this mutable value using the shared immutable strategy.
  pub fn freeze_shared(self) -> crate::shared::Utf8Bytes {
    crate::utf8_bytes::Utf8Bytes {
      inner: self.inner.freeze_with(),
    }
  }

  /// Freezes this mutable value using the compact immutable strategy.
  ///
  /// Existing heap storage is transferred without copying. In particular, a
  /// short heap view is not immediately canonicalized; the compact strategy
  /// applies to subsequent shortening or view operations.
  pub fn freeze_compact(self) -> crate::compact::Utf8Bytes {
    crate::utf8_bytes::Utf8Bytes {
      inner: self.inner.freeze_with(),
    }
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
  pub fn try_split_to(&mut self, at: usize) -> Result<Self, Utf8IndexError> {
    <Self as Utf8Buf>::try_split_to(self, at)
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
  pub fn try_split_off(&mut self, at: usize) -> Result<Self, Utf8IndexError> {
    <Self as Utf8Buf>::try_split_off(self, at)
  }

  /// Removes all bytes from the buffer and returns them in a new buffer.
  ///
  /// Afterwards, `self` will be empty, retaining any additional capacity
  /// it had before the split (see [`bytes::BytesMut::split`]).
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let mut buf = Utf8BytesMut::with_capacity(128);
  /// buf.push_str("hello");
  /// let data = buf.split();
  ///
  /// assert_eq!(data.as_str(), "hello");
  /// assert_eq!(buf.len(), 0);
  /// ```
  pub fn split(&mut self) -> Self {
    let inner = self.inner.split();
    Self { inner }
  }

  /// Merges another buffer back into this one.
  ///
  /// Heap-backed contiguous halves merge without copying. Inline and mixed
  /// pairs append their contents, promoting storage when necessary.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let mut buf = Utf8BytesMut::with_capacity(128);
  /// buf.push_str("hello world");
  /// let tail = buf.split_off(5);
  /// buf.unsplit(tail);
  /// assert_eq!(buf.as_str(), "hello world");
  /// ```
  ///
  /// Inline buffers append too:
  ///
  /// ```
  /// use smol_bytes::Utf8BytesMut;
  ///
  /// let mut buf = Utf8BytesMut::from("hi"); // inline
  /// let tail = Utf8BytesMut::from("!");
  /// buf.unsplit(tail);
  /// assert_eq!(buf.as_str(), "hi!");
  /// ```
  pub fn unsplit(&mut self, other: Self) {
    self.inner.unsplit(other.inner)
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
  pub fn try_slice(&self, range: impl RangeBounds<usize>) -> Result<Self, Utf8IndexError> {
    <Self as Utf8Buf>::try_slice(self, range)
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

#[cfg(feature = "std")]
impl AsRef<std::ffi::OsStr> for Utf8BytesMut {
  fn as_ref(&self) -> &std::ffi::OsStr {
    std::ffi::OsStr::new(self.as_str())
  }
}

#[cfg(feature = "std")]
impl AsRef<std::path::Path> for Utf8BytesMut {
  fn as_ref(&self) -> &std::path::Path {
    std::path::Path::new(self.as_str())
  }
}

// The trait impls hold the real bodies. Inherent methods (above) forward
// here via explicit `<Self as Utf8Buf>::method` / `<Self as Utf8BufMut>::method`
// UFCS so there is no ambiguity about which impl is called and no fragile
// reliance on the inherent-over-trait method resolution rule.
impl Utf8Buf for Utf8BytesMut {
  fn try_split_to(&mut self, at: usize) -> Result<Self, Utf8IndexError> {
    validate_char_boundary(self.as_str(), at)?;
    let inner = self.inner.split_to(at);
    Ok(Self { inner })
  }

  fn try_split_off(&mut self, at: usize) -> Result<Self, Utf8IndexError> {
    validate_char_boundary(self.as_str(), at)?;
    let inner = self.inner.split_off(at);
    Ok(Self { inner })
  }

  fn try_slice(&self, range: impl RangeBounds<usize>) -> Result<Self, Utf8IndexError> {
    let (start, end) = range_bounds_to_start_end(self.as_str(), range)?;
    Ok(Self {
      inner: BytesMut::from(&self.inner.as_slice()[start..end]),
    })
  }
}

impl Utf8BufMut for Utf8BytesMut {
  type CapacityError = Infallible;

  fn try_truncate(&mut self, new_len: usize) -> Result<(), Utf8IndexError> {
    if new_len >= self.len() {
      return Ok(());
    }
    if !self.as_str().is_char_boundary(new_len) {
      return Err(Utf8IndexError::InvalidCharBoundary { index: new_len });
    }
    self.inner.truncate(new_len);
    Ok(())
  }

  fn try_push(&mut self, ch: char) -> Result<(), Self::CapacityError> {
    let mut buf = [0u8; 4];
    let s = ch.encode_utf8(&mut buf);
    self.inner.put_slice(s.as_bytes());
    Ok(())
  }

  fn try_push_str(&mut self, s: &str) -> Result<(), Self::CapacityError> {
    self.inner.put_slice(s.as_bytes());
    Ok(())
  }

  fn clear(&mut self) {
    self.inner.clear();
  }
}
