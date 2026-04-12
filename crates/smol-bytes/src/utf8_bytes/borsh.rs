use std::string::String;

use super::*;

impl<S> ::borsh::BorshSerialize for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn serialize<W: ::borsh::io::Write>(&self, writer: &mut W) -> ::borsh::io::Result<()> {
    self.as_str().serialize(writer)
  }
}

impl<S> ::borsh::BorshDeserialize for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn deserialize_reader<R: ::borsh::io::Read>(reader: &mut R) -> ::borsh::io::Result<Self> {
    let s = String::deserialize_reader(reader)?;
    Ok(Self::from(s))
  }
}
