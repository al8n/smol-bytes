use super::*;

use core::cmp::Ordering;
use std::{boxed::Box, rc::Rc, string::String, sync::Arc, vec::Vec};

macro_rules! bail {
  ($($ty:ty => $method:ident($other:ty)),+$(,)?) => {
    $(
      impl<S> ::core::cmp::PartialEq<$ty> for RawSmolBytes<S>
      where
        Self: Strategy,
      {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn eq(&self, other: &$ty) -> ::core::primitive::bool {
          <Self as ::core::cmp::PartialEq<$other>>::eq(self, other.$method())
        }
      }

      impl<S> ::core::cmp::PartialEq<RawSmolBytes<S>> for $ty
      where
        RawSmolBytes<S>: Strategy,
      {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn eq(&self, other: &RawSmolBytes<S>) -> ::core::primitive::bool {
          <RawSmolBytes<S> as ::core::cmp::PartialEq<$ty>>::eq(other, self)
        }
      }

      impl<S> ::core::cmp::PartialOrd<$ty> for RawSmolBytes<S>
      where
        Self: Strategy,
      {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn partial_cmp(&self, other: &$ty) -> ::core::option::Option<::core::cmp::Ordering> {
          <Self as ::core::cmp::PartialOrd<$other>>::partial_cmp(self, other.$method())
        }
      }

      impl<S> ::core::cmp::PartialOrd<RawSmolBytes<S>> for $ty
      where
        RawSmolBytes<S>: Strategy,
      {
        #[cfg_attr(not(tarpaulin), inline(always))]
        fn partial_cmp(&self, other: &RawSmolBytes<S>) -> ::core::option::Option<::core::cmp::Ordering> {
          <$other as ::core::cmp::PartialOrd<RawSmolBytes<S>>>::partial_cmp(self.$method(), other)
        }
      }
    )*
  };
}

impl<S> Hash for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.as_slice().hash(state)
  }
}

impl<'a, T: ?Sized, S> PartialEq<&'a T> for RawSmolBytes<S>
where
  Self: PartialEq<T> + Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &&'a T) -> bool {
    self.eq(other)
  }
}

impl<'a, T: ?Sized, S> PartialOrd<&'a T> for RawSmolBytes<S>
where
  Self: PartialOrd<T> + Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &&'a T) -> Option<Ordering> {
    self.partial_cmp(other)
  }
}

impl<S> PartialEq for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.repr.ptr_eq(&other.repr) || self.as_slice() == other.as_slice()
  }
}

impl<S> Eq for RawSmolBytes<S> where Self: Strategy {}

impl<S> PartialOrd for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl<S> Ord for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_slice().cmp(other.as_slice())
  }
}

// ---- [u8] comparisons ----
impl<S> PartialEq<[u8]> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &[u8]) -> bool {
    self.as_slice() == other
  }
}

impl<S> PartialEq<RawSmolBytes<S>> for [u8]
where
  RawSmolBytes<S>: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &RawSmolBytes<S>) -> bool {
    <RawSmolBytes<S> as PartialEq<[u8]>>::eq(other, self)
  }
}

impl<S> PartialOrd<[u8]> for RawSmolBytes<S>
where
  Self: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &[u8]) -> Option<Ordering> {
    Some(self.as_slice().cmp(other))
  }
}

impl<S> PartialOrd<RawSmolBytes<S>> for [u8]
where
  RawSmolBytes<S>: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &RawSmolBytes<S>) -> Option<Ordering> {
    Some(self.cmp(other.as_slice()))
  }
}

// ---- &[u8] comparisons ----
impl<S> PartialEq<RawSmolBytes<S>> for &[u8]
where
  RawSmolBytes<S>: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &RawSmolBytes<S>) -> bool {
    <[u8] as PartialEq<RawSmolBytes<S>>>::eq(self, other)
  }
}

impl<S> PartialOrd<RawSmolBytes<S>> for &[u8]
where
  RawSmolBytes<S>: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &RawSmolBytes<S>) -> Option<Ordering> {
    <[u8] as PartialOrd<RawSmolBytes<S>>>::partial_cmp(self, other)
  }
}

// ---- &str comparisons ----
impl<S> PartialEq<RawSmolBytes<S>> for &str
where
  RawSmolBytes<S>: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn eq(&self, other: &RawSmolBytes<S>) -> bool {
    <str as PartialEq<RawSmolBytes<S>>>::eq(self, other)
  }
}

impl<S> PartialOrd<RawSmolBytes<S>> for &str
where
  RawSmolBytes<S>: Strategy,
{
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn partial_cmp(&self, other: &RawSmolBytes<S>) -> Option<Ordering> {
    <str as PartialOrd<RawSmolBytes<S>>>::partial_cmp(self, other)
  }
}

bail!(
  str => as_bytes([u8]),
  Bytes => as_ref([u8]),
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
