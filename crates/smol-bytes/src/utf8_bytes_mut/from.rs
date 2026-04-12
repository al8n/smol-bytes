use std::string::{String, ToString};

use super::*;

impl From<&str> for Utf8BytesMut {
  fn from(s: &str) -> Self {
    Self {
      inner: BytesMut::from(s.as_bytes()),
    }
  }
}

impl From<String> for Utf8BytesMut {
  fn from(s: String) -> Self {
    Self {
      inner: BytesMut::from(s.into_bytes()),
    }
  }
}

impl TryFrom<BytesMut> for Utf8BytesMut {
  type Error = core::str::Utf8Error;

  fn try_from(bytes: BytesMut) -> Result<Self, Self::Error> {
    core::str::from_utf8(bytes.as_ref())?;
    Ok(Self { inner: bytes })
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

impl From<Utf8BytesMut> for BytesMut {
  fn from(value: Utf8BytesMut) -> Self {
    value.inner
  }
}

impl From<Utf8BytesMut> for String {
  fn from(value: Utf8BytesMut) -> Self {
    value.as_str().to_string()
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  use std::{boxed::Box, rc::Rc, sync::Arc};

  impl From<Utf8BytesMut> for Box<str> {
    fn from(value: Utf8BytesMut) -> Self {
      value.as_str().into()
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
      Self::from(s.as_ref())
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
