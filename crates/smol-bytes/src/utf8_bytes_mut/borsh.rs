use super::*;

impl ::borsh::BorshSerialize for Utf8BytesMut {
  fn serialize<W: ::borsh::io::Write>(&self, writer: &mut W) -> ::borsh::io::Result<()> {
    self.as_str().serialize(writer)
  }
}

impl ::borsh::BorshDeserialize for Utf8BytesMut {
  fn deserialize_reader<R: ::borsh::io::Read>(reader: &mut R) -> ::borsh::io::Result<Self> {
    let s = String::deserialize_reader(reader)?;
    Ok(Self::from(s))
  }
}
