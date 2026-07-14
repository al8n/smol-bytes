use super::*;

impl core::fmt::Debug for BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match &self.0 {
      Repr::Inline(b) => b.fmt(f),
      Repr::Heap(b) => b.fmt(f),
    }
  }
}

impl core::fmt::LowerHex for BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match &self.0 {
      Repr::Inline(b) => b.fmt(f),
      Repr::Heap(b) => b.fmt(f),
    }
  }
}

impl core::fmt::UpperHex for BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match &self.0 {
      Repr::Inline(b) => b.fmt(f),
      Repr::Heap(b) => b.fmt(f),
    }
  }
}

impl core::fmt::Write for BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    match &mut self.0 {
      Repr::Inline(inline_storage) => inline_storage.write_str(s),
      Repr::Heap(bytes_mut) => bytes_mut.write_str(s),
    }
  }
}
