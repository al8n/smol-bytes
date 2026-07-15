use crate::strategy::ImmutableStorage;

use super::*;

impl<'a> Extend<&'a [u8]> for BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn extend<T: IntoIterator<Item = &'a [u8]>>(&mut self, iter: T) {
    for slice in iter {
      self.extend_from_slice(slice);
    }
  }
}

impl<'a> Extend<&'a u8> for BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn extend<T: IntoIterator<Item = &'a u8>>(&mut self, iter: T) {
    self.extend(iter.into_iter().copied());
  }
}

impl Extend<u8> for BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
    let iter = iter.into_iter();

    let (lower, _) = iter.size_hint();
    self.reserve(lower);

    for b in iter {
      self.put_u8(b);
    }
  }
}

impl Extend<bytes::Bytes> for BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn extend<T: IntoIterator<Item = bytes::Bytes>>(&mut self, iter: T) {
    for bytes in iter {
      self.extend_from_slice(&bytes);
    }
  }
}

impl<S> Extend<RawBytes<S>> for BytesMut
where
  RawBytes<S>: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn extend<T: IntoIterator<Item = RawBytes<S>>>(&mut self, iter: T) {
    for smol_bytes in iter {
      self.extend_from_slice(smol_bytes.as_ref());
    }
  }
}

impl FromIterator<u8> for BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
    let mut smol_bytes_mut = BytesMut::new();
    smol_bytes_mut.extend(iter);
    smol_bytes_mut
  }
}

impl<'a> FromIterator<&'a u8> for BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn from_iter<T: IntoIterator<Item = &'a u8>>(iter: T) -> Self {
    Self::from_iter(iter.into_iter().copied())
  }
}

impl<'a> FromIterator<&'a [u8]> for BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn from_iter<T: IntoIterator<Item = &'a [u8]>>(iter: T) -> Self {
    let mut smol_bytes_mut = BytesMut::new();

    for slice in iter {
      smol_bytes_mut.extend_from_slice(slice);
    }
    smol_bytes_mut
  }
}

impl IntoIterator for BytesMut {
  type Item = u8;
  type IntoIter = ::bytes::buf::IntoIter<BytesMut>;

  #[cfg_attr(not(coverage), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    ::bytes::buf::IntoIter::new(self)
  }
}

impl<'a> IntoIterator for &'a BytesMut {
  type Item = &'a u8;
  type IntoIter = core::slice::Iter<'a, u8>;

  #[cfg_attr(not(coverage), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    self.as_ref().iter()
  }
}
