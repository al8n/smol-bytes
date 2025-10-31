use super::*;

impl From<&str> for Utf8Bytes {
  fn from(s: &str) -> Self {
    Self {
      inner: Bytes::copy_from_slice(s.as_bytes()),
    }
  }
}

impl From<String> for Utf8Bytes {
  fn from(s: String) -> Self {
    Self {
      inner: Bytes::from(s.into_bytes()),
    }
  }
}

impl TryFrom<Bytes> for Utf8Bytes {
  type Error = core::str::Utf8Error;

  fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
    core::str::from_utf8(bytes.as_ref())?;
    Ok(Self { inner: bytes })
  }
}

impl From<Utf8Bytes> for Bytes {
  fn from(value: Utf8Bytes) -> Self {
    value.inner
  }
}

impl From<Utf8Bytes> for String {
  fn from(value: Utf8Bytes) -> Self {
    value.as_str().to_string()
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  use std::{boxed::Box, rc::Rc, sync::Arc};

  impl From<Utf8Bytes> for Box<str> {
    fn from(value: Utf8Bytes) -> Self {
      value.as_str().into()
    }
  }

  impl From<Utf8Bytes> for Arc<str> {
    fn from(value: Utf8Bytes) -> Self {
      value.as_str().into()
    }
  }

  impl From<Utf8Bytes> for Rc<str> {
    fn from(value: Utf8Bytes) -> Self {
      value.as_str().into()
    }
  }

  impl From<&String> for Utf8Bytes {
    fn from(s: &String) -> Self {
      Self::from(s.as_str())
    }
  }

  impl From<Box<str>> for Utf8Bytes {
    fn from(s: Box<str>) -> Self {
      Self::from(s.as_ref())
    }
  }

  impl From<Arc<str>> for Utf8Bytes {
    fn from(s: Arc<str>) -> Self {
      Self::from(s.as_ref())
    }
  }

  impl From<Rc<str>> for Utf8Bytes {
    fn from(s: Rc<str>) -> Self {
      Self::from(s.as_ref())
    }
  }
};
