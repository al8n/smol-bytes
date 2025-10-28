use super::*;

impl<S> core::fmt::Debug for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match &self.repr {
      Repr::Inline(b) => b.fmt(f),
      Repr::Heap(b) => b.fmt(f),
    }
  }
}

impl<S> core::fmt::LowerHex for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match &self.repr {
      Repr::Inline(b) => b.fmt(f),
      Repr::Heap(b) => b.fmt(f),
    }
  }
}

impl<S> core::fmt::UpperHex for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match &self.repr {
      Repr::Inline(b) => b.fmt(f),
      Repr::Heap(b) => b.fmt(f),
    }
  }
}