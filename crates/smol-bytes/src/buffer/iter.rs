use super::Buffer;

impl<'a> IntoIterator for &'a Buffer {
  type Item = &'a u8;
  type IntoIter = core::slice::Iter<'a, u8>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    self.as_slice().iter()
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl IntoIterator for Buffer {
  type Item = u8;
  type IntoIter = ::bytes::buf::IntoIter<Buffer>;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    ::bytes::buf::IntoIter::new(self)
  }
}
