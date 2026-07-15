#[cfg(any(feature = "std", feature = "alloc"))]
use std::string::ToString;

use super::*;

impl From<&str> for Utf8Buffer {
  fn from(s: &str) -> Self {
    let inner = Buffer::try_from(s.as_bytes()).expect("string too large for inline buffer");
    Self { inner }
  }
}

impl Utf8Buffer {
  /// Tries to create a `Utf8Buffer` from a string slice.
  pub fn try_from_str(s: &str) -> Result<Self, TryPutError> {
    let inner = Buffer::try_from(s.as_bytes())?;
    Ok(Self { inner })
  }
}

impl From<Utf8Buffer> for Buffer {
  #[cfg_attr(not(coverage), inline(always))]
  fn from(value: Utf8Buffer) -> Self {
    value.inner
  }
}

impl TryFrom<Buffer> for Utf8Buffer {
  type Error = core::str::Utf8Error;

  fn try_from(buffer: Buffer) -> Result<Self, Self::Error> {
    // Validate UTF-8
    core::str::from_utf8(buffer.as_slice())?;
    Ok(Self { inner: buffer })
  }
}

impl TryFrom<&[u8]> for Utf8Buffer {
  type Error = FromBytesError;

  /// Creates a `Utf8Buffer` from a byte slice, validating UTF-8 and
  /// checking inline capacity.
  fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
    let s = core::str::from_utf8(bytes)?;
    Ok(Self::try_from_str(s)?)
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  use std::{boxed::Box, rc::Rc, string::String, sync::Arc};

  impl From<Utf8Buffer> for String {
    fn from(value: Utf8Buffer) -> Self {
      value.as_str().to_string()
    }
  }

  impl From<Utf8Buffer> for Box<str> {
    fn from(value: Utf8Buffer) -> Self {
      value.as_str().into()
    }
  }

  impl From<Utf8Buffer> for Arc<str> {
    fn from(value: Utf8Buffer) -> Self {
      value.as_str().into()
    }
  }

  impl From<Utf8Buffer> for Rc<str> {
    fn from(value: Utf8Buffer) -> Self {
      value.as_str().into()
    }
  }

  impl TryFrom<String> for Utf8Buffer {
    type Error = TryPutError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
      Self::try_from_str(s.as_str())
    }
  }

  impl TryFrom<&String> for Utf8Buffer {
    type Error = TryPutError;

    fn try_from(s: &String) -> Result<Self, Self::Error> {
      Self::try_from_str(s.as_str())
    }
  }

  impl TryFrom<Box<str>> for Utf8Buffer {
    type Error = TryPutError;

    fn try_from(s: Box<str>) -> Result<Self, Self::Error> {
      Self::try_from_str(s.as_ref())
    }
  }

  impl TryFrom<Arc<str>> for Utf8Buffer {
    type Error = TryPutError;

    fn try_from(s: Arc<str>) -> Result<Self, Self::Error> {
      Self::try_from_str(s.as_ref())
    }
  }

  impl TryFrom<Rc<str>> for Utf8Buffer {
    type Error = TryPutError;

    fn try_from(s: Rc<str>) -> Result<Self, Self::Error> {
      Self::try_from_str(s.as_ref())
    }
  }
};
