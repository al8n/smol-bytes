use core::cmp::Ordering;

use super::*;

impl PartialEq for Utf8Buffer {
  #[cfg_attr(not(coverage), inline(always))]
  fn eq(&self, other: &Self) -> bool {
    self.as_str() == other.as_str()
  }
}

impl Eq for Utf8Buffer {}

impl PartialOrd for Utf8Buffer {
  #[cfg_attr(not(coverage), inline(always))]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Utf8Buffer {
  #[cfg_attr(not(coverage), inline(always))]
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_str().cmp(other.as_str())
  }
}

impl PartialEq<str> for Utf8Buffer {
  #[cfg_attr(not(coverage), inline(always))]
  fn eq(&self, other: &str) -> bool {
    self.as_str() == other
  }
}

impl PartialEq<Utf8Buffer> for str {
  #[cfg_attr(not(coverage), inline(always))]
  fn eq(&self, other: &Utf8Buffer) -> bool {
    self == other.as_str()
  }
}

impl PartialEq<&str> for Utf8Buffer {
  #[cfg_attr(not(coverage), inline(always))]
  fn eq(&self, other: &&str) -> bool {
    self.as_str() == *other
  }
}

impl PartialEq<Utf8Buffer> for &str {
  #[cfg_attr(not(coverage), inline(always))]
  fn eq(&self, other: &Utf8Buffer) -> bool {
    *self == other.as_str()
  }
}

impl PartialOrd<str> for Utf8Buffer {
  #[cfg_attr(not(coverage), inline(always))]
  fn partial_cmp(&self, other: &str) -> Option<Ordering> {
    self.as_str().partial_cmp(other)
  }
}

impl PartialOrd<Utf8Buffer> for str {
  #[cfg_attr(not(coverage), inline(always))]
  fn partial_cmp(&self, other: &Utf8Buffer) -> Option<Ordering> {
    self.partial_cmp(other.as_str())
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  use std::string::String;

  impl PartialEq<String> for Utf8Buffer {
    #[cfg_attr(not(coverage), inline(always))]
    fn eq(&self, other: &String) -> bool {
      self.as_str() == other.as_str()
    }
  }

  impl PartialEq<Utf8Buffer> for String {
    #[cfg_attr(not(coverage), inline(always))]
    fn eq(&self, other: &Utf8Buffer) -> bool {
      self.as_str() == other.as_str()
    }
  }

  impl PartialOrd<String> for Utf8Buffer {
    #[cfg_attr(not(coverage), inline(always))]
    fn partial_cmp(&self, other: &String) -> Option<Ordering> {
      self.as_str().partial_cmp(other.as_str())
    }
  }

  impl PartialOrd<Utf8Buffer> for String {
    #[cfg_attr(not(coverage), inline(always))]
    fn partial_cmp(&self, other: &Utf8Buffer) -> Option<Ordering> {
      self.as_str().partial_cmp(other.as_str())
    }
  }
};

impl core::hash::Hash for Utf8Buffer {
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.as_str().hash(state);
  }
}

// Cross-type equality with the heap-capable wrappers. Gated because
// `Utf8Bytes` and `Utf8BytesMut` require `std` or `alloc`.
#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  use crate::Utf8BytesMut;
  use crate::bytes::{RawBytes, strategy::ImmutableStorage};

  impl<S> PartialEq<crate::utf8_bytes::Utf8Bytes<S>> for Utf8Buffer
  where
    RawBytes<S>: ImmutableStorage,
  {
    #[cfg_attr(not(coverage), inline(always))]
    fn eq(&self, other: &crate::utf8_bytes::Utf8Bytes<S>) -> bool {
      self.as_str() == other.as_str()
    }
  }

  impl<S> PartialEq<Utf8Buffer> for crate::utf8_bytes::Utf8Bytes<S>
  where
    RawBytes<S>: ImmutableStorage,
  {
    #[cfg_attr(not(coverage), inline(always))]
    fn eq(&self, other: &Utf8Buffer) -> bool {
      self.as_str() == other.as_str()
    }
  }

  impl PartialEq<Utf8BytesMut> for Utf8Buffer {
    #[cfg_attr(not(coverage), inline(always))]
    fn eq(&self, other: &Utf8BytesMut) -> bool {
      self.as_str() == other.as_str()
    }
  }

  impl PartialEq<Utf8Buffer> for Utf8BytesMut {
    #[cfg_attr(not(coverage), inline(always))]
    fn eq(&self, other: &Utf8Buffer) -> bool {
      self.as_str() == other.as_str()
    }
  }
};
