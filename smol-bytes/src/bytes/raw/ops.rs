use super::*;

impl<S> core::ops::Deref for RawBytes<S>
where
  Self: ImmutableStorage,
{
  type Target = [u8];

  #[cfg_attr(not(coverage), inline(always))]
  fn deref(&self) -> &Self::Target {
    self.as_slice()
  }
}

impl<S> core::borrow::Borrow<[u8]> for RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn borrow(&self) -> &[u8] {
    self.as_ref()
  }
}

impl<S> AsRef<[u8]> for RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn as_ref(&self) -> &[u8] {
    self
  }
}
