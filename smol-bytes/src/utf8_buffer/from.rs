use core::str::FromStr;
#[cfg(any(feature = "std", feature = "alloc"))]
use std::string::ToString;

use super::*;

impl Utf8Buffer {
  /// Validates a fixed buffer as UTF-8 without discarding it on failure.
  pub fn from_utf8(buffer: Buffer) -> Result<Self, FromUtf8Error<Buffer>> {
    match core::str::from_utf8(buffer.as_slice()) {
      Ok(_) => Ok(Self { inner: buffer }),
      Err(error) => Err(FromUtf8Error::new(buffer, error)),
    }
  }

  /// Tries to create a `Utf8Buffer` from a string slice.
  pub fn try_from_str(s: &str) -> Result<Self, TryPutError> {
    let inner = Buffer::try_from(s.as_bytes())?;
    Ok(Self { inner })
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<S> crate::utf8_bytes::Utf8Bytes<S> {
  /// Tries to move this validated UTF-8 view into a fixed [`Utf8Buffer`].
  ///
  /// Inline values move their inline storage while preserving the cursor.
  /// Short heap views are copied into inline storage, while longer values are
  /// returned unchanged.
  pub fn try_into_buffer(self) -> Result<Utf8Buffer, Self> {
    match self.inner.try_into_buffer() {
      Ok(inner) => Ok(Utf8Buffer { inner }),
      Err(inner) => Err(Self { inner }),
    }
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl crate::Utf8BytesMut {
  /// Tries to move this validated mutable UTF-8 view into a fixed
  /// [`Utf8Buffer`].
  ///
  /// Inline values move their inline storage while preserving the cursor.
  /// Short heap views are copied into inline storage, while longer values are
  /// returned unchanged.
  pub fn try_into_buffer(self) -> Result<Utf8Buffer, Self> {
    match self.inner.try_into_buffer() {
      Ok(inner) => Ok(Utf8Buffer { inner }),
      Err(inner) => Err(Self { inner }),
    }
  }
}

impl TryFrom<&str> for Utf8Buffer {
  type Error = TryPutError;

  fn try_from(s: &str) -> Result<Self, Self::Error> {
    Self::try_from_str(s)
  }
}

impl FromStr for Utf8Buffer {
  type Err = TryPutError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::try_from_str(s)
  }
}

impl From<Utf8Buffer> for Buffer {
  #[cfg_attr(not(coverage), inline(always))]
  fn from(value: Utf8Buffer) -> Self {
    value.inner
  }
}

impl TryFrom<Buffer> for Utf8Buffer {
  type Error = FromUtf8Error<Buffer>;

  fn try_from(buffer: Buffer) -> Result<Self, Self::Error> {
    Self::from_utf8(buffer)
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
  use std::{boxed::Box, rc::Rc, string::String, sync::Arc, vec::Vec};

  impl From<Utf8Buffer> for String {
    fn from(value: Utf8Buffer) -> Self {
      value.as_str().to_string()
    }
  }

  impl From<Utf8Buffer> for Vec<u8> {
    fn from(value: Utf8Buffer) -> Self {
      value.into_inner().as_slice().to_vec()
    }
  }

  impl From<Utf8Buffer> for ::bytes::Bytes {
    fn from(value: Utf8Buffer) -> Self {
      ::bytes::Bytes::copy_from_slice(value.as_str().as_bytes())
    }
  }

  impl From<Utf8Buffer> for ::bytes::BytesMut {
    fn from(value: Utf8Buffer) -> Self {
      ::bytes::BytesMut::from(value.as_str().as_bytes())
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
