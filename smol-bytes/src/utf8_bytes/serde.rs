use core::{fmt, marker::PhantomData};
use std::{string::String, vec::Vec};

use serde::de::{Error, Unexpected, Visitor};
use serde_core as serde;

use super::*;

impl<S> serde::Serialize for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
  where
    Ser: serde::Serializer,
  {
    serializer.serialize_str(self.as_str())
  }
}

/// Builds the value from whatever the deserializer hands over, without an
/// intermediate `String`.
struct Utf8BytesVisitor<S>(PhantomData<S>);

impl<S> Visitor<'_> for Utf8BytesVisitor<S>
where
  RawBytes<S>: ImmutableStorage,
{
  type Value = Utf8Bytes<S>;

  // Worded like serde's `String` visitor so error messages stay identical.
  fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str("a string")
  }

  // `visit_borrowed_str` forwards here by default.
  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: Error,
  {
    Ok(Utf8Bytes::from(v))
  }

  // Forwarding to `visit_str` would copy a heap-sized buffer instead of
  // keeping it.
  fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
  where
    E: Error,
  {
    Ok(Utf8Bytes::from(v))
  }

  // `visit_borrowed_bytes` forwards here by default.
  fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
  where
    E: Error,
  {
    Utf8Bytes::<S>::try_from(v).map_err(|_| E::invalid_value(Unexpected::Bytes(v), &self))
  }

  // Forwarding to `visit_bytes` would copy a heap-sized buffer instead of
  // keeping it.
  fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
  where
    E: Error,
  {
    Utf8Bytes::<S>::try_from(v)
      .map_err(|error| E::invalid_value(Unexpected::Bytes(error.as_bytes()), &self))
  }
}

impl<'de, S> serde::Deserialize<'de> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_string(Utf8BytesVisitor(PhantomData))
  }
}
