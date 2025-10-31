use super::*;

impl core::fmt::Debug for Utf8Bytes {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    core::fmt::Debug::fmt(self.as_str(), f)
  }
}

impl core::fmt::Display for Utf8Bytes {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    core::fmt::Display::fmt(self.as_str(), f)
  }
}
