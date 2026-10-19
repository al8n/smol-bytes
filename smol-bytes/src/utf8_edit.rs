use core::ops::Range;

use crate::Buffer;

#[cfg(any(feature = "alloc", feature = "std"))]
use crate::BytesMut;

/// Private mutable storage shared by UTF-8 editing operations.
pub(crate) enum RawUtf8Storage<'a> {
  Fixed(&'a mut Buffer),
  #[cfg(any(feature = "alloc", feature = "std"))]
  Growable(&'a mut BytesMut),
}

impl<'a> RawUtf8Storage<'a> {
  pub(crate) fn fixed(storage: &'a mut Buffer) -> Self {
    Self::Fixed(storage)
  }

  #[cfg(any(feature = "alloc", feature = "std"))]
  pub(crate) fn growable(storage: &'a mut BytesMut) -> Self {
    Self::Growable(storage)
  }

  pub(crate) fn len(&self) -> usize {
    match self {
      Self::Fixed(storage) => storage.len(),
      #[cfg(any(feature = "alloc", feature = "std"))]
      Self::Growable(storage) => storage.len(),
    }
  }

  pub(crate) fn as_slice(&self) -> &[u8] {
    match self {
      Self::Fixed(storage) => storage.as_slice(),
      #[cfg(any(feature = "alloc", feature = "std"))]
      Self::Growable(storage) => storage.as_slice(),
    }
  }

  pub(crate) fn as_mut_slice(&mut self) -> &mut [u8] {
    match self {
      Self::Fixed(storage) => storage.as_mut_slice(),
      #[cfg(any(feature = "alloc", feature = "std"))]
      Self::Growable(storage) => storage.as_mut_slice(),
    }
  }

  pub(crate) fn resize(&mut self, len: usize) {
    match self {
      Self::Fixed(storage) => storage.resize(len, 0),
      #[cfg(any(feature = "alloc", feature = "std"))]
      Self::Growable(storage) => storage.resize(len, 0),
    }
  }

  pub(crate) fn truncate(&mut self, len: usize) {
    match self {
      Self::Fixed(storage) => storage.truncate(len),
      #[cfg(any(feature = "alloc", feature = "std"))]
      Self::Growable(storage) => storage.truncate(len),
    }
  }

  pub(crate) fn copy_within(&mut self, source: Range<usize>, destination: usize) {
    self.as_mut_slice().copy_within(source, destination);
  }
}

pub(crate) fn replace_bytes(
  storage: &mut RawUtf8Storage<'_>,
  start: usize,
  end: usize,
  replacement: &[u8],
) {
  let old_len = storage.len();
  let removed = end - start;
  let new_len = old_len
    .checked_sub(removed)
    .and_then(|len| len.checked_add(replacement.len()))
    .expect("buffer length overflow");

  if new_len > old_len {
    storage.resize(new_len);
  }

  storage.copy_within(end..old_len, start + replacement.len());
  storage.as_mut_slice()[start..start + replacement.len()].copy_from_slice(replacement);

  if new_len < old_len {
    storage.truncate(new_len);
  }
}

pub(crate) fn scalar_width(first: u8) -> usize {
  match first {
    0x00..=0x7f => 1,
    0xc2..=0xdf => 2,
    0xe0..=0xef => 3,
    0xf0..=0xf4 => 4,
    _ => panic!("UTF-8 invariant violated"),
  }
}

pub(crate) struct RetainGuard<'a> {
  storage: RawUtf8Storage<'a>,
  read: usize,
  write: usize,
  original_len: usize,
}

impl<'a> RetainGuard<'a> {
  pub(crate) fn new(storage: RawUtf8Storage<'a>) -> Self {
    let original_len = storage.len();
    Self {
      storage,
      read: 0,
      write: 0,
      original_len,
    }
  }

  pub(crate) fn retain<F>(&mut self, predicate: &mut F)
  where
    F: FnMut(char) -> bool,
  {
    while self.read < self.original_len {
      let next_read = {
        let bytes = self.storage.as_slice();
        let width = scalar_width(bytes[self.read]);
        self.read + width
      };
      let character = {
        let bytes = self.storage.as_slice();
        core::str::from_utf8(&bytes[self.read..next_read])
          .expect("UTF-8 invariant violated")
          .chars()
          .next()
          .expect("UTF-8 invariant violated")
      };

      if predicate(character) {
        self.storage.copy_within(self.read..next_read, self.write);
        self.write += next_read - self.read;
      }
      self.read = next_read;
    }
  }
}

impl Drop for RetainGuard<'_> {
  fn drop(&mut self) {
    self
      .storage
      .copy_within(self.read..self.original_len, self.write);
    self
      .storage
      .truncate(self.write + (self.original_len - self.read));
  }
}
