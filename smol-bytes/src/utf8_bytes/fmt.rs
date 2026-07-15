use super::*;

impl<S> core::fmt::Debug for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    core::fmt::Debug::fmt(self.as_str(), f)
  }
}

impl<S> core::fmt::Display for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    core::fmt::Display::fmt(self.as_str(), f)
  }
}
