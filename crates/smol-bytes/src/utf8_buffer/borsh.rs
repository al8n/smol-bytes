use super::*;

impl ::borsh::BorshSerialize for Utf8Buffer {
  fn serialize<W: ::borsh::io::Write>(&self, writer: &mut W) -> ::borsh::io::Result<()> {
    self.as_str().serialize(writer)
  }
}

impl ::borsh::BorshDeserialize for Utf8Buffer {
  fn deserialize_reader<R: ::borsh::io::Read>(reader: &mut R) -> ::borsh::io::Result<Self> {
    let buffer = <crate::Buffer as ::borsh::BorshDeserialize>::deserialize_reader(reader)?;
    Utf8Buffer::try_from(buffer)
      .map_err(|_| ::borsh::io::Error::new(::borsh::io::ErrorKind::InvalidData, "invalid UTF-8"))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn multibyte_roundtrip() {
    let value = Utf8Buffer::try_from_str("café €").unwrap();
    let encoded = ::borsh::to_vec(&value).unwrap();
    let decoded = ::borsh::from_slice::<Utf8Buffer>(&encoded).unwrap();

    assert_eq!(decoded.as_str(), value.as_str());
  }

  #[test]
  fn rejects_invalid_utf8() {
    let encoded = [2, 0, 0, 0, 0xc3, 0x28];
    let error = ::borsh::from_slice::<Utf8Buffer>(&encoded).unwrap_err();

    assert_eq!(error.kind(), ::borsh::io::ErrorKind::InvalidData);
  }
}
