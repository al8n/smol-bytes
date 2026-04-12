use std::string::{String, ToString};

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

impl<S> TryFrom<RawBytes<S>> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  type Error = core::str::Utf8Error;

  fn try_from(bytes: RawBytes<S>) -> Result<Self, Self::Error> {
    core::str::from_utf8(bytes.as_ref())?;
    Ok(Self { inner: bytes })
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

impl<S> From<Utf8Bytes<S>> for RawBytes<S> {
  fn from(value: Utf8Bytes<S>) -> Self {
    value.inner
  }
}

impl<S> From<Utf8Bytes<S>> for String
where
  RawBytes<S>: ImmutableStorage,
{
  fn from(value: Utf8Bytes<S>) -> Self {
    value.as_str().to_string()
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
      value.as_str().into()
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
      Self::from(s.as_ref())
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
