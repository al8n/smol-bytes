use super::*;

impl<S> core::fmt::Debug for RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match &self.repr {
      Repr::Inline(b) => b.fmt(f),
      Repr::Heap(b) => b.fmt(f),
    }
  }
}

impl<S> core::fmt::LowerHex for RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match &self.repr {
      Repr::Inline(b) => b.fmt(f),
      Repr::Heap(b) => b.fmt(f),
    }
  }
}

impl<S> core::fmt::UpperHex for RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match &self.repr {
      Repr::Inline(b) => b.fmt(f),
      Repr::Heap(b) => b.fmt(f),
    }
  }
}
