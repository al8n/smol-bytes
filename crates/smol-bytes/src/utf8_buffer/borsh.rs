use std::string::{String, ToString};

use super::*;

impl ::borsh::BorshSerialize for Utf8Buffer {
  fn serialize<W: ::borsh::io::Write>(&self, writer: &mut W) -> ::borsh::io::Result<()> {
    self.as_str().serialize(writer)
  }
}

impl ::borsh::BorshDeserialize for Utf8Buffer {
  fn deserialize_reader<R: ::borsh::io::Read>(reader: &mut R) -> ::borsh::io::Result<Self> {
    let s = String::deserialize_reader(reader)?;
    Self::try_from(s)
      .map_err(|e| ::borsh::io::Error::new(::borsh::io::ErrorKind::InvalidData, e.to_string()))
  }
}
