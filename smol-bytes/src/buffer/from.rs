use super::*;

impl<'a> TryFrom<&'a Buffer> for &'a str {
  type Error = core::str::Utf8Error;

  fn try_from(value: &'a Buffer) -> Result<Self, Self::Error> {
    core::str::from_utf8(value.as_slice())
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl TryFrom<Buffer> for std::string::String {
  type Error = core::str::Utf8Error;

  fn try_from(value: Buffer) -> Result<Self, Self::Error> {
    use std::string::ToString;
    let s = core::str::from_utf8(value.as_slice())?;
    Ok(s.to_string())
  }
}

macro_rules! from_array {
  ($($size:literal),+$(,)?) => {
    $(
      impl From<[u8; $size]> for Buffer {
        fn from(value: [u8; $size]) -> Self {
          let mut this = Self::new();
          this.put_slice(value.as_slice());
          this
        }
      }
    )*
  };
}

impl From<[u8; INLINE_CAP]> for Buffer {
  fn from(value: [u8; INLINE_CAP]) -> Self {
    Self {
      end: InlineSize::_V62,
      cur: InlineSize::_V0,
      // SAFETY: `u8` and `MaybeUninit<u8>` have identical layout, and every
      // source byte is initialized, establishing the Buffer invariant.
      buf: unsafe { transmute::<[u8; INLINE_CAP], [MaybeUninit<u8>; INLINE_CAP]>(value) },
    }
  }
}

from_array!(
  0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
  27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
  51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61,
);

macro_rules! try_from {
  ($($method:ident($ty: ty)),+$(,)?) => {
    $(
      impl TryFrom<$ty> for Buffer {
        type Error = TryPutError;

        fn try_from(value: $ty) -> Result<Self, Self::Error> {
          let mut this = Self::new();
          this.try_put_slice(value.$method()).map(|_| this)
        }
      }
    )*
  };
}

try_from!(as_bytes(&str), as_ref(&[u8]),);

#[cfg(any(feature = "std", feature = "alloc"))]
const _: () = {
  use std::{boxed::Box, rc::Rc, string::String, sync::Arc, vec::Vec};

  try_from!(
    as_slice(Vec<u8>),
    as_bytes(String),
    as_ref(Arc<[u8]>),
    as_ref(Rc<[u8]>),
    as_ref(Box<[u8]>),
    as_bytes(Arc<str>),
    as_bytes(Rc<str>),
    as_bytes(Box<str>),
    as_ref(::bytes::Bytes),
    as_ref(::bytes::BytesMut),
  );
};
