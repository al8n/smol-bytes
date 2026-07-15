use super::*;

impl<I> core::ops::Index<I> for Utf8BytesMut
where
  I: core::slice::SliceIndex<str>,
{
  type Output = I::Output;

  fn index(&self, index: I) -> &Self::Output {
    &self.as_str()[index]
  }
}
