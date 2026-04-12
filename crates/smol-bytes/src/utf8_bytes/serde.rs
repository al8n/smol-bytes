use std::string::String;

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

impl<'de, S> serde::Deserialize<'de> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    Ok(Self::from(s))
  }
}
