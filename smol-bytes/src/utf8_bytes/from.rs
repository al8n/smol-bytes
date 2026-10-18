use core::{convert::Infallible, str::FromStr};
use std::{borrow::Cow, string::String, vec::Vec};

use crate::{BytesMut, Utf8BytesMut};

use super::*;

impl<S> From<&str> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn from(s: &str) -> Self {
    Self {
      inner: RawBytes::copy_from_slice(s.as_bytes()),
    }
  }
}

impl<S> FromStr for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  type Err = Infallible;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self::from(s))
  }
}

impl<S> From<String> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn from(s: String) -> Self {
    Self {
      inner: RawBytes::from(s.into_bytes()),
    }
  }
}

impl<'a, S> From<Cow<'a, str>> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn from(value: Cow<'a, str>) -> Self {
    match value {
      Cow::Borrowed(value) => Self::from(value),
      Cow::Owned(value) => Self::from(value),
    }
  }
}

impl<S> TryFrom<RawBytes<S>> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  type Error = FromUtf8Error<RawBytes<S>>;

  fn try_from(bytes: RawBytes<S>) -> Result<Self, Self::Error> {
    Self::from_utf8(bytes)
  }
}

impl<S> TryFrom<&[u8]> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  type Error = core::str::Utf8Error;

  /// Creates a `Utf8Bytes` from a byte slice, validating UTF-8.
  fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
    let s = core::str::from_utf8(bytes)?;
    Ok(Self::from(s))
  }
}

impl<S> TryFrom<Vec<u8>> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  type Error = FromUtf8Error<Vec<u8>>;

  fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
    match core::str::from_utf8(&bytes) {
      Ok(_) => Ok(Self {
        inner: RawBytes::from(bytes),
      }),
      Err(error) => Err(FromUtf8Error::new(bytes, error)),
    }
  }
}

impl<S> TryFrom<::bytes::Bytes> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage + From<::bytes::Bytes>,
{
  type Error = FromUtf8Error<::bytes::Bytes>;

  fn try_from(bytes: ::bytes::Bytes) -> Result<Self, Self::Error> {
    match core::str::from_utf8(bytes.as_ref()) {
      Ok(_) => Ok(Self {
        inner: RawBytes::from(bytes),
      }),
      Err(error) => Err(FromUtf8Error::new(bytes, error)),
    }
  }
}

impl<S> From<Utf8Bytes<S>> for RawBytes<S> {
  fn from(value: Utf8Bytes<S>) -> Self {
    value.inner
  }
}

impl<S> From<Utf8BytesMut> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn from(value: Utf8BytesMut) -> Self {
    Self {
      inner: value.into_inner().freeze_with(),
    }
  }
}

impl<S> From<Utf8Bytes<S>> for Utf8BytesMut
where
  RawBytes<S>: ImmutableStorage,
{
  fn from(value: Utf8Bytes<S>) -> Self {
    Self {
      inner: BytesMut::from(value.into_inner()),
    }
  }
}

impl From<crate::compact::Utf8Bytes> for crate::shared::Utf8Bytes {
  fn from(value: crate::compact::Utf8Bytes) -> Self {
    Self {
      inner: value.into_inner().into(),
    }
  }
}

impl From<crate::shared::Utf8Bytes> for crate::compact::Utf8Bytes {
  fn from(value: crate::shared::Utf8Bytes) -> Self {
    Self {
      inner: value.into_inner().into(),
    }
  }
}

impl<S> From<Utf8Bytes<S>> for String
where
  RawBytes<S>: ImmutableStorage,
{
  fn from(value: Utf8Bytes<S>) -> Self {
    String::from_utf8(value.inner.into()).expect("Utf8Bytes invariant")
  }
}

impl<S> From<Utf8Bytes<S>> for Vec<u8>
where
  RawBytes<S>: ImmutableStorage,
{
  fn from(value: Utf8Bytes<S>) -> Self {
    value.into_inner().into_vec()
  }
}

impl<S> From<Utf8Bytes<S>> for ::bytes::Bytes
where
  RawBytes<S>: ImmutableStorage,
{
  fn from(value: Utf8Bytes<S>) -> Self {
    value.into_inner().into_bytes()
  }
}

impl<S> From<Utf8Bytes<S>> for ::bytes::BytesMut
where
  RawBytes<S>: ImmutableStorage,
{
  fn from(value: Utf8Bytes<S>) -> Self {
    BytesMut::from(value.into_inner()).into()
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  use std::{boxed::Box, rc::Rc, sync::Arc};

  impl<S> From<Utf8Bytes<S>> for Box<str>
  where
    RawBytes<S>: ImmutableStorage,
  {
    fn from(value: Utf8Bytes<S>) -> Self {
      String::from(value).into_boxed_str()
    }
  }

  impl<S> From<Utf8Bytes<S>> for Arc<str>
  where
    RawBytes<S>: ImmutableStorage,
  {
    fn from(value: Utf8Bytes<S>) -> Self {
      value.as_str().into()
    }
  }

  impl<S> From<Utf8Bytes<S>> for Rc<str>
  where
    RawBytes<S>: ImmutableStorage,
  {
    fn from(value: Utf8Bytes<S>) -> Self {
      value.as_str().into()
    }
  }

  impl<S> From<&String> for Utf8Bytes<S>
  where
    RawBytes<S>: ImmutableStorage,
  {
    fn from(s: &String) -> Self {
      Self::from(s.as_str())
    }
  }

  impl<S> From<Box<str>> for Utf8Bytes<S>
  where
    RawBytes<S>: ImmutableStorage,
  {
    fn from(s: Box<str>) -> Self {
      Self::from(String::from(s))
    }
  }

  impl<S> From<Arc<str>> for Utf8Bytes<S>
  where
    RawBytes<S>: ImmutableStorage,
  {
    fn from(s: Arc<str>) -> Self {
      Self::from(s.as_ref())
    }
  }

  impl<S> From<Rc<str>> for Utf8Bytes<S>
  where
    RawBytes<S>: ImmutableStorage,
  {
    fn from(s: Rc<str>) -> Self {
      Self::from(s.as_ref())
    }
  }
};
