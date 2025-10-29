use core::fmt;
use std::{string::String, vec::Vec};

use serde::de::{Deserializer, Error, Visitor};
use serde_core as serde;

use super::BytesMut;

// https://github.com/serde-rs/serde/blob/629802f2abfd1a54a6072992888fea7ca5bc209f/serde/src/private/de.rs#L56-L125
fn smol_bytes_mut<'de: 'a, 'a, D>(deserializer: D) -> Result<BytesMut, D::Error>
where
  D: Deserializer<'de>,
{
  struct BytesMutVisitor;

  impl<'a> Visitor<'a> for BytesMutVisitor {
    type Value = BytesMut;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
      formatter.write_str("a mutable bytes")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Ok(BytesMut::from(v))
    }

    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Ok(BytesMut::from(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Ok(BytesMut::from(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Ok(BytesMut::from(v))
    }

    fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Ok(BytesMut::from(v))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Ok(BytesMut::from(v))
    }
  }

  deserializer.deserialize_str(BytesMutVisitor)
}

impl serde::Serialize for BytesMut {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    self.as_slice().serialize(serializer)
  }
}

impl<'de> serde::Deserialize<'de> for BytesMut {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    smol_bytes_mut(deserializer)
  }
}
