use core::{fmt, iter::FusedIterator};

use crate::utf8_edit::{RawUtf8Storage, scalar_width};

/// A borrowing iterator over a UTF-8 range removed when the iterator drops.
///
/// The original range stays in place while this iterator is alive. Dropping it
/// removes the entire selected range, including any characters that were not
/// yielded. Forgetting it leaves the original value unchanged.
///
/// # Examples
///
/// ```
/// use smol_bytes::Utf8Buffer;
///
/// let mut text = Utf8Buffer::try_from("aéz").unwrap();
/// let removed: String = text.drain(1..3).collect();
/// assert_eq!(removed, "é");
/// assert_eq!(text.as_str(), "az");
/// ```
pub struct Utf8Drain<'a> {
  storage: RawUtf8Storage<'a>,
  start: usize,
  end: usize,
  original_len: usize,
  front: usize,
  back: usize,
}

impl<'a> Utf8Drain<'a> {
  pub(crate) fn new(storage: RawUtf8Storage<'a>, start: usize, end: usize) -> Self {
    let original_len = storage.len();
    Self {
      storage,
      start,
      end,
      original_len,
      front: start,
      back: end,
    }
  }

  /// Returns the not-yet-yielded portion of the selected range.
  pub fn as_str(&self) -> &str {
    core::str::from_utf8(&self.storage.as_slice()[self.front..self.back])
      .expect("UTF-8 invariant violated")
  }
}

impl fmt::Debug for Utf8Drain<'_> {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_tuple("Utf8Drain")
      .field(&self.as_str())
      .finish()
  }
}

impl AsRef<str> for Utf8Drain<'_> {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl AsRef<[u8]> for Utf8Drain<'_> {
  fn as_ref(&self) -> &[u8] {
    self.as_str().as_bytes()
  }
}

impl Iterator for Utf8Drain<'_> {
  type Item = char;

  fn next(&mut self) -> Option<Self::Item> {
    if self.front == self.back {
      return None;
    }

    let next = {
      let bytes = self.storage.as_slice();
      self.front + scalar_width(bytes[self.front])
    };
    let character = {
      let bytes = self.storage.as_slice();
      core::str::from_utf8(&bytes[self.front..next])
        .expect("UTF-8 invariant violated")
        .chars()
        .next()
        .expect("UTF-8 invariant violated")
    };
    self.front = next;
    Some(character)
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let bytes = self.back - self.front;
    let minimum = bytes / 4 + usize::from(!bytes.is_multiple_of(4));
    (minimum, Some(bytes))
  }
}

impl DoubleEndedIterator for Utf8Drain<'_> {
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.front == self.back {
      return None;
    }

    let start = {
      let bytes = self.storage.as_slice();
      let mut start = self.back - 1;
      for _ in 0..3 {
        if start == self.front || bytes[start] & 0b1100_0000 != 0b1000_0000 {
          break;
        }
        start -= 1;
      }
      start
    };
    let character = {
      let bytes = self.storage.as_slice();
      core::str::from_utf8(&bytes[start..self.back])
        .expect("UTF-8 invariant violated")
        .chars()
        .next()
        .expect("UTF-8 invariant violated")
    };
    self.back = start;
    Some(character)
  }
}

impl FusedIterator for Utf8Drain<'_> {}

impl Drop for Utf8Drain<'_> {
  fn drop(&mut self) {
    self
      .storage
      .copy_within(self.end..self.original_len, self.start);
    self
      .storage
      .truncate(self.original_len - (self.end - self.start));
  }
}
