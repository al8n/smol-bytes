use core::ops::RangeBounds;

use super::error::{normalize_range, Utf8Error};

/// Extension trait for UTF-8 validated buffer types.
///
/// This trait provides common functionality for buffer types that guarantee
/// valid UTF-8 content, such as `Utf8Buffer`, `Utf8Bytes`, and `Utf8BytesMut`.
pub trait Utf8Buf: AsRef<str> + Sized {
  /// Returns the contents as a string slice.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::{Utf8Buf, Utf8Bytes};
  ///
  /// let buf = Utf8Bytes::from("hello");
  /// assert_eq!(buf.as_str(), "hello");
  /// ```
  fn as_str(&self) -> &str {
    self.as_ref()
  }

  /// Returns the length of the buffer in bytes.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::{Utf8Buf, Utf8Bytes};
  ///
  /// let buf = Utf8Bytes::from("hello");
  /// assert_eq!(buf.len(), 5);
  /// ```
  fn len(&self) -> usize {
    self.as_str().len()
  }

  /// Returns `true` if the buffer has a length of 0.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::{Utf8Buf, Utf8Bytes};
  ///
  /// let buf = Utf8Bytes::new();
  /// assert!(buf.is_empty());
  /// ```
  fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Splits the buffer at the given index, returning the content before the split point.
  ///
  /// # Panics
  ///
  /// Panics if `at` does not lie on a UTF-8 character boundary or is out of bounds.
  fn split_to(&mut self, at: usize) -> Self {
    self.try_split_to(at).expect("split_to failed")
  }

  /// Tries to split the buffer at the given index.
  ///
  /// # Errors
  ///
  /// Returns an error if `at` is out of bounds or does not lie on a UTF-8 character boundary.
  fn try_split_to(&mut self, at: usize) -> Result<Self, Utf8Error>;

  /// Splits the buffer at the given index, returning the content after the split point.
  ///
  /// # Panics
  ///
  /// Panics if `at` does not lie on a UTF-8 character boundary or is out of bounds.
  fn split_off(&mut self, at: usize) -> Self {
    self.try_split_off(at).expect("split_off failed")
  }

  /// Tries to split the buffer at the given index, returning the tail.
  ///
  /// # Errors
  ///
  /// Returns an error if `at` is out of bounds or does not lie on a UTF-8 character boundary.
  fn try_split_off(&mut self, at: usize) -> Result<Self, Utf8Error>;

  /// Returns a sub-slice of the buffer.
  ///
  /// # Panics
  ///
  /// Panics if the range does not lie on UTF-8 character boundaries or is out of bounds.
  fn slice(&self, range: impl RangeBounds<usize>) -> Self {
    self.try_slice(range).expect("slice failed")
  }

  /// Tries to return a sub-slice of the buffer.
  ///
  /// # Errors
  ///
  /// Returns an error if the range is out of bounds or does not lie on UTF-8 character boundaries.
  fn try_slice(&self, range: impl RangeBounds<usize>) -> Result<Self, Utf8Error>;

  /// Helper function to convert range bounds to start and end indices.
  ///
  /// This is used internally by slice operations to validate and normalize ranges.
  fn range_bounds_to_start_end(
    &self,
    range: impl RangeBounds<usize>,
  ) -> Result<(usize, usize), Utf8Error> {
    let len = self.len();
    let (start, end) =
      normalize_range(range, len).map_err(|err| Utf8Error::OutOfBounds { at: err.end, len })?;

    let s = self.as_str();
    if !s.is_char_boundary(start) {
      return Err(Utf8Error::InvalidCharBoundary { at: start });
    }
    if !s.is_char_boundary(end) {
      return Err(Utf8Error::InvalidCharBoundary { at: end });
    }

    Ok((start, end))
  }

  /// Validates that an index is within bounds and on a character boundary.
  ///
  /// This is used internally by split operations.
  fn validate_char_boundary(&self, at: usize) -> Result<(), Utf8Error> {
    let len = self.len();

    if at > len {
      return Err(Utf8Error::OutOfBounds { at, len });
    }

    if !self.as_str().is_char_boundary(at) {
      return Err(Utf8Error::InvalidCharBoundary { at });
    }

    Ok(())
  }
}

/// Extension trait for mutable UTF-8 validated buffer types.
///
/// This trait provides common mutable operations for buffer types that can be
/// modified while maintaining UTF-8 validity.
pub trait Utf8BufMut: Utf8Buf {
  /// Appends a character to the buffer.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::{Utf8BufMut, Utf8BytesMut};
  ///
  /// let mut buf = Utf8BytesMut::new();
  /// buf.push('a');
  /// buf.push('b');
  /// assert_eq!(buf.as_str(), "ab");
  /// ```
  fn push(&mut self, ch: char);

  /// Appends a string slice to the buffer.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::{Utf8BufMut, Utf8BytesMut};
  ///
  /// let mut buf = Utf8BytesMut::new();
  /// buf.push_str("hello");
  /// assert_eq!(buf.as_str(), "hello");
  /// ```
  fn push_str(&mut self, s: &str);

  /// Clears the buffer, removing all contents.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::{Utf8BufMut, Utf8BytesMut};
  ///
  /// let mut buf = Utf8BytesMut::from("hello");
  /// buf.clear();
  /// assert!(buf.is_empty());
  /// ```
  fn clear(&mut self);
}
