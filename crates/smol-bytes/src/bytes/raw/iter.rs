use super::*;

impl<'a, S> FromIterator<&'a [u8]> for RawSmolBytes<S>
where
  Self: Strategy,
{
  fn from_iter<T: IntoIterator<Item = &'a [u8]>>(iter: T) -> Self {
    SmolBytesMut::from_iter(iter).freeze()
  }
}

impl<S> FromIterator<u8> for RawSmolBytes<S>
where
  Self: Strategy,
{
  fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
    SmolBytesMut::from_iter(iter).freeze()
  }
}

impl<'a, S> FromIterator<&'a u8> for RawSmolBytes<S>
where
  Self: Strategy,
{
  fn from_iter<T: IntoIterator<Item = &'a u8>>(iter: T) -> Self {
    SmolBytesMut::from_iter(iter).freeze()
  }
}

impl<'a, S> IntoIterator for &'a RawSmolBytes<S>
where
  RawSmolBytes<S>: Strategy,
{
  type Item = &'a u8;
  type IntoIter = core::slice::Iter<'a, u8>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    self.as_ref().iter()
  }
}

impl<S> IntoIterator for RawSmolBytes<S>
where
  RawSmolBytes<S>: Strategy,
{
  type Item = u8;
  type IntoIter = ::bytes::buf::IntoIter<RawSmolBytes<S>>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    ::bytes::buf::IntoIter::new(self)
  }
}
