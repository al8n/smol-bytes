use core::fmt;
use std::{string::String, vec::Vec};

use serde::de::{Deserializer, Error, Visitor};
use serde_core as serde;

use crate::strategy::Strategy;

use super::RawBytes;

// https://github.com/serde-rs/serde/blob/629802f2abfd1a54a6072992888fea7ca5bc209f/serde/src/private/de.rs#L56-L125
fn smol_bytes<'de: 'a, 'a, D, S>(deserializer: D) -> Result<RawBytes<S>, D::Error>
where
  D: Deserializer<'de>,
  RawBytes<S>: Strategy,
{
  struct RawBytesVisitor<S>(core::marker::PhantomData<S>);

  impl<'a, S> Visitor<'a> for RawBytesVisitor<S>
  where
    RawBytes<S>: Strategy,
  {
    type Value = RawBytes<S>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
      formatter.write_str("a bytes")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Ok(RawBytes::from(v))
    }

    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Ok(RawBytes::from(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Ok(RawBytes::from(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Ok(RawBytes::from(v))
    }

    fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Ok(RawBytes::from(v))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
      E: Error,
    {
      Ok(RawBytes::from(v))
    }
  }

  deserializer.deserialize_str(RawBytesVisitor(core::marker::PhantomData))
}

impl<St> serde::Serialize for RawBytes<St>
where
  Self: Strategy,
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    self.as_slice().serialize(serializer)
  }
}

impl<'de, S> serde::Deserialize<'de> for RawBytes<S>
where
  Self: Strategy,
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    smol_bytes(deserializer)
  }
}
