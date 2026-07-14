use super::*;

impl<S> PartialEq for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn eq(&self, other: &Self) -> bool {
    self.as_str() == other.as_str()
  }
}

impl<S> Eq for Utf8Bytes<S> where RawBytes<S>: ImmutableStorage {}

impl<S> PartialOrd for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl<S> Ord for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    self.as_str().cmp(other.as_str())
  }
}

impl<S> PartialEq<str> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn eq(&self, other: &str) -> bool {
    self.as_str() == other
  }
}

impl<S> PartialEq<Utf8Bytes<S>> for str
where
  RawBytes<S>: ImmutableStorage,
{
  fn eq(&self, other: &Utf8Bytes<S>) -> bool {
    self == other.as_str()
  }
}

impl<S> PartialEq<&str> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn eq(&self, other: &&str) -> bool {
    self.as_str() == *other
  }
}

impl<S> PartialEq<Utf8Bytes<S>> for &str
where
  RawBytes<S>: ImmutableStorage,
{
  fn eq(&self, other: &Utf8Bytes<S>) -> bool {
    *self == other.as_str()
  }
}

impl<S> PartialOrd<str> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn partial_cmp(&self, other: &str) -> Option<core::cmp::Ordering> {
    self.as_str().partial_cmp(other)
  }
}

impl<S> PartialOrd<Utf8Bytes<S>> for str
where
  RawBytes<S>: ImmutableStorage,
{
  fn partial_cmp(&self, other: &Utf8Bytes<S>) -> Option<core::cmp::Ordering> {
    self.partial_cmp(other.as_str())
  }
}

impl<S> core::hash::Hash for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.as_str().hash(state);
  }
}

// Cross-type equality between the two heap-capable wrappers.
const _: () = {
  use crate::Utf8BytesMut;

  impl<S> PartialEq<Utf8BytesMut> for Utf8Bytes<S>
  where
    RawBytes<S>: ImmutableStorage,
  {
    #[cfg_attr(not(coverage), inline(always))]
    fn eq(&self, other: &Utf8BytesMut) -> bool {
      self.as_str() == other.as_str()
    }
  }

  impl<S> PartialEq<Utf8Bytes<S>> for Utf8BytesMut
  where
    RawBytes<S>: ImmutableStorage,
  {
    #[cfg_attr(not(coverage), inline(always))]
    fn eq(&self, other: &Utf8Bytes<S>) -> bool {
      self.as_str() == other.as_str()
    }
  }
};

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  use std::string::String;

  impl<S> PartialEq<String> for Utf8Bytes<S>
  where
    RawBytes<S>: ImmutableStorage,
  {
    fn eq(&self, other: &String) -> bool {
      self.as_str() == other.as_str()
    }
  }

  impl<S> PartialEq<Utf8Bytes<S>> for String
  where
    RawBytes<S>: ImmutableStorage,
  {
    fn eq(&self, other: &Utf8Bytes<S>) -> bool {
      self.as_str() == other.as_str()
    }
  }

  impl<S> PartialOrd<String> for Utf8Bytes<S>
  where
    RawBytes<S>: ImmutableStorage,
  {
    fn partial_cmp(&self, other: &String) -> Option<core::cmp::Ordering> {
      self.as_str().partial_cmp(other.as_str())
    }
  }

  impl<S> PartialOrd<Utf8Bytes<S>> for String
  where
    RawBytes<S>: ImmutableStorage,
  {
    fn partial_cmp(&self, other: &Utf8Bytes<S>) -> Option<core::cmp::Ordering> {
      self.as_str().partial_cmp(other.as_str())
    }
  }
};
