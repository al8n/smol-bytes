use core::fmt;
#[cfg(any(feature = "std", feature = "alloc"))]
use std::{string::String, vec::Vec};

use serde::de::{Deserializer, Error, Visitor};
use serde_core as serde;

use crate::INLINE_CAP;

use super::Buffer;

// https://github.com/serde-rs/serde/blob/629802f2abfd1a54a6072992888fea7ca5bc209f/serde/src/private/de.rs#L56-L125
fn buffer<'de: 'a, 'a, D>(deserializer: D) -> Result<Buffer, D::Error>
where
  D: Deserializer<'de>,
{
  struct BufferVisitor;

  impl<'a> Visitor<'a> for BufferVisitor {
    type Value = Buffer;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
      formatter.write_str("byte array")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Buffer::try_from(v).map_err(E::custom)
    }

    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Buffer::try_from(v).map_err(E::custom)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
      A: serde_core::de::SeqAccess<'a>,
    {
      match seq.size_hint() {
        Some(hint) if hint > INLINE_CAP => Err(serde::de::Error::custom("too many bytes")),
        _ => {
          let mut this = Buffer::new();
          while let Some(byte) = seq.next_element::<u8>()? {
            this.try_put_u8(byte).map_err(serde::de::Error::custom)?;
          }

          Ok(this)
        }
      }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Buffer::try_from(v).map_err(E::custom)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Buffer::try_from(v).map_err(E::custom)
    }

    fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Buffer::try_from(v).map_err(E::custom)
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Buffer::try_from(v).map_err(E::custom)
    }
  }

  deserializer.deserialize_byte_buf(BufferVisitor)
}

impl serde::Serialize for Buffer {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_bytes(self.as_slice())
  }
}

impl<'de> serde::Deserialize<'de> for Buffer {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    buffer(deserializer)
  }
}
