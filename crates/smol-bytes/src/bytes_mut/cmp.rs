use crate::strategy::Strategy;

use core::cmp::Ordering;
use std::{boxed::Box, rc::Rc, string::String, sync::Arc, vec::Vec};

use super::*;

macro_rules! bail {
  ($($ty:ty => $method:ident($other:ty)),+$(,)?) => {
    $(
      impl ::core::cmp::PartialEq<$ty> for SmolBytesMut {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn eq(&self, other: &$ty) -> ::core::primitive::bool {
          <Self as ::core::cmp::PartialEq<$other>>::eq(self, other.$method())
        }
      }

      impl ::core::cmp::PartialEq<SmolBytesMut> for $ty {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn eq(&self, other: &SmolBytesMut) -> ::core::primitive::bool {
          <SmolBytesMut as ::core::cmp::PartialEq<$ty>>::eq(other, self)
        }
      }

      impl ::core::cmp::PartialOrd<$ty> for SmolBytesMut {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn partial_cmp(&self, other: &$ty) -> ::core::option::Option<::core::cmp::Ordering> {
          <Self as ::core::cmp::PartialOrd<$other>>::partial_cmp(self, other.$method())
        }
      }

      impl ::core::cmp::PartialOrd<SmolBytesMut> for $ty {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn partial_cmp(&self, other: &SmolBytesMut) -> ::core::option::Option<::core::cmp::Ordering> {
          <$other as ::core::cmp::PartialOrd<SmolBytesMut>>::partial_cmp(self.$method(), other)
        }
      }
    )*
  };
}

impl PartialEq for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &Self) -> bool {
    self.as_ref() == other.as_ref()
  }
}

impl Eq for SmolBytesMut {}

impl PartialOrd for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_ref().cmp(other.as_ref())
  }
}

impl core::hash::Hash for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.as_ref().hash(state);
  }
}

impl<'a, T: ?Sized> PartialEq<&'a T> for SmolBytesMut
where
  Self: PartialEq<T>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &&'a T) -> bool {
    self.eq(other)
  }
}

impl<'a, T: ?Sized> PartialOrd<&'a T> for SmolBytesMut
where
  Self: PartialOrd<T>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &&'a T) -> Option<Ordering> {
    self.partial_cmp(other)
  }
}

// ---- [u8] comparisons ----
impl PartialEq<[u8]> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &[u8]) -> bool {
    self.as_ref() == other
  }
}

impl PartialEq<SmolBytesMut> for [u8] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &SmolBytesMut) -> bool {
    self == other.as_ref()
  }
}

impl PartialOrd<[u8]> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &[u8]) -> Option<Ordering> {
    Some(self.as_ref().cmp(other))
  }
}

impl PartialOrd<SmolBytesMut> for [u8] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &SmolBytesMut) -> Option<Ordering> {
    Some(self.cmp(other.as_ref()))
  }
}

// --- RawSmolBytes comparisons ----

impl<S> PartialEq<RawSmolBytes<S>> for SmolBytesMut
where
  RawSmolBytes<S>: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &RawSmolBytes<S>) -> bool {
    self.as_ref() == other.as_ref()
  }
}

impl<S> PartialEq<SmolBytesMut> for RawSmolBytes<S>
where
  RawSmolBytes<S>: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &SmolBytesMut) -> bool {
    other.eq(self)
  }
}

impl<S> PartialOrd<RawSmolBytes<S>> for SmolBytesMut
where
  RawSmolBytes<S>: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &RawSmolBytes<S>) -> Option<Ordering> {
    Some(self.as_ref().cmp(other.as_ref()))
  }
}

impl<S> PartialOrd<SmolBytesMut> for RawSmolBytes<S>
where
  RawSmolBytes<S>: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &SmolBytesMut) -> Option<Ordering> {
    Some(self.as_ref().cmp(other.as_ref()))
  }
}

// ---- &[u8] comparisons ----
impl PartialEq<SmolBytesMut> for &[u8] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &SmolBytesMut) -> bool {
    <[u8] as PartialEq<SmolBytesMut>>::eq(self, other)
  }
}

impl PartialOrd<SmolBytesMut> for &[u8] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &SmolBytesMut) -> Option<Ordering> {
    <[u8] as PartialOrd<SmolBytesMut>>::partial_cmp(self, other)
  }
}

// ---- &str comparisons ----
impl PartialEq<SmolBytesMut> for &str {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &SmolBytesMut) -> bool {
    <str as PartialEq<SmolBytesMut>>::eq(self, other)
  }
}

impl PartialOrd<SmolBytesMut> for &str {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &SmolBytesMut) -> Option<Ordering> {
    <str as PartialOrd<SmolBytesMut>>::partial_cmp(self, other)
  }
}

bail!(
  str => as_bytes([u8]),
  Bytes => as_ref([u8]),
  BytesMut => as_ref([u8]),
  String => as_str(str),
  Vec<u8> => as_slice([u8]),
  Box<[u8]> => as_ref([u8]),
  Rc<[u8]> => as_ref([u8]),
  Arc<[u8]> => as_ref([u8]),
  Box<str> => as_ref(str),
  Rc<str> => as_ref(str),
  Arc<str> => as_ref(str),
);
