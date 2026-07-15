use super::*;

impl<S, I> core::ops::Index<I> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
  I: core::slice::SliceIndex<str>,
{
  type Output = I::Output;

  fn index(&self, index: I) -> &Self::Output {
    &self.as_str()[index]
  }
}
