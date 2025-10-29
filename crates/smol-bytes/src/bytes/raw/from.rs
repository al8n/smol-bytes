use super::*;

impl<S, const N: usize> From<[u8; N]> for RawBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(array: [u8; N]) -> Self {
    Self::from(&array)
  }
}

impl<S, const N: usize> From<&[u8; N]> for RawBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(array: &[u8; N]) -> Self {
    if N <= INLINE_CAP {
      // SAFETY: N is guaranteed to be less than or equal to INLINE_CAP
      Self::inline(unsafe { Buffer::copy_from_slice(array) })
    } else {
      Self::heap(::bytes::Bytes::copy_from_slice(array.as_slice()))
    }
  }
}

impl<S> From<&[u8]> for RawBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(slice: &[u8]) -> Self {
    Self::copy_from_slice(slice)
  }
}

impl<S> From<&str> for RawBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(v: &str) -> Self {
    Self::copy_from_slice(v.as_bytes())
  }
}

impl<S> From<Box<[u8]>> for RawBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(slice: Box<[u8]>) -> Self {
    Self::new_in(Repr::from_box(slice))
  }
}

impl<S> From<Vec<u8>> for RawBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(vec: Vec<u8>) -> Self {
    Self::new_in(Repr::from_vec(vec))
  }
}

impl<S> From<String> for RawBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(s: String) -> Self {
    Self::from(s.into_bytes())
  }
}

impl<S> From<Arc<[u8]>> for RawBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(arc: Arc<[u8]>) -> Self {
    Self::new_in(Repr::from_arc(arc))
  }
}

impl<'a, S> From<Cow<'a, [u8]>> for RawBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(cow: Cow<'a, [u8]>) -> Self {
    match cow {
      Cow::Borrowed(slice) => RawBytes::copy_from_slice(slice),
      Cow::Owned(vec) => RawBytes::from(vec),
    }
  }
}

impl<S> From<Buffer> for RawBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(value: Buffer) -> Self {
    Self::inline(value)
  }
}

impl<S> From<RawBytes<S>> for Vec<u8>
where
  RawBytes<S>: Strategy,
{
  #[inline]
  fn from(bytes: RawBytes<S>) -> Self {
    bytes.into_vec()
  }
}

impl<S> From<RawBytes<S>> for Arc<[u8]>
where
  RawBytes<S>: Strategy,
{
  #[inline]
  fn from(bytes: RawBytes<S>) -> Self {
    bytes.into_arc()
  }
}

impl<S> From<RawBytes<S>> for Bytes
where
  RawBytes<S>: Strategy,
{
  #[inline]
  fn from(bytes: RawBytes<S>) -> Self {
    bytes.into_bytes()
  }
}

impl<S> From<BytesMut> for RawBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(v: BytesMut) -> Self {
    v.freeze()
  }
}

impl<S> From<::bytes::BytesMut> for RawBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn from(v: ::bytes::BytesMut) -> Self {
    Self::from(BytesMut::from_bytes_mut(v))
  }
}
