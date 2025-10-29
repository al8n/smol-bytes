use core::fmt;
#[cfg(any(feature = "std", feature = "alloc"))]
use std::{string::String, vec::Vec};

use serde::de::{Deserializer, Error, Visitor};
use serde_core as serde;

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
      formatter.write_str("a buffer")
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

  deserializer.deserialize_str(BufferVisitor)
}

impl serde::Serialize for Buffer {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    self.as_slice().serialize(serializer)
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
