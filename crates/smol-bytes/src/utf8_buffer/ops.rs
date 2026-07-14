use core::ops::Index;

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
