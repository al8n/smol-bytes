use super::*;
use serde::de::{Deserializer, Error, Visitor};
use serde_core as serde;

impl serde::Serialize for Utf8Buffer {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(self.as_str())
  }
}

impl<'de> serde::Deserialize<'de> for Utf8Buffer {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let s = <&str as serde::Deserialize>::deserialize(deserializer)?;
    Self::try_from(s).map_err(serde::de::Error::custom)
  }
}
