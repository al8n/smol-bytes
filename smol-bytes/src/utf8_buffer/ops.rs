use core::{
  borrow::BorrowMut,
  ops::{DerefMut, Index, IndexMut},
};

use super::*;

impl<I> Index<I> for Utf8Buffer
where
  I: core::slice::SliceIndex<str>,
{
  type Output = I::Output;

  #[cfg_attr(not(coverage), inline(always))]
  fn index(&self, index: I) -> &Self::Output {
    &self.as_str()[index]
  }
}

impl AsMut<str> for Utf8Buffer {
  fn as_mut(&mut self) -> &mut str {
    self.as_mut_str()
  }
}

impl BorrowMut<str> for Utf8Buffer {
  fn borrow_mut(&mut self) -> &mut str {
    self.as_mut_str()
  }
}

impl DerefMut for Utf8Buffer {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.as_mut_str()
  }
}

impl<I> IndexMut<I> for Utf8Buffer
where
  I: core::slice::SliceIndex<str>,
{
  fn index_mut(&mut self, index: I) -> &mut Self::Output {
    &mut self.as_mut_str()[index]
  }
}
