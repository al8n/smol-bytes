use super::Buffer;

impl<'a> IntoIterator for &'a Buffer {
  type Item = &'a u8;
  type IntoIter = core::slice::Iter<'a, u8>;

  #[cfg_attr(not(coverage), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    self.as_slice().iter()
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl IntoIterator for Buffer {
  type Item = u8;
  type IntoIter = ::bytes::buf::IntoIter<Buffer>;

  #[cfg_attr(not(coverage), inline(always))]
  fn into_iter(self) -> Self::IntoIter {
    ::bytes::buf::IntoIter::new(self)
  }
}

/// Iterator over the bytes contained by the buffer.
#[cfg(not(any(feature = "std", feature = "alloc")))]
#[derive(Debug)]
pub struct IntoIter<T> {
  inner: T,
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
const _: () = {
  impl Iterator for IntoIter<Buffer> {
    type Item = u8;

    #[cfg_attr(not(coverage), inline(always))]
    fn next(&mut self) -> Option<Self::Item> {
      self.inner.try_get_u8().ok()
    }
  }

  impl IntoIterator for Buffer {
    type Item = u8;
    type IntoIter = IntoIter<Buffer>;

    #[cfg_attr(not(coverage), inline(always))]
    fn into_iter(self) -> Self::IntoIter {
      IntoIter { inner: self }
    }
  }
};
