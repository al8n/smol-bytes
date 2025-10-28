use super::*;

impl<S> core::ops::Deref for RawSmolBytes<S>
where
  Self: Strategy,
{
  type Target = [u8];

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    self.as_slice()
  }
}

impl<S> core::borrow::Borrow<[u8]> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn borrow(&self) -> &[u8] {
    self.as_ref()
  }
}

impl<S> AsRef<[u8]> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_ref(&self) -> &[u8] {
    self
  }
}
