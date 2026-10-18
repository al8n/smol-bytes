use core::{convert::Infallible, str::FromStr};
use std::{borrow::Cow, string::String, vec::Vec};

use super::*;

impl From<&str> for Utf8BytesMut {
  fn from(s: &str) -> Self {
    Self {
      inner: BytesMut::from(s.as_bytes()),
    }
  }
}

impl FromStr for Utf8BytesMut {
  type Err = Infallible;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self::from(s))
  }
}

impl From<String> for Utf8BytesMut {
  fn from(s: String) -> Self {
    Self {
      inner: BytesMut::from(s.into_bytes()),
    }
  }
}

impl<'a> From<Cow<'a, str>> for Utf8BytesMut {
  fn from(value: Cow<'a, str>) -> Self {
    match value {
      Cow::Borrowed(value) => Self::from(value),
      Cow::Owned(value) => Self::from(value),
    }
  }
}

impl TryFrom<BytesMut> for Utf8BytesMut {
  type Error = FromUtf8Error<BytesMut>;

  fn try_from(bytes: BytesMut) -> Result<Self, Self::Error> {
    Self::from_utf8(bytes)
  }
}

impl TryFrom<&[u8]> for Utf8BytesMut {
  type Error = core::str::Utf8Error;

  /// Creates a `Utf8BytesMut` from a byte slice, validating UTF-8.
  fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
    let s = core::str::from_utf8(bytes)?;
    Ok(Self::from(s))
  }
}

impl TryFrom<Vec<u8>> for Utf8BytesMut {
  type Error = FromUtf8Error<Vec<u8>>;

  fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
    match core::str::from_utf8(&bytes) {
      Ok(_) => Ok(Self {
        inner: BytesMut::from(bytes),
      }),
      Err(error) => Err(FromUtf8Error::new(bytes, error)),
    }
  }
}

impl TryFrom<::bytes::Bytes> for Utf8BytesMut {
  type Error = FromUtf8Error<::bytes::Bytes>;

  fn try_from(bytes: ::bytes::Bytes) -> Result<Self, Self::Error> {
    match core::str::from_utf8(bytes.as_ref()) {
      Ok(_) => Ok(Self {
        inner: BytesMut::from(bytes),
      }),
      Err(error) => Err(FromUtf8Error::new(bytes, error)),
    }
  }
}

impl TryFrom<::bytes::BytesMut> for Utf8BytesMut {
  type Error = FromUtf8Error<::bytes::BytesMut>;

  fn try_from(bytes: ::bytes::BytesMut) -> Result<Self, Self::Error> {
    match core::str::from_utf8(bytes.as_ref()) {
      Ok(_) => Ok(Self {
        inner: BytesMut::from(bytes),
      }),
      Err(error) => Err(FromUtf8Error::new(bytes, error)),
    }
  }
}

impl From<Utf8BytesMut> for BytesMut {
  fn from(value: Utf8BytesMut) -> Self {
    value.inner
  }
}

impl From<Utf8BytesMut> for String {
  fn from(value: Utf8BytesMut) -> Self {
    String::from_utf8(value.inner.into()).expect("Utf8Bytes invariant")
  }
}

impl From<Utf8BytesMut> for Vec<u8> {
  fn from(value: Utf8BytesMut) -> Self {
    value.into_inner().into()
  }
}

impl From<Utf8BytesMut> for ::bytes::Bytes {
  fn from(value: Utf8BytesMut) -> Self {
    value.into_inner().into()
  }
}

impl From<Utf8BytesMut> for ::bytes::BytesMut {
  fn from(value: Utf8BytesMut) -> Self {
    value.into_inner().into()
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  use std::{boxed::Box, rc::Rc, sync::Arc};

  impl From<Utf8BytesMut> for Box<str> {
    fn from(value: Utf8BytesMut) -> Self {
      String::from(value).into_boxed_str()
    }
  }

  impl From<Utf8BytesMut> for Arc<str> {
    fn from(value: Utf8BytesMut) -> Self {
      value.as_str().into()
    }
  }

  impl From<Utf8BytesMut> for Rc<str> {
    fn from(value: Utf8BytesMut) -> Self {
      value.as_str().into()
    }
  }

  impl From<&String> for Utf8BytesMut {
    fn from(s: &String) -> Self {
      Self::from(s.as_str())
    }
  }

  impl From<Box<str>> for Utf8BytesMut {
    fn from(s: Box<str>) -> Self {
      Self::from(String::from(s))
    }
  }

  impl From<Arc<str>> for Utf8BytesMut {
    fn from(s: Arc<str>) -> Self {
      Self::from(s.as_ref())
    }
  }

  impl From<Rc<str>> for Utf8BytesMut {
    fn from(s: Rc<str>) -> Self {
      Self::from(s.as_ref())
    }
  }
};
