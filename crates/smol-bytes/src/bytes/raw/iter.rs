use super::*;

impl<'a, S> FromIterator<&'a [u8]> for RawBytes<S>
where
  Self: ImmutableStorage,
{
  fn from_iter<T: IntoIterator<Item = &'a [u8]>>(iter: T) -> Self {
    BytesMut::from_iter(iter).freeze()
  }
}

impl<S> FromIterator<u8> for RawBytes<S>
where
  Self: ImmutableStorage,
{
  fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
    BytesMut::from_iter(iter).freeze()
  }
}

impl<'a, S> FromIterator<&'a u8> for RawBytes<S>
where
  Self: ImmutableStorage,
{
  fn from_iter<T: IntoIterator<Item = &'a u8>>(iter: T) -> Self {
    BytesMut::from_iter(iter).freeze()
  }
}

impl<'a, S> IntoIterator for &'a RawBytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  type Item = &'a u8;
  type IntoIter = core::slice::Iter<'a, u8>;

  #[cfg_attr(not(coverage), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    self.as_ref().iter()
  }
}

impl<S> IntoIterator for RawBytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  type Item = u8;
  type IntoIter = ::bytes::buf::IntoIter<RawBytes<S>>;

  #[cfg_attr(not(coverage), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    ::bytes::buf::IntoIter::new(self)
  }
}
