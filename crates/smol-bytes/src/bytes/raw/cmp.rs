use super::*;

use core::cmp::Ordering;
use std::{boxed::Box, rc::Rc, string::String, sync::Arc, vec::Vec};

macro_rules! bail {
  ($($ty:ty => $method:ident($other:ty)),+$(,)?) => {
    $(
      impl<S> ::core::cmp::PartialEq<$ty> for RawBytes<S>
      where
        Self: ImmutableStorage,
      {
        #[cfg_attr(not(coverage), inline(always))]
        fn eq(&self, other: &$ty) -> ::core::primitive::bool {
          <Self as ::core::cmp::PartialEq<$other>>::eq(self, other.$method())
        }
      }

      impl<S> ::core::cmp::PartialEq<RawBytes<S>> for $ty
      where
        RawBytes<S>: ImmutableStorage,
      {
        #[cfg_attr(not(coverage), inline(always))]
        fn eq(&self, other: &RawBytes<S>) -> ::core::primitive::bool {
          <RawBytes<S> as ::core::cmp::PartialEq<$ty>>::eq(other, self)
        }
      }

      impl<S> ::core::cmp::PartialOrd<$ty> for RawBytes<S>
      where
        Self: ImmutableStorage,
      {
        #[cfg_attr(not(coverage), inline(always))]
        fn partial_cmp(&self, other: &$ty) -> ::core::option::Option<::core::cmp::Ordering> {
          <Self as ::core::cmp::PartialOrd<$other>>::partial_cmp(self, other.$method())
        }
      }

      impl<S> ::core::cmp::PartialOrd<RawBytes<S>> for $ty
      where
        RawBytes<S>: ImmutableStorage,
      {
        #[cfg_attr(not(coverage), inline(always))]
        fn partial_cmp(&self, other: &RawBytes<S>) -> ::core::option::Option<::core::cmp::Ordering> {
          <$other as ::core::cmp::PartialOrd<RawBytes<S>>>::partial_cmp(self.$method(), other)
        }
      }
    )*
  };
}

impl<S> Hash for RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.as_slice().hash(state)
  }
}

impl<'a, T: ?Sized, S> PartialEq<&'a T> for RawBytes<S>
where
  Self: PartialEq<T> + ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn eq(&self, other: &&'a T) -> bool {
    self.eq(other)
  }
}

impl<'a, T: ?Sized, S> PartialOrd<&'a T> for RawBytes<S>
where
  Self: PartialOrd<T> + ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn partial_cmp(&self, other: &&'a T) -> Option<Ordering> {
    self.partial_cmp(other)
  }
}

impl<S> PartialEq for RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.repr.ptr_eq(&other.repr) || self.as_slice() == other.as_slice()
  }
}

impl<S> Eq for RawBytes<S> where Self: ImmutableStorage {}

impl<S> PartialOrd for RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl<S> Ord for RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_slice().cmp(other.as_slice())
  }
}

// ---- [u8] comparisons ----
impl<S> PartialEq<[u8]> for RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn eq(&self, other: &[u8]) -> bool {
    self.as_slice() == other
  }
}

impl<S> PartialEq<RawBytes<S>> for [u8]
where
  RawBytes<S>: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn eq(&self, other: &RawBytes<S>) -> bool {
    <RawBytes<S> as PartialEq<[u8]>>::eq(other, self)
  }
}

impl<S> PartialOrd<[u8]> for RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn partial_cmp(&self, other: &[u8]) -> Option<Ordering> {
    Some(self.as_slice().cmp(other))
  }
}

impl<S> PartialOrd<RawBytes<S>> for [u8]
where
  RawBytes<S>: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn partial_cmp(&self, other: &RawBytes<S>) -> Option<Ordering> {
    Some(self.cmp(other.as_slice()))
  }
}

impl<S, const N: usize> PartialEq<[u8; N]> for RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn eq(&self, other: &[u8; N]) -> bool {
    N == self.len() && self.as_slice() == other.as_slice()
  }
}

impl<S, const N: usize> PartialEq<RawBytes<S>> for [u8; N]
where
  RawBytes<S>: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn eq(&self, other: &RawBytes<S>) -> bool {
    other.eq(self)
  }
}

impl<S, const N: usize> PartialOrd<[u8; N]> for RawBytes<S>
where
  Self: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn partial_cmp(&self, other: &[u8; N]) -> Option<Ordering> {
    Some(self.as_slice().cmp(other.as_slice()))
  }
}

impl<S, const N: usize> PartialOrd<RawBytes<S>> for [u8; N]
where
  RawBytes<S>: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn partial_cmp(&self, other: &RawBytes<S>) -> Option<Ordering> {
    Some(self.as_slice().cmp(other.as_slice()))
  }
}

// ---- &[u8] comparisons ----
impl<S> PartialEq<RawBytes<S>> for &[u8]
where
  RawBytes<S>: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn eq(&self, other: &RawBytes<S>) -> bool {
    <[u8] as PartialEq<RawBytes<S>>>::eq(self, other)
  }
}

impl<S> PartialOrd<RawBytes<S>> for &[u8]
where
  RawBytes<S>: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn partial_cmp(&self, other: &RawBytes<S>) -> Option<Ordering> {
    <[u8] as PartialOrd<RawBytes<S>>>::partial_cmp(self, other)
  }
}

// ---- &str comparisons ----
impl<S> PartialEq<RawBytes<S>> for &str
where
  RawBytes<S>: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn eq(&self, other: &RawBytes<S>) -> bool {
    <str as PartialEq<RawBytes<S>>>::eq(self, other)
  }
}

impl<S> PartialOrd<RawBytes<S>> for &str
where
  RawBytes<S>: ImmutableStorage,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn partial_cmp(&self, other: &RawBytes<S>) -> Option<Ordering> {
    <str as PartialOrd<RawBytes<S>>>::partial_cmp(self, other)
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
