use super::*;

impl core::ops::Deref for SmolBytesMut {
  type Target = [u8];

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    match &self.0 {
      Repr::Inline(b) => b.as_slice(),
      Repr::Heap(b) => b.as_ref(),
    }
  }
}

impl core::ops::DerefMut for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref_mut(&mut self) -> &mut Self::Target {
    match &mut self.0 {
      Repr::Inline(b) => b.as_mut_slice(),
      Repr::Heap(b) => b.as_mut(),
    }
  }
}

impl core::borrow::Borrow<[u8]> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn borrow(&self) -> &[u8] {
    self.as_ref()
  }
}

impl core::borrow::BorrowMut<[u8]> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn borrow_mut(&mut self) -> &mut [u8] {
    self.as_mut()
  }
}

impl AsRef<[u8]> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_ref(&self) -> &[u8] {
    self
  }
}

impl AsMut<[u8]> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn as_mut(&mut self) -> &mut [u8] {
    self
  }
}
