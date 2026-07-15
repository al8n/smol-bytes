use core::borrow::{Borrow, BorrowMut};

use super::*;

impl core::ops::Deref for Buffer {
  type Target = [u8];

  #[cfg_attr(not(coverage), inline(always))]
  fn deref(&self) -> &Self::Target {
    self.as_slice()
  }
}

impl core::ops::DerefMut for Buffer {
  #[cfg_attr(not(coverage), inline(always))]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.as_mut_slice()
  }
}

impl AsRef<[u8]> for Buffer {
  #[cfg_attr(not(coverage), inline(always))]
  fn as_ref(&self) -> &[u8] {
    self
  }
}

impl AsMut<[u8]> for Buffer {
  #[cfg_attr(not(coverage), inline(always))]
  fn as_mut(&mut self) -> &mut [u8] {
    self
  }
}

impl Borrow<[u8]> for Buffer {
  #[cfg_attr(not(coverage), inline(always))]
  fn borrow(&self) -> &[u8] {
    self.as_ref()
  }
}

impl BorrowMut<[u8]> for Buffer {
  #[cfg_attr(not(coverage), inline(always))]
  fn borrow_mut(&mut self) -> &mut [u8] {
    self.as_mut()
  }
}
