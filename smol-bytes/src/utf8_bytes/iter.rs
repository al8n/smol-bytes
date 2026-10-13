use crate::utf8_bytes_mut::Utf8BytesMut;

use super::*;

impl<S, T> FromIterator<T> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
  Utf8BytesMut: Extend<T>,
{
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let mut bytes = Utf8BytesMut::new();
    bytes.extend(iter);
    Self {
      inner: bytes.inner.freeze::<S>(),
    }
  }
}
