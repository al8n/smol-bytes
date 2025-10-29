use crate::strategy::Strategy;

use std::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};

use super::*;

impl From<&[u8]> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(slice: &[u8]) -> Self {
    if slice.len() <= INLINE_CAP {
      // SAFETY: len is guaranteed to be less than or equal to INLINE_CAP
      Self(Repr::Inline(unsafe { Buffer::copy_from_slice(slice) }))
    } else {
      Self(Repr::Heap(BytesMut::from(slice)))
    }
  }
}

impl From<&str> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(s: &str) -> Self {
    Self::from(s.as_bytes())
  }
}

impl From<Bytes> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(bytes: Bytes) -> Self {
    Self::from_bytes(bytes)
  }
}

impl From<BytesMut> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(bytes: BytesMut) -> Self {
    Self::from_bytes_mut(bytes)
  }
}

impl<S> From<RawSmolBytes<S>> for SmolBytesMut
where
  RawSmolBytes<S>: Strategy,
{
  #[inline]
  fn from(v: RawSmolBytes<S>) -> Self {
    use crate::bytes::Repr;
    match v.repr {
      Repr::Inline(inline_storage) => SmolBytesMut::from_inline(inline_storage),
      Repr::Heap(bytes) => SmolBytesMut::from_bytes(bytes),
    }
  }
}

impl From<SmolBytesMut> for BytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(smol_bytes_mut: SmolBytesMut) -> Self {
    match smol_bytes_mut.0 {
      Repr::Inline(storage) => {
        let mut bytes_mut = BytesMut::with_capacity(storage.len());
        bytes_mut.put_slice(storage.as_slice());
        bytes_mut
      }
      Repr::Heap(b) => b,
    }
  }
}

impl From<SmolBytesMut> for Bytes {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(smol_bytes_mut: SmolBytesMut) -> Self {
    match smol_bytes_mut.0 {
      Repr::Inline(storage) => Bytes::copy_from_slice(&storage),
      Repr::Heap(b) => b.freeze(),
    }
  }
}

impl From<Vec<u8>> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(vec: Vec<u8>) -> Self {
    if vec.len() <= INLINE_CAP {
      // SAFETY: len is guaranteed to be less than or equal to INLINE_CAP
      Self(Repr::Inline(unsafe { Buffer::copy_from_slice(&vec) }))
    } else {
      Self(Repr::Heap(BytesMut::from(Bytes::from(vec))))
    }
  }
}

impl From<String> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(s: String) -> Self {
    Self::from(s.into_bytes())
  }
}

impl From<Box<[u8]>> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(vec: Box<[u8]>) -> Self {
    if vec.len() <= INLINE_CAP {
      // SAFETY: len is guaranteed to be less than or equal to INLINE_CAP
      Self(Repr::Inline(unsafe { Buffer::copy_from_slice(&vec) }))
    } else {
      Self(Repr::Heap(BytesMut::from(Bytes::from(vec))))
    }
  }
}

impl From<Arc<[u8]>> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(arc: Arc<[u8]>) -> Self {
    Self::from(arc.as_ref())
  }
}

impl From<Rc<[u8]>> for SmolBytesMut {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(rc: Rc<[u8]>) -> Self {
    Self::from(rc.as_ref())
  }
}
