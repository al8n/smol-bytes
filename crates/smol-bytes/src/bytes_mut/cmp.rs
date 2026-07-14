use crate::strategy::ImmutableStorage;

use core::cmp::Ordering;
use std::{boxed::Box, rc::Rc, string::String, sync::Arc, vec::Vec};

use super::*;

macro_rules! bail {
  ($($ty:ty => $method:ident($other:ty)),+$(,)?) => {
    $(
      impl ::core::cmp::PartialEq<$ty> for BytesMut {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn eq(&self, other: &$ty) -> ::core::primitive::bool {
          <Self as ::core::cmp::PartialEq<$other>>::eq(self, other.$method())
        }
      }

      impl ::core::cmp::PartialEq<BytesMut> for $ty {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn eq(&self, other: &BytesMut) -> ::core::primitive::bool {
          <BytesMut as ::core::cmp::PartialEq<$ty>>::eq(other, self)
        }
      }

      impl ::core::cmp::PartialOrd<$ty> for BytesMut {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn partial_cmp(&self, other: &$ty) -> ::core::option::Option<::core::cmp::Ordering> {
          <Self as ::core::cmp::PartialOrd<$other>>::partial_cmp(self, other.$method())
        }
      }

      impl ::core::cmp::PartialOrd<BytesMut> for $ty {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn partial_cmp(&self, other: &BytesMut) -> ::core::option::Option<::core::cmp::Ordering> {
          <$other as ::core::cmp::PartialOrd<BytesMut>>::partial_cmp(self.$method(), other)
        }
      }
    )*
  };
}

impl PartialEq for BytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &Self) -> bool {
    self.as_ref() == other.as_ref()
  }
}

impl Eq for BytesMut {}

impl PartialOrd for BytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for BytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_ref().cmp(other.as_ref())
  }
}

impl core::hash::Hash for BytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.as_ref().hash(state);
  }
}

impl<'a, T: ?Sized> PartialEq<&'a T> for BytesMut
where
  Self: PartialEq<T>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &&'a T) -> bool {
    self.eq(other)
  }
}

impl<'a, T: ?Sized> PartialOrd<&'a T> for BytesMut
where
  Self: PartialOrd<T>,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &&'a T) -> Option<Ordering> {
    self.partial_cmp(other)
  }
}

// ---- [u8] comparisons ----
impl PartialEq<[u8]> for BytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &[u8]) -> bool {
    self.as_ref() == other
  }
}

impl PartialEq<BytesMut> for [u8] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &BytesMut) -> bool {
    self == other.as_ref()
  }
}

impl PartialOrd<[u8]> for BytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &[u8]) -> Option<Ordering> {
    Some(self.as_ref().cmp(other))
  }
}

impl PartialOrd<BytesMut> for [u8] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &BytesMut) -> Option<Ordering> {
    Some(self.cmp(other.as_ref()))
  }
}

impl<const N: usize> PartialEq<[u8; N]> for BytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &[u8; N]) -> bool {
    self.as_ref() == other
  }
}

impl<const N: usize> PartialEq<BytesMut> for [u8; N] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &BytesMut) -> bool {
    other.eq(self)
  }
}

impl<const N: usize> PartialOrd<[u8; N]> for BytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &[u8; N]) -> Option<Ordering> {
    Some(self.as_ref().cmp(other))
  }
}

impl<const N: usize> PartialOrd<BytesMut> for [u8; N] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &BytesMut) -> Option<Ordering> {
    Some(self.as_slice().cmp(other.as_ref()))
  }
}

// --- RawBytes comparisons ----

impl<S> PartialEq<RawBytes<S>> for BytesMut
where
  RawBytes<S>: ImmutableStorage,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &RawBytes<S>) -> bool {
    self.as_ref() == other.as_ref()
  }
}

impl<S> PartialEq<BytesMut> for RawBytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &BytesMut) -> bool {
    other.eq(self)
  }
}

impl<S> PartialOrd<RawBytes<S>> for BytesMut
where
  RawBytes<S>: ImmutableStorage,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &RawBytes<S>) -> Option<Ordering> {
    Some(self.as_ref().cmp(other.as_ref()))
  }
}

impl<S> PartialOrd<BytesMut> for RawBytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &BytesMut) -> Option<Ordering> {
    Some(self.as_ref().cmp(other.as_ref()))
  }
}

// ---- &[u8] comparisons ----
impl PartialEq<BytesMut> for &[u8] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &BytesMut) -> bool {
    <[u8] as PartialEq<BytesMut>>::eq(self, other)
  }
}

impl PartialOrd<BytesMut> for &[u8] {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &BytesMut) -> Option<Ordering> {
    <[u8] as PartialOrd<BytesMut>>::partial_cmp(self, other)
  }
}

// ---- &str comparisons ----
impl PartialEq<BytesMut> for &str {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &BytesMut) -> bool {
    <str as PartialEq<BytesMut>>::eq(self, other)
  }
}

impl PartialOrd<BytesMut> for &str {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &BytesMut) -> Option<Ordering> {
    <str as PartialOrd<BytesMut>>::partial_cmp(self, other)
  }
}

bail!(
  str => as_bytes([u8]),
  bytes::Bytes => as_ref([u8]),
  bytes::BytesMut => as_ref([u8]),
  String => as_str(str),
  Vec<u8> => as_slice([u8]),
  Box<[u8]> => as_ref([u8]),
  Rc<[u8]> => as_ref([u8]),
  Arc<[u8]> => as_ref([u8]),
  Box<str> => as_ref(str),
  Rc<str> => as_ref(str),
  Arc<str> => as_ref(str),
);
