#[cfg(any(feature = "std", feature = "alloc"))]
use std::string::String;

use serde_core as serde;

use super::*;

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
    #[cfg(any(feature = "std", feature = "alloc"))]
    {
      let s = String::deserialize(deserializer)?;
      Self::try_from_str(&s).map_err(serde::de::Error::custom)
    }
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    {
      let s = <&str as serde::Deserialize>::deserialize(deserializer)?;
      Self::try_from_str(s).map_err(serde::de::Error::custom)
    }
  }
}
