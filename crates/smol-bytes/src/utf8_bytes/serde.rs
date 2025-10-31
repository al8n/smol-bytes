use super::*;

impl ::serde::Serialize for Utf8Bytes {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: ::serde::Serializer,
  {
    serializer.serialize_str(self.as_str())
  }
}

impl<'de> ::serde::Deserialize<'de> for Utf8Bytes {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: ::serde::Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    Ok(Self::from(s))
  }
}
