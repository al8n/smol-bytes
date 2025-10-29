use core::cmp::Ordering;

use super::*;

macro_rules! bail {
  ($($ty:ty => $method:ident($other:ty)),+$(,)?) => {
    $(
      impl ::core::cmp::PartialEq<$ty> for Buffer {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn eq(&self, other: &$ty) -> ::core::primitive::bool {
          <Self as ::core::cmp::PartialEq<$other>>::eq(self, other.$method())
        }
      }

      impl ::core::cmp::PartialEq<Buffer> for $ty {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn eq(&self, other: &Buffer) -> ::core::primitive::bool {
          <Buffer as ::core::cmp::PartialEq<$ty>>::eq(other, self)
        }
      }

      impl ::core::cmp::PartialOrd<$ty> for Buffer {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn partial_cmp(&self, other: &$ty) -> ::core::option::Option<::core::cmp::Ordering> {
          <Self as ::core::cmp::PartialOrd<$other>>::partial_cmp(self, other.$method())
        }
      }

      impl ::core::cmp::PartialOrd<Buffer> for $ty {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn partial_cmp(&self, other: &Buffer) -> ::core::option::Option<::core::cmp::Ordering> {
          <$other as ::core::cmp::PartialOrd<Buffer>>::partial_cmp(self.$method(), other)
        }
      }
    )*
  };
}

impl PartialEq for Buffer {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &Self) -> bool {
    self.as_ref() == other.as_ref()
  }
}

impl Eq for Buffer {}

impl PartialOrd for Buffer {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Buffer {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_ref().cmp(other.as_ref())
  }
}

impl core::hash::Hash for Buffer {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.as_ref().hash(state);
  }
}

impl<'a, T: ?Sized> PartialEq<&'a T> for Buffer
where
  Self: PartialEq<T>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &&'a T) -> bool {
    self.eq(other)
  }
}

impl<'a, T: ?Sized> PartialOrd<&'a T> for Buffer
where
  Self: PartialOrd<T>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &&'a T) -> Option<Ordering> {
    self.partial_cmp(other)
  }
}

// ---- [u8] comparisons ----
impl PartialEq<[u8]> for Buffer {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &[u8]) -> bool {
    self.as_ref() == other
  }
}

impl PartialEq<Buffer> for [u8] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &Buffer) -> bool {
    self == other.as_ref()
  }
}

impl PartialOrd<[u8]> for Buffer {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &[u8]) -> Option<Ordering> {
    Some(self.as_ref().cmp(other))
  }
}

impl PartialOrd<Buffer> for [u8] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &Buffer) -> Option<Ordering> {
    Some(self.cmp(other.as_ref()))
  }
}

// ---- &[u8] comparisons ----
impl PartialEq<Buffer> for &[u8] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &Buffer) -> bool {
    <[u8] as PartialEq<Buffer>>::eq(self, other)
  }
}

impl PartialOrd<Buffer> for &[u8] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &Buffer) -> Option<Ordering> {
    <[u8] as PartialOrd<Buffer>>::partial_cmp(self, other)
  }
}

// ---- &str comparisons ----
impl PartialEq<Buffer> for &str {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &Buffer) -> bool {
    <str as PartialEq<Buffer>>::eq(self, other)
  }
}

impl PartialOrd<Buffer> for &str {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &Buffer) -> Option<Ordering> {
    <str as PartialOrd<Buffer>>::partial_cmp(self, other)
  }
}

bail!(
  str => as_bytes([u8]),
);

#[cfg(any(feature = "alloc", feature = "std"))]
const _: () = {
  use std::{boxed::Box, rc::Rc, string::String, sync::Arc, vec::Vec};

  use crate::{bytes::RawSmolBytes, strategy::Strategy};

  bail!(
    ::bytes::Bytes => as_ref([u8]),
    ::bytes::BytesMut => as_ref([u8]),
    String => as_str(str),
    Vec<u8> => as_slice([u8]),
    Box<[u8]> => as_ref([u8]),
    Rc<[u8]> => as_ref([u8]),
    Arc<[u8]> => as_ref([u8]),
    Box<str> => as_ref(str),
    Rc<str> => as_ref(str),
    Arc<str> => as_ref(str),
  );

  // --- RawSmolBytes comparisons ----
  impl<S> PartialEq<RawSmolBytes<S>> for Buffer
  where
    RawSmolBytes<S>: Strategy,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn eq(&self, other: &RawSmolBytes<S>) -> bool {
      self.as_ref() == other.as_ref()
    }
  }

  impl<S> PartialEq<Buffer> for RawSmolBytes<S>
  where
    RawSmolBytes<S>: Strategy,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn eq(&self, other: &Buffer) -> bool {
      other.eq(self)
    }
  }

  impl<S> PartialOrd<RawSmolBytes<S>> for Buffer
  where
    RawSmolBytes<S>: Strategy,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn partial_cmp(&self, other: &RawSmolBytes<S>) -> Option<Ordering> {
      Some(self.as_ref().cmp(other.as_ref()))
    }
  }

  impl<S> PartialOrd<Buffer> for RawSmolBytes<S>
  where
    RawSmolBytes<S>: Strategy,
  {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn partial_cmp(&self, other: &Buffer) -> Option<Ordering> {
      Some(self.as_ref().cmp(other.as_ref()))
    }
  }
};
