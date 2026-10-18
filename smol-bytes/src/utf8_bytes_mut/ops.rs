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

impl AsMut<str> for Utf8BytesMut {
  fn as_mut(&mut self) -> &mut str {
    self.as_mut_str()
  }
}

impl core::borrow::BorrowMut<str> for Utf8BytesMut {
  fn borrow_mut(&mut self) -> &mut str {
    self.as_mut_str()
  }
}

impl core::ops::DerefMut for Utf8BytesMut {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.as_mut_str()
  }
}

impl<I> core::ops::IndexMut<I> for Utf8BytesMut
where
  I: core::slice::SliceIndex<str>,
{
  fn index_mut(&mut self, index: I) -> &mut Self::Output {
    &mut self.as_mut_str()[index]
  }
}
