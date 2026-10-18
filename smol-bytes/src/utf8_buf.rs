use core::ops::RangeBounds;

use super::error::{OutOfBounds, Utf8IndexError, normalize_range};

/// Implementation detail that prevents downstream crates from implementing
/// the UTF-8 buffer traits.
pub(crate) mod sealed {
  pub trait Sealed {}
}

/// Extension trait for UTF-8 validated buffer types.
///
/// This trait provides common functionality for buffer types that guarantee
/// valid UTF-8 content, such as `Utf8Buffer`, `Utf8Bytes`, and `Utf8BytesMut`.
pub trait Utf8Buf: sealed::Sealed + AsRef<str> + Sized {
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
  fn try_split_to(&mut self, at: usize) -> Result<Self, Utf8IndexError>;

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
  fn try_split_off(&mut self, at: usize) -> Result<Self, Utf8IndexError>;

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
  fn try_slice(&self, range: impl RangeBounds<usize>) -> Result<Self, Utf8IndexError>;
}

/// Extension trait for mutable UTF-8 validated buffer types.
///
/// This trait provides common mutable operations for buffer types that can be
/// modified while maintaining UTF-8 validity.
pub trait Utf8BufMut: Utf8Buf {
  /// The error returned when an append cannot fit.
  type CapacityError: core::error::Error;

  /// Truncates at a UTF-8 character boundary.
  ///
  /// A length at or beyond the current byte length is a no-op. On failure the
  /// buffer is unchanged.
  fn try_truncate(&mut self, new_len: usize) -> Result<(), Utf8IndexError>;

  /// Attempts to append a character.
  fn try_push(&mut self, ch: char) -> Result<(), Self::CapacityError>;

  /// Attempts to append a string slice.
  fn try_push_str(&mut self, s: &str) -> Result<(), Self::CapacityError>;

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
  fn push(&mut self, ch: char) {
    if let Err(error) = self.try_push(ch) {
      panic!("UTF-8 buffer capacity exceeded: {error}");
    }
  }

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
  fn push_str(&mut self, s: &str) {
    if let Err(error) = self.try_push_str(s) {
      panic!("UTF-8 buffer capacity exceeded: {error}");
    }
  }

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

/// Validates that `at` is a visible UTF-8 character boundary.
pub(crate) fn validate_char_boundary(s: &str, at: usize) -> Result<(), Utf8IndexError> {
  let len = s.len();
  if at > len {
    return Err(OutOfBounds::new(at, len).into());
  }
  if !s.is_char_boundary(at) {
    return Err(Utf8IndexError::InvalidCharBoundary { index: at });
  }
  Ok(())
}

/// Normalizes a range and validates both UTF-8 character boundaries.
pub(crate) fn range_bounds_to_start_end(
  s: &str,
  range: impl RangeBounds<usize>,
) -> Result<(usize, usize), Utf8IndexError> {
  let (start, end) = normalize_range(range, s.len())?;
  if !s.is_char_boundary(start) {
    return Err(Utf8IndexError::InvalidCharBoundary { index: start });
  }
  if !s.is_char_boundary(end) {
    return Err(Utf8IndexError::InvalidCharBoundary { index: end });
  }
  Ok((start, end))
}

/// Maps a Unicode-scalar offset to its byte offset in `s`.
///
/// `chars == number of chars` maps to `s.len()`; anything larger is out of range.
///
/// Kept crate-level (rather than inside the `pyo3`-gated bindings) so its unit
/// tests run under the default test configuration. Only the Python bindings call
/// it, so it is dead code in non-`pyo3`, non-test builds.
#[allow(dead_code)]
pub(crate) fn char_offset_to_byte(s: &str, chars: usize) -> Option<usize> {
  if chars == s.chars().count() {
    Some(s.len())
  } else {
    s.char_indices().nth(chars).map(|(index, _)| index)
  }
}

#[cfg(test)]
mod char_offset_tests {
  use super::char_offset_to_byte;

  #[test]
  fn maps_scalar_offset_to_byte_offset() {
    assert_eq!(char_offset_to_byte("abc", 1), Some(1));
    assert_eq!(char_offset_to_byte("é", 1), Some(2));
    assert_eq!(char_offset_to_byte("aé", 1), Some(1));
    assert_eq!(char_offset_to_byte("aé", 2), Some(3));
    assert_eq!(char_offset_to_byte("", 0), Some(0));
    assert_eq!(char_offset_to_byte("a", 2), None);
    assert_eq!(char_offset_to_byte("🦀x", 1), Some(4));
  }
}
