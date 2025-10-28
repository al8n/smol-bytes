use super::*;

impl<S> From<&[u8]> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(slice: &[u8]) -> Self {
    Self::copy_from_slice(slice)
  }
}

impl<S> From<&str> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(v: &str) -> Self {
    Self::copy_from_slice(v.as_bytes())
  }
}

impl<S> From<Box<[u8]>> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(slice: Box<[u8]>) -> Self {
    Self::new_in(Repr::from_box(slice))
  }
}

impl<S> From<Vec<u8>> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(vec: Vec<u8>) -> Self {
    Self::new_in(Repr::from_vec(vec))
  }
}

impl<S> From<String> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(s: String) -> Self {
    Self::from(s.into_bytes())
  }
}

impl<S> From<Arc<[u8]>> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(arc: Arc<[u8]>) -> Self {
    Self::new_in(Repr::from_arc(arc))
  }
}

impl<'a, S> From<Cow<'a, [u8]>> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(cow: Cow<'a, [u8]>) -> Self {
    match cow {
      Cow::Borrowed(slice) => RawSmolBytes::copy_from_slice(slice),
      Cow::Owned(vec) => RawSmolBytes::from(vec),
    }
  }
}

impl<S> From<RawSmolBytes<S>> for Vec<u8>
where
  RawSmolBytes<S>: Strategy,
{
  #[inline]
  fn from(bytes: RawSmolBytes<S>) -> Self {
    bytes.into_vec()
  }
}

impl<S> From<RawSmolBytes<S>> for Arc<[u8]>
where
  RawSmolBytes<S>: Strategy,
{
  #[inline]
  fn from(bytes: RawSmolBytes<S>) -> Self {
    bytes.into_arc()
  }
}

impl<S> From<RawSmolBytes<S>> for Bytes
where
  RawSmolBytes<S>: Strategy,
{
  #[inline]
  fn from(bytes: RawSmolBytes<S>) -> Self {
    bytes.into_bytes()
  }
}

impl<S> From<SmolBytesMut> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(v: SmolBytesMut) -> Self {
    v.freeze()
  }
}

impl<S> From<::bytes::BytesMut> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(v: ::bytes::BytesMut) -> Self {
    Self::from(SmolBytesMut::from_bytes_mut(v))
  }
}
