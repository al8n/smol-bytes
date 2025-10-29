use crate::strategy::Strategy;

use super::*;

impl<'a> Extend<&'a [u8]> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn extend<T: IntoIterator<Item = &'a [u8]>>(&mut self, iter: T) {
    for slice in iter {
      self.extend_from_slice(slice);
    }
  }
}

impl Extend<Bytes> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn extend<T: IntoIterator<Item = Bytes>>(&mut self, iter: T) {
    for bytes in iter {
      self.extend_from_slice(&bytes);
    }
  }
}

impl<S> Extend<RawSmolBytes<S>> for SmolBytesMut
where
  RawSmolBytes<S>: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn extend<T: IntoIterator<Item = RawSmolBytes<S>>>(&mut self, iter: T) {
    for smol_bytes in iter {
      self.extend_from_slice(smol_bytes.as_ref());
    }
  }
}

impl Extend<u8> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
    let iter = iter.into_iter();

    let (lower, _) = iter.size_hint();
    self.reserve(lower);

    for b in iter {
      self.put_u8(b);
    }
  }
}

impl FromIterator<u8> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
    let mut smol_bytes_mut = SmolBytesMut::new();
    smol_bytes_mut.extend(iter);
    smol_bytes_mut
  }
}

impl<'a> FromIterator<&'a u8> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_iter<T: IntoIterator<Item = &'a u8>>(iter: T) -> Self {
    Self::from_iter(iter.into_iter().copied())
  }
}

impl<'a> FromIterator<&'a [u8]> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from_iter<T: IntoIterator<Item = &'a [u8]>>(iter: T) -> Self {
    let mut smol_bytes_mut = SmolBytesMut::new();

    for slice in iter {
      smol_bytes_mut.extend_from_slice(slice);
    }
    smol_bytes_mut
  }
}

impl IntoIterator for SmolBytesMut {
  type Item = u8;
  type IntoIter = ::bytes::buf::IntoIter<SmolBytesMut>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    ::bytes::buf::IntoIter::new(self)
  }
}

impl<'a> IntoIterator for &'a SmolBytesMut {
  type Item = &'a u8;
  type IntoIter = core::slice::Iter<'a, u8>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    self.as_ref().iter()
  }
}
