use super::*;

impl PartialEq for Utf8Bytes {
  fn eq(&self, other: &Self) -> bool {
    self.as_str() == other.as_str()
  }
}

impl Eq for Utf8Bytes {}

impl PartialOrd for Utf8Bytes {
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Utf8Bytes {
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    self.as_str().cmp(other.as_str())
  }
}

impl PartialEq<str> for Utf8Bytes {
  fn eq(&self, other: &str) -> bool {
    self.as_str() == other
  }
}

impl PartialEq<Utf8Bytes> for str {
  fn eq(&self, other: &Utf8Bytes) -> bool {
    self == other.as_str()
  }
}

impl PartialEq<&str> for Utf8Bytes {
  fn eq(&self, other: &&str) -> bool {
    self.as_str() == *other
  }
}

impl PartialEq<Utf8Bytes> for &str {
  fn eq(&self, other: &Utf8Bytes) -> bool {
    *self == other.as_str()
  }
}

impl PartialOrd<str> for Utf8Bytes {
  fn partial_cmp(&self, other: &str) -> Option<core::cmp::Ordering> {
    self.as_str().partial_cmp(other)
  }
}

impl PartialOrd<Utf8Bytes> for str {
  fn partial_cmp(&self, other: &Utf8Bytes) -> Option<core::cmp::Ordering> {
    self.partial_cmp(other.as_str())
  }
}

impl core::hash::Hash for Utf8Bytes {
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.as_str().hash(state);
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  use std::string::String;

  impl PartialEq<String> for Utf8Bytes {
    fn eq(&self, other: &String) -> bool {
      self.as_str() == other.as_str()
    }
  }

  impl PartialEq<Utf8Bytes> for String {
    fn eq(&self, other: &Utf8Bytes) -> bool {
      self.as_str() == other.as_str()
    }
  }

  impl PartialOrd<String> for Utf8Bytes {
    fn partial_cmp(&self, other: &String) -> Option<core::cmp::Ordering> {
      self.as_str().partial_cmp(other.as_str())
    }
  }

  impl PartialOrd<Utf8Bytes> for String {
    fn partial_cmp(&self, other: &Utf8Bytes) -> Option<core::cmp::Ordering> {
      self.as_str().partial_cmp(other.as_str())
    }
  }
};
